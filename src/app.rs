use crate::{
    bridge::{Bridge, Event, Response},
    catalog::{self, Command, Field},
    model::{self, CalendarFolder, Mailbox, TenantProfile},
    presentation as p,
};
use eframe::egui::{self, Color32, RichText};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    time::Duration,
};

const BLUE: Color32 = Color32::from_rgb(0, 103, 192);
const TEXT: Color32 = Color32::from_rgb(32, 36, 43);
const MUTED: Color32 = Color32::from_rgb(97, 104, 115);
const BG: Color32 = Color32::from_rgb(246, 248, 251);
const LINE: Color32 = Color32::from_rgb(225, 230, 236);
const TINT: Color32 = Color32::from_rgb(232, 241, 252);

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Mailboxes,
    Organization,
    Activity,
}
#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Access,
    Booking,
    Preferences,
    Sharing,
    Diagnostics,
}
#[derive(Clone)]
struct Job {
    action: String,
    payload: Value,
    label: String,
}
struct Pending {
    id: u64,
    job: Job,
}
struct Editor {
    index: usize,
    set: String,
    fields: BTreeMap<String, Field>,
    baseline: Value,
    search: String,
    list_inputs: BTreeMap<String, String>,
    recipient_query: BTreeMap<String, String>,
    loaded: bool,
    preview: bool,
}
struct Review {
    command: Command,
    args: Map<String, Value>,
    title: String,
}
struct Activity {
    time: String,
    label: String,
    result: String,
}

pub struct CalendarAdmin {
    page: Page,
    tab: Tab,
    demo: bool,
    catalog: Vec<Command>,
    available: BTreeSet<String>,
    schemas: BTreeSet<String>,
    bridge: Option<Bridge>,
    queue: VecDeque<Job>,
    pending: Option<Pending>,
    connected: bool,
    connection: Value,
    profiles: Vec<TenantProfile>,
    profile: usize,
    delegated: bool,
    device: bool,
    disable_wam: bool,
    pwsh: String,
    connections_open: bool,
    runtime: String,
    notices: VecDeque<String>,
    mailboxes: Vec<Mailbox>,
    directory_loaded: bool,
    directory_error: Option<String>,
    recipients: Vec<Mailbox>,
    zones: Vec<(String, String)>,
    selected: Option<usize>,
    folders: Vec<CalendarFolder>,
    folder: Option<usize>,
    query: String,
    kind: String,
    cache: BTreeMap<String, Value>,
    requested: BTreeSet<String>,
    editor: Option<Editor>,
    review: Option<Review>,
    report: Option<(String, Value)>,
    message: Option<(bool, String)>,
    details: Option<String>,
    activity: Vec<Activity>,
    org_tab: usize,
}

fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text).size(12.0).color(MUTED));
}
fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(14.0);
    ui.label(RichText::new(title).size(18.0).strong().color(TEXT));
    ui.add_space(4.0);
}
fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, LINE))
        .corner_radius(7)
        .inner_margin(18)
}
fn primary(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Color32::WHITE))
            .fill(BLUE)
            .min_size(egui::vec2(90.0, 33.0)),
    )
}
fn first(v: &Value) -> Value {
    v.as_array()
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or_else(|| v.clone())
}
fn rows(v: &Value) -> Vec<Value> {
    v.as_array()
        .cloned()
        .unwrap_or_else(|| if v.is_null() { vec![] } else { vec![v.clone()] })
}

