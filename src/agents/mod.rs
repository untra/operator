#![allow(dead_code)] // Active module - some methods reserved for future agent lifecycle features
#![allow(unused_imports)]

pub mod activity;
pub mod agent_switcher;
pub mod artifact_detector;
pub mod cmux;
mod generator;
pub mod hooks;
pub mod idle_detector;
mod launcher;
mod monitor;
mod pr_workflow;
mod session;
mod sync;
pub mod terminal_wrapper;
pub mod tmux;
pub mod tmux_config;
mod visual_review;
pub mod vscode_types;
pub mod zellij;

// Activity detection
pub use activity::{
    CmuxActivityDetector, LlmHookDetector, MockActivityDetector, TmuxActivityDetector,
    ZellijActivityDetector,
};

// Agent switching
pub use agent_switcher::AgentSwitcher;

// Ticket/agent generation
pub use generator::{
    AgentTicketCreator, AgentTicketResult, AssessTicketCreator, AssessTicketResult, AGENT_TOOLS,
};

// Launcher
pub use launcher::{LaunchOptions, Launcher, PreparedLaunch, RelaunchOptions};

// Artifact detection
pub use artifact_detector::{ArtifactDetector, ArtifactStatus};

// Monitoring
pub use monitor::{HealthCheckResult, ReconciliationResult, SessionMonitor};

// Workflows
pub use pr_workflow::PrWorkflow;
pub use session::Session;
pub use sync::{SyncAction, SyncResult, TicketSessionSync};
pub use visual_review::{VisualReviewHandler, VisualReviewResult};

// Terminal wrapper abstraction
pub use terminal_wrapper::{
    ActivityConfig, ActivityDetector, ComposedSession, SessionError, SessionInfo, SessionWrapper,
    WrapperType,
};

// Tmux implementation
pub use tmux::{
    sanitize_session_name, MockTmuxClient, SystemTmuxClient, TmuxClient, TmuxError, TmuxSession,
    TmuxVersion, TmuxWrapper,
};

// Cmux implementation
pub use cmux::{
    CmuxClient, CmuxError, CmuxWindow, CmuxWorkspace, CmuxWrapper, MockCmuxClient, SystemCmuxClient,
};

pub use tmux_config::{generate_status_script, generate_tmux_conf};

// Zellij implementation
pub use zellij::{MockZellijClient, SystemZellijClient, ZellijClient, ZellijError, ZellijWrapper};

// VSCode extension types (for webhook API contract)
pub use vscode_types::{
    VsCodeActivityResponse, VsCodeActivityState, VsCodeErrorResponse, VsCodeExistsResponse,
    VsCodeHealthResponse, VsCodeLaunchOptions, VsCodeListResponse, VsCodeModelOption,
    VsCodeSendCommandRequest, VsCodeSessionInfo, VsCodeSuccessResponse,
    VsCodeTerminalCreateOptions, VsCodeTerminalState, VsCodeTicketInfo, VsCodeTicketMetadata,
    VsCodeTicketStatus,
};
