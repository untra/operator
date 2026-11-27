//! Operator - Multi-agent orchestration for AI coding assistants
//!
//! This library module exports types for the `generate_types` binary
//! and potential future library consumers.

// Per-item #[allow(dead_code)] is used where needed; no blanket suppression

// Public modules for type generation
pub mod agents;
pub mod api;
pub mod auth;
pub mod campaigns;
pub mod collections;
pub mod config;
pub mod editors;
pub mod git;
pub mod http_client;
pub mod licensing;
pub mod ownership;
pub mod profiles;
pub mod queue;
pub mod rest;
pub mod setup;
pub mod startup;
pub mod state;
pub mod trust_verify;
pub mod types;

// Internal modules required by public modules
mod issuetypes;
mod llm;
mod notifications;
mod permissions;
mod pr_config;
pub mod projects;
mod runtime_scripts;
pub mod services;
mod steps;
#[allow(dead_code)]
pub mod taxonomy;
pub mod telemetry;
mod templates;
pub mod version;
#[cfg(windows)]
mod windows_files;

// Integration surface inventory (capability parity across surfaces)
pub mod integrations;

// MCP server bridge
pub mod mcp;

// ACP agent bridge (Agent Client Protocol - editor-hosted sessions over stdio)
pub mod acp;

// Re-export env_vars for potential external use
pub mod env_vars;

// Relay hub and channel client
pub mod relay;

// Workflow export (ticket + issuetype -> Claude dynamic workflow .js).
// Declared here (in addition to the bin) so the REST layer, which compiles in
// both the lib and bin crates, can reach it.
pub mod workflow_gen;

// Endpoint client uplink protocol (Windows/macOS tray clients)
pub mod endpoint;

#[cfg(test)]
#[path = "../tests/it/suites/endpoint_uplink.rs"]
mod endpoint_uplink;
