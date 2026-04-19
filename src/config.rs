use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysinfo::System;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Config {
    /// List of projects operator can assign work to
    #[serde(default)]
    pub projects: Vec<String>,
    pub agents: AgentsConfig,
    pub notifications: NotificationsConfig,
    pub queue: QueueConfig,
    pub paths: PathsConfig,
    pub ui: UiConfig,
    pub launch: LaunchConfig,
    pub templates: TemplatesConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub tmux: TmuxConfig,
    /// Session wrapper configuration (tmux, vscode, or cmux)
    #[serde(default)]
    pub sessions: SessionsConfig,
    #[serde(default)]
    pub llm_tools: LlmToolsConfig,
    #[serde(default)]
    pub backstage: BackstageConfig,
    #[serde(default)]
    pub rest_api: RestApiConfig,
    #[serde(default)]
    pub git: GitConfig,
    /// Kanban provider configuration for syncing issues from Jira, Linear, etc.
    #[serde(default)]
    pub kanban: KanbanConfig,
    /// Version check configuration for automatic update notifications
    #[serde(default)]
    pub version_check: VersionCheckConfig,
    /// Agent delegator configurations for autonomous ticket launching
    #[serde(default)]
    pub delegators: Vec<Delegator>,
    /// User-declared model servers (ollama, lmstudio, any OpenAI-compat host).
    /// Implicit builtin servers exist for each `llm_tool`'s vendor API and do not need declaration.
    #[serde(default)]
    pub model_servers: Vec<ModelServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct AgentsConfig {
    pub max_parallel: usize,
    pub cores_reserved: usize,
    pub health_check_interval: u64,
    /// Timeout in seconds for each agent generation (default: 300 = 5 min)
    #[serde(default = "default_generation_timeout")]
    pub generation_timeout_secs: u64,
    /// Interval in seconds between ticket-session syncs (default: 60)
    #[serde(default = "default_sync_interval")]
    pub sync_interval: u64,
    /// Maximum seconds a step can run before timing out (default: 1800 = 30 min)
    #[serde(default = "default_step_timeout")]
    pub step_timeout: u64,
    /// Seconds of tmux silence before considering agent awaiting input (default: 30)
    #[serde(default = "default_silence_threshold")]
    pub silence_threshold: u64,
}

fn default_generation_timeout() -> u64 {
    300 // 5 minutes
}

fn default_sync_interval() -> u64 {
    60 // 1 minute
}

fn default_step_timeout() -> u64 {
    1800 // 30 minutes
}

fn default_silence_threshold() -> u64 {
    6 // 6 seconds
}

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct QueueConfig {
    pub auto_assign: bool,
    pub priority_order: Vec<String>,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct PathsConfig {
    pub tickets: String,
    pub projects: String,
    pub state: String,
    /// Base directory for per-ticket worktrees (default: ~/.operator/worktrees)
    #[serde(default = "default_worktrees_dir")]
    pub worktrees: String,
}

fn default_worktrees_dir() -> String {
    dirs::home_dir().map_or_else(
        || ".operator/worktrees".to_string(),
        |h| h.join(".operator/worktrees").to_string_lossy().to_string(),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct UiConfig {
    pub refresh_rate_ms: u64,
    pub completed_history_hours: u64,
    pub summary_max_length: usize,
    #[serde(default)]
    pub panel_names: PanelNamesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct PanelNamesConfig {
    #[serde(default = "default_status_name")]
    pub status: String,
    #[serde(default = "default_todo_name")]
    pub queue: String,
    #[serde(default = "default_in_progress_name", alias = "agents")]
    pub in_progress: String,
    #[serde(default = "default_done_name")]
    pub completed: String,
}

fn default_status_name() -> String {
    "STATUS".to_string()
}

fn default_todo_name() -> String {
    "TODO QUEUE".to_string()
}

fn default_in_progress_name() -> String {
    "IN PROGRESS".to_string()
}

fn default_done_name() -> String {
    "DONE".to_string()
}

impl Default for PanelNamesConfig {
    fn default() -> Self {
        Self {
            status: default_status_name(),
            queue: default_todo_name(),
            in_progress: default_in_progress_name(),
            completed: default_done_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct LaunchConfig {
    pub confirm_autonomous: bool,
    pub confirm_paired: bool,
    pub launch_delay_ms: u64,
    /// Docker execution configuration
    #[serde(default)]
    pub docker: DockerConfig,
    /// YOLO (auto-accept) mode configuration
    #[serde(default)]
    pub yolo: YoloConfig,
}

/// Docker execution configuration for running agents in containers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct DockerConfig {
    /// Whether docker mode option is available in launch dialog
    #[serde(default)]
    pub enabled: bool,
    /// Docker image to use (required if enabled)
    #[serde(default = "default_docker_image")]
    pub image: String,
    /// Additional docker run arguments
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Container mount path for the project (default: /workspace)
    #[serde(default = "default_mount_path")]
    pub mount_path: String,
    /// Environment variables to pass through to the container
    #[serde(default)]
    pub env_vars: Vec<String>,
}

fn default_docker_image() -> String {
    String::new()
}

fn default_mount_path() -> String {
    "/workspace".to_string()
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            image: default_docker_image(),
            extra_args: Vec::new(),
            mount_path: default_mount_path(),
            env_vars: Vec::new(),
        }
    }
}

/// YOLO (auto-accept) mode configuration for fully autonomous execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, TS)]
#[ts(export)]
pub struct YoloConfig {
    /// Whether YOLO mode option is available in launch dialog
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, TS)]
#[ts(export)]
pub struct TmuxConfig {
    /// Whether custom tmux config has been generated
    #[serde(default)]
    pub config_generated: bool,
}

/// Session wrapper type for terminal session management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SessionWrapperType {
    /// Standalone tmux sessions (default)
    #[default]
    Tmux,
    /// VS Code integrated terminal (via extension webhook)
    Vscode,
    /// cmux macOS terminal multiplexer
    Cmux,
    /// Zellij terminal workspace manager
    Zellij,
}

impl SessionWrapperType {
    /// Short display name for the wrapper (used in header bar, logs)
    pub fn display_name(&self) -> &'static str {
        match self {
            SessionWrapperType::Tmux => "tmux",
            SessionWrapperType::Vscode => "vscode",
            SessionWrapperType::Cmux => "cmux",
            SessionWrapperType::Zellij => "zellij",
        }
    }
}

impl std::fmt::Display for SessionWrapperType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Session wrapper configuration
///
/// Controls how operator creates and manages terminal sessions for agents.
/// Four modes are supported:
/// - tmux: Standalone tmux sessions (default)
/// - vscode: VS Code integrated terminal (requires extension)
/// - cmux: macOS terminal multiplexer (requires running inside cmux)
/// - zellij: Zellij terminal workspace manager
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SessionsConfig {
    /// Which session wrapper to use
    #[serde(default)]
    pub wrapper: SessionWrapperType,

    /// Tmux-specific configuration
    #[serde(default)]
    pub tmux: SessionsTmuxConfig,

    /// VS Code-specific configuration
    #[serde(default)]
    pub vscode: SessionsVSCodeConfig,

    /// cmux-specific configuration
    #[serde(default)]
    pub cmux: SessionsCmuxConfig,

    /// Zellij-specific configuration
    #[serde(default)]
    pub zellij: SessionsZellijConfig,
}

impl Default for SessionsConfig {
    fn default() -> Self {
        Self {
            wrapper: SessionWrapperType::Tmux,
            tmux: SessionsTmuxConfig::default(),
            vscode: SessionsVSCodeConfig::default(),
            cmux: SessionsCmuxConfig::default(),
            zellij: SessionsZellijConfig::default(),
        }
    }
}

/// Tmux-specific session configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SessionsTmuxConfig {
    /// Whether custom tmux config has been generated
    #[serde(default)]
    pub config_generated: bool,

    /// Socket name for session isolation
    #[serde(default = "default_socket_name")]
    pub socket_name: String,
}

fn default_socket_name() -> String {
    "operator".to_string()
}

impl Default for SessionsTmuxConfig {
    fn default() -> Self {
        Self {
            config_generated: false,
            socket_name: default_socket_name(),
        }
    }
}

/// VS Code extension session configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SessionsVSCodeConfig {
    /// Port for extension webhook server
    #[serde(default = "default_vscode_webhook_port")]
    pub webhook_port: u16,

    /// Connection timeout in milliseconds
    #[serde(default = "default_vscode_connect_timeout")]
    pub connect_timeout_ms: u64,
}

