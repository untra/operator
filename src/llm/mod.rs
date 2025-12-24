//! LLM CLI tool detection and configuration
//!
//! This module handles detection of LLM CLI tools (Claude Code, Gemini, Codex)
//! and provides configuration for which tool/model pairs are available.
//!
//! Tool configurations are defined in JSON files under `tools/` and loaded
//! at compile time. Detection checks if binaries exist on the system PATH.

mod detection;
mod tool_config;

pub use detection::detect_all_tools;
