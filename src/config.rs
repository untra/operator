#[path = "config/agent_profile.rs"]
pub mod agent_profile;
#[path = "config/git_config.rs"]
pub mod git_config;
#[path = "config/kanban.rs"]
pub mod kanban;
#[path = "config/llm_tools.rs"]
pub mod llm_tools;
#[path = "config/notifications_config.rs"]
pub mod notifications_config;
#[path = "config/sessions.rs"]
pub mod sessions;

pub use agent_profile::*;
pub use git_config::*;
pub use kanban::*;
pub use llm_tools::*;
pub use notifications_config::*;
pub use sessions::*;

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
    /// Relay MCP injection configuration
    #[serde(default)]
    pub relay: RelayConfig,
    /// Model Context Protocol (MCP) server configuration
    #[serde(default)]
    pub mcp: McpConfig,
    /// Agent Client Protocol (ACP) agent configuration
    #[serde(default)]
    pub acp: AcpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct AgentsConfig {
    pub max_parallel: usize,
    pub cores_reserved: usize,
    /// Maximum concurrent agents per project/repo (default: 1).
    /// Requires `git.use_worktrees` = true when > 1 to avoid conflicts.
    #[serde(default = "default_max_agents_per_repo")]
    pub max_agents_per_repo: usize,
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

fn default_max_agents_per_repo() -> usize {
    1
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

/// REST API server configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct RestApiConfig {
    /// Whether the REST API is enabled
    #[serde(default = "default_rest_enabled")]
    pub enabled: bool,
    /// Address the REST API binds to. Defaults to `127.0.0.1` (local only) so
    /// the server — which reports the project directory name — is not reachable
    /// from other hosts. Set to `0.0.0.0` to expose it on all interfaces.
    #[serde(default = "default_rest_host")]
    pub host: String,
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

fn default_rest_host() -> String {
    "127.0.0.1".to_string()
}

fn default_rest_port() -> u16 {
    7008
}

impl Default for RestApiConfig {
    fn default() -> Self {
        Self {
            enabled: default_rest_enabled(),
            host: default_rest_host(),
            port: default_rest_port(),
            cors_origins: Vec::new(),
        }
    }
}

impl RestApiConfig {
    /// Parse the configured `host` into an `IpAddr`, falling back to localhost
    /// if it is not a valid IP literal (we never bind a wider scope by accident).
    pub fn host_ip(&self) -> std::net::IpAddr {
        self.host
            .parse()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
    }
}

/// Model Context Protocol (MCP) server configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct McpConfig {
    /// Whether to mount MCP HTTP/SSE endpoints on the REST API server.
    /// Toggling requires an API restart (no hot-swap of the axum router).
    #[serde(default = "default_true")]
    pub http_enabled: bool,
    /// Whether the descriptor endpoint advertises the `operator mcp` stdio
    /// command. Set to false on multi-tenant/remote deployments where clients
    /// shouldn't spawn local subprocesses.
    #[serde(default = "default_true")]
    pub stdio_advertised: bool,
    /// Whether to expose ticket-mutating tools (claim, complete, return-to-queue,
    /// create) over MCP. Defaults to `false` because any MCP client can call them.
    #[serde(default)]
    pub expose_ticket_write_tools: bool,
    /// External MCP servers to inject into spawned agent sessions.
    /// Each entry produces a separate `--mcp-config` file alongside the
    /// relay config when launching Claude Code agents.
    #[serde(default)]
    pub external_servers: Vec<ExternalMcpServer>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            http_enabled: true,
            stdio_advertised: true,
            expose_ticket_write_tools: false,
            external_servers: Vec::new(),
        }
    }
}

/// An external MCP server to inject into spawned agent sessions.
///
/// Values in `command`, `args`, and `env` support `${VAR}` interpolation,
/// expanded at spawn time from the operator process environment.
///
/// When `discover_from` is set, operator reads an MCP server spec from that
/// JSON sidecar file at spawn time. The sidecar must contain a top-level
/// `mcpServer` object with `command`, `args`, and `env` fields. If the file
/// is absent and `command` is empty, the server is silently skipped.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ExternalMcpServer {
    /// Server name used as the key in the `mcpServers` JSON object
    /// (e.g., "kanbots"). Must be unique across all external servers.
    pub name: String,
    /// Command to execute. Supports `${VAR}` interpolation.
    #[serde(default)]
    pub command: String,
    /// Command arguments. Each element supports `${VAR}` interpolation.
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables passed to the MCP server process.
    /// Values support `${VAR}` interpolation.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Whether this server is enabled. Allows disabling without removing config.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Path to a JSON sidecar discovery file. Relative paths resolve from
    /// the project directory. The sidecar must contain `{ "mcpServer": { ... } }`.
    /// When the file exists, its `mcpServer` spec is used verbatim (overriding
    /// `command`/`args`/`env`). When absent and `command` is empty, the server
    /// is silently skipped.
    #[serde(default)]
    pub discover_from: Option<String>,
}

