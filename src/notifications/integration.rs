//! Notification integration trait definition.

use anyhow::Result;
use async_trait::async_trait;

use super::NotificationEvent;

/// Trait for notification integrations.
///
/// Each integration (OS notifications, webhooks, etc.) implements this trait
/// to receive and handle notification events.
#[async_trait]
pub trait NotificationIntegration: Send + Sync {
    /// Integration name (for logging and config identification)
    fn name(&self) -> &str;

    /// Check if this integration handles the given event type.
    ///
    /// Returns true if this integration should receive the event.
    /// Used for per-integration event filtering.
    fn handles_event(&self, event: &NotificationEvent) -> bool;

    /// Check if this integration is enabled.
    fn is_enabled(&self) -> bool;

    /// Send a notification event.
    ///
    /// This is fire-and-forget - implementations should log errors
    /// but not fail the overall notification dispatch.
    async fn send(&self, event: &NotificationEvent) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock integration for testing
    struct MockIntegration {
        name: String,
        enabled: bool,
        events: Vec<String>,
        sent_events: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl NotificationIntegration for MockIntegration {
        fn name(&self) -> &str {
            &self.name
        }

        fn handles_event(&self, event: &NotificationEvent) -> bool {
            self.events.is_empty() || self.events.contains(&event.event_type().to_string())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        async fn send(&self, event: &NotificationEvent) -> Result<()> {
            self.sent_events
                .lock()
                .unwrap()
                .push(event.event_type().to_string());
            Ok(())
        }
    }

    #[test]
    fn test_handles_event_empty_filter_matches_all() {
        let sent = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let integration = MockIntegration {
            name: "test".into(),
            enabled: true,
            events: vec![], // Empty = all events
            sent_events: sent,
        };

        let event = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        assert!(integration.handles_event(&event));
    }

    #[test]
    fn test_handles_event_specific_filter() {
        let sent = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let integration = MockIntegration {
            name: "test".into(),
            enabled: true,
            events: vec!["agent.completed".into()],
            sent_events: sent,
        };

        let started_event = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        let completed_event = NotificationEvent::AgentCompleted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            pr_url: None,
            duration_seconds: None,
        };

        assert!(!integration.handles_event(&started_event));
        assert!(integration.handles_event(&completed_event));
    }
}
