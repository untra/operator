use super::*;
use serde_json::json;

const KEY: [u8; 32] = [42; 32];
fn policy(enable_redaction: bool) -> RedactionPolicy {
    RedactionPolicy { enable_redaction }
}

#[test]
fn test_redaction_fixed_matrix_and_false_default() {
    assert!(!RedactionPolicy::default().enable_redaction);
    let redactor = Redactor::new(&KEY).unwrap();
    for enabled in [false, true] {
        for (key, value, class) in [
            ("operator.exit_code", json!(0), RedactionClass::Public),
            (
                "operator.ticket.id",
                json!("TASK-1"),
                RedactionClass::Internal,
            ),
            (
                "user.email",
                json!("person@example.test"),
                RedactionClass::Personal,
            ),
            (
                "operator.summary",
                json!("private text"),
                RedactionClass::Content,
            ),
            (
                "unknown",
                json!({"nested": "private text"}),
                RedactionClass::Content,
            ),
            (
                "operator.auth.token",
                json!("opaque"),
                RedactionClass::Secret,
            ),
        ] {
            let result = redactor.redact_attribute(key, &value, policy(enabled));
            if class == RedactionClass::Secret || (enabled && class == RedactionClass::Content) {
                assert_eq!(result.value, None, "{key} {enabled}");
                assert_eq!(result.stats.dropped, 1);
            } else if enabled && class == RedactionClass::Personal {
                assert!(result.value.unwrap().as_str().unwrap().starts_with("hmac:"));
                assert_eq!(result.stats.hashed, 1);
            } else {
                assert_eq!(result.value, Some(value));
            }
        }
    }
}

#[test]
fn test_secret_patterns_drop_whole_attributes_and_scrub_bodies() {
    let redactor = Redactor::new(&KEY).unwrap();
    let allowed = policy(false);
    for secret in [
        "opk_v1_01234567-89ab-cdef-0123-456789abcdef_randomSecret123",
        "opk_legacySecret123",
        "sk-ant-test123",
        "ghp_test123",
        "gho_test123",
        "github_pat_test123",
        "AKIAABCDEFGHIJKLMNOP",
        "ASIAABCDEFGHIJKLMNOP",
        "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ4In0.c2ln",
        "Bearer test-token",
        "bearer test-token",
        "Bearer%20test-token",
    ] {
        let value = json!({"nested": [format!("before {secret} after")]});
        assert_eq!(
            redactor
                .redact_attribute("service.name", &value, allowed)
                .value,
            None,
            "{secret}"
        );
        let body = redactor.redact_body(&value, allowed);
        assert_eq!(
            body.value,
            Some(json!({"nested": ["before [REDACTED] after"]})),
            "{secret}"
        );
        assert_eq!(body.stats.scrubbed, 1);
        assert_eq!(
            redactor.redact_attribute(secret, &json!(1), allowed).value,
            None
        );
    }
}

#[test]
fn test_secret_scrubber_benign_near_matches() {
    let redactor = Redactor::new(&KEY).unwrap();
    for text in [
        "bearer",
        "sk-ant-",
        "ghp_",
        "AKIAABCDEFGHIJKLMNO",
        "ordinary text",
        "hmac:0123456789abcdef",
    ] {
        let value = json!(text);
        assert_eq!(
            redactor.redact_body(&value, policy(false)).value,
            Some(value)
        );
    }
}

#[test]
fn test_body_content_gate_and_secret_keys() {
    let redactor = Redactor::new(&KEY).unwrap();
    assert_eq!(
        redactor.redact_body(&json!("private"), policy(true)).value,
        None
    );
    assert_eq!(
        redactor.redact_body(&json!("private"), policy(true)).value,
        None
    );
    let result = redactor.redact_body(
        &json!({"operator.auth.token": "opaque", "ghp_test123": "value", "ok": "retained"}),
        policy(false),
    );
    assert_eq!(result.value, Some(json!({"ok": "retained"})));
    assert_eq!(result.stats.dropped, 2);
}

#[test]
fn test_hmac_stability_after_key_reload_and_distinct_keys() {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), KEY).unwrap();
    let loaded: [u8; 32] = std::fs::read(file.path()).unwrap().try_into().unwrap();
    let value = json!("person@example.test");
    let p = policy(true);
    let first = Redactor::new(&KEY)
        .unwrap()
        .redact_attribute("user.email", &value, p);
    let second = Redactor::new(&loaded)
        .unwrap()
        .redact_attribute("user.email", &value, p);
    assert_eq!(first, second);
    assert_ne!(
        first,
        Redactor::new(&[43; 32])
            .unwrap()
            .redact_attribute("user.email", &value, p)
    );
    assert_eq!(
        first.value.unwrap().as_str().unwrap().len(),
        "hmac:".len() + 16
    );
}

#[test]
fn test_hmac_known_answer_and_typed_canonical_values() {
    let redactor = Redactor::new(&KEY).unwrap();
    let p = policy(true);
    assert_eq!(
        redactor
            .redact_attribute("user.email", &json!("person@example.test"), p)
            .value,
        Some(json!("hmac:5534452358910456"))
    );
    assert_ne!(redactor.hash(&json!(1)), redactor.hash(&json!("1")));
    assert_eq!(
        redactor.hash(&json!({"a":1,"b":2})),
        redactor.hash(&json!({"b":2,"a":1}))
    );
}

#[test]
fn test_modes_do_not_mutate_input() {
    let redactor = Redactor::new(&KEY).unwrap();
    let value = json!("person@example.test");
    let window = redactor.redact_attribute("user.email", &value, policy(false));
    let export = redactor.redact_attribute("user.email", &value, policy(true));
    assert_eq!(window.value, Some(value.clone()));
    assert_ne!(window.value, export.value);
    assert_eq!(value, json!("person@example.test"));
}

#[test]
fn test_public_scalar_attribute_rejects_content_object() {
    let redactor = Redactor::new(&KEY).unwrap();
    let result = redactor.redact_attribute(
        "service.name",
        &json!({"prompt": "private text"}),
        policy(true),
    );
    assert_eq!(result.value, None);
    assert_eq!(result.stats.dropped, 1);
}
