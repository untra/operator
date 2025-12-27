use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    #[serde(default)]
    pub llm_tools: LlmToolsConfig,
    #[serde(default)]
    pub backstage: BackstageConfig,
    #[serde(default)]
    pub rest_api: RestApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    30 // 30 seconds
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub on_agent_start: bool,
    pub on_agent_complete: bool,
    pub on_agent_needs_input: bool,
    pub on_pr_created: bool,
    pub on_investigation_created: bool,
    pub sound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueueConfig {
    pub auto_assign: bool,
    pub priority_order: Vec<String>,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PathsConfig {
    pub tickets: String,
    pub projects: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiConfig {
    pub refresh_rate_ms: u64,
    pub completed_history_hours: u64,
    pub summary_max_length: usize,
    #[serde(default)]
    pub panel_names: PanelNamesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PanelNamesConfig {
    #[serde(default = "default_todo_name")]
    pub queue: String,
    #[serde(default = "default_doing_name")]
    pub agents: String,
    #[serde(default = "default_awaiting_name")]
    pub awaiting: String,
    #[serde(default = "default_done_name")]
    pub completed: String,
}

fn default_todo_name() -> String {
    "TODO QUEUE".to_string()
}

fn default_doing_name() -> String {
    "DOING".to_string()
}

fn default_awaiting_name() -> String {
    "AWAITING".to_string()
}

fn default_done_name() -> String {
    "DONE".to_string()
}

impl Default for PanelNamesConfig {
    fn default() -> Self {
        Self {
            queue: default_todo_name(),
            agents: default_doing_name(),
            awaiting: default_awaiting_name(),
            completed: default_done_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct YoloConfig {
    /// Whether YOLO mode option is available in launch dialog
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct TmuxConfig {
    /// Whether custom tmux config has been generated
    #[serde(default)]
    pub config_generated: bool,
}

/// Backstage integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BackstageConfig {
    /// Whether Backstage integration is enabled
    #[serde(default = "default_backstage_enabled")]
    pub enabled: bool,
    /// Port for the Backstage server
    #[serde(default = "default_backstage_port")]
    pub port: u16,
    /// Auto-start Backstage server when TUI launches
    #[serde(default)]
    pub auto_start: bool,
    /// Subdirectory within state_path for Backstage installation
    #[serde(default = "default_backstage_subpath")]
    pub subpath: String,
    /// Subdirectory within backstage path for branding customization
    #[serde(default = "default_branding_subpath")]
    pub branding_subpath: String,
    /// Base URL for downloading backstage-server binary
    #[serde(default = "default_backstage_release_url")]
    pub release_url: String,
    /// Optional local path to backstage-server binary
    /// If set, this is used instead of downloading from release_url
    #[serde(default)]
    pub local_binary_path: Option<String>,
    /// Branding and theming configuration
    #[serde(default)]
    pub branding: BrandingConfig,
}

/// Branding configuration for Backstage portal
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
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
}

/// A detected CLI tool (e.g., claude binary)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectedTool {
    /// Tool name (e.g., "claude")
    pub name: String,
    /// Path to the binary
    pub path: String,
    /// Version string
    pub version: String,
    /// Available model aliases (e.g., ["opus", "sonnet", "haiku"])
    pub model_aliases: Vec<String>,
    /// Command template with {{model}}, {{session_id}}, {{prompt_file}} placeholders
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ToolCapabilities {
    /// Whether the tool supports session continuity via UUID
    #[serde(default)]
    pub supports_sessions: bool,
    /// Whether the tool can run in headless/non-interactive mode
    #[serde(default)]
    pub supports_headless: bool,
}

/// A {tool, model} pair that can be selected when launching tickets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct LlmProvider {
    /// CLI tool name (e.g., "claude")
    pub tool: String,
    /// Model alias or name (e.g., "opus", "sonnet")
    pub model: String,
    /// Optional display name for UI (e.g., "Claude Opus")
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Predefined issue type collections
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollectionPreset {
    /// Simple tasks only: TASK
    Simple,
    /// Developer kanban: TASK, FEAT, FIX
    DevKanban,
    /// DevOps kanban: TASK, SPIKE, INV, FEAT, FIX
    #[default]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemplatesConfig {
    /// Named preset for issue type collection
    /// Options: simple, dev_kanban, devops_kanban, custom
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
            preset: CollectionPreset::DevopsKanban,
            collection: Vec::new(),
            active_collection: None,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        config
            .try_deserialize()
            .context("Failed to deserialize configuration")
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
            notifications: NotificationsConfig {
                enabled: true,
                on_agent_start: true,
                on_agent_complete: true,
                on_agent_needs_input: true,
                on_pr_created: true,
                on_investigation_created: true,
                sound: false,
            },
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
            llm_tools: LlmToolsConfig::default(),
            backstage: BackstageConfig::default(),
            rest_api: RestApiConfig::default(),
        }
    }
}