impl CalendarAdmin {
    pub fn new(cc: &eframe::CreationContext<'_>, demo: bool) -> Self {
        Self::with_context(&cc.egui_ctx, demo)
    }
    fn with_context(ctx: &egui::Context, demo: bool) -> Self {
        install_font(ctx);
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = Color32::WHITE;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.selection.bg_fill = TINT;
        style.visuals.selection.stroke = egui::Stroke::new(1.0, BLUE);
        style.visuals.hyperlink_color = BLUE;
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.interact_size.y = 31.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(27.0));
        ctx.set_style(style);
        let profiles = std::fs::read(profile_path())
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<TenantProfile>>(&b).ok())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| {
                vec![TenantProfile {
                    label: "My organization".into(),
                    ..Default::default()
                }]
            });
        let mailboxes = if demo {
            model::demo_mailboxes()
        } else {
            vec![]
        };
        Self {
            page: Page::Mailboxes,
            tab: Tab::Overview,
            demo,
            catalog: catalog::load(),
            available: BTreeSet::new(),
            schemas: BTreeSet::new(),
            bridge: None,
            queue: VecDeque::new(),
            pending: None,
            connected: false,
            connection: Value::Null,
            profiles,
            profile: 0,
            delegated: false,
            device: false,
            disable_wam: false,
            pwsh: default_pwsh(),
            connections_open: false,
            runtime: String::new(),
            notices: VecDeque::new(),
            zones: time_zones()
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
            directory_loaded: demo,
            directory_error: None,
            recipients: mailboxes.clone(),
            mailboxes,
            selected: None,
            folders: vec![],
            folder: None,
            query: String::new(),
            kind: "All mailboxes".into(),
            cache: BTreeMap::new(),
            requested: BTreeSet::new(),
            editor: None,
            review: None,
            report: None,
            message: None,
            details: None,
            activity: vec![],
            org_tab: 0,
        }
    }
    fn scope_key(&self) -> String {
        if self.page == Page::Mailboxes {
            format!("{}|{}", self.target(), self.folder_target())
        } else {
            "organization".into()
        }
    }
    fn target(&self) -> String {
        self.selected
            .and_then(|i| self.mailboxes.get(i))
            .map(|m| m.email.clone())
            .unwrap_or_default()
    }
    fn folder_target(&self) -> String {
        self.folder
            .and_then(|i| self.folders.get(i))
            .map(|f| catalog::folder_identity(&self.target(), &f.path))
            .unwrap_or_default()
    }
    fn tenant(&self) -> String {
        if self.demo {
            return "Contoso · Demo".into();
        }
        if !self.connected {
            return "Connect Microsoft 365".into();
        }
        let profile = &self.profiles[self.profile];
        if !profile.label.trim().is_empty()
            && !matches!(
                profile.label.as_str(),
                "My organization" | "New organization"
            )
        {
            return profile.label.clone();
        }
        let v = first(&self.connection["connections"]);
        if let Some(domain) = v["DelegatedOrganization"]
            .as_str()
            .filter(|s| !s.is_empty())
        {
            return domain.into();
        }
        if let Some(domain) = v["UserPrincipalName"]
            .as_str()
            .and_then(|s| s.split_once('@').map(|(_, domain)| domain))
        {
            return domain.into();
        }
        "My organization".into()
    }

    fn busy(&self) -> bool {
        self.pending.is_some() || !self.queue.is_empty()
    }
    fn enqueue(&mut self, action: &str, payload: Value, label: impl Into<String>) {
        if action == "directory" {
            self.directory_loaded = false;
            self.directory_error = None;
        }
        self.queue.push_back(Job {
            action: action.into(),
            payload,
            label: label.into(),
        });
    }
    fn clear(&mut self) {
        self.mailboxes.clear();
        self.directory_loaded = false;
        self.directory_error = None;
        self.recipients.clear();
        self.selected = None;
        self.folders.clear();
        self.folder = None;
        self.cache.clear();
        self.requested.clear();
        self.editor = None;
        self.review = None;
        self.report = None;
        self.available.clear();
        self.schemas.clear();
        self.connection = Value::Null;
        self.catalog = catalog::load();
    }
    fn poll(&mut self) {
        let events: Vec<_> = self
            .bridge
            .as_ref()
            .map(|b| b.events.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Response(r) => self.response(r),
                Event::Notice(s) => {
                    if self.notices.len() > 80 {
                        self.notices.pop_front();
                    }
                    self.notices.push_back(s);
                }
                Event::Closed => {
                    self.connected = false;
                    self.pending = None;
                    self.queue.clear();
                    self.clear();
                    self.bridge.take();
                    self.message = Some((
                        false,
                        "Your connection ended. Reconnect to continue.".into(),
                    ));
                }
            }
        }
        if self.pending.is_none() {
            if let Some(job) = self.queue.pop_front() {
                if let Some(b) = self.bridge.as_mut() {
                    match b.send(&job.action, job.payload.clone()) {
                        Ok(id) => self.pending = Some(Pending { id, job }),
                        Err(e) => {
                            self.details = Some(e);
                            self.message = Some((
                                false,
                                "We couldn't reach Microsoft 365. Reconnect and try again.".into(),
                            ));
                            self.queue.clear();
                        }
                    }
                }
            }
        }
    }
    fn response(&mut self, r: Response) {
        if r.id == 0 {
            self.runtime = if r.ok {
                format!(
                    "Ready · PowerShell {} · Exchange {}",
                    p::value(&r.data["powershell"]),
                    p::value(&r.data["module"])
                )
            } else {
                r.message
            };
            return;
        }
        if self.pending.as_ref().map(|p| p.id) != Some(r.id) {
            return;
        }
        let job = self.pending.take().unwrap().job;
        self.activity.push(Activity {
            time: chrono::Local::now().format("%H:%M").to_string(),
            label: job.label.clone(),
            result: if r.ok {
                "Completed".into()
            } else {
                "Couldn't complete".into()
            },
        });
        if !r.ok {
            if job.action == "directory" {
                self.directory_loaded = true;
                self.directory_error = Some(r.message.clone());
            }
            self.details = Some(r.message);
            self.message = Some((
                false,
                format!(
                    "We couldn't {}. Check your connection or permissions.",
                    job.label.to_lowercase()
                ),
            ));
            if job.action == "connect" {
                self.connected = false;
                self.queue.clear();
            }
            return;
        }
        match job.action.as_str() {
            "connect" => {
                self.connected = true;
                self.demo = false;
                self.connection = r.data;
                self.available = self.connection["commands"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect();
                self.connections_open = false;
                self.enqueue("zones", json!({}), "Load time zones");
                self.enqueue("directory", json!({}), "Load mailboxes");
                self.enqueue("recipients", json!({}), "Load people and groups");
                self.message = Some((true, "You're connected. Loading your mailboxes…".into()));
            }
            "disconnect" => {
                self.connected = false;
                self.clear();
                self.message = Some((true, "Disconnected.".into()));
            }
            "directory" => {
                if self
                    .message
                    .as_ref()
                    .is_some_and(|(ok, text)| *ok && text.contains("Loading your mailboxes"))
                {
                    self.message = None;
                }
                self.directory_loaded = true;
                match model::directory_rows(&r.data) {
                    Ok(mut mailboxes) => {
                        let previous = self.target();
                        mailboxes.sort_by_key(|m| m.name.to_lowercase());
                        self.selected = mailboxes.iter().position(|m| m.email == previous);
                        self.mailboxes = mailboxes;
                        self.directory_error = None;
                        if self.recipients.is_empty() {
                            self.recipients = self.mailboxes.clone();
                        }
                        if let Some(activity) = self.activity.last_mut() {
                            activity.result = format!("{} mailboxes loaded", self.mailboxes.len());
                        }
                    }
                    Err(error) => {
                        self.directory_error = Some(error.clone());
                        self.details = Some(error);
                        if let Some(activity) = self.activity.last_mut() {
                            activity.result = "Couldn't read directory".into();
                        }
                    }
                }
            }
            "zones" => {
                let zones: Vec<_> = rows(&r.data)
                    .iter()
                    .filter_map(|v| {
                        Some((v["id"].as_str()?.to_owned(), v["name"].as_str()?.to_owned()))
                    })
                    .collect();
                if !zones.is_empty() {
                    self.zones = zones;
                }
            }
            "recipients" => {
                if let Ok(recipients) = model::directory_rows(&r.data) {
                    if !recipients.is_empty() {
                        self.recipients = recipients;
                    }
                }
                self.recipients
                    .retain(|m| !m.kind.contains("DistributionGroup"));
                self.recipients.sort_by_key(|m| m.name.to_lowercase());
            }
            "calendars" => {
                if job.payload["mailbox"].as_str() == Some(&self.target()) {
                    self.folders = serde_json::from_value(r.data).unwrap_or_default();
                    self.folder = if self.folders.is_empty() {
                        None
                    } else {
                        Some(0)
                    };
                    self.load_tab();
                }
            }
            "schema" => {
                if let Some(name) = job.payload["command"].as_str() {
                    if let Some(i) = self.catalog.iter().position(|c| c.name == name) {
                        if let Ok(mut parameters) = serde_json::from_value::<Vec<catalog::Parameter>>(
                            r.data["parameters"].clone(),
                        ) {
                            for parameter in &mut parameters {
                                if matches!(
                                    parameter.kind.as_str(),
                                    "System.Object" | "System.Object[]"
                                ) {
                                    if let Some(reference) = self.catalog[i]
                                        .parameters
                                        .iter()
                                        .find(|p| p.name == parameter.name)
                                    {
                                        parameter.kind = reference.kind.clone();
                                    }
                                }
                            }
                            self.catalog[i].parameters = parameters;
                            self.schemas.insert(name.into());
                            if self.editor.as_ref().is_some_and(|e| e.index == i) {
                                let set = r.data["default_set"]
                                    .as_str()
                                    .unwrap_or("Default")
                                    .to_owned();
                                let baseline = self.editor.as_ref().unwrap().baseline.clone();
                                self.editor = Some(self.make_editor(i, Some(set), baseline));
                            }
                        }
                    }
                }
            }
            "invoke" => {
                let name = job.payload["command"].as_str().unwrap_or("").to_owned();
                if name.starts_with("Get-") {
                    if let Some(scope) = job.payload.get("scope").and_then(Value::as_str) {
                        if scope != self.scope_key() {
                            return;
                        }
                    }
                    self.cache.insert(name.clone(), r.data.clone());
                    if job.payload["report"].as_bool() == Some(true) {
                        self.report = Some((job.label, r.data.clone()));
                    }
                    if let Some(e) = self.editor.as_ref() {
                        if paired_read(&self.catalog[e.index].name) == Some(name.as_str()) {
                            let index = e.index;
                            let set = e.set.clone();
                            self.editor = Some(self.make_editor(index, Some(set), first(&r.data)));
                        }
                    }
                } else {
                    let preview = job.payload["arguments"]["PreviewOnly"].as_bool() == Some(true);
                    if preview || name.starts_with("Test-") {
                        self.report = Some((job.label, r.data));
                    } else {
                        self.message = Some((true, "Changes saved.".into()));
                        self.editor = None;
                        self.cache.clear();
                        self.requested.clear();
                        self.load_tab();
                    }
                }
            }
            _ => {}
        }
    }
    fn query_command(&mut self, name: &str, title: &str, report: bool) {
        if self.demo || !self.connected {
            return;
        }
        if !self.available.contains(name) {
            return;
        }
        let Some(c) = self.catalog.iter().find(|c| c.name == name) else {
            return;
        };
        let mut args = Map::new();
        if c.supports("Identity")
            && self.page == Page::Mailboxes
            && !self.target().is_empty()
            && c.category != "Sharing & availability"
        {
            let target = if c
                .parameters
                .iter()
                .any(|p| p.name == "Identity" && p.kind.contains("FolderId"))
            {
                self.folder_target()
            } else {
                self.target()
            };
            if !target.is_empty() {
                args.insert("Identity".into(), json!(target));
            }
        }
        if matches!(
            name,
            "Get-MailboxFolderStatistics" | "Get-EXOMailboxFolderStatistics"
        ) {
            args.insert("FolderScope".into(), json!("Calendar"));
        }
        self.enqueue(
            "invoke",
            json!({"command":name,"arguments":args,"report":report,"scope":self.scope_key()}),
            title,
        );
    }
    fn load_tab(&mut self) {
        let list: Vec<(&str, &str)> = match self.tab {
            Tab::Overview => vec![
                (
                    "Get-MailboxCalendarConfiguration",
                    "Load calendar preferences",
                ),
                ("Get-MailboxRegionalConfiguration", "Load regional settings"),
            ],
            Tab::Access => vec![("Get-EXOMailboxFolderPermission", "Load calendar access")],
            Tab::Booking => vec![("Get-CalendarProcessing", "Load booking settings")],
            Tab::Preferences => vec![
                (
                    "Get-MailboxCalendarConfiguration",
                    "Load calendar preferences",
                ),
                ("Get-MailboxRegionalConfiguration", "Load regional settings"),
                ("Get-EventsFromEmailConfiguration", "Load automatic events"),
            ],
            Tab::Sharing => vec![("Get-MailboxCalendarFolder", "Load sharing settings")],
            Tab::Diagnostics => vec![],
        };
        for (n, t) in list {
            if !self.cache.contains_key(n) {
                self.query_command(n, t, false);
            }
        }
    }
    fn select_mailbox(&mut self, index: usize) {
        self.selected = Some(index);
        self.tab = Tab::Overview;
        self.folder = None;
        self.folders.clear();
        self.cache.clear();
        self.requested.clear();
        self.editor = None;
        if self.demo {
            self.folders = vec![
                CalendarFolder {
                    name: "Calendar".into(),
                    path: "/Calendar".into(),
                    count: json!(24),
                    kind: "Calendar".into(),
                    size: String::new(),
                },
                CalendarFolder {
                    name: "Team projects".into(),
                    path: "/Team projects".into(),
                    count: json!(8),
                    kind: "User Created".into(),
                    size: String::new(),
                },
            ];
            self.folder = Some(0);
            self.seed_demo();
        } else {
            self.enqueue(
                "calendars",
                json!({"mailbox":self.target()}),
                "Load calendars",
            );
        }
    }
    fn seed_demo(&mut self) {
        self.cache.insert("Get-EXOMailboxFolderPermission".into(),json!([{"User":"Everyone in your organization","AccessRights":["AvailabilityOnly"],"SharingPermissionFlags":"None"},{"User":"Megan Bowen","AccessRights":["Editor"],"SharingPermissionFlags":"Delegate"}]));
        self.cache.insert(
            "Get-MailboxCalendarConfiguration".into(),
            json!([
                serde_json::from_str::<Value>(include_str!("../data/demo-calendar.json"))
                    .expect("bundled calendar sample")
            ]),
        );
        self.cache.insert("Get-MailboxRegionalConfiguration".into(),json!([{"TimeZone":"Mountain Standard Time","Language":"en-US","DateFormat":"M/d/yyyy","TimeFormat":"h:mm tt"}]));
        self.cache.insert("Get-CalendarProcessing".into(),json!([{"AutomateProcessing":"AutoAccept","AllowRecurringMeetings":true,"AllowConflicts":false,"BookingWindowInDays":180,"MaximumDurationInMinutes":120,"AllBookInPolicy":true,"ProcessExternalMeetingMessages":false}]));
        self.cache.insert("Get-MailboxCalendarFolder".into(),json!([{"PublishEnabled":false,"DetailLevel":"AvailabilityOnly","SearchableUrlEnabled":false}]));
        self.cache.insert(
            "Get-EventsFromEmailConfiguration".into(),
            json!([{"EventsFromEmailEnabled":true,"CreateEventsFromEmailAsPrivate":true}]),
        );
    }
    fn open_editor(&mut self, name: &str, baseline: Option<Value>) {
        if self.connected && !self.available.contains(name) {
            self.message = Some((
                false,
                "Your account doesn't have permission to change these settings.".into(),
            ));
            return;
        }
        let Some(i) = self.catalog.iter().position(|c| c.name == name) else {
            return;
        };
        let base = baseline.unwrap_or_else(|| {
            paired_read(name)
                .and_then(|n| self.cache.get(n))
                .map(first)
                .unwrap_or(Value::Null)
        });
        self.editor = Some(self.make_editor(i, None, base));
        if self.connected && self.available.contains(name) {
            if !self.schemas.contains(name) {
                self.enqueue("schema", json!({"command":name}), "Prepare settings");
            }
            if let Some(read) = paired_read(name) {
                if !self.cache.contains_key(read) {
                    self.query_command(read, "Load current settings", false);
                }
            }
        }
    }
    fn make_editor(&self, index: usize, set: Option<String>, baseline: Value) -> Editor {
        let c = &self.catalog[index];
        let sets = c.sets();
        let set = set
            .filter(|s| sets.contains(s))
            .unwrap_or_else(|| sets[0].clone());
        let mut fields = BTreeMap::new();
        for param in &c.parameters {
            let mut field = Field::default();
            if let Some(value) = baseline.get(&param.name) {
                if !value.is_null() {
                    field.text = p::edit_value(&param.name, value).unwrap_or_default();
                }
            }
            if param.name == "Identity"
                && self.page == Page::Mailboxes
                && c.category != "Sharing & availability"
            {
                field.text = if param.kind.contains("FolderId") {
                    self.folder_target()
                } else {
                    self.target()
                };
                field.enabled = !field.text.is_empty();
            }
            if param.name == "Parent" && !self.folder_target().is_empty() {
                field.text = self.folder_target();
                field.enabled = true;
            }
            if param.required_in(&set) {
                if param.kind.contains("SwitchParameter") {
                    field.text = "true".into();
                }
                field.enabled = true;
            }
            if param.boolean() && field.text.is_empty() {
                field.text = "false".into();
            }
            if param.name == "SendNotificationToUser" && c.name == "Add-MailboxFolderPermission" {
                field.enabled = true;
                field.text = "true".into();
            }
            if param.name == "PreviewOnly" {
                field.enabled = true;
                field.text = "true".into();
            }
            if param.name == "AccessRights" && field.text.is_empty() {
                field.text = "Reviewer".into();
                field.enabled = true;
            }
            if param.name == "CancelOrganizedMeetings" {
                field.text = "true".into();
                field.enabled = true;
            }
            fields.insert(param.name.clone(), field);
        }
        Editor {
            index,
            set,
            fields,
            baseline,
            search: String::new(),
            list_inputs: BTreeMap::new(),
            recipient_query: BTreeMap::new(),
            loaded: self.schemas.contains(&c.name) || self.demo,
            preview: c.supports("PreviewOnly"),
        }
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar")
            .exact_height(60.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(19, 37, 61))
                    .inner_margin(egui::Margin::symmetric(22, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let (r, _) =
                        ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 5, BLUE);
                    ui.painter().rect_stroke(
                        r.shrink(5.0),
                        2,
                        egui::Stroke::new(1.4, Color32::WHITE),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().line_segment(
                        [
                            r.min + egui::vec2(5.0, 11.0),
                            r.min + egui::vec2(21.0, 11.0),
                        ],
                        egui::Stroke::new(1.4, Color32::WHITE),
                    );
                    ui.label(
                        RichText::new("Calendar admin center")
                            .size(19.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(self.tenant()).color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(40, 61, 87))
                                .min_size(egui::vec2(190.0, 34.0)),
                            )
                            .clicked()
                        {
                            self.connections_open = true;
                        }
                        if self.connected {
                            ui.label(
                                RichText::new("● Connected")
                                    .size(12.0)
                                    .color(Color32::from_rgb(139, 220, 171)),
                            );
                        }
                    });
                });
            });
    }
    fn navigation(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("nav")
            .exact_width(190.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(249, 250, 252))
                    .inner_margin(14),
            )
            .show(ctx, |ui| {
                ui.add_space(20.0);
                muted(ui, "MANAGE");
                ui.add_space(10.0);
                for (page, label) in [
                    (Page::Mailboxes, "Mailboxes"),
                    (Page::Organization, "Organization"),
                    (Page::Activity, "Recent activity"),
                ] {
                    let selected = self.page == page;
                    if ui
                        .add_sized(
                            [162.0, 42.0],
                            egui::Button::new(RichText::new(label).color(if selected {
                                BLUE
                            } else {
                                TEXT
                            }))
                            .fill(if selected {
                                TINT
                            } else {
                                Color32::TRANSPARENT
                            }),
                        )
                        .clicked()
                    {
                        self.page = page;
                    }
                }
                if self.demo {
                    ui.add_space(28.0);
                    ui.separator();
                    ui.add_space(12.0);
                    muted(ui, "You're exploring a demo.");
                    if ui.link("Connect your organization").clicked() {
                        self.connections_open = true;
                    }
                }
                if self.busy() {
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        muted(ui, "Updating…");
                    });
                }
            });
    }
    fn directory_toolbar(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text("Search mailboxes")
                .desired_width(if compact { 270.0 } else { 350.0 }),
        );
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_salt("mailbox_type")
                .selected_text(&self.kind)
                .show_ui(ui, |ui| {
                    for k in ["All mailboxes", "People", "Shared", "Rooms", "Equipment"] {
                        ui.selectable_value(&mut self.kind, k.into(), k);
                    }
                });
            if !compact
                && ui
                    .add_enabled(self.connected && !self.busy(), egui::Button::new("Refresh"))
                    .clicked()
            {
                self.enqueue("directory", json!({}), "Refresh mailboxes");
            }
        });
    }
    fn directory_list(&mut self, ui: &mut egui::Ui, compact: bool) {
        self.directory_toolbar(ui, compact);
        ui.add_space(8.0);
        let query = self.query.to_lowercase();
        let mut clicked = None;
        let mut count = 0;
        egui::ScrollArea::vertical()
            .id_salt("mailbox_list")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, m) in self.mailboxes.iter().enumerate() {
                    if !(m.name.to_lowercase().contains(&query)
                        || m.email.to_lowercase().contains(&query))
                    {
                        continue;
                    }
                    if !match self.kind.as_str() {
                        "People" => m.kind == "UserMailbox",
                        "Shared" => m.kind.contains("Shared"),
                        "Rooms" => m.kind.contains("Room"),
                        "Equipment" => m.kind.contains("Equipment"),
                        _ => true,
                    } {
                        continue;
                    }
                    count += 1;
                    let selected = self.selected == Some(i);
                    let fill = if selected { TINT } else { Color32::WHITE };
                    let response = egui::Frame::new()
                        .fill(fill)
                        .inner_margin(12)
                        .corner_radius(4)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(
                                    egui::vec2(34.0, 34.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    r.center(),
                                    17.0,
                                    Color32::from_rgb(227, 234, 245),
                                );
                                let initials: String = m
                                    .name
                                    .split_whitespace()
                                    .take(2)
                                    .filter_map(|s| s.chars().next())
                                    .collect();
                                ui.painter().text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    initials,
                                    egui::FontId::proportional(12.0),
                                    BLUE,
                                );
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&m.name).strong());
                                    muted(ui, &m.email);
                                });
                                if !compact {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(RichText::new("Open  ›").color(BLUE));
                                            muted(ui, m.kind.replace("Mailbox", ""));
                                        },
                                    );
                                }
                            });
                        })
                        .response;
                    let r = ui.interact(
                        response.rect,
                        ui.id().with(("mailbox", i)),
                        egui::Sense::click(),
                    );
                    r.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &m.name)
                    });
                    if r.clicked() {
                        clicked = Some(i);
                    }
                    ui.add_space(3.0);
                }
                if count == 0 {
                    muted(ui, "No matching mailboxes.");
                }
            });
        if let Some(i) = clicked {
            self.select_mailbox(i);
        }
    }
    fn mailboxes(&mut self, ui: &mut egui::Ui) {
        if self.mailboxes.is_empty() && self.connected {
            ui.heading("Mailboxes");
            muted(ui, format!("Connected to {}", self.tenant()));
            ui.add_space(25.0);
            let loading = self
                .pending
                .as_ref()
                .is_some_and(|p| p.job.action == "directory")
                || self.queue.iter().any(|job| job.action == "directory");
            card().show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                if loading{ui.horizontal(|ui|{ui.spinner();ui.label("Loading your mailboxes…");});muted(ui,"Your sign-in succeeded. The directory is being retrieved.");}
                else{
                    if self.directory_error.is_some(){ui.label(RichText::new("We couldn't load your mailboxes").size(20.0).strong());muted(ui,"Your account is still connected. Try loading the directory again.");}
                    else if self.directory_loaded{ui.label(RichText::new("No mailboxes were returned").size(20.0).strong());muted(ui,"Microsoft 365 returned no mailboxes for this account. You can refresh or check the account in connection details.");}
                    else{ui.label("Your account is connected. Load the mailbox directory to continue.");}
                    ui.horizontal(|ui|{if primary(ui,"Load mailboxes").clicked(){self.enqueue("directory",json!({}),"Load mailboxes");}
if ui.button("Connection details").clicked(){self.connections_open=true;}});
                    if let Some(error)=&self.directory_error{ui.collapsing("Error details",|ui|{ui.label(error);});}
                }
            });
            return;
        }
        if self.mailboxes.is_empty() && self.busy() {
            ui.heading("Mailboxes");
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Connecting to Microsoft 365…");
            });
            return;
        }
        if self.mailboxes.is_empty() {
            ui.heading("Mailboxes");
            ui.label(RichText::new("Manage calendar access, sharing and booking.").color(MUTED));
            ui.add_space(55.0);
            card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(25.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Your calendars, all in one place")
                            .size(25.0)
                            .strong(),
                    );
                    ui.label("Connect Microsoft 365 to find a person, team or room.");
                    ui.add_space(18.0);
                    if primary(ui, "Connect Microsoft 365").clicked() {
                        self.connections_open = true;
                    }
                    if ui.link("Explore the demo").clicked() {
                        self.demo = true;
                        self.mailboxes = model::demo_mailboxes();
                        self.recipients = self.mailboxes.clone();
                    }
                });
                ui.add_space(30.0);
            });
            return;
        }
        if self.selected.is_none() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Mailboxes");
                    muted(
                        ui,
                        "Choose a person, shared mailbox or room to manage their calendars.",
                    );
                });
            });
            ui.add_space(20.0);
            card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                self.directory_list(ui, false);
            });
        } else {
            egui::SidePanel::left("mailbox_inner")
                .resizable(true)
                .default_width(310.0)
                .min_width(270.0)
                .max_width(420.0)
                .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(14))
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Mailboxes").size(20.0).strong());
                        if ui.small_button("All").clicked() {
                            self.selected = None;
                        }
                    });
                    ui.add_space(12.0);
                    self.directory_list(ui, true);
                });
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin {
                    left: 24,
                    right: 0,
                    top: 0,
                    bottom: 0,
                }))
                .show_inside(ui, |ui| {
                    self.mailbox_detail(ui);
                });
        }
    }
    fn mailbox_detail(&mut self, ui: &mut egui::Ui) {
        let Some(m) = self.selected.and_then(|i| self.mailboxes.get(i)).cloned() else {
            return;
        };
        ui.heading(&m.name);
        ui.horizontal_wrapped(|ui| {
            muted(ui, &m.email);
            muted(ui, m.kind.replace("Mailbox", " mailbox"));
        });
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Calendar").strong());
            let old = self.folder;
            let selected = self
                .folder
                .and_then(|i| self.folders.get(i))
                .map(|f| f.name.as_str())
                .unwrap_or(if self.busy() {
                    "Loading calendars…"
                } else {
                    "No calendar found"
                });
            egui::ComboBox::from_id_salt("active_calendar")
                .width(245.0)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (i, f) in self.folders.iter().enumerate() {
                        ui.selectable_value(&mut self.folder, Some(i), &f.name);
                    }
                });
            if old != self.folder {
                self.cache.remove("Get-EXOMailboxFolderPermission");
                self.cache.remove("Get-MailboxCalendarFolder");
                if self.demo {
                    self.seed_demo();
                } else {
                    self.load_tab();
                }
            }
            if ui
                .add_enabled(self.connected && !self.busy(), egui::Button::new("Refresh"))
                .clicked()
            {
                self.cache.clear();
                self.requested.clear();
                self.enqueue("calendars", json!({"mailbox":m.email}), "Refresh calendars");
            }
        });
        ui.add_space(13.0);
        ui.horizontal_wrapped(|ui| {
            for (tab, title) in [
                (Tab::Overview, "Overview"),
                (Tab::Access, "People & access"),
                (Tab::Booking, "Booking"),
                (Tab::Preferences, "Preferences"),
                (Tab::Sharing, "Sharing"),
                (Tab::Diagnostics, "Troubleshooting"),
            ] {
                if tab == Tab::Booking && !m.kind.contains("Room") && !m.kind.contains("Equipment")
                {
                    continue;
                }
                if ui.selectable_label(self.tab == tab, title).clicked() {
                    self.tab = tab;
                    self.load_tab();
                }
            }
        });
        ui.separator();
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .id_salt("calendar_details")
            .auto_shrink([false, false])
            .show(ui, |ui| match self.tab {
                Tab::Overview => self.overview(ui),
                Tab::Access => self.access(ui),
                Tab::Booking => self.booking(ui),
                Tab::Preferences => self.preferences(ui),
                Tab::Sharing => self.sharing(ui),
                Tab::Diagnostics => self.diagnostics(ui),
            });
    }
    fn settings_card(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        read: &str,
        write: &str,
        keys: &[&str],
    ) {
        let value = self.cache.get(read).map(first).unwrap_or(Value::Null);
        let mut edit = false;
        card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Edit").clicked() {
                        edit = true;
                    }
                });
            });
            if value.is_null() {
                muted(
                    ui,
                    if self.busy() {
                        "Loading settings…"
                    } else {
                        "Settings aren't available yet."
                    },
                );
            } else {
                egui::Grid::new(ui.id().with(("settings_values", title)))
                    .num_columns(2)
                    .spacing(egui::vec2(28.0, 10.0))
                    .show(ui, |ui| {
                        for key in keys {
                            if let Some(value) = value.get(key) {
                                ui.label(RichText::new(p::label(key)).color(MUTED));
                                ui.label(p::field_value(key, value));
                                ui.end_row();
                            }
                        }
                    });
            }
        });
        if edit {
            self.open_editor(write, Some(value));
        }
        ui.add_space(12.0);
    }
    fn overview(&mut self, ui: &mut egui::Ui) {
        section(ui, "Calendar overview");
        if let Some(f) = self.folder.and_then(|i| self.folders.get(i)) {
            card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new(&f.name).size(20.0).strong());
                muted(ui, format!("{} items", p::value(&f.count)));
            });
            ui.add_space(12.0);
        }
        self.settings_card(
            ui,
            "Working hours",
            "Get-MailboxCalendarConfiguration",
            "Set-MailboxCalendarConfiguration",
            &[
                "WorkDays",
                "WorkingHoursStartTime",
                "WorkingHoursEndTime",
                "WorkingHoursTimeZone",
            ],
        );
        self.settings_card(
            ui,
            "Language & time zone",
            "Get-MailboxRegionalConfiguration",
            "Set-MailboxRegionalConfiguration",
            &["TimeZone", "Language"],
        );
        section(ui, "More mailbox options");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Mailbox details").clicked() {
                self.query_command("Get-Mailbox", "Mailbox details", true);
            }
            if ui.button("Mailbox calendar settings").clicked() {
                self.open_editor("Set-Mailbox", None);
            }
            ui.menu_button("Folder options", |ui| {
                for (label, cmd) in [
                    ("View calendar folders", "Get-EXOMailboxFolderStatistics"),
                    ("Folder report", "Get-MailboxFolderStatistics"),
                    ("Browse folders", "Get-MailboxFolder"),
                    ("Create a mailbox folder", "New-MailboxFolder"),
                ] {
                    if ui.button(label).clicked() {
                        self.open_editor(cmd, None);
                        ui.close_menu();
                    }
                }
            });
        });
    }
    fn access(&mut self, ui: &mut egui::Ui) {
        section(ui, "People with access");
        muted(ui, "Choose who can see or change this calendar.");
        ui.add_space(10.0);
        if primary(ui, "Add people").clicked() {
            self.open_editor("Add-MailboxFolderPermission", Some(json!({})));
        }
        ui.add_space(13.0);
        let permissions = self
            .cache
            .get("Get-EXOMailboxFolderPermission")
            .or_else(|| self.cache.get("Get-MailboxFolderPermission"))
            .map(rows)
            .unwrap_or_default();
        if permissions.is_empty() {
            card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                muted(
                    ui,
                    if self.busy() {
                        "Loading people…"
                    } else {
                        "No access entries to show."
                    },
                );
            });
        }
        for permission in permissions {
            let user = p::value(&permission["User"]);
            let role = rows(&permission["AccessRights"])
                .first()
                .map(p::value)
                .unwrap_or_else(|| "None".into());
            let mut action = None;
            card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(if user == "Default" || user == "Anonymous" {
                                p::option(&user)
                            } else {
                                user.clone()
                            })
                            .strong(),
                        );
                        muted(ui, p::option(&role));
                        if p::value(&permission["SharingPermissionFlags"]).contains("Delegate") {
                            muted(ui, "Delegate");
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !matches!(
                            user.as_str(),
                            "Default"
                                | "Anonymous"
                                | "Everyone in your organization"
                                | "People outside your organization"
                        ) && ui.button("Remove").clicked()
                        {
                            action = Some("Remove-MailboxFolderPermission");
                        }
                        if ui.button("Edit").clicked() {
                            action = Some("Set-MailboxFolderPermission");
                        }
                    });
                });
            });
            if let Some(cmd) = action {
                let mut baseline = permission.clone();
                let identifier = match user.as_str() {
                    "Everyone in your organization" => "Default".to_owned(),
                    "People outside your organization" => "Anonymous".to_owned(),
                    _ => self
                        .recipients
                        .iter()
                        .find(|m| m.name == user)
                        .map(|m| m.email.clone())
                        .unwrap_or(user.clone()),
                };
                baseline["User"] = json!(identifier);
                self.open_editor(cmd, Some(baseline));
                if let Some(e) = self.editor.as_mut() {
                    if let Some(f) = e.fields.get_mut("User") {
                        f.enabled = true;
                    }
                }
            }
            ui.add_space(8.0);
        }
    }
    fn booking(&mut self, ui: &mut egui::Ui) {
        section(ui, "Resource booking");
        self.settings_card(
            ui,
            "Booking rules",
            "Get-CalendarProcessing",
            "Set-CalendarProcessing",
            &[
                "AutomateProcessing",
                "BookingWindowInDays",
                "MaximumDurationInMinutes",
                "AllowRecurringMeetings",
                "AllowConflicts",
                "AllBookInPolicy",
                "ResourceDelegates",
            ],
        );
        if ui.button("Edit room details").clicked() {
            self.open_editor("Set-Place", None);
        }
        if ui.button("View room details").clicked() {
            self.query_command("Get-Place", "Room details", true);
        }
    }
    fn preferences(&mut self, ui: &mut egui::Ui) {
        section(ui, "Calendar preferences");
        self.settings_card(
            ui,
            "Working hours & display",
            "Get-MailboxCalendarConfiguration",
            "Set-MailboxCalendarConfiguration",
            &[
                "WorkDays",
                "WorkingHoursStartTime",
                "WorkingHoursEndTime",
                "DefaultReminderTime",
                "RemindersEnabled",
                "WeekStartDay",
            ],
        );
        self.settings_card(
            ui,
            "Language & region",
            "Get-MailboxRegionalConfiguration",
            "Set-MailboxRegionalConfiguration",
            &["TimeZone", "Language", "DateFormat", "TimeFormat"],
        );
        self.settings_card(
            ui,
            "Events from email",
            "Get-EventsFromEmailConfiguration",
            "Set-EventsFromEmailConfiguration",
            &[
                "EventsFromEmailEnabled",
                "CreateEventsFromEmailAsPrivate",
                "FlightReservationProcessingLevel",
                "HotelReservationProcessingLevel",
            ],
        );
    }
    fn sharing(&mut self, ui: &mut egui::Ui) {
        section(ui, "Publish & share");
        self.settings_card(
            ui,
            "Calendar publishing",
            "Get-MailboxCalendarFolder",
            "Set-MailboxCalendarFolder",
            &[
                "PublishEnabled",
                "DetailLevel",
                "SearchableUrlEnabled",
                "PublishedCalendarUrl",
                "PublishedICalUrl",
            ],
        );
        muted(ui,"Publishing makes the selected calendar available through a link. Your organization's sharing policy still applies.");
        ui.add_space(12.0);
        if ui.button("Choose a sharing policy").clicked() {
            self.open_editor("Set-Mailbox", None);
            if let Some(e) = self.editor.as_mut() {
                e.search = "sharing".into();
            }
        }
    }
    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        section(ui, "Find and fix calendar issues");
        for (title, desc, cmd) in [
            (
                "Find meeting activity",
                "Search the history of a meeting to investigate changes or missing updates.",
                "Get-CalendarDiagnosticObjects",
            ),
            (
                "Cancel upcoming meetings",
                "Review and cancel meetings organized by this mailbox.",
                "Remove-CalendarEvents",
            ),
            (
                "Update event time zones",
                "Move existing events to a new time zone while preserving their local time.",
                "Invoke-CalendarEventMigration",
            ),
        ] {
            card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new(title).size(18.0).strong());
                muted(ui, desc);
                if ui
                    .button(if cmd.starts_with("Get-") {
                        "Search activity"
                    } else {
                        "Get started"
                    })
                    .clicked()
                {
                    self.open_editor(cmd, None);
                }
            });
            ui.add_space(12.0);
        }
        ui.menu_button("Other reports", |ui| {
            if ui.button("Calendar permissions report").clicked() {
                self.query_command("Get-MailboxFolderPermission", "Calendar permissions", true);
                ui.close_menu();
            }
        });
    }
    fn organization(&mut self, ui: &mut egui::Ui) {
        ui.heading("Organization");
        muted(ui, "Manage how calendars work across your organization.");
        ui.add_space(18.0);
        ui.horizontal_wrapped(|ui| {
            for (i, title) in [
                "External sharing",
                "Availability",
                "Rooms & equipment",
                "Defaults",
            ]
            .iter()
            .enumerate()
            {
                if ui.selectable_label(self.org_tab == i, *title).clicked() {
                    self.org_tab = i;
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| match self.org_tab {
            0 => {
                self.org_card(
                    ui,
                    "Sharing policies",
                    "Control the details people can share outside your organization.",
                    "Get-SharingPolicy",
                    &[
                        ("Create policy", "New-SharingPolicy"),
                        ("Edit policy", "Set-SharingPolicy"),
                        ("Remove policy", "Remove-SharingPolicy"),
                    ],
                );
                self.org_card(
                    ui,
                    "Partner organizations",
                    "Share availability with another Microsoft 365 organization.",
                    "Get-OrganizationRelationship",
                    &[
                        ("Add organization", "New-OrganizationRelationship"),
                        ("Edit organization", "Set-OrganizationRelationship"),
                        ("Remove organization", "Remove-OrganizationRelationship"),
                        ("Check connection", "Test-OrganizationRelationship"),
                    ],
                );
            }
            1 => {
                self.org_card(
                    ui,
                    "Availability sharing",
                    "Manage your organization's availability-sharing configuration.",
                    "Get-AvailabilityConfig",
                    &[
                        ("Set up sharing", "New-AvailabilityConfig"),
                        ("Edit settings", "Set-AvailabilityConfig"),
                        ("Remove settings", "Remove-AvailabilityConfig"),
                    ],
                );
                self.org_card(
                    ui,
                    "Availability connections",
                    "Connect free/busy lookups with partner domains.",
                    "Get-AvailabilityAddressSpace",
                    &[
                        ("Add connection", "Add-AvailabilityAddressSpace"),
                        ("Edit connection", "Set-AvailabilityConfig"),
                        ("Remove connection", "Remove-AvailabilityAddressSpace"),
                    ],
                );
            }
            2 => {
                self.org_card(
                    ui,
                    "Rooms & equipment",
                    "View and update the details people use to find a room.",
                    "Get-Place",
                    &[("Edit a room", "Set-Place")],
                );
                self.org_card(
                    ui,
                    "Resource attributes",
                    "Manage the custom attributes available for rooms and equipment.",
                    "Get-ResourceConfig",
                    &[("Edit attributes", "Set-ResourceConfig")],
                );
            }
            _ => {
                self.org_card(
                    ui,
                    "Calendar defaults",
                    "Manage organization-wide calendar and meeting settings.",
                    "Get-OrganizationConfig",
                    &[("Edit defaults", "Set-OrganizationConfig")],
                );
                if ui.button("Refresh mailbox directory").clicked() {
                    self.query_command("Get-EXOMailbox", "Mailbox directory", true);
                }
            }
        });
    }
    fn org_card(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        desc: &str,
        read: &str,
        actions: &[(&str, &str)],
    ) {
        section(ui, title);
        if self.connected && self.requested.insert(read.into()) {
            self.query_command(read, title, false);
        }
        let objects = self.cache.get(read).map(rows).unwrap_or_default();
        card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            muted(ui, desc);
            ui.horizontal_wrapped(|ui| {
                if ui.button("View current settings").clicked() {
                    self.query_command(read, title, true);
                }
                for (label, cmd) in actions {
                    if !objects.is_empty()
                        && (cmd.starts_with("Set-") || cmd.starts_with("Remove-"))
                    {
                        continue;
                    }
                    if ui.button(*label).clicked() {
                        self.open_editor(cmd, None);
                    }
                }
            });
            for object in objects {
                ui.separator();
                let title = object
                    .get("DisplayName")
                    .or_else(|| object.get("Name"))
                    .or_else(|| object.get("ForestName"))
                    .map(p::value)
                    .unwrap_or_else(|| "Current settings".into());
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(title).strong());
                    for (label, cmd) in actions
                        .iter()
                        .filter(|(_, c)| c.starts_with("Set-") || c.starts_with("Remove-"))
                    {
                        if ui.button(*label).clicked() {
                            self.open_editor(cmd, Some(object.clone()));
                        }
                    }
                });
            }
        });
        ui.add_space(12.0);
    }
    fn editor_window(&mut self, ctx: &egui::Context) {
        let Some(mut editor) = self.editor.take() else {
            return;
        };
        let command = self.catalog[editor.index].clone();
        let mut open = true;
        let mut submit = false;
        let mut preview = false;
        let mut cancel = false;
        let title = editor_title(&command.name, &command.title);
        let target = if self.page == Page::Mailboxes {
            self.target()
        } else {
            String::new()
        };
        let calendar = self
            .folder
            .and_then(|i| self.folders.get(i))
            .map(|f| f.name.clone())
            .unwrap_or_default();
        let modal=egui::Modal::new(egui::Id::new("settings_editor")).frame(card()).show(ctx,|ui|{
ui.set_width((ctx.screen_rect().width()-100.0).clamp(400.0,620.0));
ui.horizontal(|ui|{ui.label(RichText::new(&title).size(22.0).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("Close").clicked(){cancel=true;}});});ui.add_space(8.0);
   if !target.is_empty(){muted(ui,format!("{}  ·  {}",self.selected.and_then(|i|self.mailboxes.get(i)).map(|m|m.name.as_str()).unwrap_or(&target),calendar));ui.separator();}
   if command.name=="Remove-CalendarEvents"{ui.label("Choose the period to clear. Only meetings organized by this person will be canceled.");}
   if command.name=="Invoke-CalendarEventMigration"{ui.label("Review the affected events before moving them to a new time zone.");}
   if command.name=="Remove-MailboxFolderPermission"{ui.label("This person will lose their current access to the selected calendar.");}
   if !editor.loaded&&self.connected{ui.horizontal(|ui|{ui.spinner();muted(ui,"Loading current settings…");});}
   if command.name=="Get-MailboxFolder" {let mut recursive=editor.set=="Recurse";if ui.checkbox(&mut recursive,"Include all subfolders").changed(){editor=self.make_editor(editor.index,Some(if recursive{"Recurse"}else{"GetChildren"}.into()),editor.baseline.clone());}}
   if command.parameters.len()>12{ui.add(egui::TextEdit::singleline(&mut editor.search).hint_text("Find a setting").desired_width(f32::INFINITY));}
   let context_identity=!target.is_empty()&&command.category!="Sharing & availability";
   let mut params:Vec<_>=command.parameters.iter().filter(|param|param.in_set(&editor.set)&&!matches!(param.name.as_str(),"Confirm"|"WhatIf"|"PreviewOnly")&&!(param.name=="Identity"&&context_identity)&&calendar_field(&command.name,&param.name)&&!(param.name=="Parent"&&context_identity)&&!((param.name=="GetChildren"||param.name=="Recurse")&&param.required_in(&editor.set))).cloned().collect();
   params.sort_by_key(|param|(field_order(&command.name,&param.name),p::label(&param.name)));
   let query=editor.search.to_lowercase();
   egui::ScrollArea::vertical().id_salt("edit_settings").max_height((ctx.screen_rect().height()-300.0).clamp(240.0,510.0)).show(ui,|ui|{
    let mut more=vec![];
    for param in &params{
     if !query.is_empty()&&!p::label(&param.name).to_lowercase().contains(&query){continue;}
     if query.is_empty()&&field_order(&command.name,&param.name)>=50&&!param.required_in(&editor.set){more.push(param);continue;}
     render_field(ui,param,&mut editor,&self.recipients,&self.zones,&self.cache);
    }
    if !more.is_empty(){ui.add_space(8.0);ui.collapsing("More options",|ui|{for param in more{render_field(ui,param,&mut editor,&self.recipients,&self.zones,&self.cache);}});}
   });
   ui.separator();
   if self.demo{muted(ui,"Demo · Connect your organization to save changes.");}
   let ready=self.connected&&!self.demo&&editor.loaded&&self.available.contains(&command.name)&&!self.busy();
   ui.horizontal(|ui|{
    let button=if command.name.starts_with("Get-")||command.name.starts_with("Test-"){"Search"}else if command.name.starts_with("Remove-"){"Continue"}else if command.name.starts_with("Add-")||command.name.starts_with("New-"){"Add"}else{"Save changes"};
    if editor.preview&&ui.add_enabled(ready,egui::Button::new("Preview changes")).clicked(){preview=true;}
    if ui.add_enabled(ready,egui::Button::new(RichText::new(button).color(Color32::WHITE)).fill(BLUE)).clicked(){submit=true;}
    if ui.button("Cancel").clicked(){cancel=true;}
   });
  });
        if modal.should_close() {
            cancel = true;
        }
        if submit || preview {
            normalize_access(&mut editor);
            match catalog::collect(&command, &editor.set, &editor.fields) {
                Ok(mut args) => {
                    if command.supports("PreviewOnly") {
                        args.insert("PreviewOnly".into(), json!(preview));
                    }
                    if !preview
                        && (command.name.starts_with("Remove-")
                            || command.name == "Invoke-CalendarEventMigration"
                            || command.name == "Set-OrganizationConfig")
                    {
                        self.review = Some(Review {
                            command: command.clone(),
                            args,
                            title: title.clone(),
                        });
                    } else {
                        self.enqueue("invoke",json!({"command":command.name,"arguments":args,"report":!command.write()}),if preview{format!("Preview {}",title.to_lowercase())}else{title.clone()});
                        if !preview {
                            open = false;
                        }
                    }
                }
                Err(e) => {
                    self.message = Some((false, friendly_validation(&e, &command)));
                }
            }
        }
        if open && !cancel {
            self.editor = Some(editor);
        }
    }
    fn review_window(&mut self, ctx: &egui::Context) {
        let Some(review) = self.review.take() else {
            return;
        };
        let open = true;
        let mut apply = false;
        let mut cancel = false;
        let modal=egui::Modal::new(egui::Id::new("confirm_changes")).frame(card()).show(ctx,|ui|{ui.set_width(500.0);
   ui.label(RichText::new(&review.title).size(21.0).strong());muted(ui,self.tenant());if !self.target().is_empty(){ui.label(self.target());}
   if review.command.name=="Remove-CalendarEvents"{ui.colored_label(Color32::from_rgb(164,38,44),"Attendees will receive cancellation notices. This can't be undone from here.");}
   if review.command.name=="Invoke-CalendarEventMigration"{ui.label("Affected attendees may receive meeting updates.");}
   ui.add_space(10.0);
   for(key,value)in &review.args{if matches!(key.as_str(),"Identity"|"PreviewOnly"|"CancelOrganizedMeetings"){continue;}ui.horizontal_wrapped(|ui|{ui.label(RichText::new(p::label(key)).strong());ui.label(if review.command.parameters.iter().any(|p|p.name==*key&&p.secret()){"••••••••".into()}else{display_field_value(key,value,&self.recipients)});});}
   ui.add_space(14.0);ui.horizontal(|ui|{if primary(ui,if review.command.name.starts_with("Remove-"){"Confirm removal"}else{"Apply changes"}).clicked(){apply=true;}
if ui.button("Go back").clicked(){cancel=true;}});
  });
        if modal.should_close() {
            cancel = true;
        }
        if apply {
            self.enqueue(
                "invoke",
                json!({"command":review.command.name,"arguments":review.args}),
                review.title,
            );
            self.editor = None;
        } else if open && !cancel {
            self.review = Some(review);
        }
    }
    fn connection_window(&mut self, ctx: &egui::Context) {
        if !self.connections_open {
            return;
        }
        let mut open = true;
        let modal=egui::Modal::new(egui::Id::new("tenant_connection")).frame(card()).show(ctx,|ui|{ui.set_width(550.0);ui.horizontal(|ui|{ui.label(RichText::new("Microsoft 365 connection").size(22.0).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("Close").clicked(){open=false;}});});ui.add_space(12.0);
   egui::ScrollArea::vertical().max_height(630.0).show(ui,|ui|{
    if self.connected{ui.label(RichText::new(self.tenant()).size(20.0).strong());muted(ui,"Connected");let info=first(&self.connection["connections"]);if let Some(account)=info["UserPrincipalName"].as_str(){ui.label(format!("Signed in as {account}"));}
if let Some(customer)=info["DelegatedOrganization"].as_str().filter(|s|!s.is_empty()){ui.label(format!("Customer: {customer}"));}
if primary(ui,"Disconnect").clicked(){self.enqueue("disconnect",json!({}),"Disconnect");}ui.add_space(10.0);ui.label("Disconnect to switch organizations.");}
    else{
     ui.label(RichText::new("Choose your organization").size(20.0).strong());ui.add_space(10.0);
     ui.add_enabled_ui(!self.busy(),|ui|{
      ui.horizontal(|ui|{egui::ComboBox::from_id_salt("profiles").selected_text(&self.profiles[self.profile].label).show_ui(ui,|ui|{for(i,p)in self.profiles.iter().enumerate(){ui.selectable_value(&mut self.profile,i,&p.label);}});if ui.button("Add organization").clicked(){self.profiles.push(TenantProfile{label:"New organization".into(),..Default::default()});self.profile=self.profiles.len()-1;}});
      ui.label("Connection name");ui.add(egui::TextEdit::singleline(&mut self.profiles[self.profile].label).desired_width(f32::INFINITY));ui.label("Your account");ui.add(egui::TextEdit::singleline(&mut self.profiles[self.profile].upn).hint_text("you@company.com").desired_width(f32::INFINITY));
      ui.checkbox(&mut self.delegated,"I'm managing a customer organization");
      if self.delegated{ui.label("Customer domain");ui.add(egui::TextEdit::singleline(&mut self.profiles[self.profile].organization).hint_text("customer.onmicrosoft.com").desired_width(f32::INFINITY));muted(ui,"Uses your existing delegated access.");}
      ui.add_space(10.0);
      ui.horizontal(|ui|{if primary(ui,"Sign in").clicked(){let profile=self.profiles[self.profile].clone();if self.delegated&&profile.organization.trim().is_empty(){self.message=Some((false,"Enter the customer domain.".into()));}else{self.demo=false;self.clear();self.notices.clear();if self.bridge.is_none(){match Bridge::start(&self.pwsh){Ok(b)=>self.bridge=Some(b),Err(e)=>{self.details=Some(e);self.message=Some((false,"We couldn't start the Microsoft 365 connection. Open connection troubleshooting for help.".into()));}}}
if self.bridge.is_some(){self.enqueue("connect",json!({"upn":profile.upn,"organization":if self.delegated{profile.organization}else{String::new()},"device":self.device,"disable_wam":self.disable_wam}),"Sign in");}}}
       if ui.button("Save connection").clicked(){let path=profile_path();let result=(||->Result<(),String>{if let Some(parent)=path.parent(){std::fs::create_dir_all(parent).map_err(|e|e.to_string())?;}std::fs::write(&path,serde_json::to_vec_pretty(&self.profiles).map_err(|e|e.to_string())?).map_err(|e|e.to_string())})();self.message=Some(match result{Ok(())=>(true,"Connection saved.".into()),Err(e)=>{self.details=Some(e);(false,"We couldn't save this connection.".into())}});}
      });
     });
    }
    if self.busy(){ui.add_space(12.0);ui.horizontal(|ui|{ui.spinner();ui.label("Complete sign-in in your browser.");});if ui.button("Cancel connection").clicked(){if let Some(b)=self.bridge.as_mut(){b.stop();}self.bridge.take();self.connected=false;self.pending=None;self.queue.clear();self.clear();}}
    if !self.notices.is_empty(){ui.add_space(12.0);for line in &self.notices{ui.label(line);}
if self.device{ui.hyperlink_to("Open sign-in page","https://microsoft.com/devicelogin");}}
    ui.add_space(16.0);ui.collapsing("Connection troubleshooting",|ui|{
     ui.checkbox(&mut self.device,"Sign in with a device code instead");if cfg!(target_os="windows"){ui.checkbox(&mut self.disable_wam,"Use browser sign-in");}
     ui.label("PowerShell location");ui.add_enabled(self.bridge.is_none(),egui::TextEdit::singleline(&mut self.pwsh).desired_width(f32::INFINITY));
     if !self.runtime.is_empty(){muted(ui,&self.runtime);}ui.label("Requires PowerShell 7.4+ and the Exchange Online management module.");ui.hyperlink_to("Installation help","https://learn.microsoft.com/en-us/powershell/exchange/exchange-online-powershell-v2?view=exchange-ps");
     if let Some(details)=&self.details{ui.label(details);}
    });
   });
  });
        self.connections_open = open && !modal.should_close();
    }
    fn report_window(&mut self, ctx: &egui::Context) {
        let Some((title, data)) = self.report.take() else {
            return;
        };
        let mut open = true;
        egui::Window::new(&title)
            .open(&mut open)
            .default_size([880.0, 540.0])
            .show(ctx, |ui| {
                let objects = rows(&data);
                if objects.is_empty() {
                    ui.label("No matching results.");
                }
                let columns: BTreeSet<_> = objects
                    .iter()
                    .take(100)
                    .filter_map(Value::as_object)
                    .flat_map(|o| o.keys().cloned())
                    .collect();
                ui.horizontal(|ui| {
                    muted(ui, format!("{} results", objects.len()));
                    if ui.button("Copy table").clicked() {
                        let mut text = columns
                            .iter()
                            .map(|k| p::label(k))
                            .collect::<Vec<_>>()
                            .join("\t");
                        for row in &objects {
                            text.push('\n');
                            text.push_str(
                                &columns
                                    .iter()
                                    .map(|k| p::value(&row[k]).replace(['\n', '\t'], " "))
                                    .collect::<Vec<_>>()
                                    .join("\t"),
                            );
                        }
                        ctx.copy_text(text);
                    }
                });
                egui::ScrollArea::both().show(ui, |ui| {
                    if columns.is_empty() {
                        for row in &objects {
                            ui.label(p::value(row));
                        }
                    } else {
                        egui::Grid::new("report_grid")
                            .striped(true)
                            .max_col_width(260.0)
                            .show(ui, |ui| {
                                for k in &columns {
                                    ui.label(RichText::new(p::label(k)).strong());
                                }
                                ui.end_row();
                                for row in &objects {
                                    for k in &columns {
                                        ui.label(p::value(&row[k]));
                                    }
                                    ui.end_row();
                                }
                            });
                    }
                });
            });
        if open {
            self.report = Some((title, data));
        }
    }
    fn activity(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recent activity");
        muted(ui, "Changes and requests from this session.");
        ui.add_space(18.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.activity.is_empty() {
                muted(ui, "No activity yet.");
            }
            for a in self.activity.iter().rev() {
                card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        muted(ui, &a.time);
                        ui.label(&a.label);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            muted(ui, &a.result);
                        });
                    });
                });
                ui.add_space(6.0);
            }
        });
    }
}

