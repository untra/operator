use operator_telemetry::{class_of, RedactionClass};

#[test]
fn test_class_of_known_and_unknown_attributes() {
    assert_eq!(class_of("operator.ticket.id"), RedactionClass::Internal);
    assert_eq!(class_of("operator.exit_code"), RedactionClass::Public);
    assert_eq!(class_of("user.email"), RedactionClass::Personal);
    assert_eq!(class_of("operator.auth.token"), RedactionClass::Secret);
    assert_eq!(class_of("unreviewed.attribute"), RedactionClass::Content);
    assert_eq!(class_of("claude_code.unreviewed"), RedactionClass::Content);
}

#[test]
fn test_class_serde() {
    let classes = [
        RedactionClass::Public,
        RedactionClass::Internal,
        RedactionClass::Personal,
        RedactionClass::Content,
        RedactionClass::Secret,
    ];
    for (class, name) in classes
        .into_iter()
        .zip(["public", "internal", "personal", "content", "secret"])
    {
        assert_eq!(
            serde_json::to_string(&class).unwrap(),
            format!("\"{name}\"")
        );
        assert_eq!(
            serde_json::from_str::<RedactionClass>(&format!("\"{name}\"")).unwrap(),
            class
        );
    }
}

#[test]
fn test_catalog_signal_coverage_and_attribute_references() {
    use operator_telemetry::{EventName, MetricName, SpanName, ATTRIBUTES, SIGNALS};
    let expected = [
        "agent.started",
        "agent.completed",
        "agent.failed",
        "agent.awaiting_input",
        "agent.session_lost",
        "pr.created",
        "pr.merged",
        "pr.closed",
        "pr.ready_to_merge",
        "pr.changes_requested",
        "ticket.returned",
        "investigation.created",
        "step.started",
        "step.completed",
        "auth.bootstrap",
        "auth.login",
        "auth.logout",
        "auth.password_reset",
        "auth.session_revoked",
        "auth.device_code_issued",
        "auth.device_approved",
        "auth.refresh_token_reuse",
        "auth.access_key_exchange",
        "auth.access_key_created",
        "auth.access_key_revoked",
        "auth.admin_created",
    ];
    for name in expected {
        assert!(
            EventName::ALL
                .iter()
                .any(|e| e.as_str() == format!("operator.{name}")),
            "{name}"
        );
    }
    assert_eq!(SpanName::ALL.len(), 4);
    assert_eq!(MetricName::ALL.len(), 7);
    for signal in SIGNALS {
        for key in signal.attributes {
            assert!(ATTRIBUTES.iter().any(|a| a.name == *key), "{key}");
            if signal.kind == "event" {
                assert_ne!(class_of(key), RedactionClass::Secret);
            }
        }
    }
}
