use crate::{auth::audit::AuditAction, notifications::NotificationEvent};
use opentelemetry::logs::{LogRecord, Logger, Severity};
use opentelemetry::KeyValue;
use operator_telemetry::{attr, EventName};

#[cfg(test)]
thread_local! { static EMITTED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }

pub fn emit_event(name: EventName, attributes: Vec<KeyValue>) {
    #[cfg(test)]
    EMITTED.with(|count| count.set(count.get() + 1));
    let Some(logger) =
        super::self_export::active().and_then(super::self_export::SelfExport::logger)
    else {
        return;
    };
    let mut record = logger.create_log_record();
    record.set_event_name(name.as_str());
    record.set_timestamp(std::time::SystemTime::now());
    record.set_severity_number(Severity::Info);
    for kv in attributes {
        let value: opentelemetry::logs::AnyValue = match kv.value {
            opentelemetry::Value::Bool(v) => v.into(),
            opentelemetry::Value::I64(v) => v.into(),
            opentelemetry::Value::F64(v) => v.into(),
            opentelemetry::Value::String(v) => v.as_str().to_owned().into(),
            other => other.to_string().into(),
        };
        record.add_attribute(kv.key, value);
    }
    logger.emit(record);
}

pub fn audit_event(action: AuditAction, succeeded: bool, detail: Option<&str>) {
    let mut attributes = vec![KeyValue::new(attr::OPERATOR_AUTH_SUCCEEDED, succeeded)];
    if let Some(detail) = detail {
        attributes.push(KeyValue::new(attr::OPERATOR_AUTH_DETAIL, detail.to_owned()));
    }
    emit_event(action.event(), attributes);
}

pub fn notification_event(event: &NotificationEvent) -> (EventName, Vec<KeyValue>) {
    match event {
        NotificationEvent::AgentStarted {
            project,
            ticket_type,
            ticket_id,
            session_name,
            launch_mode,
        } => {
            let mut attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_ISSUETYPE_KEY, ticket_type.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_SESSION_NAME, session_name.clone()),
            ];
            if let Some(launch_mode) = launch_mode {
                attributes.push(KeyValue::new(
                    attr::OPERATOR_LAUNCH_MODE,
                    launch_mode.clone(),
                ));
            }
            (EventName::OperatorAgentStarted, attributes)
        }
        NotificationEvent::AgentCompleted {
            project,
            ticket_type,
            ticket_id,
            pr_url,
            duration_seconds,
        } => {
            let mut attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_ISSUETYPE_KEY, ticket_type.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
            ];
            if let Some(pr_url) = pr_url {
                attributes.push(KeyValue::new(attr::OPERATOR_PR_URL, pr_url.clone()));
            }
            if let Some(duration_seconds) = duration_seconds {
                attributes.push(KeyValue::new(
                    attr::OPERATOR_DURATION,
                    (*duration_seconds).min(i64::MAX as u64) as i64,
                ));
            }
            (EventName::OperatorAgentCompleted, attributes)
        }
        NotificationEvent::AgentFailed {
            project,
            ticket_id,
            error,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_ERROR, error.clone()),
            ];
            (EventName::OperatorAgentFailed, attributes)
        }
        NotificationEvent::AgentAwaitingInput {
            project,
            ticket_type,
            ticket_id,
            reason,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_ISSUETYPE_KEY, ticket_type.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_REASON, reason.clone()),
            ];
            (EventName::OperatorAgentAwaitingInput, attributes)
        }
        NotificationEvent::AgentSessionLost { session_name } => {
            let attributes = vec![KeyValue::new(
                attr::OPERATOR_SESSION_NAME,
                session_name.clone(),
            )];
            (EventName::OperatorAgentSessionLost, attributes)
        }
        NotificationEvent::PrCreated {
            project,
            ticket_id,
            pr_url,
            pr_number,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_PR_URL, pr_url.clone()),
                KeyValue::new(attr::OPERATOR_PR_NUMBER, *pr_number),
            ];
            (EventName::OperatorPrCreated, attributes)
        }
        NotificationEvent::TicketReturned {
            project,
            ticket_id,
            summary,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_SUMMARY, summary.clone()),
            ];
            (EventName::OperatorTicketReturned, attributes)
        }
        NotificationEvent::InvestigationCreated {
            source,
            severity,
            summary,
            ticket_id,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_INVESTIGATION_SOURCE, source.clone()),
                KeyValue::new(attr::OPERATOR_INVESTIGATION_SEVERITY, severity.clone()),
                KeyValue::new(attr::OPERATOR_SUMMARY, summary.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
            ];
            (EventName::OperatorInvestigationCreated, attributes)
        }
        NotificationEvent::PrMerged {
            project,
            ticket_id,
            pr_number,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_PR_NUMBER, *pr_number),
            ];
            (EventName::OperatorPrMerged, attributes)
        }
        NotificationEvent::PrClosed {
            project,
            ticket_id,
            pr_number,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_PR_NUMBER, *pr_number),
            ];
            (EventName::OperatorPrClosed, attributes)
        }
        NotificationEvent::PrReadyToMerge {
            project,
            ticket_id,
            pr_number,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_PR_NUMBER, *pr_number),
            ];
            (EventName::OperatorPrReadyToMerge, attributes)
        }
        NotificationEvent::PrChangesRequested {
            project,
            ticket_id,
            pr_number,
        } => {
            let attributes = vec![
                KeyValue::new(attr::OPERATOR_PROJECT, project.clone()),
                KeyValue::new(attr::OPERATOR_TICKET_ID, ticket_id.clone()),
                KeyValue::new(attr::OPERATOR_PR_NUMBER, *pr_number),
            ];
            (EventName::OperatorPrChangesRequested, attributes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disabled_notifications_still_emit() {
        let service = crate::notifications::NotificationService::disabled();
        let before = EMITTED.with(std::cell::Cell::get);
        service.notify(NotificationEvent::AgentSessionLost {
            session_name: "s".into(),
        });
        service.notify_sync(NotificationEvent::AgentSessionLost {
            session_name: "s".into(),
        });
        assert_eq!(EMITTED.with(std::cell::Cell::get), before + 2);
    }
    #[test]
    fn test_notification_mappings_match_catalog() {
        for event in [
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
        ] {
            let event: NotificationEvent = serde_json::from_value(serde_json::json!({"event":event,"data":{
                "project":"p","ticket_type":"TASK","ticket_id":"t","session_name":"s","launch_mode":"auto", "pr_url":"http://local/pr/1","duration_seconds":1,"error":"e","reason":"r","pr_number":1,"summary":"s","source":"s","severity":"info"
            }})).unwrap();
            let (name, attributes) = notification_event(&event);
            let spec = operator_telemetry::SIGNALS
                .iter()
                .find(|s| s.name == name.as_str())
                .unwrap();
            for attribute in attributes {
                assert!(
                    spec.attributes.contains(&attribute.key.as_str()),
                    "{}: {}",
                    spec.name,
                    attribute.key
                );
            }
        }
    }
    #[test]
    fn test_audit_mapping_preserves_legacy_strings() {
        assert_eq!(
            AuditAction::ALL.map(AuditAction::as_str),
            [
                "bootstrap",
                "login",
                "logout",
                "password reset",
                "session revoked",
                "device code issued",
                "device approved",
                "refresh token reuse",
                "access key exchange",
                "access key created",
                "access key revoked",
                "admin created"
            ]
        );
        for action in AuditAction::ALL {
            assert!(operator_telemetry::SIGNALS
                .iter()
                .any(|s| s.name == action.event().as_str()));
        }
    }
}
