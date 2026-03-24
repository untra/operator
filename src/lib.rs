//! Operator - Multi-agent orchestration for AI coding assistants
//!
//! This library module exports types for the `generate_types` binary
//! and potential future library consumers.

// Per-item #[allow(dead_code)] is used where needed; no blanket suppression

// Public modules for type generation
pub mod agents;
pub mod api;
pub mod config;
pub mod git;
pub mod queue;
pub mod rest;
pub mod state;
pub mod types;

// Internal modules required by public modules
mod backstage;
mod collections;
mod issuetypes;
mod llm;
mod notifications;
mod permissions;
mod pr_config;
mod projects;
mod services;
mod startup;
mod templates;
pub mod version;

// MCP server bridge
pub mod mcp;

// Re-export env_vars for potential external use
pub mod env_vars;
