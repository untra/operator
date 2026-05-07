//! Relay hub and channel-session types for operator and opr8r.
//!
//! This crate is shared between the `operator` TUI and the `opr8r` step-wrapper
//! so that relay tooling can be distributed via the signed `opr8r` binary without
//! pulling in the full TUI/REST dependency stack.

#[cfg(unix)]
pub mod channel_session;
#[cfg(unix)]
pub mod client;
#[cfg(unix)]
pub mod hub;
pub mod protocol;
pub mod session_name;
pub mod socket_path;
