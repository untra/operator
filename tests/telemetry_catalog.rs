use operator_telemetry::{EventName, RedactionClass};
use ts_rs::TS;

#[test]
fn export_bindings_redaction_class() {
    RedactionClass::export(&ts_rs::Config::default()).unwrap();
}

#[test]
fn test_catalog_covers_existing_notification_and_audit_names() {
    let notification_source = include_str!("../src/notifications/mod.rs");
    let notification = regex::Regex::new(r#"#\[serde\(rename = "([a-z_]+\.[a-z_]+)"\)\]"#).unwrap();
    let audit = regex::Regex::new(r#"\.audit\(\s*"([a-z ]+)""#).unwrap();
    for capture in notification.captures_iter(notification_source) {
        let name = format!("operator.{}", &capture[1]);
        assert!(
            EventName::ALL.iter().any(|event| event.as_str() == name),
            "{name}"
        );
    }
    for source in [
        include_str!("../src/rest/routes/auth.rs"),
        include_str!("../src/main.rs"),
        include_str!("../src/app/tickets.rs"),
    ] {
        for capture in audit.captures_iter(source) {
            let name = format!("operator.auth.{}", capture[1].replace(' ', "_"));
            assert!(
                EventName::ALL.iter().any(|event| event.as_str() == name),
                "{name}"
            );
        }
    }
}

#[test]
fn test_generate_types_check_rejects_stale_catalog_without_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in [
        "shared/types.ts",
        "docs/schemas/config.json",
        "docs/schemas/state.json",
        "docs/schemas/jira-api.json",
        "docs/telemetry/catalog.md",
    ] {
        let dest = temp.path().join(path);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::copy(root.join(path), dest).unwrap();
    }
    let check = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_generate_types"))
            .arg("--check")
            .current_dir(temp.path())
            .output()
            .unwrap()
    };
    let fresh = check();
    assert!(
        fresh.status.success(),
        "{}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let catalog = temp.path().join("docs/telemetry/catalog.md");
    std::fs::write(&catalog, "stale catalog").unwrap();
    let stale = check();
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("docs/telemetry/catalog.md"));
    assert_eq!(std::fs::read_to_string(&catalog).unwrap(), "stale catalog");
    std::fs::remove_file(&catalog).unwrap();
    assert!(!check().status.success());
    assert!(!catalog.exists());
}