impl eframe::App for CalendarAdmin {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.poll();
        ctx.request_repaint_after(Duration::from_millis(150));
        self.header(ctx);
        self.navigation(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(24))
            .show(ctx, |ui| {
                if let Some((ok, text)) = self.message.clone() {
                    egui::Frame::new()
                        .fill(if ok {
                            Color32::from_rgb(228, 245, 234)
                        } else {
                            Color32::from_rgb(255, 239, 232)
                        })
                        .inner_margin(11)
                        .corner_radius(5)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(text);
                                if !ok && ui.link("Get help").clicked() {
                                    self.connections_open = true;
                                }
                                if ui.small_button("Dismiss").clicked() {
                                    self.message = None;
                                }
                            });
                        });
                    ui.add_space(12.0);
                }
                match self.page {
                    Page::Mailboxes => self.mailboxes(ui),
                    Page::Organization => self.organization(ui),
                    Page::Activity => self.activity(ui),
                }
            });
        self.editor_window(ctx);
        self.review_window(ctx);
        self.report_window(ctx);
        self.connection_window(ctx);
    }
}

fn editor_title(name: &str, fallback: &str) -> String {
    match name {
        "Add-MailboxFolderPermission" => "Add people to this calendar",
        "Set-MailboxFolderPermission" => "Edit calendar access",
        "Remove-MailboxFolderPermission" => "Remove calendar access",
        "Set-MailboxCalendarConfiguration" => "Calendar preferences",
        "Set-MailboxRegionalConfiguration" => "Language & region",
        "Set-CalendarProcessing" => "Booking rules",
        "Set-MailboxCalendarFolder" => "Calendar publishing",
        "Set-EventsFromEmailConfiguration" => "Events from email",
        "Get-CalendarDiagnosticObjects" => "Find meeting activity",
        "Remove-CalendarEvents" => "Cancel upcoming meetings",
        "Invoke-CalendarEventMigration" => "Update event time zones",
        "Set-Mailbox" => "Mailbox calendar settings",
        "Get-MailboxFolder" => "Browse mailbox folders",
        "Get-MailboxFolderStatistics" => "Calendar folder report",
        "Get-EXOMailboxFolderStatistics" => "Calendar folders",
        _ => return fallback.into(),
    }
    .into()
}
fn paired_read(name: &str) -> Option<&'static str> {
    match name {
        "Set-MailboxCalendarConfiguration" => Some("Get-MailboxCalendarConfiguration"),
        "Set-MailboxRegionalConfiguration" => Some("Get-MailboxRegionalConfiguration"),
        "Set-CalendarProcessing" => Some("Get-CalendarProcessing"),
        "Set-MailboxCalendarFolder" => Some("Get-MailboxCalendarFolder"),
        "Set-EventsFromEmailConfiguration" => Some("Get-EventsFromEmailConfiguration"),
        "Set-Mailbox" => Some("Get-Mailbox"),
        "Set-Place" => Some("Get-Place"),
        "Set-ResourceConfig" => Some("Get-ResourceConfig"),
        "Set-OrganizationConfig" => Some("Get-OrganizationConfig"),
        "Set-SharingPolicy" | "Remove-SharingPolicy" => Some("Get-SharingPolicy"),
        "Set-OrganizationRelationship"
        | "Remove-OrganizationRelationship"
        | "Test-OrganizationRelationship" => Some("Get-OrganizationRelationship"),
        "Set-AvailabilityConfig" | "Remove-AvailabilityConfig" => Some("Get-AvailabilityConfig"),
        "Remove-AvailabilityAddressSpace" => Some("Get-AvailabilityAddressSpace"),
        _ => None,
    }
}
fn field_order(command: &str, name: &str) -> u8 {
    if name == "User" {
        return 0;
    }
    if name == "AccessRights" {
        return 1;
    }
    if name == "SharingPermissionFlags" {
        return 2;
    }
    if name == "SendNotificationToUser" {
        return 3;
    }
    let common = [
        "Name",
        "Identity",
        "Subject",
        "StartDate",
        "EndDate",
        "QueryStartDate",
        "QueryWindowInDays",
        "FromWindowsTimeZoneId",
        "ToWindowsTimeZoneId",
        "CancelOrganizedMeetings",
        "AutomateProcessing",
        "AllBookInPolicy",
        "BookingWindowInDays",
        "MaximumDurationInMinutes",
        "AllowRecurringMeetings",
        "AllowConflicts",
        "ResourceDelegates",
        "WorkingHoursStartTime",
        "WorkingHoursEndTime",
        "WorkingHoursTimeZone",
        "WorkDays",
        "DefaultReminderTime",
        "RemindersEnabled",
        "WeekStartDay",
        "TimeZone",
        "Language",
        "DateFormat",
        "TimeFormat",
        "PublishEnabled",
        "DetailLevel",
        "PublishDateRangeFrom",
        "PublishDateRangeTo",
        "SearchableUrlEnabled",
        "EventsFromEmailEnabled",
        "CreateEventsFromEmailAsPrivate",
        "Domains",
        "Enabled",
        "DomainNames",
        "FreeBusyAccessEnabled",
        "FreeBusyAccessLevel",
        "Capacity",
        "Building",
        "Floor",
        "Label",
        "SharingPolicy",
        "GrantSendOnBehalfTo",
    ];
    if let Some(i) = common.iter().position(|x| *x == name) {
        return (i + 4).min(49) as u8;
    }
    if command == "Set-Mailbox" && !name.to_lowercase().contains("calendar") {
        return 90;
    }
    60
}
fn friendly_validation(message: &str, command: &Command) -> String {
    let mut result = message.to_owned();
    for param in &command.parameters {
        result = result
            .replace(&catalog::humanize(&param.name), &p::label(&param.name))
            .replace(&param.name, &p::label(&param.name));
    }
    result.replace("Default", "this action")
}
fn display_field_value(key: &str, value: &Value, recipients: &[Mailbox]) -> String {
    if key == "AccessRights" {
        return rows(value)
            .iter()
            .map(|v| p::option(&p::value(v)))
            .collect::<Vec<_>>()
            .join(", ");
    }
    let raw = p::value(value);
    recipients
        .iter()
        .find(|m| m.email == raw)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| {
            if value.is_string() {
                p::option(&raw)
            } else {
                raw
            }
        })
}
fn render_field(
    ui: &mut egui::Ui,
    param: &catalog::Parameter,
    editor: &mut Editor,
    recipients: &[Mailbox],
    zones: &[(String, String)],
    cache: &BTreeMap<String, Value>,
) {
    let key = &param.name;
    ui.push_id(key,|ui|{
  ui.add_space(7.0);ui.label(RichText::new(p::label(key)).strong());
  let description=p::description(key);if !description.is_empty(){muted(ui,description);}
  if let Some(value)=editor.baseline.get(key).filter(|v|!v.is_null()) {if p::edit_value(key,value).is_none(){muted(ui,"This setting couldn't be read. Refresh before changing it.");return;}}
  let field=editor.fields.entry(key.clone()).or_default();
  if key=="User"||is_recipient(param){
   recipient_picker(ui,key,field,editor.recipient_query.entry(key.clone()).or_default(),recipients,param.multiple());
  }else if key=="AccessRights"{
   let current=field.text.lines().next().unwrap_or("Reviewer").to_owned();let mut value=current.clone();
   egui::ComboBox::from_id_salt("access_level").width(365.0).selected_text(p::option(&current)).show_ui(ui,|ui|{for option in ["AvailabilityOnly","LimitedDetails","Reviewer","Editor","Owner","Contributor","Author","NonEditingAuthor","PublishingAuthor","PublishingEditor","None"]{ui.selectable_value(&mut value,option.into(),p::option(option));}});
   if value!=current{field.text=value;field.enabled=true;field.clear=false;}
  }else if key=="SharingPermissionFlags"{
   let mut delegate=field.text.contains("Delegate");let mut private=field.text.contains("CanViewPrivateItems");let a=ui.checkbox(&mut delegate,"Let this person receive and respond to meeting requests").changed();let b=ui.add_enabled(delegate,egui::Checkbox::new(&mut private,"Let this person see private events")).changed();
   if a||b{field.text=if delegate{if private{"Delegate, CanViewPrivateItems"}else{"Delegate"}}else{"None"}.into();field.enabled=true;field.clear=false;}
  }else if p::is_clock_field(key){
   clock_picker(ui,key,field);
  }else if key=="WorkDays"{
   let previous=field.text.clone();let mut days:Vec<String>=p::working_days(&Value::String(previous.clone())).unwrap_or_default().into_iter().map(str::to_owned).collect();
   ui.horizontal_wrapped(|ui|{for day in ["Monday","Tuesday","Wednesday","Thursday","Friday","Saturday","Sunday"]{let mut selected=days.iter().any(|d|d==day);if ui.checkbox(&mut selected,&day[..3]).changed(){days.retain(|d|d!=day);if selected{days.push(day.into());}}}});
   let next=if days.is_empty(){"None".into()}else{days.join(", ")};if next!=previous{field.text=next;field.enabled=true;field.clear=false;}
  }else if key.contains("TimeZone")||key.contains("WindowsTimeZoneId"){
   let previous=field.text.clone();let title=zones.iter().find(|(value,_)|*value==previous).map(|(_,label)|label.to_owned()).unwrap_or_else(||if previous.is_empty(){"Choose a time zone".into()}else{previous.clone()});
   egui::ComboBox::from_id_salt("time_zone").width(365.0).selected_text(title).show_ui(ui,|ui|{let query=editor.list_inputs.entry(key.clone()).or_default();ui.add(egui::TextEdit::singleline(query).hint_text("Search time zones"));let query=query.to_lowercase();for(value,label)in zones{if query.is_empty()||label.to_lowercase().contains(&query)||value.to_lowercase().contains(&query){ui.selectable_value(&mut field.text,value.clone(),label);}}});
   if previous!=field.text{field.enabled=true;field.clear=false;}
  }else if key=="Identity"&&cache.values().any(|v|rows(v).iter().any(|r|r.get("Name").is_some())){
   let before=field.text.clone();let mut options=vec![];for data in cache.values(){for row in rows(data){if let Some(name)=row.get("Name"){options.push((p::value(name),row.get("Identity").map(p::value).unwrap_or_else(||p::value(name))));}}}options.sort();options.dedup();
   egui::ComboBox::from_id_salt("object_identity").selected_text(if field.text.is_empty(){"Choose an item"}else{&field.text}).width(365.0).show_ui(ui,|ui|{for(label,value)in options{ui.selectable_value(&mut field.text,value,label);}});if before!=field.text{field.enabled=true;}
  }else if param.boolean(){
   let mut value=field.text=="true";let toggle_label=if value{"On"}else{"Off"};if ui.checkbox(&mut value,toggle_label).changed(){field.text=value.to_string();field.enabled=true;field.clear=false;}
  }else if param.credential(){
   let a=ui.add(egui::TextEdit::singleline(&mut field.text).hint_text("Username").desired_width(f32::INFINITY)).changed();let b=ui.add(egui::TextEdit::singleline(&mut field.password).hint_text("Password").password(true).desired_width(f32::INFINITY)).changed();if a||b{field.enabled=true;field.clear=false;}
  }else if !param.values.is_empty()&&!param.multiple(){
   let previous=field.text.clone();egui::ComboBox::from_id_salt("choice").width(365.0).selected_text(if field.text.is_empty(){"Choose an option".into()}else if key=="AutomateProcessing"&&field.text=="None"{"Don't process bookings".into()}else{p::option(&field.text)}).show_ui(ui,|ui|{for value in &param.values{ui.selectable_value(&mut field.text,value.clone(),if key=="AutomateProcessing"&&value=="None"{"Don't process bookings".into()}else{p::option(value)});}});if previous!=field.text{field.enabled=true;field.clear=false;}
  }else if key=="AutomateProcessing"{
   let before=field.text.clone();egui::ComboBox::from_id_salt("booking_approval").selected_text(p::option(&field.text)).show_ui(ui,|ui|{for(value,label)in[("AutoAccept","Accept or decline automatically"),("AutoUpdate","Let someone respond to requests"),("None","Don't process bookings")]{ui.selectable_value(&mut field.text,value.to_owned(),label);}});if before!=field.text{field.enabled=true;field.clear=false;}
  }else if param.kind.contains("Hashtable"){
   let mut values:BTreeMap<String,String>=serde_json::from_str(&field.text).unwrap_or_default();let mut changed=false;let mut remove=None;
   for(key,value)in values.iter_mut(){ui.horizontal(|ui|{ui.label(key);if ui.text_edit_singleline(value).changed(){changed=true;}
if ui.small_button("Remove").clicked(){remove=Some(key.clone());}});}
if let Some(key)=remove{values.remove(&key);changed=true;}
   let input=editor.list_inputs.entry(key.clone()).or_default();ui.horizontal(|ui|{ui.add(egui::TextEdit::singleline(input).hint_text("New setting name"));if ui.button("Add").clicked()&&!input.trim().is_empty(){values.insert(input.trim().into(),String::new());input.clear();changed=true;}});
   if changed{field.text=serde_json::to_string(&values).unwrap_or_default();field.enabled=true;field.clear=false;}
  }else if key=="Domains"{
   let mut level=editor.list_inputs.get("domain-level").cloned().unwrap_or_else(||"CalendarSharingFreeBusySimple".into());
   domain_picker(ui,field,editor.list_inputs.entry(key.clone()).or_default(),&mut level);
   editor.list_inputs.insert("domain-level".into(),level);
  }else if param.multiple(){
   let mut values:Vec<String>=field.text.lines().filter(|s|!s.is_empty()).map(str::to_owned).collect();let mut remove=None;for(i,value)in values.iter().enumerate(){ui.horizontal_wrapped(|ui|{ui.label(value);if ui.small_button("Remove").clicked(){remove=Some(i);}});}
if let Some(i)=remove{values.remove(i);field.text=values.join("\n");field.enabled=true;}
   let input=editor.list_inputs.entry(key.clone()).or_default();ui.horizontal(|ui|{let response=ui.add(egui::TextEdit::singleline(input).hint_text(if key=="Domains"{"Domain and allowed sharing, e.g. example.com:CalendarSharingFreeBusySimple"}else{"Add a value"}).desired_width(360.0));if (ui.button("Add").clicked()||(response.lost_focus()&&ui.input(|i|i.key_pressed(egui::Key::Enter))))&&!input.trim().is_empty(){values.push(input.trim().into());field.text=values.join("\n");field.enabled=true;field.clear=false;input.clear();}});
  }else if param.kind.contains("Int")&&!param.kind.contains("IdParameter")&&!param.kind.contains("Interval"){
   let mut n=field.text.parse::<i64>().unwrap_or_default();if ui.add(egui::DragValue::new(&mut n).speed(1).range(0..=i32::MAX as i64)).changed(){field.text=n.to_string();field.enabled=true;field.clear=false;}
  }else{
   let hint=if key.contains("Date"){ "Choose a date (YYYY-MM-DD)"}else if key.contains("Time"){"e.g. 09:00"}else if key=="Subject"{"Search by meeting title"}else{"Enter a value"};
   let edit=if key.contains("Response")||key.contains("Description"){egui::TextEdit::multiline(&mut field.text).desired_rows(3)}else{egui::TextEdit::singleline(&mut field.text)};
   if ui.add(edit.hint_text(hint).password(param.secret()).desired_width(f32::INFINITY)).changed(){field.enabled=true;field.clear=false;}
  }
  if editor.baseline.get(key).is_some()&&!param.required_in(&editor.set)&&!matches!(key.as_str(),"User"|"AccessRights"|"SharingPermissionFlags")
   && ui.small_button("Reset to default").clicked(){field.enabled=true;field.clear=true;field.text.clear();}
  ui.add_space(4.0);
 });
}

