#![allow(dead_code)]
#![allow(unused_imports)]

mod generator;
mod launcher;
mod monitor;
mod session;
mod sync;
pub mod tmux;
pub mod tmux_config;

pub use generator::{AgentTicketCreator, AgentTicketResult, AGENT_TOOLS};
pub use launcher::Launcher;
pub use monitor::{HealthCheckResult, ReconciliationResult, SessionMonitor};
pub use session::Session;
pub use sync::{SyncAction, SyncResult, TicketSessionSync};
pub use tmux::{
    sanitize_session_name, MockTmuxClient, SystemTmuxClient, TmuxClient, TmuxError, TmuxSession,
    TmuxVersion,
};
pub use tmux_config::{generate_status_script, generate_tmux_conf};
