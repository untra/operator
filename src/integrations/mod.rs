//! Integration surface inventory.
//!
//! Re-exports the capability inventory used by surface parity tests
//! to ensure slash commands, MCP tools, REST routes, and TUI actions
//! stay aligned.

pub mod inventory;

pub use inventory::{all_capabilities, Capability};
