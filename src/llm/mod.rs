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

pub use detection::verify_tool_health;
#[allow(unused_imports)] // Used by main.rs binary
pub use detection::{detect_all_tools, refresh_tool_detection};
pub use skill_deployer::deploy_skills;

/// Refresh cached tool detection and persist only when the detected state changed.
#[allow(dead_code)] // Shared by the binary's TUI and API entry points.
pub fn refresh_config_detection(config: &mut crate::config::Config) -> bool {
    let refreshed = refresh_tool_detection(&config.llm_tools);
    let changed =
        serde_json::to_value(&refreshed).ok() != serde_json::to_value(&config.llm_tools).ok();
    config.llm_tools = refreshed;

    for tool in &config.llm_tools.detected {
        tracing::info!(
            tool = %tool.name,
            version = %tool.version,
            path = %tool.path,
            "LLM tool detected"
        );
    }
    for provider in &config.llm_tools.providers {
        tracing::debug!(tool = %provider.tool, model = %provider.model, "LLM provider available");
    }
    if changed {
        if let Err(error) = config.save() {
            tracing::warn!(%error, "Failed to save LLM detection results");
        }
    }
    changed
}
