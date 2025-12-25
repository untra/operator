//! Backstage integration for project cataloging and taxonomy management.
//!
//! This module provides:
//! - A 24-Kind project taxonomy (source of truth in `taxonomy.toml`)
//! - catalog-info.yaml generation for Backstage catalog
//! - Backstage scaffold generator for local deployment
//! - Static branding defaults
//! - Project analysis and Kind detection
//! - Bun-based server lifecycle management

pub mod analyzer;
pub mod branding;
pub mod scaffold;
pub mod server;
pub mod taxonomy;

// Re-exports for TUI integration and testing
pub use server::{BackstageServer, ServerStatus};

// Additional re-exports for tests and future use
#[allow(unused_imports)]
pub use server::{BackstageError, BunClient, BunVersion, MockBunClient, SystemBunClient};
