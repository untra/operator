#[path = "../registry.rs"]
mod registry;

use registry::{Model, Registry};
use std::path::Path;

fn catalog() -> Registry {
    Registry::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../telemetry")).unwrap()
}

#[test]
fn test_registry_generation_is_deterministic() {
    let first = catalog();
    let second = catalog();
    assert_eq!(first.rust(), second.rust());
    assert_eq!(first.markdown(), second.markdown());
    assert_eq!(first.markdown(), operator_telemetry::CATALOG_MARKDOWN);
    assert!(first.markdown().contains("title: OTLP Event Catalog"));
    assert!(!first.markdown().lines().any(|l| l.starts_with("# ")));
}

#[test]
fn test_registry_rejects_missing_and_unknown_classes() {
    let mut r = catalog();
    let a = r
        .groups
        .iter_mut()
        .flat_map(|(_, g)| &mut g.attributes)
        .find(|a| a.id.is_some())
        .unwrap();
    a.annotations = None;
    assert!(r.validate().unwrap_err().contains("class"));
    let mut r = catalog();
    let a = r
        .groups
        .iter_mut()
        .flat_map(|(_, g)| &mut g.attributes)
        .find(|a| a.id.is_some())
        .unwrap();
    a.annotations.as_mut().unwrap().operator.redaction_class = "publci".into();
    assert!(r.validate().unwrap_err().contains("class"));
}

#[test]
fn test_registry_rejects_duplicate_names_and_symbol_collisions() {
    let mut r = catalog();
    r.groups.push(r.groups[0].clone());
    assert!(r.validate().unwrap_err().contains("duplicate"));
    let mut r = catalog();
    let g = &mut r
        .groups
        .iter_mut()
        .find(|(_, g)| g.kind == "attribute_group")
        .unwrap()
        .1;
    let mut a = g.attributes[0].clone();
    a.id = Some(a.id.unwrap().replace('.', "_"));
    g.attributes.push(a);
    assert!(r.validate().unwrap_err().contains("duplicate"));
}

#[test]
fn test_registry_rejects_unresolved_refs_and_secret_event_attributes() {
    for key in ["not.defined", "operator.auth.token"] {
        let mut r = catalog();
        let g = &mut r
            .groups
            .iter_mut()
            .find(|(_, g)| g.kind == "event")
            .unwrap()
            .1;
        g.attributes[0].reference = Some(key.into());
        let error = r.validate().unwrap_err();
        assert!(error.contains("unresolved") || error.contains("secret attribute"));
    }
}

#[test]
fn test_registry_rejects_invalid_patterns_and_class_actions() {
    let mut r = catalog();
    r.patterns.patterns[0].regex = "[".into();
    assert!(r.validate().unwrap_err().contains("pattern"));
    r.patterns.patterns[0].regex = "a*".into();
    assert!(r.validate().unwrap_err().contains("pattern"));
    let mut r = catalog();
    r.classes.classes[0].when_enabled = "unsupported".into();
    assert!(r.validate().unwrap_err().contains("classes or actions"));
}

#[test]
fn test_registry_rejects_malformed_and_unsupported_yaml() {
    assert!(serde_saphyr::from_str::<Model>("groups: [").is_err());
    let model = "groups:\n  - id: example\n    type: attribute_group\n    brief: Example\n    attributes: []\n    extends: unreviewed\n";
    assert!(serde_saphyr::from_str::<Model>(model).is_err());
    let mut r = catalog();
    r.groups[0].1.kind = "unsupported".into();
    assert!(r.validate().is_err());
}

#[test]
fn test_registry_actions_match_the_fixed_contract() {
    let r = catalog();
    for (name, enabled, disabled) in [
        ("public", "keep", "keep"),
        ("internal", "keep", "keep"),
        ("personal", "hash", "keep"),
        ("content", "drop", "keep"),
        ("secret", "drop", "drop"),
    ] {
        let class = r.classes.classes.iter().find(|c| c.name == name).unwrap();
        assert_eq!(class.when_enabled, enabled);
        assert_eq!(class.when_disabled, disabled);
    }
    let mut invalid = r;
    invalid
        .classes
        .classes
        .iter_mut()
        .find(|c| c.name == "secret")
        .unwrap()
        .when_disabled = "keep".into();
    assert!(invalid.validate().is_err());
}
