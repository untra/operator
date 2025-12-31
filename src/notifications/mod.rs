//! Notification system for operator events.
//!
//! This module provides a notification integration architecture where multiple
//! integrations (OS notifications, webhooks) can receive events.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

mod integration;
mod os_integration;
mod service;
mod webhook_integration;

// Public API exports for extensibility
#[allow(unused_imports)]
pub use integration::NotificationIntegration;
#[allow(unused_imports)]
pub use os_integration::OsIntegration;
pub use service::NotificationService;
#[allow(unused_imports)]
pub use webhook_integration::WebhookIntegration;

/// Send a notification using the platform-specific implementation.
///
/// This is a compatibility shim - new code should use NotificationService.notify().
#[deprecated(note = "Use NotificationService.notify() instead")]
pub fn send(title: &str, subtitle: &str, message: &str, sound: bool) -> Result<()> {
    send_os_notification(title, subtitle, message, sound)
}

/// Send a notification using the platform-specific implementation.
/// This is a low-level function used by OsIntegration.
pub fn send_os_notification(title: &str, subtitle: &str, message: &str, sound: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::send_notification(title, subtitle, message, sound)
    }

    #[cfg(target_os = "linux")]
    {
        linux::send_notification(title, subtitle, message, sound)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        // Fall back to just logging on other systems
        let _ = sound; // suppress unused warning
        tracing::info!("Notification: {} - {} - {}", title, subtitle, message);
        Ok(())
    }
}

/// All notification events that can be dispatched to integrations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", content = "data")]
pub enum NotificationEvent {
    /// Agent has started working on a ticket
    #[serde(rename = "agent.started")]
    AgentStarted {
        project: String,
        ticket_type: String,
        ticket_id: String,
        session_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        launch_mode: Option<String>,
    },

    /// Agent has completed work on a ticket
    #[serde(rename = "agent.completed")]
    AgentCompleted {
        project: String,
        ticket_type: String,
        ticket_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pr_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_seconds: Option<u64>,
    },

    /// Agent encountered an error
    #[serde(rename = "agent.failed")]
    AgentFailed {
        project: String,
        ticket_id: String,
        error: String,
    },

    /// Agent is waiting for user input
    #[serde(rename = "agent.awaiting_input")]
    AgentAwaitingInput {
        project: String,
        ticket_type: String,
        ticket_id: String,
        reason: String,
    },

    /// Agent's tmux session was lost
    #[serde(rename = "agent.session_lost")]
    AgentSessionLost { session_name: String },

    /// Pull request was created
    #[serde(rename = "pr.created")]
    PrCreated {
        project: String,
        ticket_id: String,
        pr_url: String,
        pr_number: i64,
    },

    /// Pull request was merged
    #[serde(rename = "pr.merged")]
    PrMerged {
        project: String,
        ticket_id: String,
        pr_number: i64,
    },

    /// Pull request was closed without merging
    #[serde(rename = "pr.closed")]
    PrClosed {
        project: String,
        ticket_id: String,
        pr_number: i64,
    },

    /// Pull request is approved and ready to merge
    #[serde(rename = "pr.ready_to_merge")]
    PrReadyToMerge {
        project: String,
        ticket_id: String,
        pr_number: i64,
    },

    /// Pull request has changes requested
    #[serde(rename = "pr.changes_requested")]
    PrChangesRequested {
        project: String,
        ticket_id: String,
        pr_number: i64,
    },

    /// Ticket was returned to queue
    #[serde(rename = "ticket.returned")]
    TicketReturned {
        project: String,
        ticket_id: String,
        summary: String,
    },

    /// Investigation ticket was created
    #[serde(rename = "investigation.created")]
    InvestigationCreated {
        source: String,
        severity: String,
        summary: String,
        ticket_id: String,
    },
}

