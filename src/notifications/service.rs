//! Central notification service that dispatches events to all integrations.

use std::sync::Arc;

use anyhow::Result;

use super::integration::NotificationIntegration;
use super::os_integration::OsIntegration;
use super::webhook_integration::WebhookIntegration;
use super::NotificationEvent;
use crate::config::Config;

/// Central notification dispatcher.
///
/// Receives events and dispatches them to all enabled integrations
/// that handle the given event type.
pub struct NotificationService {
    integrations: Vec<Arc<dyn NotificationIntegration>>,
    enabled: bool,
}

impl NotificationService {
    /// Create a new notification service from config.
    pub fn from_config(config: &Config) -> Result<Self> {
        let mut integrations: Vec<Arc<dyn NotificationIntegration>> = Vec::new();

        // Add OS integration
        let os_integration = OsIntegration::new(&config.notifications.os);
        integrations.push(Arc::new(os_integration));

        // Add single webhook if configured
        if let Some(ref webhook_config) = config.notifications.webhook {
            if webhook_config.enabled && !webhook_config.url.is_empty() {
                match WebhookIntegration::new(webhook_config) {
                    Ok(webhook) => integrations.push(Arc::new(webhook)),
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to create webhook integration");
                    }
                }
            }
        }

        // Add multiple webhooks if configured
        for webhook_config in &config.notifications.webhooks {
            if webhook_config.enabled && !webhook_config.url.is_empty() {
                match WebhookIntegration::new(webhook_config) {
                    Ok(webhook) => integrations.push(Arc::new(webhook)),
                    Err(e) => {
                        tracing::warn!(
                            webhook = webhook_config.name.as_deref().unwrap_or("unnamed"),
                            error = %e,
                            "Failed to create webhook integration"
                        );
                    }
                }
            }
        }

