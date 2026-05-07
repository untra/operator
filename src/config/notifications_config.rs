use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Notifications configuration with support for multiple integrations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct NotificationsConfig {
    /// Global enabled flag for all notifications
    pub enabled: bool,

    /// OS notification configuration
    #[serde(default)]
    pub os: OsNotificationConfig,

    /// Single webhook configuration (for simple setups)
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,

    /// Multiple webhook configurations
    #[serde(default)]
    pub webhooks: Vec<WebhookConfig>,

    // Legacy fields for backwards compatibility
    // These are deprecated but still supported for existing configs
    #[serde(default = "default_true")]
    #[schemars(skip)]
    #[ts(skip)]
    pub on_agent_start: bool,
    #[serde(default = "default_true")]
    #[schemars(skip)]
    #[ts(skip)]
    pub on_agent_complete: bool,
    #[serde(default = "default_true")]
    #[schemars(skip)]
    #[ts(skip)]
    pub on_agent_needs_input: bool,
    #[serde(default = "default_true")]
    #[schemars(skip)]
    #[ts(skip)]
    pub on_pr_created: bool,
    #[serde(default = "default_true")]
    #[schemars(skip)]
    #[ts(skip)]
    pub on_investigation_created: bool,
    #[serde(default)]
    #[schemars(skip)]
    #[ts(skip)]
    pub sound: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            os: OsNotificationConfig::default(),
            webhook: None,
            webhooks: Vec::new(),
            // Legacy fields
            on_agent_start: true,
            on_agent_complete: true,
            on_agent_needs_input: true,
            on_pr_created: true,
            on_investigation_created: true,
            sound: false,
        }
    }
}

/// OS notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct OsNotificationConfig {
    /// Whether OS notifications are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Play sound with notifications
    #[serde(default)]
    pub sound: bool,

    /// Events to send (empty = all events)
    /// Possible values: agent.started, agent.completed, agent.failed,
    /// `agent.awaiting_input`, `agent.session_lost`, pr.created, pr.merged,
    /// pr.closed, `pr.ready_to_merge`, `pr.changes_requested`,
    /// ticket.returned, investigation.created
    #[serde(default)]
    pub events: Vec<String>,
}

impl Default for OsNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: false,
            events: Vec::new(), // All events
        }
    }
}

/// Webhook notification configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct WebhookConfig {
    /// Optional name for this webhook (for logging)
    #[serde(default)]
    pub name: Option<String>,

    /// Whether this webhook is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Webhook URL
    #[serde(default)]
    pub url: String,

    /// Authentication type: "bearer" or "basic"
    #[serde(default)]
    pub auth_type: Option<String>,

    /// Environment variable containing the bearer token
    #[serde(default)]
    pub token_env: Option<String>,

    /// Username for basic auth
    #[serde(default)]
    pub username: Option<String>,

    /// Environment variable containing the password for basic auth
    #[serde(default)]
    pub password_env: Option<String>,

    /// Events to send (empty = all events)
    #[serde(default)]
    pub events: Option<Vec<String>>,
}