fn default_vscode_webhook_port() -> u16 {
    7009
}

fn default_vscode_connect_timeout() -> u64 {
    5000
}

impl Default for SessionsVSCodeConfig {
    fn default() -> Self {
        Self {
            webhook_port: default_vscode_webhook_port(),
            connect_timeout_ms: default_vscode_connect_timeout(),
        }
    }
}

/// Placement policy for cmux sessions: where to create new agent terminals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum CmuxPlacementPolicy {
    /// Automatically choose: 0-1 windows → new workspace, >1 windows → new window
    #[default]
    Auto,
    /// Always create a new workspace in the active window
    Workspace,
    /// Always create a new window for each ticket
    Window,
}

impl std::fmt::Display for CmuxPlacementPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmuxPlacementPolicy::Auto => write!(f, "auto"),
            CmuxPlacementPolicy::Workspace => write!(f, "workspace"),
            CmuxPlacementPolicy::Window => write!(f, "window"),
        }
    }
}

/// cmux macOS terminal multiplexer session configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SessionsCmuxConfig {
    /// Path to the cmux binary
    #[serde(default = "default_cmux_binary_path")]
    pub binary_path: String,

    /// Require running inside cmux (`CMUX_WORKSPACE_ID` env var present)
    #[serde(default = "default_true_val")]
    pub require_in_cmux: bool,

    /// Where to place new agent sessions: "auto", "workspace", or "window"
    #[serde(default)]
    pub placement: CmuxPlacementPolicy,
}

fn default_cmux_binary_path() -> String {
    "/Applications/cmux.app/Contents/Resources/bin/cmux".to_string()
}

fn default_true_val() -> bool {
    true
}

impl Default for SessionsCmuxConfig {
    fn default() -> Self {
        Self {
            binary_path: default_cmux_binary_path(),
            require_in_cmux: default_true_val(),
            placement: CmuxPlacementPolicy::default(),
        }
    }
}

/// Zellij terminal workspace manager session configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SessionsZellijConfig {
    /// Require running inside Zellij (ZELLIJ env var present)
    #[serde(default = "default_true_val")]
    pub require_in_zellij: bool,
}

impl Default for SessionsZellijConfig {
    fn default() -> Self {
        Self {
            require_in_zellij: default_true_val(),
        }
    }
}

/// Backstage integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct BackstageConfig {
    /// Whether Backstage integration is enabled
    #[serde(default = "default_backstage_enabled")]
    pub enabled: bool,
    /// Whether to show Backstage in the Connections status section
    #[serde(default)]
    pub display: bool,
    /// Port for the Backstage server
    #[serde(default = "default_backstage_port")]
    pub port: u16,
    /// Auto-start Backstage server when TUI launches
    #[serde(default)]
    pub auto_start: bool,
    /// Subdirectory within `state_path` for Backstage installation
    #[serde(default = "default_backstage_subpath")]
    pub subpath: String,
    /// Subdirectory within backstage path for branding customization
    #[serde(default = "default_branding_subpath")]
    pub branding_subpath: String,
    /// Base URL for downloading backstage-server binary
    #[serde(default = "default_backstage_release_url")]
    pub release_url: String,
    /// Optional local path to backstage-server binary
    /// If set, this is used instead of downloading from `release_url`
    #[serde(default)]
    pub local_binary_path: Option<String>,
    /// Branding and theming configuration
    #[serde(default)]
    pub branding: BrandingConfig,
}

/// Branding configuration for Backstage portal
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct BrandingConfig {
    /// App title shown in header
    #[serde(default = "default_app_title")]
    pub app_title: String,
    /// Organization name
    #[serde(default = "default_org_name")]
    pub org_name: String,
    /// Path to logo SVG (relative to branding path)
    #[serde(default)]
    pub logo_path: Option<String>,
    /// Theme colors (uses Operator defaults if not set)
    #[serde(default)]
    pub colors: ThemeColors,
}

fn default_app_title() -> String {
    "Operator Portal".to_string()
}

fn default_org_name() -> String {
    "Operator".to_string()
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            app_title: default_app_title(),
            org_name: default_org_name(),
            logo_path: Some("logo.svg".to_string()),
            colors: ThemeColors::default(),
        }
    }
}

/// Theme color configuration for Backstage
/// Default colors match Operator's tmux theme
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ThemeColors {
    /// Primary/accent color (default: salmon #cc6c55)
    #[serde(default = "default_color_primary")]
    pub primary: String,
    /// Secondary color (default: dark teal #114145)
    #[serde(default = "default_color_secondary")]
    pub secondary: String,
    /// Accent/highlight color (default: cream #f4dbb7)
    #[serde(default = "default_color_accent")]
    pub accent: String,
    /// Warning/error color (default: coral #d46048)
    #[serde(default = "default_color_warning")]
    pub warning: String,
    /// Muted text color (default: darker salmon #8a4a3a)
    #[serde(default = "default_color_muted")]
    pub muted: String,
}

fn default_color_primary() -> String {
    "#cc6c55".to_string() // salmon
}

fn default_color_secondary() -> String {
    "#114145".to_string() // dark teal
}

fn default_color_accent() -> String {
    "#f4dbb7".to_string() // cream
}

fn default_color_warning() -> String {
    "#d46048".to_string() // coral
}

fn default_color_muted() -> String {
    "#8a4a3a".to_string() // darker salmon
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: default_color_primary(),
            secondary: default_color_secondary(),
            accent: default_color_accent(),
            warning: default_color_warning(),
            muted: default_color_muted(),
        }
    }
}

fn default_backstage_enabled() -> bool {
    true
}

fn default_backstage_port() -> u16 {
    7007
}

fn default_backstage_subpath() -> String {
    "backstage".to_string()
}

fn default_branding_subpath() -> String {
    "branding".to_string()
}

fn default_backstage_release_url() -> String {
    "https://github.com/untra/operator/releases/latest/download".to_string()
}

impl Default for BackstageConfig {
    fn default() -> Self {
        Self {
            enabled: default_backstage_enabled(),
            display: false,
            port: default_backstage_port(),
            auto_start: false,
            subpath: default_backstage_subpath(),
            branding_subpath: default_branding_subpath(),
            release_url: default_backstage_release_url(),
            local_binary_path: None,
            branding: BrandingConfig::default(),
        }
    }
}

/// REST API server configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct RestApiConfig {
    /// Whether the REST API is enabled
    #[serde(default = "default_rest_enabled")]
    pub enabled: bool,
    /// Port for the REST API server
    #[serde(default = "default_rest_port")]
    pub port: u16,
    /// CORS allowed origins (empty = allow all)
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

fn default_rest_enabled() -> bool {
    true
}

fn default_rest_port() -> u16 {
    7008
}

impl Default for RestApiConfig {
    fn default() -> Self {
        Self {
            enabled: default_rest_enabled(),
            port: default_rest_port(),
            cors_origins: Vec::new(),
        }
    }
}

/// LLM CLI tools configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, TS)]
#[ts(export)]
pub struct LlmToolsConfig {
    /// Detected CLI tools (populated on first startup)
    #[serde(default)]
    pub detected: Vec<DetectedTool>,

    /// Available {tool, model} pairs for launching tickets
    /// Built from detected tools + their model aliases
    #[serde(default)]
    pub providers: Vec<LlmProvider>,

    /// Whether detection has been completed
    #[serde(default)]
    pub detection_complete: bool,