impl NotificationEvent {
    /// Get the event type string for filtering (e.g., "agent.started")
    pub fn event_type(&self) -> &'static str {
        match self {
            NotificationEvent::AgentStarted { .. } => "agent.started",
            NotificationEvent::AgentCompleted { .. } => "agent.completed",
            NotificationEvent::AgentFailed { .. } => "agent.failed",
            NotificationEvent::AgentAwaitingInput { .. } => "agent.awaiting_input",
            NotificationEvent::AgentSessionLost { .. } => "agent.session_lost",
            NotificationEvent::PrCreated { .. } => "pr.created",
            NotificationEvent::PrMerged { .. } => "pr.merged",
            NotificationEvent::PrClosed { .. } => "pr.closed",
            NotificationEvent::PrReadyToMerge { .. } => "pr.ready_to_merge",
            NotificationEvent::PrChangesRequested { .. } => "pr.changes_requested",
            NotificationEvent::TicketReturned { .. } => "ticket.returned",
            NotificationEvent::InvestigationCreated { .. } => "investigation.created",
        }
    }

    /// Format for OS notification display.
    /// Returns (title, subtitle, message).
    pub fn to_os_notification(&self) -> (String, String, String) {
        match self {
            NotificationEvent::AgentStarted {
                project,
                ticket_type,
                ticket_id,
                session_name,
                launch_mode,
            } => {
                let mode_suffix = launch_mode
                    .as_ref()
                    .map(|m| format!(" [{}]", m))
                    .unwrap_or_default();
                (
                    "Agent Started".to_string(),
                    format!(
                        "{} - {} (tmux: {}){}",
                        project, ticket_type, session_name, mode_suffix
                    ),
                    ticket_id.clone(),
                )
            }

            NotificationEvent::AgentCompleted {
                project,
                ticket_type,
                ticket_id,
                pr_url,
                ..
            } => {
                let message = if let Some(url) = pr_url {
                    format!("{} complete - PR: {}", ticket_id, url)
                } else {
                    format!("{} complete", ticket_id)
                };
                (
                    "Agent Complete".to_string(),
                    format!("{} - {}", project, ticket_type),
                    message,
                )
            }

            NotificationEvent::AgentFailed {
                project,
                ticket_id,
                error,
            } => (
                "Agent Failed".to_string(),
                format!("{} - {}", project, ticket_id),
                error.clone(),
            ),

            NotificationEvent::AgentAwaitingInput {
                project,
                ticket_type,
                ticket_id,
                reason,
            } => (
                "Agent Awaiting Input".to_string(),
                format!("{} - {} ({})", project, ticket_type, ticket_id),
                reason.clone(),
            ),

            NotificationEvent::AgentSessionLost { session_name } => (
                "Agent Session Lost".to_string(),
                session_name.clone(),
                "The tmux session for this agent has terminated unexpectedly.".to_string(),
            ),

            NotificationEvent::PrCreated {
                project,
                ticket_id,
                pr_url,
                pr_number,
            } => (
                "PR Created".to_string(),
                format!("{} - {}", project, ticket_id),
                format!("PR #{}: {}", pr_number, pr_url),
            ),

            NotificationEvent::PrMerged {
                project,
                ticket_id,
                pr_number,
            } => (
                "PR Merged".to_string(),
                project.clone(),
                format!("PR #{} merged for {}", pr_number, ticket_id),
            ),

            NotificationEvent::PrClosed {
                ticket_id,
                pr_number,
                ..
            } => (
                "PR Closed".to_string(),
                ticket_id.clone(),
                format!("PR #{} closed without merge", pr_number),
            ),

            NotificationEvent::PrReadyToMerge {
                ticket_id,
                pr_number,
                ..
            } => (
                "PR Ready to Merge".to_string(),
                ticket_id.clone(),
                format!("PR #{} is approved and ready to merge", pr_number),
            ),

            NotificationEvent::PrChangesRequested {
                ticket_id,
                pr_number,
                ..
            } => (
                "PR Changes Requested".to_string(),
                ticket_id.clone(),
                format!("PR #{} has changes requested", pr_number),
            ),

            NotificationEvent::TicketReturned {
                project,
                ticket_id,
                summary,
            } => (
                "Ticket Returned to Queue".to_string(),
                project.clone(),
                format!("{} - {}", ticket_id, summary),
            ),

            NotificationEvent::InvestigationCreated {
                source,
                severity,
                summary,
                ticket_id,
            } => (
                "Investigation Created".to_string(),
                format!("{}-{} [{}] from {}", "INV", ticket_id, severity, source),
                summary.clone(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_strings() {
        let test_cases = vec![
            (
                NotificationEvent::AgentStarted {
                    project: "test".into(),
                    ticket_type: "FEAT".into(),
                    ticket_id: "123".into(),
                    session_name: "tmux-123".into(),
                    launch_mode: None,
                },
                "agent.started",
            ),
            (
                NotificationEvent::AgentCompleted {
                    project: "test".into(),
                    ticket_type: "FEAT".into(),
                    ticket_id: "123".into(),
                    pr_url: None,
                    duration_seconds: None,
                },
                "agent.completed",
            ),
            (
                NotificationEvent::AgentFailed {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    error: "error".into(),
                },
                "agent.failed",
            ),
            (
                NotificationEvent::AgentAwaitingInput {
                    project: "test".into(),
                    ticket_type: "SPIKE".into(),
                    ticket_id: "123".into(),
                    reason: "question".into(),
                },
                "agent.awaiting_input",
            ),
            (
                NotificationEvent::AgentSessionLost {
                    session_name: "tmux-123".into(),
                },
                "agent.session_lost",
            ),
            (
                NotificationEvent::PrCreated {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    pr_url: "https://github.com/...".into(),
                    pr_number: 42,
                },
                "pr.created",
            ),
            (
                NotificationEvent::PrMerged {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    pr_number: 42,
                },
                "pr.merged",
            ),
            (
                NotificationEvent::PrClosed {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    pr_number: 42,
                },
                "pr.closed",
            ),
            (
                NotificationEvent::PrReadyToMerge {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    pr_number: 42,
                },
                "pr.ready_to_merge",
            ),
            (
                NotificationEvent::PrChangesRequested {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    pr_number: 42,
                },
                "pr.changes_requested",
            ),
            (
                NotificationEvent::TicketReturned {
                    project: "test".into(),
                    ticket_id: "123".into(),
                    summary: "summary".into(),
                },
                "ticket.returned",
            ),
            (
                NotificationEvent::InvestigationCreated {
                    source: "pagerduty".into(),
                    severity: "S1".into(),
                    summary: "error".into(),
                    ticket_id: "456".into(),
                },
                "investigation.created",
            ),
        ];

        for (event, expected_type) in test_cases {
            assert_eq!(
                event.event_type(),
                expected_type,
                "Event {:?} should have type '{}'",
                event,
                expected_type
            );
        }
    }

    #[test]
    fn test_event_serialization() {
        let event = NotificationEvent::AgentStarted {
            project: "backend".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "FEAT-042".into(),
            session_name: "op-backend-feat-042".into(),
            launch_mode: Some("docker".into()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event\":\"agent.started\""));
        assert!(json.contains("\"project\":\"backend\""));

        // Round-trip
        let deserialized: NotificationEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_to_os_notification_agent_started() {
        let event = NotificationEvent::AgentStarted {
            project: "backend".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "FEAT-042".into(),
            session_name: "op-backend-feat-042".into(),
            launch_mode: Some("docker".into()),
        };

        let (title, subtitle, message) = event.to_os_notification();
        assert_eq!(title, "Agent Started");
        assert!(subtitle.contains("backend"));
        assert!(subtitle.contains("docker"));
        assert_eq!(message, "FEAT-042");
    }

    #[test]
    fn test_to_os_notification_pr_merged() {
        let event = NotificationEvent::PrMerged {
            project: "backend".into(),
            ticket_id: "FEAT-042".into(),
            pr_number: 123,
        };

        let (title, subtitle, message) = event.to_os_notification();
        assert_eq!(title, "PR Merged");
        assert_eq!(subtitle, "backend");
        assert!(message.contains("123"));
        assert!(message.contains("FEAT-042"));
    }
}
