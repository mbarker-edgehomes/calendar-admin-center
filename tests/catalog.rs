use calendar_admin_center::catalog::{self, Field};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn every_calendar_reference_entry_is_classified() {
    let baseline: Vec<String> =
        serde_json::from_str(include_str!("../data/source-calendar-cmdlets.json")).unwrap();
    let catalog = catalog::load();
    for name in baseline {
        assert!(
            catalog.iter().any(|c| c.name == name),
            "Unclassified calendar command: {name}"
        );
    }
    let names: BTreeSet<_> = catalog.iter().map(|c| &c.name).collect();
    assert_eq!(names.len(), catalog.len());
    for command in &catalog {
        assert!(
            !command.parameters.is_empty()
                || matches!(
                    command.name.as_str(),
                    "Get-ResourceConfig" | "Get-OrganizationConfig"
                )
        );
        assert!(command.url.starts_with("https://learn.microsoft.com/"));
        let names: BTreeSet<_> = command.parameters.iter().map(|p| &p.name).collect();
        assert_eq!(
            names.len(),
            command.parameters.len(),
            "Duplicate parameter in {}",
            command.name
        );
    }
}

#[test]
fn on_premises_and_discontinued_commands_are_excluded() {
    let catalog = catalog::load();
    for name in [
        "Get-CalendarDiagnosticAnalysis",
        "Get-CalendarDiagnosticLog",
        "Test-CalendarConnectivity",
        "Get-CalendarNotification",
        "Set-CalendarNotification",
    ] {
        assert!(catalog
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .unavailable
            .is_some());
    }
    assert!(catalog
        .iter()
        .find(|c| c.name == "Invoke-CalendarEventMigration")
        .unwrap()
        .unavailable
        .is_none());
}

#[test]
fn folder_identity_preserves_localized_paths_and_apostrophes() {
    assert_eq!(
        catalog::folder_identity("o'connor@example.com", "/Kalender/Team A"),
        "o'connor@example.com:\\Kalender\\Team A"
    );
}

#[test]
fn absent_false_clear_and_arrays_are_distinct() {
    let command = catalog::load()
        .into_iter()
        .find(|c| c.name == "Set-CalendarProcessing")
        .unwrap();
    let mut fields = BTreeMap::new();
    fields.insert(
        "Identity".into(),
        Field {
            enabled: true,
            text: "room@example.com".into(),
            ..Default::default()
        },
    );
    fields.insert(
        "AllowConflicts".into(),
        Field {
            enabled: true,
            text: "false".into(),
            ..Default::default()
        },
    );
    fields.insert(
        "ResourceDelegates".into(),
        Field {
            enabled: true,
            text: "a@example.com\nb@example.com".into(),
            ..Default::default()
        },
    );
    fields.insert(
        "AdditionalResponse".into(),
        Field {
            enabled: true,
            clear: true,
            ..Default::default()
        },
    );
    let args = catalog::collect(&command, "Default", &fields).unwrap();
    assert_eq!(args["AllowConflicts"], json!(false));
    assert_eq!(args["AdditionalResponse"], json!(null));
    assert_eq!(
        args["ResourceDelegates"],
        json!(["a@example.com", "b@example.com"])
    );
    assert!(!args.contains_key("DeleteSubject"));
}

#[test]
fn required_fields_and_parameter_sets_are_enforced() {
    let command = catalog::load()
        .into_iter()
        .find(|c| c.name == "Set-CalendarProcessing")
        .unwrap();
    assert!(catalog::collect(&command, "Default", &BTreeMap::new()).is_err());
    let command = catalog::load()
        .into_iter()
        .find(|c| c.name == "Get-MailboxFolder")
        .unwrap();
    let mut fields = BTreeMap::new();
    fields.insert(
        "GetChildren".into(),
        Field {
            enabled: true,
            text: "true".into(),
            ..Default::default()
        },
    );
    assert!(catalog::collect(&command, "GetChildren", &fields).is_ok());
    assert!(catalog::collect(&command, "Recurse", &fields).is_err());
}

#[test]
fn secrets_are_redacted_in_reviews() {
    let mut command = catalog::load()
        .into_iter()
        .find(|c| c.name == "Add-AvailabilityAddressSpace")
        .unwrap();
    command.parameters.push(catalog::Parameter {
        name: "Credentials".into(),
        kind: "PSCredential".into(),
        required: false,
        sets: vec!["(All)".into()],
        values: vec![],
        mandatory_sets: vec![],
    });
    let args =
        json!({"Credentials":{"username":"person","password":"secret"},"ForestName":"example.com"});
    let redacted = catalog::redact(&command, args.as_object().unwrap()).to_string();
    assert!(!redacted.contains("secret"));
    assert!(redacted.contains("example.com"));
}

#[test]
fn collection_updates_and_hostile_strings_remain_data() {
    let command = catalog::load()
        .into_iter()
        .find(|c| c.name == "Set-CalendarProcessing")
        .unwrap();
    let mut fields = BTreeMap::new();
    let hostile = "'; Remove-Mailbox *; $(Get-Process) #";
    fields.insert(
        "Identity".into(),
        Field {
            enabled: true,
            text: hostile.into(),
            ..Default::default()
        },
    );
    fields.insert(
        "ResourceDelegates".into(),
        Field {
            enabled: true,
            text: r#"{"Add":["a@example.com"],"Remove":["b@example.com"]}"#.into(),
            ..Default::default()
        },
    );
    let args = catalog::collect(&command, "Default", &fields).unwrap();
    assert_eq!(args["Identity"], json!(hostile));
    assert_eq!(args["ResourceDelegates"]["Add"][0], "a@example.com");
}