    /// User's preferred default LLM tool (e.g., "claude")
    #[serde(default)]
    pub default_tool: Option<String>,

    /// User's preferred default model alias (e.g., "opus")
    #[serde(default)]
    pub default_model: Option<String>,

    /// Per-tool overrides for skill directories (keyed by `tool_name`)
    #[serde(default)]
    pub skill_directory_overrides: std::collections::HashMap<String, SkillDirectoriesOverride>,
}

/// A detected CLI tool (e.g., claude binary)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DetectedTool {
    /// Tool name (e.g., "claude")
    pub name: String,
    /// Path to the binary
    pub path: String,
    /// Version string
    pub version: String,
    /// Minimum required version for Operator compatibility
    #[serde(default)]
    pub min_version: Option<String>,
    /// Whether the installed version meets the minimum requirement
    #[serde(default)]
    pub version_ok: bool,
    /// Available model aliases (e.g., ["opus", "sonnet", "haiku"])
    #[serde(default)]
    pub model_aliases: Vec<String>,
    /// Command template with {{model}}, {{`session_id`}}, {{`prompt_file`}} placeholders
    #[serde(default)]
    pub command_template: String,
    /// Tool capabilities
    #[serde(default)]
    pub capabilities: ToolCapabilities,
    /// CLI flags for YOLO (auto-accept) mode
    #[serde(default)]
    pub yolo_flags: Vec<String>,
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ToolCapabilities {
    /// Whether the tool supports session continuity via UUID
    #[serde(default)]
    pub supports_sessions: bool,
    /// Whether the tool can run in headless/non-interactive mode
    #[serde(default)]
    pub supports_headless: bool,
}

/// A {tool, model} pair that can be selected when launching tickets.
/// Includes optional variant fields adopted from vibe-kanban's profile system.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema, TS)]
#[ts(export)]
pub struct LlmProvider {
    /// CLI tool name (e.g., "claude", "codex", "gemini")
    pub tool: String,
    /// Model alias or name (e.g., "opus", "sonnet", "gpt-4.1")
    pub model: String,
    /// Optional display name for UI (e.g., "Claude Opus", "Codex High")
    #[serde(default)]
    pub display_name: Option<String>,

    // ─── Variant fields (all optional) ───────────────────────────────
    /// Additional CLI flags for this provider (e.g., ["--dangerously-skip-permissions"])
    #[serde(default)]
    pub flags: Vec<String>,

    /// Environment variables to set when launching
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Whether this provider requires approval gates
    #[serde(default)]
    pub approvals: bool,

    /// Whether to run in plan-only mode
    #[serde(default)]
    pub plan_only: bool,

    /// Reasoning effort level (Codex: "low", "medium", "high")
    #[serde(default)]
    pub reasoning_effort: Option<String>,

    /// Sandbox mode (Codex: "danger-full-access", "workspace-write")
    #[serde(default)]
    pub sandbox: Option<String>,
}

/// Per-tool skill directory overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SkillDirectoriesOverride {
    /// Additional global skill directories
    #[serde(default)]
    pub global: Vec<String>,
    /// Additional project-relative skill directories
    #[serde(default)]
    pub project: Vec<String>,
}

/// Agent delegator configuration for autonomous ticket launching
///
/// A delegator is a named {tool, model} pairing with optional launch configuration
/// that can be used to launch agents for tickets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Delegator {
    /// Unique name for this delegator (e.g., "claude-opus-auto")
    pub name: String,
    /// LLM tool name (must match a detected tool, e.g., "claude", "codex")
    pub llm_tool: String,
    /// Model alias (e.g., "opus", "sonnet", "gpt-4o")
    pub model: String,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
    /// Arbitrary model properties (e.g., `reasoning_effort`, sandbox)
    #[serde(default)]
    pub model_properties: std::collections::HashMap<String, String>,
    /// Optional launch configuration
    #[serde(default)]
    pub launch_config: Option<DelegatorLaunchConfig>,
    /// Name of a declared `ModelServer` (from `Config.model_servers`).
    /// `None` means use the `llm_tool`'s implicit vendor default
    /// (claude → anthropic-api, codex → openai-api, gemini → google-api).
    #[serde(default)]
    pub model_server: Option<String>,
}

/// A named host that serves models via an inference API.
///
/// Model servers are orthogonal to `llm_tools`: a delegator pairs an agentic CLI
/// (`llm_tool`, e.g. claude/codex/gemini) with a model-serving endpoint
/// (`model_server`, e.g. ollama-local, openai-api, a custom vllm host).
///
/// Implicit builtin servers (`anthropic-api`, `openai-api`, `google-api`) are
/// returned by [`implicit_model_server_for_tool`] and do not need to be declared
/// in config.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ModelServer {
    /// Unique name (e.g., "ollama-local", "vllm-gpu1")
    pub name: String,
    /// Kind: "ollama", "openai-compat", "anthropic-api", "openai-api", "google-api", "lmstudio"
    pub kind: String,
    /// Base URL of the inference endpoint (e.g., `http://localhost:11434`).
    /// `None` for implicit vendor servers means use the SDK default.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Name of an env var providing the API key (e.g., `OLLAMA_API_KEY`)
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Additional environment variables set when spawning agents that use this server
    #[serde(default)]
    pub extra_env: std::collections::HashMap<String, String>,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Returns the implicit builtin `ModelServer` associated with a given `llm_tool`.
///
/// Used when a `Delegator` has no explicit `model_server`. Unknown tools
/// fall back to an `"openai-api"` server so arbitrary future tools still resolve.
pub fn implicit_model_server_for_tool(tool: &str) -> ModelServer {
    let (name, kind) = match tool {
        "claude" => ("anthropic-api", "anthropic-api"),
        "codex" => ("openai-api", "openai-api"),
        "gemini" => ("google-api", "google-api"),
        _ => ("openai-api", "openai-api"),
    };
    ModelServer {
        name: name.to_string(),
        kind: kind.to_string(),
        base_url: None,
        api_key_env: None,
        extra_env: std::collections::HashMap::new(),
        display_name: None,
    }
}

/// Launch configuration for a delegator
///
/// Controls how the delegator launches agents. Optional fields use tri-state
/// semantics: `None` = inherit from global config, `Some(true/false)` = override.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct DelegatorLaunchConfig {
    /// Run in YOLO (auto-accept) mode
    #[serde(default)]
    pub yolo: bool,
    /// Permission mode override
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Additional CLI flags
    #[serde(default)]
    pub flags: Vec<String>,
    /// Override global `git.use_worktrees` per-delegator (None = use global setting)
    #[serde(default)]
    pub use_worktrees: Option<bool>,
    /// Whether to create a git branch for the ticket (None = default behavior)
    #[serde(default)]
    pub create_branch: Option<bool>,
    /// Run in docker container (None = use global `launch.docker.enabled`)
    #[serde(default)]
    pub docker: Option<bool>,
    /// Prompt text to prepend before the generated step prompt
    #[serde(default)]
    pub prompt_prefix: Option<String>,
    /// Prompt text to append after the generated step prompt
    #[serde(default)]
    pub prompt_suffix: Option<String>,
}

/// Predefined issue type collections
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum CollectionPreset {
    /// Simple tasks only: TASK
    Simple,
    /// Developer kanban: TASK, FEAT, FIX
    #[default]
    DevKanban,
    /// DevOps kanban: TASK, SPIKE, INV, FEAT, FIX
    DevopsKanban,
    /// Custom collection (use the collection field)
    Custom,
}

impl CollectionPreset {
    /// Get the issue types for this preset
    pub fn issue_types(&self) -> Vec<String> {
        match self {
            CollectionPreset::Simple => vec!["TASK".to_string()],
            CollectionPreset::DevKanban => {
                vec!["TASK".to_string(), "FEAT".to_string(), "FIX".to_string()]
            }
            CollectionPreset::DevopsKanban => vec![
                "TASK".to_string(),
                "SPIKE".to_string(),
                "INV".to_string(),
                "FEAT".to_string(),
                "FIX".to_string(),
            ],
            CollectionPreset::Custom => Vec::new(), // Use collection field
        }
    }

