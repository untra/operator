//! Integration surface inventory.
//!
//! Re-exports the capability inventory used by surface parity tests
//! to ensure slash commands, MCP tools, REST routes, and TUI actions
//! stay aligned.

pub mod catalog;
pub mod inventory;
pub mod support_status;

pub use catalog::{all_integrations, entry_for, CatalogEntry, Vertical};
pub use inventory::{all_capabilities, Capability};
pub use support_status::SupportStatus;
