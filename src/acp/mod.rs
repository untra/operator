//! Agent Client Protocol (ACP) integration for Operator.
//!
//! Operator runs as an ACP agent that editors (Zed, `JetBrains`, Emacs,
//! Kiro, `OpenCode`, marimo, Eclipse) launch as a stdio subprocess. Each
//! ACP session maps to one operator ticket (created or attached when
//! the editor's cwd matches an in-progress ticket).
//!
//! Phase A (this file's current scope): only `initialize` is handled.
//! Sessions, prompts, and editor config snippets land in Phase B.
//!
//! See: <https://agentclientprotocol.com/>

pub mod agent;
pub mod client_configs;
pub mod server;
pub mod session;
pub mod translator;

pub use agent::run_stdio;
pub use server::{AcpAgentServer, AcpAgentStatus};
// SessionRegistry and AcpSession are intentionally not re-exported at the
// `acp::*` root: they're internal to the agent runtime. Callers that need
// them can use `acp::session::*`.