        Ok(Self {
            integrations,
            enabled: config.notifications.enabled,
        })
    }

    /// Create a disabled notification service (for testing).
    #[allow(dead_code)]
    pub fn disabled() -> Self {
        Self {
            integrations: Vec::new(),
            enabled: false,
        }
    }

    /// Check if notifications are globally enabled.
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the number of registered integrations.
    #[allow(dead_code)]
    pub fn integration_count(&self) -> usize {
        self.integrations.len()
    }

    /// Dispatch a notification to all enabled integrations that handle the event.
    ///
    /// This is fire-and-forget - each integration is spawned as a separate task
    /// and errors are logged but not propagated.
    pub async fn notify(&self, event: NotificationEvent) {
        if !self.enabled {
            return;
        }

        for integration in &self.integrations {
            if integration.is_enabled() && integration.handles_event(&event) {
                let integration = integration.clone();
                let event = event.clone();

                // Fire-and-forget - spawn task and don't await
                tokio::spawn(async move {
                    if let Err(e) = integration.send(&event).await {
                        tracing::warn!(
                            integration = %integration.name(),
                            event = %event.event_type(),
                            error = %e,
                            "Notification delivery failed"
                        );
                    }
                });
            }
        }
    }

    /// Dispatch a notification synchronously (blocking).
    ///
    /// This is useful for contexts where async is not available.
    /// Only dispatches to OS integration (webhooks require async).
    pub fn notify_sync(&self, event: NotificationEvent) {
        if !self.enabled {
            return;
        }

        // For sync contexts, only dispatch to OS integration
        for integration in &self.integrations {
            if integration.is_enabled()
                && integration.handles_event(&event)
                && integration.name() == "os"
            {
                let integration = integration.clone();
                let event = event.clone();

                // Try to get current runtime handle
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    handle.spawn(async move {
                        let _ = integration.send(&event).await;
                    });
                } else {
                    // No runtime available - send synchronously via blocking task
                    tracing::debug!("No tokio runtime available for sync notification");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{NotificationsConfig, OsNotificationConfig, WebhookConfig};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Create a minimal test config
    fn make_test_config() -> Config {
        let mut config = Config::default();
        config.notifications = NotificationsConfig {
            enabled: true,
            os: OsNotificationConfig {
                enabled: true,
                sound: false,
                events: vec![],
            },
            webhook: None,
            webhooks: vec![],
            on_agent_start: true,
            on_agent_complete: true,
            on_agent_needs_input: true,
            on_pr_created: true,
            on_investigation_created: true,
            sound: false,
        };
        config
    }

    #[test]
    fn test_service_from_config() {
        let config = make_test_config();
        let service = NotificationService::from_config(&config).unwrap();

        assert!(service.is_enabled());
        assert_eq!(service.integration_count(), 1); // Just OS integration
    }

    #[test]
    fn test_service_disabled() {
        let service = NotificationService::disabled();

        assert!(!service.is_enabled());
        assert_eq!(service.integration_count(), 0);
    }

    #[test]
    fn test_service_with_webhooks() {
        let mut config = make_test_config();
        config.notifications.webhooks = vec![
            WebhookConfig {
                name: Some("slack".into()),
                enabled: true,
                url: "https://hooks.slack.com/test".into(),
                auth_type: None,
                token_env: None,
                username: None,
                password_env: None,
                events: Some(vec!["agent.completed".into()]),
            },
            WebhookConfig {
                name: Some("pagerduty".into()),
                enabled: true,
                url: "https://events.pagerduty.com/test".into(),
                auth_type: Some("bearer".into()),
                token_env: Some("PD_TOKEN".into()),
                username: None,
                password_env: None,
                events: Some(vec!["agent.failed".into()]),
            },
        ];

        let service = NotificationService::from_config(&config).unwrap();

        assert!(service.is_enabled());
        assert_eq!(service.integration_count(), 3); // OS + 2 webhooks
    }

    #[test]
    fn test_service_skips_disabled_webhooks() {
        let mut config = make_test_config();
        config.notifications.webhooks = vec![
            WebhookConfig {
                name: Some("enabled".into()),
                enabled: true,
                url: "https://example.com/1".into(),
                auth_type: None,
                token_env: None,
                username: None,
                password_env: None,
                events: None,
            },
            WebhookConfig {
                name: Some("disabled".into()),
                enabled: false, // Disabled
                url: "https://example.com/2".into(),
                auth_type: None,
                token_env: None,
                username: None,
                password_env: None,
                events: None,
            },
        ];

        let service = NotificationService::from_config(&config).unwrap();

        assert_eq!(service.integration_count(), 2); // OS + 1 enabled webhook
    }

    #[test]
    fn test_service_skips_empty_url_webhooks() {
        let mut config = make_test_config();
        config.notifications.webhooks = vec![WebhookConfig {
            name: Some("no-url".into()),
            enabled: true,
            url: "".into(), // Empty URL
            auth_type: None,
            token_env: None,
            username: None,
            password_env: None,
            events: None,
        }];

        let service = NotificationService::from_config(&config).unwrap();

        assert_eq!(service.integration_count(), 1); // Just OS
    }

    /// Mock integration for testing dispatch behavior
    struct MockIntegration {
        name: String,
        enabled: bool,
        events: Vec<String>,
        send_count: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
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

        async fn send(&self, _event: &NotificationEvent) -> Result<()> {
            self.send_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_notify_dispatches_to_matching_integrations() {
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        let service = NotificationService {
            integrations: vec![
                Arc::new(MockIntegration {
                    name: "all".into(),
                    enabled: true,
                    events: vec![], // All events
                    send_count: count1.clone(),
                }),
                Arc::new(MockIntegration {
                    name: "completed-only".into(),
                    enabled: true,
                    events: vec!["agent.completed".into()],
                    send_count: count2.clone(),
                }),
            ],
            enabled: true,
        };

        let event = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        service.notify(event).await;

        // Give spawned tasks time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert_eq!(count1.load(Ordering::SeqCst), 1); // "all" received it
        assert_eq!(count2.load(Ordering::SeqCst), 0); // "completed-only" filtered it out
    }

    #[tokio::test]
    async fn test_notify_skips_disabled_integrations() {
        let count = Arc::new(AtomicUsize::new(0));

        let service = NotificationService {
            integrations: vec![Arc::new(MockIntegration {
                name: "disabled".into(),
                enabled: false,
                events: vec![],
                send_count: count.clone(),
            })],
            enabled: true,
        };

        let event = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        service.notify(event).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_notify_respects_global_enabled() {
        let count = Arc::new(AtomicUsize::new(0));

        let service = NotificationService {
            integrations: vec![Arc::new(MockIntegration {
                name: "test".into(),
                enabled: true,
                events: vec![],
                send_count: count.clone(),
            })],
            enabled: false, // Globally disabled
        };

        let event = NotificationEvent::AgentStarted {
            project: "test".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "123".into(),
            session_name: "tmux".into(),
            launch_mode: None,
        };

        service.notify(event).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert_eq!(count.load(Ordering::SeqCst), 0);
    }
}
