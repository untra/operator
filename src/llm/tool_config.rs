//! Tool configuration loading and templating
//!
//! This module loads LLM CLI tool configurations from embedded JSON files
//! and provides template-based command building.

use serde::{Deserialize, Serialize};

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCapabilities {
    /// Whether the tool supports session continuity via UUID
    pub supports_sessions: bool,
    /// Whether the tool can run in headless/non-interactive mode
    #[serde(default)]
    pub supports_headless: bool,
}

/// Argument mapping for CLI flags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArgMapping {
    /// Flag for prompt/instruction (e.g., "-p", "--prompt")
    pub prompt: String,
    /// Flag for model selection (e.g., "--model", "-m")
    pub model: String,
    /// Flag for session ID (e.g., "--session-id", "--resume")
    #[serde(default)]
    pub session_id: Option<String>,
    /// Flag for quiet/non-interactive mode
    #[serde(default)]
    pub quiet: Option<String>,
}

/// Tool configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Binary name (e.g., "claude", "gemini", "codex")
    pub tool_name: String,
    /// Human-readable display name (e.g., "Claude Code")
    #[serde(default)]
    pub display_name: Option<String>,
    /// Command to get version (e.g., "claude --version")
    pub version_command: String,
    /// Tool capabilities
    pub capabilities: ToolCapabilities,
    /// Available model aliases (e.g., ["opus", "sonnet", "haiku"])
    pub model_aliases: Vec<String>,
    /// Mapping of argument names to CLI flags
    pub arg_mapping: ArgMapping,
    /// Template for building the CLI command
    /// Variables: {{model}}, {{model_flag}}, {{session_id}}, {{prompt_file}}
    pub command_template: String,
    /// CLI flags for YOLO (auto-accept) mode
    #[serde(default)]
    pub yolo_flags: Vec<String>,
}

impl ToolConfig {
    /// Get the display name, falling back to tool_name
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.tool_name)
    }

    /// Build a command string by substituting template variables
    #[allow(dead_code)] // Used in tests
    pub fn build_command(&self, model: &str, session_id: &str, prompt_file: &str) -> String {
        let model_flag = if self.arg_mapping.model.is_empty() {
            String::new()
        } else {
            format!("{} {} ", self.arg_mapping.model, model)
        };

        self.command_template
            .replace("{{model_flag}}", &model_flag)
            .replace("{{model}}", model)
            .replace("{{session_id}}", session_id)
            .replace("{{prompt_file}}", prompt_file)
    }
}

/// Load all embedded tool configurations
pub fn load_all_tool_configs() -> Vec<ToolConfig> {
    let mut configs = Vec::new();

    // Load Claude config
    if let Ok(config) = serde_json::from_str::<ToolConfig>(include_str!("tools/claude.json")) {
        configs.push(config);
    } else {
        tracing::warn!("Failed to parse claude.json tool config");
    }

    // Load Gemini config
    if let Ok(config) = serde_json::from_str::<ToolConfig>(include_str!("tools/gemini.json")) {
        configs.push(config);
    } else {
        tracing::warn!("Failed to parse gemini.json tool config");
    }

    // Load Codex config
    if let Ok(config) = serde_json::from_str::<ToolConfig>(include_str!("tools/codex.json")) {
        configs.push(config);
    } else {
        tracing::warn!("Failed to parse codex.json tool config");
    }

    configs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_tool_configs() {
        let configs = load_all_tool_configs();
        assert_eq!(configs.len(), 3);

        let names: Vec<_> = configs.iter().map(|c| c.tool_name.as_str()).collect();
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"gemini"));
        assert!(names.contains(&"codex"));
    }

    #[test]
    fn test_build_command_claude() {
        let configs = load_all_tool_configs();
        let claude = configs.iter().find(|c| c.tool_name == "claude").unwrap();

        let cmd = claude.build_command("opus", "abc-123", "/tmp/prompt.txt");
        assert!(cmd.contains("--model opus"));
        assert!(cmd.contains("--session-id abc-123"));
        assert!(cmd.contains("/tmp/prompt.txt"));
    }

    #[test]
    fn test_build_command_codex() {
        let configs = load_all_tool_configs();
        let codex = configs.iter().find(|c| c.tool_name == "codex").unwrap();

        let cmd = codex.build_command("gpt-4o", "xyz-789", "/tmp/prompt.txt");
        assert!(cmd.contains("-m gpt-4o"));
        assert!(cmd.contains("--resume xyz-789"));
        assert!(cmd.contains("/tmp/prompt.txt"));
    }

    #[test]
    fn test_display_name() {
        let configs = load_all_tool_configs();
        let claude = configs.iter().find(|c| c.tool_name == "claude").unwrap();
        assert_eq!(claude.display_name(), "Claude Code");
    }
}
