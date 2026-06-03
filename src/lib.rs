//! Operator - Multi-agent orchestration for AI coding assistants
//!
//! This library module exports types for the `generate_types` binary
//! and potential future library consumers.

// Per-item #[allow(dead_code)] is used where needed; no blanket suppression

// Public modules for type generation
pub mod agents;
pub mod api;
pub mod config;
pub mod editors;
pub mod git;
pub mod queue;
pub mod rest;
pub mod state;
pub mod types;

// Internal modules required by public modules
mod collections;
mod issuetypes;
mod llm;
mod notifications;
mod permissions;
mod pr_config;
mod projects;
mod services;
mod startup;
mod steps;
#[allow(dead_code)]
pub mod taxonomy;
mod templates;
pub mod version;

// Integration surface inventory (capability parity across surfaces)
pub mod integrations;

// MCP server bridge
pub mod mcp;

// ACP agent bridge (Agent Client Protocol — editor-hosted sessions over stdio)
pub mod acp;

// Re-export env_vars for potential external use
pub mod env_vars;

// Relay hub and channel client
pub mod relay;

// Workflow export (ticket + issuetype -> Claude dynamic workflow .js).
// Declared here (in addition to the bin) so the REST layer, which compiles in
// both the lib and bin crates, can reach it.
pub mod workflow_gen;
