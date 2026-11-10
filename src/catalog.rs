use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    pub name: String,
    pub title: String,
    pub category: String,
    pub unavailable: Option<String>,
    pub url: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub required: bool,
    pub sets: Vec<String>,
    pub values: Vec<String>,
    #[serde(default)]
    pub mandatory_sets: Vec<String>,
}

impl Parameter {
    pub fn in_set(&self, set: &str) -> bool {
        self.sets
            .iter()
            .any(|s| s == set || s == "(All)" || s == "__AllParameterSets")
    }
    pub fn required_in(&self, set: &str) -> bool {
        if self.mandatory_sets.is_empty() {
            self.required && self.in_set(set)
        } else {
            self.mandatory_sets
                .iter()
                .any(|s| s == set || s == "__AllParameterSets")
        }
    }
    pub fn boolean(&self) -> bool {
        self.kind.to_lowercase().contains("boolean") || self.kind.contains("SwitchParameter")
    }
    pub fn multiple(&self) -> bool {
        self.kind.ends_with("[]") || self.kind.contains("MultiValued")
    }
    pub fn credential(&self) -> bool {
        self.kind.contains("PSCredential")
    }
    pub fn secret(&self) -> bool {
        self.credential()
            || self.kind.contains("SecureString")
            || self.name.to_lowercase().contains("password")
    }
}

impl Command {
    pub fn write(&self) -> bool {
        !self.name.starts_with("Get-") && !self.name.starts_with("Test-")
    }
    pub fn supports(&self, name: &str) -> bool {
        self.parameters.iter().any(|p| p.name == name)
    }
    pub fn sets(&self) -> Vec<String> {
        let mut sets: Vec<_> = self
            .parameters
            .iter()
            .flat_map(|p| p.sets.clone())
            .filter(|s| s != "(All)" && s != "__AllParameterSets")
            .collect();
        sets.sort();
        sets.dedup();
        if sets.is_empty() {
            sets.push("Default".into());
        }
        sets
    }
}

pub fn load() -> Vec<Command> {
    serde_json::from_str(include_str!("../data/commands.json")).expect("bundled command catalog")
}

pub fn humanize(value: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in value.chars() {
        if c.is_uppercase() && prev_lower {
            out.push(' ');
        }
        out.push(c);
        prev_lower = c.is_lowercase() || c.is_numeric();
    }
    out
}

pub fn help(name: &str) -> &'static str {
    match name {
        "Identity" => "Target mailbox, folder, policy or relationship. Folder identities use mailbox:\\localized folder path.",
        "User" => "User principal name, Default (everyone in the organization), or Anonymous (external users).",
        "AccessRights" => "Common roles: AvailabilityOnly, LimitedDetails, Reviewer, Editor, PublishingEditor, Owner, None. One per line.",
        "SharingPermissionFlags" => "Delegate grants meeting delegation. CanViewPrivateItems requires Delegate and Editor access. None removes delegate flags.",
        "SendNotificationToUser" => "Send a sharing invitation when supported. Review Exchange's role and notification requirements.",
        "AutomateProcessing" => "None, AutoUpdate, or AutoAccept. AutoAccept enables automatic resource booking decisions.",
        "BookingWindowInDays" => "How far ahead a resource can be booked, in days.",
        "MaximumDurationInMinutes" => "Longest permitted resource reservation, in minutes.",
        "BookInPolicy" | "RequestInPolicy" | "RequestOutOfPolicy" | "ResourceDelegates" => "Enter recipient addresses, one per line. JSON {\"Add\":[...],\"Remove\":[...]} updates a multivalue collection where supported.",
        "QueryStartDate" | "StartDate" | "EndDate" => "Use an unambiguous ISO date/time, e.g. 2026-11-10T09:00:00. Diagnostic dates filter log activity, not a complete agenda.",
        "CancelOrganizedMeetings" => "Cancellation notices can be sent to attendees. PreviewOnly is recommended before submitting cancellations.",
        "PreviewOnly" => "Preview eligible changes without applying them. Inspect the returned results before executing a live operation.",
        "FromWindowsTimeZoneId" | "ToWindowsTimeZoneId" | "TimeZone" => "Use a Windows time zone ID supported by Exchange, e.g. Pacific Standard Time or UTC.",
        "PublishEnabled" => "Expose published calendar information to external viewers according to publishing and sharing policies.",
        "DetailLevel" => "Choose AvailabilityOnly, LimitedDetails, or FullDetails according to the supported metadata.",
        "Domains" => "Sharing domains and actions, one per line, e.g. example.com:CalendarSharingFreeBusySimple.",
        "Credential" | "Credentials" => "Credentials for a remote availability service, not this app's Microsoft 365 sign-in. Never saved by this application.",
        _ => "Optional settings remain unchanged until their Include checkbox is selected. Open Microsoft reference for exact semantics and limits.",
    }
}

#[derive(Clone, Default)]
pub struct Field {
    pub enabled: bool,
    pub text: String,
    pub password: String,
    pub clear: bool,
}

pub fn collect(
    command: &Command,
    set: &str,
    fields: &BTreeMap<String, Field>,
) -> Result<Map<String, Value>, String> {
    let mut args = Map::new();
    for p in &command.parameters {
        if matches!(p.name.as_str(), "Confirm" | "WhatIf") || !p.in_set(set) {
            continue;
        }
        let field = fields.get(&p.name).cloned().unwrap_or_default();
        if !field.enabled {
            if p.required_in(set) {
                return Err(format!("{} is required for {}", humanize(&p.name), set));
            }
            continue;
        }
        if field.clear {
            args.insert(p.name.clone(), Value::Null);
            continue;
        }
        let s = field.text.trim();
        let value = if p.credential() {
            if s.is_empty() || field.password.is_empty() {
                return Err(format!("{} needs a username and password", p.name));
            }
            serde_json::json!({"username":s,"password":field.password})
        } else if p.boolean() {
            Value::Bool(
                s.parse::<bool>()
                    .map_err(|_| format!("{} must be true or false", p.name))?,
            )
        } else if p.kind.contains("Int")
            && !p.kind.contains("Interval")
            && !p.kind.contains("IdParameter")
        {
            Value::Number(
                s.parse::<i64>()
                    .map_err(|_| format!("{} must be a whole number", p.name))?
                    .into(),
            )
        } else if s.starts_with('{') || (p.multiple() && s.starts_with('[')) {
            serde_json::from_str(s).map_err(|e| format!("{}: invalid JSON: {}", p.name, e))?
        } else if p.multiple() {
            Value::Array(
                s.lines()
                    .filter(|x| !x.trim().is_empty())
                    .map(|x| Value::String(x.trim().into()))
                    .collect(),
            )
        } else {
            if p.required_in(set) && s.is_empty() {
                return Err(format!("{} cannot be empty", humanize(&p.name)));
            }
            Value::String(field.text.clone())
        };
        args.insert(p.name.clone(), value);
    }
    Ok(args)
}

pub fn redact(command: &Command, args: &Map<String, Value>) -> Value {
    Value::Object(
        args.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    if command
                        .parameters
                        .iter()
                        .any(|p| p.name == *k && p.secret())
                    {
                        Value::String("[redacted]".into())
                    } else {
                        v.clone()
                    },
                )
            })
            .collect(),
    )
}

pub fn folder_identity(mailbox: &str, path: &str) -> String {
    format!(
        "{}:\\{}",
        mailbox,
        path.trim_start_matches(['/', '\\']).replace('/', "\\")
    )
}