fn is_recipient(param: &catalog::Parameter) -> bool {
    param.kind.contains("RecipientIdParameter")
        || matches!(
            param.name.as_str(),
            "ResourceDelegates"
                | "GrantSendOnBehalfTo"
                | "BookInPolicy"
                | "RequestInPolicy"
                | "RequestOutOfPolicy"
        )
}

fn calendar_field(command: &str, name: &str) -> bool {
    if matches!(name, "Identity" | "Confirm" | "WhatIf") {
        return true;
    }
    let lower = name.to_lowercase();
    match command {
        "Set-Mailbox" => {
            lower.contains("calendar")
                || matches!(
                    name,
                    "SharingPolicy" | "GrantSendOnBehalfTo" | "MessageCopyForSendOnBehalfEnabled"
                )
        }
        "Set-OrganizationConfig" => [
            "calendar",
            "meeting",
            "booking",
            "room",
            "event",
            "workday",
            "workinghours",
        ]
        .iter()
        .any(|part| lower.contains(part)),
        _ => true,
    }
}

fn normalize_access(editor: &mut Editor) {
    let flags = editor
        .fields
        .get("SharingPermissionFlags")
        .cloned()
        .unwrap_or_default();
    let rights = editor
        .fields
        .get("AccessRights")
        .cloned()
        .unwrap_or_default();
    if flags.enabled && flags.text.contains("Delegate") {
        if let Some(field) = editor.fields.get_mut("AccessRights") {
            field.enabled = true;
            field.text = "Editor".into();
        }
    } else if rights.enabled && rights.text != "Editor" && flags.text.contains("Delegate") {
        if let Some(field) = editor.fields.get_mut("SharingPermissionFlags") {
            field.enabled = true;
            field.text = "None".into();
        }
    }
    let role = editor
        .fields
        .get("AccessRights")
        .map(|f| f.text.as_str())
        .unwrap_or("");
    if !matches!(
        role,
        "AvailabilityOnly" | "LimitedDetails" | "Reviewer" | "Editor"
    ) {
        if let Some(field) = editor.fields.get_mut("SendNotificationToUser") {
            field.enabled = true;
            field.text = "false".into();
        }
    }
}
fn clock_picker(ui: &mut egui::Ui, key: &str, field: &mut Field) {
    use chrono::Timelike;
    let Some(time) = p::clock(&Value::String(field.text.clone())) else {
        if ui.button("Choose a time").clicked() {
            field.text = if key == "WorkingHoursEndTime" {
                "17:00:00"
            } else {
                "09:00:00"
            }
            .into();
            field.enabled = true;
            field.clear = false;
        }
        return;
    };
    let mut hour = match time.hour() % 12 {
        0 => 12,
        h => h,
    };
    let mut minute = time.minute();
    let mut afternoon = time.hour() >= 12;
    let before = (hour, minute, afternoon);
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("hour")
            .width(60.0)
            .selected_text(hour.to_string())
            .show_ui(ui, |ui| {
                for h in 1..=12 {
                    ui.selectable_value(&mut hour, h, h.to_string());
                }
            });
        ui.label(":");
        egui::ComboBox::from_id_salt("minute")
            .width(60.0)
            .selected_text(format!("{minute:02}"))
            .show_ui(ui, |ui| {
                for m in 0..60 {
                    ui.selectable_value(&mut minute, m, format!("{m:02}"));
                }
            });
        egui::ComboBox::from_id_salt("period")
            .width(65.0)
            .selected_text(if afternoon { "PM" } else { "AM" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut afternoon, false, "AM");
                ui.selectable_value(&mut afternoon, true, "PM");
            });
    });
    if before != (hour, minute, afternoon) {
        let hour = hour % 12 + if afternoon { 12 } else { 0 };
        field.text = format!("{hour:02}:{minute:02}:00");
        field.enabled = true;
        field.clear = false;
    }
}