    /// Get display name for this preset
    pub fn display_name(&self) -> &'static str {
        match self {
            CollectionPreset::Simple => "Simple (TASK only)",
            CollectionPreset::DevKanban => "Dev Kanban (TASK, FEAT, FIX)",
            CollectionPreset::DevopsKanban => "DevOps Kanban (TASK, SPIKE, INV, FEAT, FIX)",
            CollectionPreset::Custom => "Custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct TemplatesConfig {
    /// Named preset for issue type collection
    /// Options: simple, `dev_kanban`, `devops_kanban`, custom
    #[serde(default)]
    pub preset: CollectionPreset,
    /// Custom issuetype collection (only used when preset = custom)
    /// List of issue type keys: TASK, FEAT, FIX, SPIKE, INV
    #[serde(default)]
    pub collection: Vec<String>,
    /// Active collection name (overrides preset if set)
    /// Can be a builtin preset name or a user-defined collection
    #[serde(default)]
    pub active_collection: Option<String>,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            preset: CollectionPreset::DevKanban,
            collection: Vec::new(),
            active_collection: None,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct LoggingConfig {
    /// Log level filter (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Whether to log to file in TUI mode (false = stderr for debugging)
    #[serde(default = "default_log_to_file")]
    pub to_file: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_to_file() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            to_file: default_log_to_file(),
        }
    }
}

/// API integrations configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ApiConfig {
    /// Interval in seconds between PR status checks (default: 60)
    #[serde(default = "default_pr_check_interval")]
    pub pr_check_interval_secs: u64,
    /// Interval in seconds between rate limit checks (default: 300)
    #[serde(default = "default_rate_limit_check_interval")]
    pub rate_limit_check_interval_secs: u64,
    /// Show warning when rate limit remaining is below this percentage (default: 0.2)
    #[serde(default = "default_rate_limit_warning_threshold")]
    pub rate_limit_warning_threshold: f32,
}

fn default_pr_check_interval() -> u64 {
    60 // 1 minute
}

fn default_rate_limit_check_interval() -> u64 {
    300 // 5 minutes
}

fn default_rate_limit_warning_threshold() -> f32 {
    0.2 // 20%
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            pr_check_interval_secs: default_pr_check_interval(),
            rate_limit_check_interval_secs: default_rate_limit_check_interval(),
            rate_limit_warning_threshold: default_rate_limit_warning_threshold(),
        }
    }
}

// ─── Kanban Provider Configuration ─────────────────────────────────────────

/// Kanban provider configuration for syncing issues from external systems
///
/// Providers are keyed by domain/workspace:
/// - Jira: keyed by domain (e.g., "foobar.atlassian.net")
/// - Linear: keyed by workspace slug (e.g., "myworkspace")
/// - GitHub Projects: keyed by owner login (e.g., "my-org")
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct KanbanConfig {
    /// Jira Cloud instances keyed by domain (e.g., "foobar.atlassian.net")
    #[serde(default)]
    pub jira: std::collections::HashMap<String, JiraConfig>,
    /// Linear instances keyed by workspace slug
    #[serde(default)]
    pub linear: std::collections::HashMap<String, LinearConfig>,
    /// GitHub Projects v2 instances keyed by owner login (user or org)
    ///
    /// NOTE: This is the *kanban* GitHub integration (Projects v2), distinct
    /// from `GitHubConfig` which is the *git provider* used for PRs and
    /// branches. The two use different env vars and different scopes — see
    /// `docs/getting-started/kanban/github.md` for the full disambiguation.
    #[serde(default)]
    pub github: std::collections::HashMap<String, GithubProjectsConfig>,
}

/// Jira Cloud provider configuration
///
/// The domain is specified as the `HashMap` key in KanbanConfig.jira
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name containing the API key (default: `OPERATOR_JIRA_API_KEY`)
    #[serde(default = "default_jira_api_key_env")]
    pub api_key_env: String,
    /// Atlassian account email for authentication
    #[serde(default)]
    pub email: String,
    /// Per-project sync configuration
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectSyncConfig>,
}

fn default_jira_api_key_env() -> String {
    "OPERATOR_JIRA_API_KEY".to_string()
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_jira_api_key_env(),
            email: String::new(),
            projects: std::collections::HashMap::new(),
        }
    }
}

/// Linear provider configuration
///
/// The workspace slug is specified as the `HashMap` key in KanbanConfig.linear
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct LinearConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name containing the API key (default: `OPERATOR_LINEAR_API_KEY`)
    #[serde(default = "default_linear_api_key_env")]
    pub api_key_env: String,
    /// Per-team sync configuration
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectSyncConfig>,
}

fn default_linear_api_key_env() -> String {
    "OPERATOR_LINEAR_API_KEY".to_string()
}

impl Default for LinearConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_linear_api_key_env(),
            projects: std::collections::HashMap::new(),
        }
    }
}

/// GitHub Projects v2 (kanban) provider configuration
///
/// The owner login (user or org) is specified as the `HashMap` key in
/// `KanbanConfig.github`. Project keys inside `projects` are `GraphQL` node
/// IDs (e.g., `PVT_kwDOABcdefg`) — opaque, stable identifiers used directly
/// by every GitHub Projects v2 mutation without needing a lookup.
///
/// **Distinct from `GitHubConfig`** (the git provider used for PR/branch
/// operations). They live in different parts of the config tree, use
/// different env vars (`OPERATOR_GITHUB_TOKEN` vs `GITHUB_TOKEN`), and
/// require different OAuth scopes (`project` vs `repo`). See
/// `docs/getting-started/kanban/github.md` for the full rationale.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct GithubProjectsConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name containing the GitHub token (default:
    /// `OPERATOR_GITHUB_TOKEN`). The token must have `project` (or
    /// `read:project`) scope, NOT just `repo` — see the disambiguation
    /// guide in the kanban github docs.
    #[serde(default = "default_github_projects_api_key_env")]
    pub api_key_env: String,
    /// Per-project sync configuration. Keys are `GraphQL` project node IDs.
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectSyncConfig>,
}

fn default_github_projects_api_key_env() -> String {
    "OPERATOR_GITHUB_TOKEN".to_string()
}

impl Default for GithubProjectsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_github_projects_api_key_env(),
            projects: std::collections::HashMap::new(),
        }
    }
}

impl KanbanConfig {
    /// Insert or update a Jira project entry in the config.
    ///
    /// If the workspace (keyed by domain) doesn't exist, it is created with
    /// `enabled = true` and the provided email + `api_key_env`. If it already
    /// exists, the email and `api_key_env` are updated and the project is
    /// upserted into its `projects` map without clobbering sibling projects.
    pub fn upsert_jira_project(
        &mut self,
        domain: &str,
        email: &str,
        api_key_env: &str,
        project_key: &str,
        sync_user_id: &str,
    ) {
        let entry = self.jira.entry(domain.to_string()).or_default();
        entry.enabled = true;
        entry.email = email.to_string();
        entry.api_key_env = api_key_env.to_string();
        entry.projects.insert(
            project_key.to_string(),
            ProjectSyncConfig {
                sync_user_id: sync_user_id.to_string(),
                sync_statuses: Vec::new(),
                collection_name: None,
                type_mappings: std::collections::HashMap::new(),
            },
        );
    }

    /// Insert or update a Linear team entry in the config.
    ///
    /// If the workspace (keyed by workspace slug) doesn't exist, it is
    /// created with `enabled = true` and the provided `api_key_env`. If it
    /// already exists, the `api_key_env` is updated and the project/team is
    /// upserted into its `projects` map without clobbering siblings.
    pub fn upsert_linear_project(
        &mut self,
        workspace: &str,
        api_key_env: &str,
        project_key: &str,
        sync_user_id: &str,
    ) {
        let entry = self.linear.entry(workspace.to_string()).or_default();
        entry.enabled = true;
        entry.api_key_env = api_key_env.to_string();
        entry.projects.insert(
            project_key.to_string(),
            ProjectSyncConfig {
                sync_user_id: sync_user_id.to_string(),
                sync_statuses: Vec::new(),
                collection_name: None,
                type_mappings: std::collections::HashMap::new(),
            },
        );
    }

