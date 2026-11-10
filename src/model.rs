use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Mailbox {
    #[serde(rename = "DisplayName")]
    pub name: String,
    #[serde(rename = "PrimarySmtpAddress")]
    pub email: String,
    #[serde(rename = "RecipientTypeDetails")]
    pub kind: String,
    #[serde(
        rename = "ExternalDirectoryObjectId",
        deserialize_with = "nullable_string"
    )]
    pub id: String,
}

fn nullable_string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

pub fn directory_rows(value: &Value) -> Result<Vec<Mailbox>, String> {
    fn unwrap(value: &Value, depth: usize) -> Result<Vec<Mailbox>, String> {
        if depth > 4 {
            return Err("The directory response could not be read.".into());
        }
        match value {
            Value::Null => Ok(vec![]),
            Value::Array(_) => serde_json::from_value(value.clone()).map_err(|_| {
                "Microsoft 365 returned a directory record the app couldn't read.".into()
            }),
            Value::Object(object) => {
                for key in [
                    "mailboxes",
                    "recipients",
                    "rows",
                    "value",
                    "Value",
                    "Results",
                    "Items",
                    "Data",
                ] {
                    if let Some(value) = object.get(key) {
                        return unwrap(value, depth + 1);
                    }
                }
                if object.contains_key("PrimarySmtpAddress") {
                    return serde_json::from_value(value.clone())
                        .map(|mailbox| vec![mailbox])
                        .map_err(|_| "The directory record could not be read.".into());
                }
                Err("Microsoft 365 returned an unexpected directory response.".into())
            }
            Value::String(text) => serde_json::from_str::<Value>(text)
                .map_err(|_| "The directory response could not be read.".to_owned())
                .and_then(|value| unwrap(&value, depth + 1)),
            _ => Err("Microsoft 365 returned an unexpected directory response.".into()),
        }
    }
    unwrap(value, 0)
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CalendarFolder {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "FolderPath")]
    pub path: String,
    #[serde(rename = "FolderType")]
    pub kind: String,
    #[serde(rename = "ItemsInFolder")]
    pub count: Value,
    #[serde(rename = "FolderSize")]
    pub size: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TenantProfile {
    pub label: String,
    pub organization: String,
    pub upn: String,
}

pub fn demo_mailboxes() -> Vec<Mailbox> {
    [
        ("Adele Vance", "adele", "UserMailbox"),
        ("Alex Wilber", "alex", "UserMailbox"),
        ("Boardroom · Denver", "boardroom", "RoomMailbox"),
        ("Design studio", "studio", "RoomMailbox"),
        ("Equipment · Camera kit", "camera", "EquipmentMailbox"),
        ("Marketing team", "marketing", "SharedMailbox"),
        ("Megan Bowen", "megan", "UserMailbox"),
        ("Operations", "operations", "SharedMailbox"),
    ]
    .into_iter()
    .map(|(name, email, kind)| Mailbox {
        name: name.into(),
        email: format!("{email}@contoso.example"),
        kind: kind.into(),
        id: email.into(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn directory_decoding_accepts_envelopes_single_rows_and_nullable_ids() {
        let record = json!({"DisplayName":"Example Person","PrimarySmtpAddress":"person@company.example","RecipientTypeDetails":"UserMailbox","ExternalDirectoryObjectId":null});
        for value in [
            record.clone(),
            json!([record.clone()]),
            json!({"mailboxes":[record.clone()]}),
            json!({"value":[record.clone()],"Count":1}),
            json!({"recipients":[record.clone()]}),
        ] {
            let rows = directory_rows(&value).unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].email, "person@company.example");
            assert_eq!(rows[0].id, "");
        }
        assert!(directory_rows(&json!({"mailboxes":[]})).unwrap().is_empty());
        assert!(directory_rows(&json!({"unexpected":true})).is_err());
        assert!(directory_rows(&json!([{"DisplayName":false}])).is_err());
    }
}