/// Agent Client Protocol (ACP) agent configuration.
///
/// Operator runs as an ACP agent over stdio when editors (Zed, `JetBrains`,
/// Emacs `agent-shell`, Kiro, etc.) spawn `operator acp`. Each ACP session
/// maps to an in-progress ACP ticket and a delegator subprocess.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AcpConfig {
    /// Whether the dashboard advertises the `operator acp` stdio entrypoint
    /// (and editor-config snippet actions). Set to false on machines that
    /// shouldn't be used as ACP agents.
    #[serde(default = "default_true")]
    pub stdio_advertised: bool,
    /// Name of the delegator (from `[[delegators]]`) to use for ACP prompts.
    /// If unset or not found, falls back to the operator's default delegator
    /// resolution.
    #[serde(default)]
    pub default_delegator: Option<String>,
    /// Maximum number of concurrent ACP sessions. New `session/new` requests
    /// beyond this limit are rejected with a JSON-RPC error.
    #[serde(default = "default_acp_max_sessions")]
    pub max_concurrent_sessions: usize,
}

impl Default for AcpConfig {
    fn default() -> Self {
        Self {
            stdio_advertised: true,
            default_delegator: None,
            max_concurrent_sessions: default_acp_max_sessions(),
        }
    }
}

fn default_acp_max_sessions() -> usize {
    8
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

    /// Enable fetching hosted issuetype collections during setup.
    /// When disabled, only the embedded (offline) collections are offered.
    #[serde(default = "default_true")]
    pub collections_fetch_enabled: bool,

    /// URL of the hosted collection index manifest, fetched during setup.
    /// Points at a `CollectionIndex` JSON document listing available collections.
    #[serde(default = "default_collections_manifest_url")]
    pub collections_manifest_url: Option<String>,

    /// Timeout in seconds for hosted collection fetch HTTP requests.
    #[serde(default = "default_collections_fetch_timeout")]
    pub collections_fetch_timeout_secs: u64,
}

fn default_collections_manifest_url() -> Option<String> {
    Some("https://operator.untra.io/collections/index.json".to_string())
}

fn default_collections_fetch_timeout() -> u64 {
    5
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            preset: CollectionPreset::DevKanban,
            collection: Vec::new(),
            active_collection: None,
            collections_fetch_enabled: true,
            collections_manifest_url: default_collections_manifest_url(),
            collections_fetch_timeout_secs: 5,
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

fn default_true() -> bool {
    true
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

// ─── Relay Configuration ─────────────────────────────────────────────────────

/// Relay MCP injection configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct RelayConfig {
    /// When true, automatically inject the relay MCP server for all delegators.
    /// When false (default), relay injection is opt-in per delegator.
    #[serde(default)]
    pub auto_inject_mcp: bool,
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
        let cfg: Self = config.try_deserialize().map_err(|e| {
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
        })?;

        if cfg.agents.max_agents_per_repo > 1 && !cfg.git.use_worktrees {
            tracing::warn!(
                max_agents_per_repo = cfg.agents.max_agents_per_repo,
                "max_agents_per_repo > 1 without git.use_worktrees = true; \
                 multiple agents on the same repo without worktrees will cause git conflicts"
            );
        }

        Ok(cfg)
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

    pub fn effective_max_agents_per_repo(&self) -> usize {
        self.agents.max_agents_per_repo.max(1)
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
                max_agents_per_repo: 1,
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
            rest_api: RestApiConfig::default(),
            git: GitConfig::default(),
            kanban: KanbanConfig::default(),
            version_check: VersionCheckConfig::default(),
            delegators: Vec::new(),
            model_servers: Vec::new(),
            relay: RelayConfig::default(),
            mcp: McpConfig::default(),
            acp: AcpConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Default value function tests (private functions — must stay inline) ---

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
}

#[cfg(test)]
#[path = "config/config_tests.rs"]
mod config_tests;
