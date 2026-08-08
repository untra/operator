//! LLM CLI tool detection and configuration
//!
//! This module handles detection of LLM CLI tools (Claude Code, Gemini, Codex,
//! plus user-defined tools) and provides configuration for which tool/model
//! pairs are available.
//!
//! Builtin tool configurations are embedded from JSON files under `tools/`;
//! user tool configurations are loaded at runtime from
//! `<config dir>/operator/tools/*.json`. Detection checks if binaries exist on
//! the system PATH unless a tool opts out via `detection.mode: "always"`.

mod detection;
pub mod skill_deployer;
pub mod tool_config;

#[allow(unused_imports)] // Used by main.rs binary
pub use detection::{detect_all_tools, refresh_tool_detection};
pub use skill_deployer::deploy_skills;
