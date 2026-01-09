//! Operator - Multi-agent orchestration for AI coding assistants
//!
//! This library module exports types for the generate_types binary
//! and potential future library consumers.

// Allow dead code in the library - some internal modules are only used by main.rs
#![allow(dead_code)]
#![allow(unused_imports)]

// Public modules for type generation
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
mod permissions;
mod projects;
mod startup;
mod templates;
pub mod version;

// Re-export env_vars for potential external use
pub mod env_vars;
