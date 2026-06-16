use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
    /// The canonical list of session wrappers, in display order. Single source of
    /// truth mirrored by the vertical catalog (`crate::integrations::catalog`);
    /// `vscode` is advertised under the Editor vertical.
    ///
    /// Consumed by `tests/vertical_parity.rs` to enforce catalog coverage; reads
    /// as unused in the bin crate, which has no blanket dead-code allowance here.
    #[allow(dead_code)]
    pub const ALL: [SessionWrapperType; 4] = [
        SessionWrapperType::Tmux,
        SessionWrapperType::Vscode,
        SessionWrapperType::Cmux,
        SessionWrapperType::Zellij,
    ];

    /// Short display name for the wrapper (used in header bar, logs)
    pub fn display_name(&self) -> &'static str {
        match self {
            SessionWrapperType::Tmux => "tmux",
            SessionWrapperType::Vscode => "vscode",
            SessionWrapperType::Cmux => "cmux",
            SessionWrapperType::Zellij => "zellij",
        }
    }

    /// Whether the operator process is currently running *inside* this wrapper's
    /// session context, detected from process-global environment markers.
    ///
    /// Reports the control wrapper of the API: launched tickets are coordinated
    /// through this wrapper, so "active" means operator can actually drive it.
    /// tmux sets `TMUX`, cmux sets `CMUX_WORKSPACE_ID`, zellij sets `ZELLIJ`, and
    /// VS Code's integrated terminal sets `TERM_PROGRAM=vscode`. These are the
    /// same env names checked by the wrapper detection in `status_panel` and
    /// `agents::{cmux,zellij}` — reuse, don't invent new ones.
    pub fn is_active_context(&self) -> bool {
        match self {
            SessionWrapperType::Tmux => std::env::var("TMUX").is_ok(),
            SessionWrapperType::Vscode => {
                std::env::var("TERM_PROGRAM").is_ok_and(|v| v == "vscode")
            }
            SessionWrapperType::Cmux => std::env::var("CMUX_WORKSPACE_ID").is_ok(),
            SessionWrapperType::Zellij => std::env::var("ZELLIJ").is_ok(),
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
