//! Webhook notification integration.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Serialize;

use super::integration::NotificationIntegration;
use super::NotificationEvent;
use crate::config::WebhookConfig;

/// Webhook authentication type.
#[derive(Debug, Clone)]
pub enum WebhookAuth {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
}

/// Webhook notification integration.
///
/// Sends HTTP POST requests to configured endpoints when events occur.
pub struct WebhookIntegration {
    name: String,
    url: String,
    auth: WebhookAuth,
    subscribed_events: Vec<String>,
    enabled: bool,
    client: Client,
}

/// Webhook payload format.
#[derive(Debug, Serialize)]
struct WebhookPayload {
    /// Event type string (e.g., "agent.started")
    event: String,
    /// ISO 8601 timestamp
    timestamp: String,
    /// Event-specific data
    data: serde_json::Value,
}

impl WebhookIntegration {
    /// Create a new webhook integration from config.
    pub fn new(config: &WebhookConfig) -> Result<Self> {
        let auth = match config.auth_type.as_deref() {
            Some("bearer") => {
                let token_env = config.token_env.as_deref().unwrap_or("");
                let token = std::env::var(token_env).unwrap_or_default();
                if token.is_empty() && !token_env.is_empty() {
                    tracing::warn!(
                        webhook = config.name.as_deref().unwrap_or("unnamed"),
                        env_var = token_env,
                        "Bearer token environment variable is not set or empty"
                    );
                }
                WebhookAuth::Bearer { token }
            }
            Some("basic") => {
                let password_env = config.password_env.as_deref().unwrap_or("");
                let password = std::env::var(password_env).unwrap_or_default();
                if password.is_empty() && !password_env.is_empty() {
                    tracing::warn!(
                        webhook = config.name.as_deref().unwrap_or("unnamed"),
                        env_var = password_env,
                        "Basic auth password environment variable is not set or empty"
                    );
                }
                WebhookAuth::Basic {
                    username: config.username.clone().unwrap_or_default(),
                    password,
                }
            }
            _ => WebhookAuth::None,
        };

        Ok(Self {
            name: config.name.clone().unwrap_or_else(|| "webhook".to_string()),
            url: config.url.clone(),
            auth,
            subscribed_events: config.events.clone().unwrap_or_default(),
            enabled: config.enabled,
            client: Client::new(),
        })
    }

    /// Create a webhook for testing.
    #[cfg(test)]
    pub fn new_test(name: &str, url: &str, events: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            auth: WebhookAuth::None,
            subscribed_events: events,
            enabled: true,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl NotificationIntegration for WebhookIntegration {
    fn name(&self) -> &str {
        &self.name
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
        // Build payload
        let payload = WebhookPayload {
            event: event.event_type().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::to_value(event)?,
        };

        // Build request
        let mut request = self.client.post(&self.url).json(&payload);

        // Apply authentication
        request = match &self.auth {
            WebhookAuth::Bearer { token } => request.bearer_auth(token),
            WebhookAuth::Basic { username, password } => {
                request.basic_auth(username, Some(password))
            }
            WebhookAuth::None => request,
        };

        // Fire-and-forget with logging
        match request.send().await {
            Ok(response) if response.status().is_success() => {
                tracing::debug!(
                    webhook = %self.name,
                    event = %event.event_type(),
                    status = %response.status(),
                    "Webhook delivered successfully"
                );
            }
            Ok(response) => {
                tracing::warn!(
                    webhook = %self.name,
                    event = %event.event_type(),
                    status = %response.status(),
                    "Webhook returned non-success status"
                );
            }
            Err(e) => {
                tracing::warn!(
                    webhook = %self.name,
                    event = %event.event_type(),
                    error = %e,
                    "Webhook delivery failed"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config(
        name: &str,
        url: &str,
        events: Option<Vec<String>>,
        auth_type: Option<&str>,
    ) -> WebhookConfig {
        WebhookConfig {
            name: Some(name.to_string()),
            enabled: true,
            url: url.to_string(),
            auth_type: auth_type.map(String::from),
            token_env: Some("TEST_TOKEN".to_string()),
            username: Some("testuser".to_string()),
            password_env: Some("TEST_PASSWORD".to_string()),
            events,
        }
    }

    #[test]
    fn test_webhook_integration_from_config() {
        let config = make_test_config("test-hook", "https://example.com/webhook", None, None);
        let integration = WebhookIntegration::new(&config).unwrap();

        assert!(integration.is_enabled());
        assert_eq!(integration.name(), "test-hook");
    }

    #[test]
    fn test_webhook_handles_event_empty_filter() {
        let integration = WebhookIntegration::new_test("test", "https://example.com", vec![]);

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
    fn test_webhook_handles_event_specific_filter_match() {
        let integration = WebhookIntegration::new_test(
            "test",
            "https://example.com",
            vec!["agent.started".into()],
        );

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
    fn test_webhook_handles_event_specific_filter_no_match() {
        let integration = WebhookIntegration::new_test(
            "test",
            "https://example.com",
            vec!["agent.completed".into()],
        );

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
    fn test_webhook_payload_format() {
        let event = NotificationEvent::AgentStarted {
            project: "backend".into(),
            ticket_type: "FEAT".into(),
            ticket_id: "FEAT-042".into(),
            session_name: "op-backend-feat-042".into(),
            launch_mode: Some("docker".into()),
        };

        let payload = WebhookPayload {
            event: event.event_type().to_string(),
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            data: serde_json::to_value(&event).unwrap(),
        };

        let json = serde_json::to_string(&payload).unwrap();

        assert!(json.contains("\"event\":\"agent.started\""));
        assert!(json.contains("\"timestamp\":\"2024-01-15T10:30:00Z\""));
        assert!(json.contains("\"project\":\"backend\""));
    }

    #[test]
    fn test_auth_bearer_from_config() {
        // Set test env var
        std::env::set_var("TEST_BEARER_TOKEN", "secret123");

        let config = WebhookConfig {
            name: Some("test".into()),
            enabled: true,
            url: "https://example.com".into(),
            auth_type: Some("bearer".into()),
            token_env: Some("TEST_BEARER_TOKEN".into()),
            username: None,
            password_env: None,
            events: None,
        };

        let integration = WebhookIntegration::new(&config).unwrap();

        match &integration.auth {
            WebhookAuth::Bearer { token } => assert_eq!(token, "secret123"),
            _ => panic!("Expected Bearer auth"),
        }

        std::env::remove_var("TEST_BEARER_TOKEN");
    }

    #[test]
    fn test_auth_basic_from_config() {
        // Set test env var
        std::env::set_var("TEST_BASIC_PASSWORD", "password123");

        let config = WebhookConfig {
            name: Some("test".into()),
            enabled: true,
            url: "https://example.com".into(),
            auth_type: Some("basic".into()),
            token_env: None,
            username: Some("myuser".into()),
            password_env: Some("TEST_BASIC_PASSWORD".into()),
            events: None,
        };

        let integration = WebhookIntegration::new(&config).unwrap();

        match &integration.auth {
            WebhookAuth::Basic { username, password } => {
                assert_eq!(username, "myuser");
                assert_eq!(password, "password123");
            }
            _ => panic!("Expected Basic auth"),
        }

        std::env::remove_var("TEST_BASIC_PASSWORD");
    }
}