    /// Insert or update a GitHub Projects v2 entry in the config.
    ///
    /// If the owner (keyed by login) doesn't exist, it is created with
    /// `enabled = true` and the provided `api_key_env`. If it already
    /// exists, the `api_key_env` is updated and the project is upserted
    /// into its `projects` map without clobbering siblings.
    ///
    /// `project_key` is the `GraphQL` project node ID (e.g., `PVT_kwDO...`)
    /// and `sync_user_id` is the user's numeric GitHub `databaseId`.
    pub fn upsert_github_project(
        &mut self,
        owner: &str,
        api_key_env: &str,
        project_key: &str,
        sync_user_id: &str,
    ) {
        let entry = self.github.entry(owner.to_string()).or_default();
        entry.enabled = true;
        entry.api_key_env = api_key_env.to_string();
        entry.projects.insert(
            project_key.to_string(),
            ProjectSyncConfig {
                sync_user_id: sync_user_id.to_string(),
                sync_statuses: Vec::new(),
                collection_name: None,
                type_mappings: std::collections::HashMap::new(),
            },
        );
    }

    /// Provider-neutral upsert dispatcher.
    ///
    /// Delegates to the provider-specific upsert method based on the
    /// `WorkspaceExtra` variant in the validated workspace.
    #[allow(dead_code)] // Will be used by onboarding service in Phase 1b
    pub fn upsert_project(
        &mut self,
        workspace: &crate::api::providers::kanban::ValidatedWorkspace,
        project: &crate::api::providers::kanban::DiscoveredProject,
    ) {
        use crate::api::providers::kanban::WorkspaceExtra;
        match &workspace.extra {
            WorkspaceExtra::Jira { email } => self.upsert_jira_project(
                &workspace.workspace_key,
                email,
                &workspace.api_key_env,
                &project.project_key,
                &workspace.sync_user_id,
            ),
            WorkspaceExtra::Linear => self.upsert_linear_project(
                &workspace.workspace_key,
                &workspace.api_key_env,
                &project.project_key,
                &workspace.sync_user_id,
            ),
            WorkspaceExtra::Github => self.upsert_github_project(
                &workspace.workspace_key,
                &workspace.api_key_env,
                &project.project_key,
                &workspace.sync_user_id,
            ),
        }
    }
}

/// Per-project/team sync configuration for a kanban provider
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ProjectSyncConfig {
    /// User ID to sync issues for (provider-specific format)
    /// - Jira: accountId (e.g., "5e3f7acd9876543210abcdef")
    /// - Linear: user ID (e.g., "abc12345-6789-0abc-def0-123456789abc")
    /// - GitHub Projects: numeric GitHub `databaseId` (e.g., "12345678")
    #[serde(default)]
    pub sync_user_id: String,
    /// Workflow statuses to sync (empty = default/first status only)
    #[serde(default)]
    pub sync_statuses: Vec<String>,
    /// Optional `IssueTypeCollection` name this project maps to.
    /// Not required for kanban onboarding or sync.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
    /// Explicit mapping: kanban issue type ID → operator issue type key (e.g., TASK, FEAT, FIX).
    /// Multiple kanban types can map to the same operator template.
    #[serde(default)]
    pub type_mappings: std::collections::HashMap<String, String>,
}

// ─── Git Provider Configuration ────────────────────────────────────────────

/// Git provider configuration for PR/MR operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct GitConfig {
    /// Active provider (auto-detected from remote URL if not specified)
    #[serde(default)]
    pub provider: Option<GitProviderConfig>,
    /// GitHub-specific configuration
    #[serde(default)]
    pub github: GitHubConfig,
    /// GitLab-specific configuration (planned)
    #[serde(default)]
    pub gitlab: GitLabConfig,
    /// Branch naming format (e.g., "{type}/{ticket_id}-{slug}")
    #[serde(default = "default_branch_format")]
    pub branch_format: String,
    /// Whether to use git worktrees for per-ticket isolation (default: false)
    /// When false, tickets work directly in the project directory with branches
    #[serde(default)]
    pub use_worktrees: bool,
}

fn default_branch_format() -> String {
    "{type}/{ticket_id}".to_string()
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            provider: None,
            github: GitHubConfig::default(),
            gitlab: GitLabConfig::default(),
            branch_format: default_branch_format(),
            use_worktrees: false,
        }
    }
}

/// Git provider selection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum GitProviderConfig {
    /// GitHub (github.com)
    GitHub,
    /// GitLab (gitlab.com or self-hosted)
    GitLab,
    /// Bitbucket (bitbucket.org)
    Bitbucket,
    /// Azure DevOps (dev.azure.com)
    AzureDevOps,
}

/// GitHub-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct GitHubConfig {
    /// Whether GitHub integration is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Environment variable containing the GitHub token (default: `GITHUB_TOKEN`)
    #[serde(default = "default_github_token_env")]
    pub token_env: String,
}

fn default_github_token_env() -> String {
    "GITHUB_TOKEN".to_string()
}

/// GitLab-specific configuration (planned)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct GitLabConfig {
    /// Whether GitLab integration is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable containing the GitLab token (default: `GITLAB_TOKEN`)
    #[serde(default = "default_gitlab_token_env")]
    pub token_env: String,
    /// GitLab host (default: gitlab.com, can be self-hosted)
    #[serde(default)]
    pub host: Option<String>,
}

fn default_gitlab_token_env() -> String {
    "GITLAB_TOKEN".to_string()
}

// ─── Version Check Configuration ────────────────────────────────────────────

/// Version check configuration for automatic update notifications
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct VersionCheckConfig {
    /// Enable automatic version checking on startup
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// URL to fetch latest version from (optional, can be removed)
    #[serde(default = "default_version_check_url")]
    pub url: Option<String>,

    /// Timeout in seconds for version check HTTP request
    #[serde(default = "default_version_check_timeout")]
    pub timeout_secs: u64,
}

fn default_version_check_url() -> Option<String> {
    Some("https://operator.untra.io/VERSION".to_string())
}

fn default_version_check_timeout() -> u64 {
    3
}

impl Default for VersionCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            url: default_version_check_url(),
            timeout_secs: 3,
        }
    }
}

impl Config {
    /// Path to the operator config file within .tickets/
    pub fn operator_config_path() -> PathBuf {
        PathBuf::from(".tickets/operator/config.toml")
    }

    pub fn load(config_path: Option<&str>) -> Result<Self> {
        // Start with embedded defaults so operator works without config files
        let defaults = Config::default();
        let defaults_json =
            serde_json::to_string(&defaults).context("Failed to serialize default config")?;

        let mut builder = config::Config::builder().add_source(config::File::from_str(
            &defaults_json,
            config::FileFormat::Json,
        ));

        // Operator config in .tickets/operator/ (primary config location)
        let operator_config = Self::operator_config_path();
        if operator_config.exists() {
            builder = builder.add_source(config::File::from(operator_config));
        }

        // User config in ~/.config/operator/ (optional global overrides)
        if let Some(config_dir) = dirs::config_dir() {
            let user_config = config_dir.join("operator").join("config.toml");
            if user_config.exists() {
                builder = builder.add_source(config::File::from(user_config));
            }
        }

        // Explicit config file (CLI override)
        if let Some(path) = config_path {
            builder = builder.add_source(config::File::with_name(path));
        }

        // Environment variables with OPERATOR_ prefix
        builder = builder.add_source(
            config::Environment::with_prefix("OPERATOR")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder.build().context("Failed to load configuration")?;
        config.try_deserialize().map_err(|e| {
            let mut sources = vec![];
            let operator_config = Self::operator_config_path();
            if operator_config.exists() {
                sources.push(format!("  - {}", operator_config.display()));
            }
            if let Some(config_dir) = dirs::config_dir() {
                let user_config = config_dir.join("operator").join("config.toml");
                if user_config.exists() {
                    sources.push(format!("  - {}", user_config.display()));
                }
            }
            if let Some(path) = config_path {
                sources.push(format!("  - {path}"));
            }
            let sources_str = if sources.is_empty() {
                String::from("  (no config files found)")
            } else {
                sources.join("\n")
            };
            anyhow::anyhow!(
                "Failed to deserialize configuration: {e}\n\nConfig files loaded:\n{sources_str}\n\nCheck these files for missing or invalid fields."
            )
        })
    }

    /// Save config to .tickets/operator/config.toml
    pub fn save(&self) -> Result<()> {
        let config_path = Self::operator_config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create operator config directory")?;
        }

        let toml_str =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        std::fs::write(&config_path, toml_str).context("Failed to write config file")?;

        Ok(())
    }