fn recipient_row(ui: &mut egui::Ui, recipient: &Mailbox, highlighted: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
    if highlighted || response.hovered() {
        ui.painter().rect_filled(rect, 4, TINT);
    }
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(10.0, 4.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.add(egui::Label::new(RichText::new(&recipient.name).strong().size(14.0)).truncate());
            ui.add(
                egui::Label::new(RichText::new(&recipient.email).color(MUTED).size(12.0))
                    .truncate(),
            );
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            format!("{} ({})", recipient.name, recipient.email),
        )
    });
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn recipient_matches<'a>(
    recipients: &'a [Mailbox],
    query: &str,
    selected: &[String],
) -> Vec<&'a Mailbox> {
    let query = query.trim().to_lowercase();
    recipients
        .iter()
        .filter(|m| {
            !m.email.is_empty()
                && !selected.contains(&m.email)
                && (query.is_empty()
                    || m.name.to_lowercase().contains(&query)
                    || m.email.to_lowercase().contains(&query))
        })
        .take(if query.is_empty() { 6 } else { 12 })
        .collect()
}

fn recipient_picker(
    ui: &mut egui::Ui,
    key: &str,
    field: &mut Field,
    query: &mut String,
    recipients: &[Mailbox],
    multiple: bool,
) {
    let selected: Vec<String> = field
        .text
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .collect();
    if !multiple && matches!(field.text.as_str(), "Default" | "Anonymous") {
        ui.label(p::option(&field.text));
        return;
    }
    let mut remove = None;
    for (i, value) in selected.iter().enumerate() {
        ui.horizontal_wrapped(|ui| {
            let label = recipients
                .iter()
                .find(|m| m.email == *value || m.name == *value)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| p::option(value));
            egui::Frame::new()
                .fill(TINT)
                .corner_radius(5)
                .inner_margin(8)
                .show(ui, |ui| {
                    ui.label(RichText::new(label).color(BLUE));
                });
            if ui
                .small_button(if multiple { "Remove" } else { "Change" })
                .clicked()
            {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        let mut values = selected.clone();
        values.remove(i);
        field.text = values.join("\n");
        field.enabled = true;
        field.clear = values.is_empty();
    }
    if !multiple && !field.text.is_empty() {
        return;
    }
    let popup_id = ui.make_persistent_id(("directory_picker", key));
    let cursor_id = popup_id.with("cursor");
    let was_open = ui.memory(|memory| memory.is_popup_open(popup_id));
    let down =
        was_open && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
    let up = was_open && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
    let enter =
        was_open && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    let response = ui.add(
        egui::TextEdit::singleline(query)
            .hint_text("Search people or groups…")
            .desired_width(ui.available_width().min(420.0)),
    );
    if response.clicked() || response.changed() {
        ui.memory_mut(|m| m.open_popup(popup_id));
    }
    let matches = recipient_matches(recipients, query, &selected);
    let mut cursor = ui
        .data(|d| d.get_temp::<usize>(cursor_id))
        .unwrap_or(0)
        .min(matches.len().saturating_sub(1));
    if response.changed() {
        cursor = 0;
    } else if down {
        cursor = (cursor + 1).min(matches.len().saturating_sub(1));
    } else if up {
        cursor = cursor.saturating_sub(1);
    }
    let mut chosen = if enter {
        matches.get(cursor).map(|m| m.email.clone())
    } else {
        None
    };
    egui::popup_below_widget(
        ui,
        popup_id,
        &response,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            egui::ScrollArea::vertical()
                .max_height(224.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (i, recipient) in matches.iter().enumerate() {
                        let row = recipient_row(ui, recipient, i == cursor);
                        if row.clicked() {
                            chosen = Some(recipient.email.clone());
                        }
                        if row.hovered() {
                            cursor = i;
                        }
                        if (down || up) && i == cursor {
                            row.scroll_to_me(Some(egui::Align::Center));
                        }
                    }
                    if matches.is_empty() {
                        muted(ui, "No matching people or groups.");
                    }
                });
        },
    );
    ui.data_mut(|d| d.insert_temp(cursor_id, cursor));
    if let Some(value) = chosen {
        let mut values = if multiple { selected } else { vec![] };
        values.push(value);
        field.text = values.join("\n");
        field.enabled = true;
        field.clear = false;
        query.clear();
        ui.memory_mut(|m| m.close_popup());
        response.surrender_focus();
    }
}

