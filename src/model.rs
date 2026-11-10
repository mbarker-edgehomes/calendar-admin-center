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
    #[serde(rename = "ExternalDirectoryObjectId")]
    pub id: String,
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
