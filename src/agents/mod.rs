#![allow(dead_code)] // Active module - some methods reserved for future agent lifecycle features
#![allow(unused_imports)]

mod generator;
pub mod hooks;
pub mod idle_detector;
mod launcher;
mod monitor;
mod pr_workflow;
mod session;
mod sync;
pub mod tmux;
pub mod tmux_config;
mod visual_review;

pub use generator::{
    AgentTicketCreator, AgentTicketResult, AssessTicketCreator, AssessTicketResult, AGENT_TOOLS,
};
pub use launcher::{LaunchOptions, Launcher, RelaunchOptions};
pub use monitor::{HealthCheckResult, ReconciliationResult, SessionMonitor};
pub use pr_workflow::PrWorkflow;
pub use session::Session;
pub use sync::{SyncAction, SyncResult, TicketSessionSync};
pub use tmux::{
    sanitize_session_name, MockTmuxClient, SystemTmuxClient, TmuxClient, TmuxError, TmuxSession,
    TmuxVersion,
};
pub use tmux_config::{generate_status_script, generate_tmux_conf};
pub use visual_review::{VisualReviewHandler, VisualReviewResult};