fn time_zones() -> Vec<(&'static str, &'static str)> {
    vec![
        ("UTC", "Coordinated Universal Time"),
        ("Hawaiian Standard Time", "Hawaii"),
        ("Alaskan Standard Time", "Alaska"),
        ("Pacific Standard Time", "Pacific Time (US & Canada)"),
        ("US Mountain Standard Time", "Arizona"),
        ("Mountain Standard Time", "Mountain Time (US & Canada)"),
        ("Central Standard Time", "Central Time (US & Canada)"),
        ("Eastern Standard Time", "Eastern Time (US & Canada)"),
        ("Atlantic Standard Time", "Atlantic Time (Canada)"),
        ("Newfoundland Standard Time", "Newfoundland"),
        ("SA Eastern Standard Time", "Cayenne, Fortaleza"),
        ("E. South America Standard Time", "Brasilia"),
        ("Argentina Standard Time", "Buenos Aires"),
        ("GMT Standard Time", "Dublin, Edinburgh, Lisbon, London"),
        ("W. Europe Standard Time", "Amsterdam, Berlin, Rome, Vienna"),
        (
            "Romance Standard Time",
            "Brussels, Copenhagen, Madrid, Paris",
        ),
        ("South Africa Standard Time", "Harare, Pretoria"),
        ("FLE Standard Time", "Helsinki, Kyiv, Riga, Sofia"),
        ("Israel Standard Time", "Jerusalem"),
        ("Arab Standard Time", "Kuwait, Riyadh"),
        ("Russian Standard Time", "Moscow, St. Petersburg"),
        ("Arabian Standard Time", "Abu Dhabi, Muscat"),
        ("Pakistan Standard Time", "Islamabad, Karachi"),
        ("India Standard Time", "Chennai, Kolkata, Mumbai, New Delhi"),
        ("Bangladesh Standard Time", "Dhaka"),
        ("SE Asia Standard Time", "Bangkok, Hanoi, Jakarta"),
        ("China Standard Time", "Beijing, Hong Kong"),
        ("Singapore Standard Time", "Kuala Lumpur, Singapore"),
        ("Tokyo Standard Time", "Osaka, Sapporo, Tokyo"),
        ("Korea Standard Time", "Seoul"),
        ("AUS Central Standard Time", "Darwin"),
        ("AUS Eastern Standard Time", "Canberra, Melbourne, Sydney"),
        ("E. Australia Standard Time", "Brisbane"),
        ("New Zealand Standard Time", "Auckland, Wellington"),
    ]
}
fn install_font(ctx: &egui::Context) {
    #[cfg(target_os = "macos")]
    let path = PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf");
    #[cfg(target_os = "windows")]
    let path = PathBuf::from(std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into()))
        .join("Fonts/segoeui.ttf");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let path = PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    if let Ok(bytes) = std::fs::read(path) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "system-ui".into(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system-ui".into());
        ctx.set_fonts(fonts);
    }
}
fn default_pwsh() -> String {
    if let Ok(path) = std::env::var("CAC_PWSH") {
        return path;
    }
    #[cfg(target_os = "macos")]
    for path in ["/opt/homebrew/bin/pwsh", "/usr/local/bin/pwsh"] {
        if std::path::Path::new(path).exists() {
            return path.into();
        }
    }
    #[cfg(target_os = "windows")]
    if let Some(root) = std::env::var_os("ProgramFiles") {
        let path = PathBuf::from(root).join("PowerShell/7/pwsh.exe");
        if path.exists() {
            return path.to_string_lossy().into_owned();
        }
    }
    "pwsh".into()
}
fn profile_path() -> PathBuf {
    if let Some(path) = std::env::var_os("CAC_CONFIG_DIR") {
        return PathBuf::from(path).join("connections.json");
    }
    #[cfg(target_os = "macos")]
    let root = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Library/Application Support");
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let root = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join(".config")
        });
    root.join("calendar-admin-center/connections.json")
}

