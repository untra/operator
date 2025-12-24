use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub on_agent_start: bool,
    pub on_agent_complete: bool,
    pub on_agent_needs_input: bool,
    pub on_pr_created: bool,
    pub on_investigation_created: bool,
    pub sound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub auto_assign: bool,
    pub priority_order: Vec<String>,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub tickets: String,
    pub projects: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub refresh_rate_ms: u64,
    pub completed_history_hours: u64,
    pub summary_max_length: usize,
    #[serde(default)]
    pub panel_names: PanelNamesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    pub confirm_autonomous: bool,
    pub confirm_paired: bool,
    pub launch_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmuxConfig {
    /// Whether custom tmux config has been generated
    #[serde(default)]
    pub config_generated: bool,
}

/// LLM CLI tools configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCapabilities {
    /// Whether the tool supports session continuity via UUID
    #[serde(default)]
    pub supports_sessions: bool,
    /// Whether the tool can run in headless/non-interactive mode
    #[serde(default)]
    pub supports_headless: bool,
}

/// A {tool, model} pair that can be selected when launching tickets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmProvider {
    /// CLI tool name (e.g., "claude")
    pub tool: String,
    /// Model alias or name (e.g., "opus", "sonnet")
    pub model: String,
    /// Optional display name for UI (e.g., "Claude Opus")
    #[serde(default)]
    pub display_name: Option<String>,
}

impl LlmProvider {
    /// Get the display name, falling back to "tool model" format
    pub fn display(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| format!("{} {}", self.tool, self.model))
    }
}

/// Predefined issue type collections
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Get all available presets
    pub fn all() -> &'static [CollectionPreset] {
        &[
            CollectionPreset::Simple,
            CollectionPreset::DevKanban,
            CollectionPreset::DevopsKanban,
            CollectionPreset::Custom,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Named preset for issue type collection
    /// Options: simple, dev_kanban, devops_kanban, custom
    #[serde(default)]
    pub preset: CollectionPreset,
    /// Custom issuetype collection (only used when preset = custom)
    /// List of issue type keys: TASK, FEAT, FIX, SPIKE, INV
    #[serde(default)]
    pub collection: Vec<String>,
}

impl TemplatesConfig {
    /// Get the effective issue types based on preset or custom collection
    pub fn effective_issue_types(&self) -> Vec<String> {
        match self.preset {
            CollectionPreset::Custom => self.collection.clone(),
            _ => self.preset.issue_types(),
        }
    }
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            preset: CollectionPreset::DevopsKanban,
            collection: Vec::new(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            },
            templates: TemplatesConfig::default(),
            api: ApiConfig::default(),
            logging: LoggingConfig::default(),
            tmux: TmuxConfig::default(),
            llm_tools: LlmToolsConfig::default(),
        }
    }
}
