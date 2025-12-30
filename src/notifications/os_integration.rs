//! OS-native notification integration (macOS/Linux).

use anyhow::Result;
use async_trait::async_trait;

use super::integration::NotificationIntegration;
use super::NotificationEvent;
use crate::config::OsNotificationConfig;

/// OS-native notification integration.
///
/// Sends notifications using the platform's native notification system:
/// - macOS: Uses `mac-notification-sys`
/// - Linux: Uses `notify-rust` (freedesktop notifications)
pub struct OsIntegration {
    enabled: bool,
    sound: bool,
    subscribed_events: Vec<String>,
}

impl OsIntegration {
    /// Create a new OS integration from config.
    pub fn new(config: &OsNotificationConfig) -> Self {
        Self {
            enabled: config.enabled,
            sound: config.sound,
            subscribed_events: config.events.clone(),
        }
    }

    /// Create a disabled OS integration.
    #[allow(dead_code)]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            sound: false,
            subscribed_events: Vec::new(),
        }
    }
}

#[async_trait]
impl NotificationIntegration for OsIntegration {
    fn name(&self) -> &str {
        "os"
    }

    fn handles_event(&self, event: &NotificationEvent) -> bool {
        // Empty subscription list means handle all events
        self.subscribed_events.is_empty()
            || self
                .subscribed_events
                .contains(&event.event_type().to_string())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn send(&self, event: &NotificationEvent) -> Result<()> {
        let (title, subtitle, message) = event.to_os_notification();

        // Call the platform-specific send function
        if let Err(e) = super::send_os_notification(&title, &subtitle, &message, self.sound) {
            tracing::warn!(
                integration = "os",
                event = %event.event_type(),
                error = %e,
                "Failed to send OS notification"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config(enabled: bool, events: Vec<String>) -> OsNotificationConfig {
        OsNotificationConfig {
            enabled,
            sound: false,
            events,
        }
    }

    #[test]
    fn test_os_integration_from_config() {
        let config = make_test_config(true, vec!["agent.started".into()]);
        let integration = OsIntegration::new(&config);

        assert!(integration.is_enabled());
        assert_eq!(integration.name(), "os");
    }

    #[test]
    fn test_os_integration_disabled() {
        let integration = OsIntegration::disabled();
        assert!(!integration.is_enabled());
    }

    #[test]
    fn test_handles_event_empty_filter() {
        let config = make_test_config(true, vec![]);
        let integration = OsIntegration::new(&config);

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
    fn test_handles_event_specific_filter_match() {
        let config = make_test_config(true, vec!["agent.started".into()]);
        let integration = OsIntegration::new(&config);

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
    fn test_handles_event_specific_filter_no_match() {
        let config = make_test_config(true, vec!["agent.completed".into()]);
        let integration = OsIntegration::new(&config);

        let event = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        assert!(!integration.handles_event(&event));
    }

    #[test]
    fn test_handles_event_multiple_filters() {
        let config = make_test_config(true, vec!["agent.started".into(), "agent.completed".into()]);
        let integration = OsIntegration::new(&config);

        let started = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        let completed = NotificationEvent::AgentCompleted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            pr_url: None,
            duration_seconds: None,
        };

        let failed = NotificationEvent::AgentFailed {
            project: "test".into(),
            ticket_id: "123".into(),
            error: "test".into(),
        };

        assert!(integration.handles_event(&started));
        assert!(integration.handles_event(&completed));
        assert!(!integration.handles_event(&failed));
    }
}