fn domain_picker(ui: &mut egui::Ui, field: &mut Field, input: &mut String, level: &mut String) {
    let choices = [
        ("CalendarSharingFreeBusySimple", "Availability only"),
        ("CalendarSharingFreeBusyDetail", "Titles and locations"),
        ("CalendarSharingFreeBusyReviewer", "All event details"),
    ];
    let mut values: Vec<String> = field
        .text
        .lines()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let mut remove = None;
    for (i, value) in values.iter().enumerate() {
        let (domain, action) = value.split_once(':').unwrap_or((value, ""));
        let label = choices
            .iter()
            .find(|(v, _)| *v == action)
            .map(|(_, l)| *l)
            .unwrap_or("Custom sharing level");
        ui.horizontal_wrapped(|ui| {
            ui.label(if domain == "*" {
                "All external domains"
            } else {
                domain
            });
            muted(ui, label);
            if ui.small_button("Remove").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        values.remove(i);
        field.text = values.join("\n");
        field.enabled = true;
        field.clear = values.is_empty();
    }
    ui.horizontal_wrapped(|ui| {
        ui.add(
            egui::TextEdit::singleline(input)
                .hint_text("Domain, e.g. example.com (or * for all)")
                .desired_width(260.0),
        );
        egui::ComboBox::from_id_salt("sharing_level")
            .selected_text(
                choices
                    .iter()
                    .find(|(v, _)| *v == level)
                    .map(|(_, l)| *l)
                    .unwrap_or("Availability only"),
            )
            .show_ui(ui, |ui| {
                for (value, label) in choices {
                    ui.selectable_value(level, value.into(), label);
                }
            });
        if ui.button("Add domain").clicked() && !input.trim().is_empty() {
            values.push(format!("{}:{}", input.trim(), level));
            field.text = values.join("\n");
            field.enabled = true;
            field.clear = false;
            input.clear();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> CalendarAdmin {
        CalendarAdmin::with_context(&egui::Context::default(), true)
    }
    #[test]
    fn access_selection_targets_the_current_calendar() {
        let mut app = app();
        app.select_mailbox(0);
        app.folder = Some(1);
        app.open_editor("Add-MailboxFolderPermission", None);
        let mut editor = app.editor.take().unwrap();
        editor.fields.get_mut("User").unwrap().text = "alex@contoso.example".into();
        editor.fields.get_mut("AccessRights").unwrap().text = "Editor".into();
        let args =
            catalog::collect(&app.catalog[editor.index], &editor.set, &editor.fields).unwrap();
        assert_eq!(args["Identity"], "adele@contoso.example:\\Team projects");
        assert_eq!(args["User"], "alex@contoso.example");
        assert_eq!(args["AccessRights"], json!(["Editor"]));
    }
    #[test]
    fn delegate_options_apply_a_compatible_access_level() {
        let mut app = app();
        app.select_mailbox(0);
        app.open_editor("Add-MailboxFolderPermission", None);
        let mut editor = app.editor.take().unwrap();
        let flags = editor.fields.get_mut("SharingPermissionFlags").unwrap();
        flags.enabled = true;
        flags.text = "Delegate, CanViewPrivateItems".into();
        normalize_access(&mut editor);
        assert_eq!(editor.fields["AccessRights"].text, "Editor");
        assert!(editor.fields["AccessRights"].enabled);
    }
    #[test]
    fn optional_existing_settings_are_saved_only_when_changed() {
        let mut app = app();
        app.select_mailbox(0);
        app.open_editor("Set-CalendarProcessing", None);
        let editor = app.editor.take().unwrap();
        let args =
            catalog::collect(&app.catalog[editor.index], &editor.set, &editor.fields).unwrap();
        assert_eq!(args.len(), 1);
        assert!(args.contains_key("Identity"));
    }
    #[test]
    fn stale_mailbox_reads_cannot_replace_current_settings() {
        let mut app = app();
        app.select_mailbox(0);
        let scope = app.scope_key();
        app.select_mailbox(1);
        app.cache.clear();
        app.pending = Some(Pending {
            id: 1,
            job: Job {
                action: "invoke".into(),
                payload: json!({"command":"Get-MailboxCalendarConfiguration","scope":scope}),
                label: "Load calendar preferences".into(),
            },
        });
        app.response(Response {
            id: 1,
            ok: true,
            data: json!([{"TimeZone":"WRONG"}]),
            message: String::new(),
        });
        assert!(app.cache.is_empty());
    }
    #[test]
    fn disconnect_clears_all_tenant_specific_data() {
        let mut app = app();
        app.select_mailbox(0);
        app.open_editor("Add-MailboxFolderPermission", None);
        app.schemas.insert("test".into());
        app.clear();
        assert!(app.mailboxes.is_empty());
        assert!(app.recipients.is_empty());
        assert!(app.cache.is_empty());
        assert!(app.editor.is_none());
        assert!(app.schemas.is_empty());
        assert!(app.folder.is_none());
    }
    #[test]
    fn each_online_operation_has_a_workflow() {
        let source = include_str!("app.rs").to_owned() + include_str!("powershell.rs");
        for command in catalog::load().iter().filter(|c| c.unavailable.is_none()) {
            assert!(
                source.contains(&command.name),
                "No workflow for {}",
                command.name
            );
        }
    }
    fn text(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(s) => {
                out.push_str(&s.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    text(s, out)
                }
            }
            _ => {}
        }
    }
    #[test]
    fn access_dialog_uses_plain_language_and_hides_the_command_interface() {
        let ctx = egui::Context::default();
        let mut app = CalendarAdmin::with_context(&ctx, true);
        app.select_mailbox(0);
        app.open_editor("Add-MailboxFolderPermission", None);
        let mut content = String::new();
        for _ in 0..2 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1440.0, 940.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    app.editor_window(ctx);
                },
            );
            for shape in output.shapes {
                text(&shape.shape, &mut content);
            }
        }
        assert!(content.contains("Person or group"));
        assert!(content.contains("Access level"));
        assert!(content.contains("Can view all details"));
        for technical in [
            "WhatIf",
            "JSON",
            "Identity",
            "AccessRights",
            "MailboxFolder",
            "documented settings",
            "Parameter set",
        ] {
            assert!(
                !content.contains(technical),
                "Technical detail leaked into UI: {technical}"
            );
        }
    }
    fn rendered(app: &mut CalendarAdmin, ctx: &egui::Context, edit: bool) -> String {
        let mut content = String::new();
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1440.0, 940.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    if edit {
                        app.editor_window(ctx);
                    } else {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            app.settings_card(
                                ui,
                                "Working hours",
                                "Get-MailboxCalendarConfiguration",
                                "Set-MailboxCalendarConfiguration",
                                &[
                                    "WorkDays",
                                    "WorkingHoursStartTime",
                                    "WorkingHoursEndTime",
                                    "WorkingHoursTimeZone",
                                ],
                            );
                        });
                    }
                },
            );
            for shape in output.shapes {
                text(&shape.shape, &mut content);
            }
        }
        content
    }
    #[test]
    fn settings_and_editor_handle_real_exchange_time_objects_without_dirtying_them() {
        let ctx = egui::Context::default();
        let mut app = CalendarAdmin::with_context(&ctx, true);
        app.select_mailbox(0);
        let view = rendered(&mut app, &ctx, false);
        for expected in [
            "8:00 AM",
            "5:00 PM",
            "Monday–Friday",
            "Pacific Time (US & Canada)",
        ] {
            assert!(view.contains(expected), "Missing {expected}");
        }
        for detail in [
            "Ticks",
            "Nanoseconds",
            "Total hours",
            "Milliseconds",
            "Days: 0",
        ] {
            assert!(!view.contains(detail));
        }
        app.open_editor("Set-MailboxCalendarConfiguration", None);
        assert_eq!(
            app.editor.as_ref().unwrap().fields["WorkingHoursStartTime"].text,
            "08:00:00"
        );
        assert_eq!(
            app.editor.as_ref().unwrap().fields["WorkingHoursEndTime"].text,
            "17:00:00"
        );
        assert_eq!(
            app.editor.as_ref().unwrap().fields["DefaultReminderTime"].text,
            "15"
        );
        let edit = rendered(&mut app, &ctx, true);
        for detail in [
            "Ticks",
            "Nanoseconds",
            "Total hours",
            "Milliseconds",
            "Days: 0",
        ] {
            assert!(!edit.contains(detail));
        }
        let editor = app.editor.as_ref().unwrap();
        let args =
            catalog::collect(&app.catalog[editor.index], &editor.set, &editor.fields).unwrap();
        assert_eq!(
            args.len(),
            1,
            "Opening the editor must not submit unchanged working hours"
        );
    }
    #[test]
    fn live_parameter_sets_do_not_add_a_name_dropdown_to_the_access_dialog() {
        let ctx = egui::Context::default();
        let mut app = CalendarAdmin::with_context(&ctx, true);
        app.select_mailbox(0);
        let index = app
            .catalog
            .iter()
            .position(|c| c.name == "Add-MailboxFolderPermission")
            .unwrap();
        app.catalog[index]
            .parameters
            .iter_mut()
            .find(|p| p.name == "Identity")
            .unwrap()
            .sets = vec!["Identity".into(), "Instance".into()];
        app.open_editor("Add-MailboxFolderPermission", None);
        let content = rendered(&mut app, &ctx, true);
        assert!(content.contains("Person or group"));
        for internal in ["Name", "Instance", "Identity", "Parameter set"] {
            assert!(
                !content.lines().any(|line| line == internal),
                "Exposed an internal command option: {internal}"
            );
        }
    }
    #[test]
    fn directory_rows_are_compact_and_left_aligned() {
        let ctx = egui::Context::default();
        let person = Mailbox {
            name: "Example Person".into(),
            email: "example.person@company.example".into(),
            ..Default::default()
        };
        let mut row_rect = egui::Rect::NOTHING;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0, 400.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.set_width(420.0);
                    row_rect = recipient_row(ui, &person, false).rect;
                });
            },
        );
        assert_eq!(row_rect.height(), 44.0);
        let mut positions = vec![];
        for shape in output.shapes {
            if let egui::Shape::Text(shape) = shape.shape {
                if shape.galley.job.text == person.name || shape.galley.job.text == person.email {
                    positions.push(shape.pos.x);
                }
            }
        }
        assert_eq!(positions.len(), 2);
        for x in positions {
            assert!(
                (x - row_rect.left() - 10.0).abs() < 1.0,
                "Directory text drifted away from the left edge"
            );
        }
    }
    #[test]
    fn directory_search_supports_filtering_and_keyboard_selection_without_special_access_entries() {
        let ctx = egui::Context::default();
        let people = vec![
            Mailbox {
                name: "Ada Example".into(),
                email: "ada@company.example".into(),
                ..Default::default()
            },
            Mailbox {
                name: "Adam Example".into(),
                email: "adam@company.example".into(),
                ..Default::default()
            },
            Mailbox {
                name: "Other Person".into(),
                email: "other@company.example".into(),
                ..Default::default()
            },
        ];
        let mut query = "ad".to_owned();
        let mut field = Field::default();
        let mut content = String::new();
        for frame in 0..4 {
            let key = match frame {
                2 => Some(egui::Key::ArrowDown),
                3 => Some(egui::Key::Enter),
                _ => None,
            };
            let events = key
                .map(|key| {
                    vec![egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }]
                })
                .unwrap_or_default();
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000.0, 700.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if frame == 0 {
                            let id = ui.make_persistent_id(("directory_picker", "User"));
                            ui.memory_mut(|memory| memory.open_popup(id));
                        }
                        recipient_picker(ui, "User", &mut field, &mut query, &people, false);
                    });
                },
            );
            for shape in output.shapes {
                text(&shape.shape, &mut content);
            }
        }
        assert_eq!(field.text, "adam@company.example");
        assert!(field.enabled);
        assert!(query.is_empty());
        assert!(content.contains("Ada Example"));
        assert!(content.contains("Adam Example"));
        assert!(!content.contains("Other Person"));
        assert!(!content.contains("Everyone in your organization"));
        assert!(!content.contains("People outside your organization"));
        assert_eq!(
            recipient_matches(&people, "ADAM@COMPANY", &[])[0].name,
            "Adam Example"
        );
    }
    #[test]
    fn working_hours_values_stay_in_a_single_aligned_column() {
        let ctx = egui::Context::default();
        let mut app = CalendarAdmin::with_context(&ctx, true);
        app.select_mailbox(0);
        let mut positions = vec![];
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1440.0, 940.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        app.settings_card(
                            ui,
                            "Working hours",
                            "Get-MailboxCalendarConfiguration",
                            "Set-MailboxCalendarConfiguration",
                            &[
                                "WorkDays",
                                "WorkingHoursStartTime",
                                "WorkingHoursEndTime",
                                "WorkingHoursTimeZone",
                            ],
                        );
                    });
                },
            );
            positions.clear();
            for shape in output.shapes {
                if let egui::Shape::Text(shape) = shape.shape {
                    if [
                        "Monday–Friday",
                        "8:00 AM",
                        "5:00 PM",
                        "Pacific Time (US & Canada)",
                    ]
                    .contains(&shape.galley.job.text.as_str())
                    {
                        positions.push(shape.pos.x);
                    }
                }
            }
        }
        assert_eq!(positions.len(), 4);
        for x in &positions {
            assert!(
                (x - positions[0]).abs() < 1.0,
                "Settings values lost their shared alignment"
            );
        }
    }
    #[test]
    fn connected_empty_loading_and_failed_directories_never_show_the_sign_in_welcome() {
        let ctx = egui::Context::default();
        let mut app = CalendarAdmin::with_context(&ctx, false);
        app.connected = true;
        for (state, expected) in [
            (0, "Loading your mailboxes"),
            (1, "No mailboxes were returned"),
            (2, "We couldn't load your mailboxes"),
        ] {
            app.queue.clear();
            app.directory_loaded = state != 0;
            app.directory_error = (state == 2).then_some("Test failure".into());
            if state == 0 {
                app.enqueue("directory", json!({}), "Load mailboxes");
            }
            let mut content = String::new();
            for _ in 0..2 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1440.0, 940.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| app.mailboxes(ui));
                    },
                );
                for shape in output.shapes {
                    text(&shape.shape, &mut content);
                }
            }
            assert!(content.contains(expected));
            assert!(!content.contains("Explore the demo"));
            assert!(!content.contains("Connect Microsoft 365"));
        }
    }
    #[test]
    fn mailbox_parse_failures_are_visible_and_do_not_disconnect_the_user() {
        let mut app = app();
        app.demo = false;
        app.connected = true;
        app.mailboxes.clear();
        app.pending = Some(Pending {
            id: 1,
            job: Job {
                action: "directory".into(),
                payload: json!({}),
                label: "Load mailboxes".into(),
            },
        });
        app.response(Response {
            id: 1,
            ok: true,
            data: json!({"unexpected":true}),
            message: String::new(),
        });
        assert!(app.connected);
        assert!(app.directory_error.is_some());
        assert!(app.directory_loaded);
        app.enqueue("directory", json!({}), "Load mailboxes");
        assert!(app.directory_error.is_none());
        assert!(!app.directory_loaded);
    }
}
