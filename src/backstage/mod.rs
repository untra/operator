//! Backstage integration for project cataloging and taxonomy management.
//!
//! This module provides:
//! - A 24-Kind project taxonomy (source of truth in `taxonomy.toml`)
//! - catalog-info.yaml generation for Backstage catalog
//! - Backstage scaffold generator for local deployment
//! - Static branding defaults
//! - Project analysis and Kind detection
//! - Bun-based server lifecycle management
//! - Compiled binary runtime management

pub mod analyzer;
pub mod branding;
pub mod runtime;
pub mod scaffold;
pub mod server;
pub mod taxonomy;

// Re-exports for TUI integration and testing
pub use server::{BackstageServer, ServerStatus};

// Runtime management re-exports (public API for future use)
#[allow(unused_imports)]
pub use runtime::{BackstageRuntime, Platform, RuntimeError};

// Additional re-exports for tests and future use
#[allow(unused_imports)]
pub use server::{
    copy_default_logo, generate_branding_config, BackstageError, BunClient, BunVersion,
    MockBunClient, RuntimeBinaryClient, SystemBunClient,
};
