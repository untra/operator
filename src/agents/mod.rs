#![allow(dead_code)] // Active module - some methods reserved for future agent lifecycle features
#![allow(unused_imports)]

pub mod activity;
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

// Activity detection
pub use activity::{LlmHookDetector, MockActivityDetector, TmuxActivityDetector};

// Ticket/agent generation
pub use generator::{
    AgentTicketCreator, AgentTicketResult, AssessTicketCreator, AssessTicketResult, AGENT_TOOLS,
};

// Launcher
pub use launcher::{LaunchOptions, Launcher, PreparedLaunch, RelaunchOptions};

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
pub use tmux_config::{generate_status_script, generate_tmux_conf};