    /// Calculate effective max agents based on CPU cores
    pub fn effective_max_agents(&self) -> usize {
        let cpu_count = System::new_all().cpus().len();
        let core_based_max = cpu_count.saturating_sub(self.agents.cores_reserved);
        self.agents.max_parallel.min(core_based_max).max(1)
    }

    /// Get absolute path to tickets directory
    pub fn tickets_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.paths.tickets);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }

    /// Get absolute path to projects directory
    pub fn projects_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.paths.projects);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }

    /// Get absolute path to state directory
    pub fn state_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.paths.state);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }

    /// Get absolute path to worktrees directory
    #[allow(dead_code)] // Will be used when WorktreeManager is wired into launcher
    pub fn worktrees_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.paths.worktrees);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }

    /// Get absolute path to logs directory
    pub fn logs_path(&self) -> PathBuf {
        self.state_path().join("logs")
    }

    /// Get path to operator's custom tmux config
    pub fn tmux_config_path(&self) -> PathBuf {
        self.tickets_path().join("operator").join(".tmux.conf")
    }

    /// Get path to tmux status script
    pub fn tmux_status_script_path(&self) -> PathBuf {
        self.tickets_path().join("operator").join("tmux-status.sh")
    }

    /// Get absolute path to Backstage installation directory
    pub fn backstage_path(&self) -> PathBuf {
        self.state_path().join(&self.backstage.subpath)
    }

    /// Get absolute path to Backstage branding directory
    #[allow(dead_code)]
    pub fn backstage_branding_path(&self) -> PathBuf {
        self.backstage_path().join(&self.backstage.branding_subpath)
    }

    /// Get priority index for a ticket type (lower = higher priority)
    pub fn priority_index(&self, ticket_type: &str) -> usize {
        self.queue
            .priority_order
            .iter()
            .position(|t| t == ticket_type)
            .unwrap_or(usize::MAX)
    }

    /// Discover projects by finding subdirectories with CLAUDE.md files
    pub fn discover_projects(&self) -> Vec<String> {
        crate::projects::discover_projects(&self.projects_path())
    }

    /// Discover projects with full git and LLM tool information
    ///
    /// Returns projects found by scanning for .git directories and LLM marker files.
    /// Each project includes git repo info (remote URL, default branch, GitHub info)
    /// and a list of available LLM tools.
    #[allow(dead_code)] // For future integration
    pub fn discover_projects_full(&self) -> Vec<crate::projects::DiscoveredProject> {
        crate::projects::discover_projects_with_git(&self.projects_path())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            projects: Vec::new(), // Populated during setup
            agents: AgentsConfig {
                max_parallel: 5,
                cores_reserved: 1,
                health_check_interval: 30,
                generation_timeout_secs: 300, // 5 minutes
                sync_interval: 60,            // 1 minute
                step_timeout: 1800,           // 30 minutes
                silence_threshold: 30,        // 30 seconds
            },
            notifications: NotificationsConfig::default(),
            queue: QueueConfig {
                auto_assign: true,
                priority_order: vec![
                    "INV".to_string(),
                    "FIX".to_string(),
                    "TASK".to_string(),
                    "FEAT".to_string(),
                    "SPIKE".to_string(),
                ],
                poll_interval_ms: 1000,
            },
            paths: PathsConfig {
                tickets: ".tickets".to_string(), // Relative to cwd
                projects: ".".to_string(),       // cwd is projects root
                state: ".tickets/operator".to_string(),
                worktrees: default_worktrees_dir(),
            },
            ui: UiConfig {
                refresh_rate_ms: 250,
                completed_history_hours: 24,
                summary_max_length: 40,
                panel_names: PanelNamesConfig::default(),
            },
            launch: LaunchConfig {
                confirm_autonomous: true,
                confirm_paired: true,
                launch_delay_ms: 2000,
                docker: DockerConfig::default(),
                yolo: YoloConfig::default(),
            },
            templates: TemplatesConfig::default(),
            api: ApiConfig::default(),
            logging: LoggingConfig::default(),
            tmux: TmuxConfig::default(),
            sessions: SessionsConfig::default(),
            llm_tools: LlmToolsConfig::default(),
            backstage: BackstageConfig::default(),
            rest_api: RestApiConfig::default(),
            git: GitConfig::default(),
            kanban: KanbanConfig::default(),
            version_check: VersionCheckConfig::default(),
            delegators: Vec::new(),
            model_servers: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preset_is_dev_kanban() {
        assert_eq!(CollectionPreset::default(), CollectionPreset::DevKanban);
    }

    #[test]
    fn test_templates_config_default_uses_dev_kanban() {
        let config = TemplatesConfig::default();
        assert_eq!(config.preset, CollectionPreset::DevKanban);
    }

    #[test]
    fn test_dev_kanban_has_three_issue_types() {
        let types = CollectionPreset::DevKanban.issue_types();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&"TASK".to_string()));
        assert!(types.contains(&"FEAT".to_string()));
        assert!(types.contains(&"FIX".to_string()));
    }

    #[test]
    fn test_delegator_serde_roundtrip() {
        let delegator = Delegator {
            name: "claude-opus-auto".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Claude Opus Auto".to_string()),
            model_properties: std::collections::HashMap::new(),
            launch_config: Some(DelegatorLaunchConfig {
                yolo: true,
                permission_mode: Some("delegate".to_string()),
                flags: vec!["--verbose".to_string()],
                ..Default::default()
            }),
            model_server: None,
        };

        let json = serde_json::to_string(&delegator).unwrap();
        let parsed: Delegator = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "claude-opus-auto");
        assert_eq!(parsed.llm_tool, "claude");
        assert_eq!(parsed.model, "opus");
        assert!(parsed.launch_config.unwrap().yolo);
        assert!(parsed.model_server.is_none());
    }

    #[test]
    fn test_model_server_toml_roundtrip() {
        let toml_str = r#"
            name = "ollama-local"
            kind = "ollama"
            base_url = "http://localhost:11434"
            display_name = "Ollama (local)"
        "#;
        let server: ModelServer = toml::from_str(toml_str).unwrap();
        assert_eq!(server.name, "ollama-local");
        assert_eq!(server.kind, "ollama");
        assert_eq!(server.base_url.as_deref(), Some("http://localhost:11434"));
        assert_eq!(server.display_name.as_deref(), Some("Ollama (local)"));
        assert!(server.extra_env.is_empty());
        assert!(server.api_key_env.is_none());
    }

    #[test]
    fn test_delegator_with_model_server_ref_roundtrip() {
        let toml_str = r#"
            name = "codex-local-qwen"
            llm_tool = "codex"
            model = "qwen2.5-coder"
            model_server = "ollama-local"
        "#;
        let d: Delegator = toml::from_str(toml_str).unwrap();
        assert_eq!(d.name, "codex-local-qwen");
        assert_eq!(d.model_server.as_deref(), Some("ollama-local"));
    }

    #[test]
    fn test_delegator_without_model_server_field_still_parses() {
        let toml_str = r#"
            name = "claude-opus-auto"
            llm_tool = "claude"
            model = "opus"
        "#;
        let d: Delegator = toml::from_str(toml_str).unwrap();
        assert_eq!(d.name, "claude-opus-auto");
        assert!(d.model_server.is_none());
    }

    #[test]
    fn test_implicit_model_server_for_known_tools() {
        assert_eq!(
            implicit_model_server_for_tool("claude").kind,
            "anthropic-api"
        );
        assert_eq!(implicit_model_server_for_tool("codex").kind, "openai-api");
        assert_eq!(implicit_model_server_for_tool("gemini").kind, "google-api");
        assert_eq!(implicit_model_server_for_tool("unknown").kind, "openai-api");
    }

    #[test]
    fn test_config_without_model_servers_field_still_parses() {
        let toml_str = r#"
            [agents]
            max_parallel = 1
            cores_reserved = 0
            health_check_interval = 5
            [notifications]
            enabled = false
            [queue]
            auto_assign = true
            priority_order = []
            poll_interval_ms = 1000
            [paths]
            tickets = ".tickets"
            projects = "."
            state = ".tickets/operator"
            worktrees = ".worktrees"
            [ui]
            refresh_rate_ms = 100
            completed_history_hours = 1
            summary_max_length = 40
            [launch]
            confirm_autonomous = false
            confirm_paired = false
            launch_delay_ms = 0
            [templates]
        "#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert!(cfg.model_servers.is_empty());
    }

    #[test]
    fn test_skill_directories_override_default() {
        let override_config = SkillDirectoriesOverride::default();
        assert!(override_config.global.is_empty());
        assert!(override_config.project.is_empty());
    }

    #[test]
    fn test_session_wrapper_type_cmux_display() {
        assert_eq!(SessionWrapperType::Cmux.to_string(), "cmux");
    }

    #[test]
    fn test_session_wrapper_type_cmux_serde_roundtrip() {
        let json = serde_json::to_string(&SessionWrapperType::Cmux).unwrap();
        let parsed: SessionWrapperType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SessionWrapperType::Cmux);
    }

    #[test]
    fn test_sessions_cmux_config_defaults() {
        let config = SessionsCmuxConfig::default();
        assert_eq!(
            config.binary_path,
            "/Applications/cmux.app/Contents/Resources/bin/cmux"
        );
        assert!(config.require_in_cmux);
        assert_eq!(config.placement, CmuxPlacementPolicy::Auto);
    }

    #[test]
    fn test_cmux_placement_policy_display() {
        assert_eq!(CmuxPlacementPolicy::Auto.to_string(), "auto");
        assert_eq!(CmuxPlacementPolicy::Workspace.to_string(), "workspace");
        assert_eq!(CmuxPlacementPolicy::Window.to_string(), "window");
    }

    #[test]
    fn test_config_deserialize_with_cmux_wrapper() {
        let toml_str = r#"
            wrapper = "cmux"
            [cmux]
            binary_path = "/usr/local/bin/cmux"
            require_in_cmux = false
            placement = "window"
        "#;
        let config: SessionsConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.wrapper, SessionWrapperType::Cmux);
        assert_eq!(config.cmux.binary_path, "/usr/local/bin/cmux");
        assert!(!config.cmux.require_in_cmux);
        assert_eq!(config.cmux.placement, CmuxPlacementPolicy::Window);
    }

    #[test]
    fn test_devops_kanban_has_five_issue_types() {
        let types = CollectionPreset::DevopsKanban.issue_types();
        assert_eq!(types.len(), 5);
        assert!(types.contains(&"TASK".to_string()));
        assert!(types.contains(&"FEAT".to_string()));
        assert!(types.contains(&"FIX".to_string()));
        assert!(types.contains(&"SPIKE".to_string()));
        assert!(types.contains(&"INV".to_string()));
    }

    // --- effective_max_agents tests ---

    #[test]
    fn test_effective_max_agents_never_returns_zero() {
        let mut config = Config::default();
        config.agents.max_parallel = 0;
        config.agents.cores_reserved = 100;
        assert!(config.effective_max_agents() >= 1);
    }

    #[test]
    fn test_effective_max_agents_respects_max_parallel() {
        let mut config = Config::default();
        config.agents.max_parallel = 2;
        config.agents.cores_reserved = 0;
        assert!(config.effective_max_agents() <= 2);
    }

    #[test]
    fn test_effective_max_agents_reserves_cores() {
        let config = Config::default();
        let cpu_count = sysinfo::System::new_all().cpus().len();
        let effective = config.effective_max_agents();
        assert!(effective <= cpu_count.saturating_sub(config.agents.cores_reserved));
    }

    // --- Path resolution tests ---

    #[test]
    fn test_tickets_path_absolute_passthrough() {
        let mut config = Config::default();
        config.paths.tickets = "/absolute/path/tickets".to_string();
        assert_eq!(
            config.tickets_path(),
            std::path::PathBuf::from("/absolute/path/tickets")
        );
    }

    #[test]
    fn test_tickets_path_relative_resolves() {
        let config = Config::default();
        let path = config.tickets_path();
        assert!(path.is_absolute());
        assert!(path.ends_with(".tickets"));
    }

    #[test]
    fn test_projects_path_absolute_passthrough() {
        let mut config = Config::default();
        config.paths.projects = "/my/projects".to_string();
        assert_eq!(
            config.projects_path(),
            std::path::PathBuf::from("/my/projects")
        );
    }

    #[test]
    fn test_state_path_relative_resolves() {
        let config = Config::default();
        let path = config.state_path();
        assert!(path.is_absolute());
        assert!(path.ends_with("operator"));
    }

    // --- priority_index tests ---

    #[test]
    fn test_priority_index_known_types() {
        let config = Config::default();
        assert_eq!(config.priority_index("INV"), 0);
        assert_eq!(config.priority_index("FIX"), 1);
        assert_eq!(config.priority_index("TASK"), 2);
        assert_eq!(config.priority_index("FEAT"), 3);
        assert_eq!(config.priority_index("SPIKE"), 4);
    }

    #[test]
    fn test_priority_index_unknown_returns_max() {
        let config = Config::default();
        assert_eq!(config.priority_index("UNKNOWN"), usize::MAX);
    }

    #[test]
    fn test_priority_index_empty_order() {
        let mut config = Config::default();
        config.queue.priority_order.clear();
        assert_eq!(config.priority_index("INV"), usize::MAX);
    }

    // --- Default value function tests ---

    #[test]
    fn test_default_generation_timeout_is_300() {
        assert_eq!(default_generation_timeout(), 300);
    }

    #[test]
    fn test_default_sync_interval_is_60() {
        assert_eq!(default_sync_interval(), 60);
    }

    #[test]
    fn test_default_step_timeout_is_1800() {
        assert_eq!(default_step_timeout(), 1800);
    }

    #[test]
    fn test_default_silence_threshold_is_6() {
        assert_eq!(default_silence_threshold(), 6);
    }

    #[test]
    fn test_default_worktrees_dir_contains_worktrees() {
        let dir = default_worktrees_dir();
        assert!(dir.contains("worktrees"));
    }

    #[test]
    fn test_upsert_jira_project_inserts_new_workspace() {
        let mut kanban = KanbanConfig::default();
        kanban.upsert_jira_project(
            "acme.atlassian.net",
            "user@acme.com",
            "OPERATOR_JIRA_API_KEY",
            "PROJ",
            "acct-123",
        );

        let ws = kanban
            .jira
            .get("acme.atlassian.net")
            .expect("workspace should be inserted");
        assert!(ws.enabled);
        assert_eq!(ws.email, "user@acme.com");
        assert_eq!(ws.api_key_env, "OPERATOR_JIRA_API_KEY");

        let project = ws.projects.get("PROJ").expect("project should exist");
        assert_eq!(project.sync_user_id, "acct-123");
    }

    #[test]
    fn test_upsert_jira_project_adds_to_existing_workspace_without_clobber() {
        let mut kanban = KanbanConfig::default();
        // Seed with an existing workspace and project
        kanban.upsert_jira_project(
            "acme.atlassian.net",
            "user@acme.com",
            "OPERATOR_JIRA_API_KEY",
            "EXISTING",
            "acct-existing",
        );

        // Add a second project to the same workspace
        kanban.upsert_jira_project(
            "acme.atlassian.net",
            "user@acme.com",
            "OPERATOR_JIRA_API_KEY",
            "NEWONE",
            "acct-new",
        );

        let ws = kanban.jira.get("acme.atlassian.net").unwrap();
        assert_eq!(ws.projects.len(), 2, "both projects should be preserved");
        assert_eq!(ws.projects["EXISTING"].sync_user_id, "acct-existing");
        assert_eq!(ws.projects["NEWONE"].sync_user_id, "acct-new");
    }

    #[test]
    fn test_upsert_jira_project_replaces_existing_project_entry() {
        let mut kanban = KanbanConfig::default();
        kanban.upsert_jira_project(
            "acme.atlassian.net",
            "user@acme.com",
            "OPERATOR_JIRA_API_KEY",
            "PROJ",
            "acct-old",
        );
        // Upsert same project with new sync_user_id
        kanban.upsert_jira_project(
            "acme.atlassian.net",
            "user@acme.com",
            "OPERATOR_JIRA_API_KEY",
            "PROJ",
            "acct-new",
        );

        let ws = kanban.jira.get("acme.atlassian.net").unwrap();
        assert_eq!(ws.projects.len(), 1);
        assert_eq!(ws.projects["PROJ"].sync_user_id, "acct-new");
    }

    #[test]
    fn test_upsert_linear_project_inserts_new_workspace() {
        let mut kanban = KanbanConfig::default();
        kanban.upsert_linear_project(
            "myworkspace",
            "OPERATOR_LINEAR_API_KEY",
            "ENG",
            "user-uuid-1",
        );

        let ws = kanban.linear.get("myworkspace").unwrap();
        assert!(ws.enabled);
        assert_eq!(ws.api_key_env, "OPERATOR_LINEAR_API_KEY");
        assert_eq!(ws.projects["ENG"].sync_user_id, "user-uuid-1");
    }

    #[test]
    fn test_upsert_linear_project_adds_to_existing_workspace_without_clobber() {
        let mut kanban = KanbanConfig::default();
        kanban.upsert_linear_project("myworkspace", "OPERATOR_LINEAR_API_KEY", "ENG", "user-a");
        kanban.upsert_linear_project("myworkspace", "OPERATOR_LINEAR_API_KEY", "DESIGN", "user-b");

        let ws = kanban.linear.get("myworkspace").unwrap();
        assert_eq!(ws.projects.len(), 2);
        assert_eq!(ws.projects["ENG"].sync_user_id, "user-a");
        assert_eq!(ws.projects["DESIGN"].sync_user_id, "user-b");
    }

    #[test]
    fn test_upsert_jira_does_not_touch_other_workspaces() {
        let mut kanban = KanbanConfig::default();
        kanban.upsert_jira_project(
            "first.atlassian.net",
            "u1@first.com",
            "OPERATOR_JIRA_API_KEY",
            "FIRST",
            "acct-1",
        );
        kanban.upsert_jira_project(
            "second.atlassian.net",
            "u2@second.com",
            "OPERATOR_JIRA_SECOND_API_KEY",
            "SECOND",
            "acct-2",
        );

        assert_eq!(kanban.jira.len(), 2);
        assert_eq!(kanban.jira["first.atlassian.net"].email, "u1@first.com");
        assert_eq!(
            kanban.jira["second.atlassian.net"].api_key_env,
            "OPERATOR_JIRA_SECOND_API_KEY"
        );
    }

    #[test]
    fn test_upsert_project_jira() {
        use crate::api::providers::kanban::{
            DiscoveredProject, KanbanProviderType, ValidatedWorkspace, WorkspaceExtra,
        };

        let mut kanban = KanbanConfig::default();
        let ws = ValidatedWorkspace {
            provider_kind: KanbanProviderType::Jira,
            workspace_key: "acme.atlassian.net".to_string(),
            workspace_display_name: "Acme Corp".to_string(),
            sync_user_id: "acct-123".to_string(),
            sync_user_display_name: "Alice".to_string(),
            api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
            prefetched_projects: None,
            extra: WorkspaceExtra::Jira {
                email: "alice@acme.com".to_string(),
            },
        };
        let project = DiscoveredProject {
            workspace_key: "acme.atlassian.net".to_string(),
            project_key: "PROJ".to_string(),
            project_display_name: "My Project".to_string(),
            provider_url: None,
            provider_native_id: None,
        };

        kanban.upsert_project(&ws, &project);

        let entry = kanban
            .jira
            .get("acme.atlassian.net")
            .expect("workspace should be created");
        assert!(entry.enabled);
        assert_eq!(entry.email, "alice@acme.com");
        assert_eq!(entry.api_key_env, "OPERATOR_JIRA_API_KEY");
        let proj = entry.projects.get("PROJ").expect("project should exist");
        assert_eq!(proj.sync_user_id, "acct-123");
    }

    #[test]
    fn test_upsert_project_linear() {
        use crate::api::providers::kanban::{
            DiscoveredProject, KanbanProviderType, ValidatedWorkspace, WorkspaceExtra,
        };

        let mut kanban = KanbanConfig::default();
        let ws = ValidatedWorkspace {
            provider_kind: KanbanProviderType::Linear,
            workspace_key: "acme".to_string(),
            workspace_display_name: "Acme Inc".to_string(),
            sync_user_id: "user-uuid-1".to_string(),
            sync_user_display_name: "Bob".to_string(),
            api_key_env: "OPERATOR_LINEAR_API_KEY".to_string(),
            prefetched_projects: None,
            extra: WorkspaceExtra::Linear,
        };
        let project = DiscoveredProject {
            workspace_key: "acme".to_string(),
            project_key: "ENG".to_string(),
            project_display_name: "Engineering".to_string(),
            provider_url: None,
            provider_native_id: None,
        };

        kanban.upsert_project(&ws, &project);

        let entry = kanban
            .linear
            .get("acme")
            .expect("workspace should be created");
        assert!(entry.enabled);
        assert_eq!(entry.api_key_env, "OPERATOR_LINEAR_API_KEY");
        let proj = entry.projects.get("ENG").expect("project should exist");
        assert_eq!(proj.sync_user_id, "user-uuid-1");
    }

    #[test]
    fn test_upsert_project_github() {
        use crate::api::providers::kanban::{
            DiscoveredProject, KanbanProviderType, ValidatedWorkspace, WorkspaceExtra,
        };

        let mut kanban = KanbanConfig::default();
        let ws = ValidatedWorkspace {
            provider_kind: KanbanProviderType::Github,
            workspace_key: "my-org".to_string(),
            workspace_display_name: "github.com".to_string(),
            sync_user_id: "12345678".to_string(),
            sync_user_display_name: "octocat".to_string(),
            api_key_env: "OPERATOR_GITHUB_TOKEN".to_string(),
            prefetched_projects: None,
            extra: WorkspaceExtra::Github,
        };
        let project = DiscoveredProject {
            workspace_key: "my-org".to_string(),
            project_key: "PVT_abc".to_string(),
            project_display_name: "My Board".to_string(),
            provider_url: None,
            provider_native_id: None,
        };

        kanban.upsert_project(&ws, &project);

        let entry = kanban
            .github
            .get("my-org")
            .expect("workspace should be created");
        assert!(entry.enabled);
        assert_eq!(entry.api_key_env, "OPERATOR_GITHUB_TOKEN");
        let proj = entry.projects.get("PVT_abc").expect("project should exist");
        assert_eq!(proj.sync_user_id, "12345678");
    }
}
