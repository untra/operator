//! Tool configuration loading and templating
//!
//! This module loads LLM CLI tool configurations — embedded builtin JSONs plus
//! user JSONs from `<config dir>/operator/tools/` — and provides template-based
//! command building. User configs are only ever read from the user-global
//! config dir, never from repo-local paths (see [`load_user_tool_configs`]).

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

/// Hook configuration for tools that support hooks (Claude, Gemini)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Hook event name (e.g., "Stop" for Claude, "`AfterAgent`" for Gemini)
    pub event_name: String,
    /// Path to hook script (e.g., "~/.claude/hooks/operator-stop.sh")
    pub script_path: String,
    /// Settings file path for this tool (e.g., "~/.claude/settings.json")
    pub settings_path: String,
}

/// Well-known directories where a tool stores skill/command files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillDirectories {
    /// Global directories (absolute paths, ~ expanded to home)
    #[serde(default)]
    pub global: Vec<String>,
    /// Project-relative directories
    #[serde(default)]
    pub project: Vec<String>,
}

/// Configuration for idle state detection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdleDetectionConfig {
    /// Regex patterns that indicate tool is idle/waiting for input (e.g., prompt chars)
    #[serde(default)]
    pub idle_patterns: Vec<String>,
    /// Regex patterns that indicate tool is actively working (spinners, status messages)
    #[serde(default)]
    pub activity_patterns: Vec<String>,
    /// Hook configuration for this tool (if supported)
    #[serde(default)]
    pub hook_config: Option<HookConfig>,
}

/// How a tool's presence is determined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectionMode {
    /// Gate on a successful `which <tool_name>` lookup (default)
    #[default]
    Which,
    /// Skip the PATH lookup; `tool_name` is used as the invocation path.
    Always,
}

/// Detection behavior overrides for a tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectionConfig {
    #[serde(default)]
    pub mode: DetectionMode,
    /// Health check run every startup. Which-mode tools store their verified presence and only need this if set;
    /// always-mode tools have nothing locally verifiable and stay unhealthy until this passes.
    #[serde(default)]
    pub health_command: Option<String>,
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
    /// Minimum required version for Operator compatibility
    #[serde(default)]
    pub min_version: Option<String>,
    /// Tool capabilities
    pub capabilities: ToolCapabilities,
    /// Available model aliases (e.g., ["opus", "sonnet", "haiku"])
    pub model_aliases: Vec<String>,
    /// Mapping of argument names to CLI flags
    pub arg_mapping: ArgMapping,
    /// Template for building the CLI command
    /// Variables: {{model}}, {{`model_flag`}}, {{`session_id`}}, {{`prompt_file`}}
    pub command_template: String,
    /// CLI flags for YOLO (auto-accept) mode
    #[serde(default)]
    pub yolo_flags: Vec<String>,
    /// Configuration for idle/awaiting state detection
    #[serde(default)]
    pub idle_detection: Option<IdleDetectionConfig>,
    /// Well-known directories for skill/command files
    #[serde(default)]
    pub skill_directories: Option<SkillDirectories>,
    /// Detection behavior (None = which-gated, no health check)
    #[serde(default)]
    pub detection: Option<DetectionConfig>,
}

impl ToolConfig {
    /// Get the display name, falling back to `tool_name`
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.tool_name)
    }

    /// Effective detection mode (default: `Which`)
    pub fn detection_mode(&self) -> DetectionMode {
        self.detection.as_ref().map(|d| d.mode).unwrap_or_default()
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

/// Operator's directory under the platform config dir (matches `src/config.rs`)
const OPERATOR_CONFIG_DIR_NAME: &str = "operator";
/// Subdirectory scanned for user tool configs
const USER_TOOLS_DIR_NAME: &str = "tools";
const TOOL_CONFIG_EXT: &str = "json";

const BUILTIN_TOOL_CONFIGS: &[(&str, &str)] = &[
    ("claude", include_str!("tools/claude.json")),
    ("gemini", include_str!("tools/gemini.json")),
    ("codex", include_str!("tools/codex.json")),
];

/// The user tool-config directory: `<platform config dir>/operator/tools`
pub fn user_tools_dir() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join(OPERATOR_CONFIG_DIR_NAME).join(USER_TOOLS_DIR_NAME))
}

/// Load all tool configurations: embedded builtins plus user JSONs from
/// [`user_tools_dir`]. A user config whose `tool_name` matches a builtin fully
/// replaces it; new names append.
pub fn load_all_tool_configs() -> Vec<ToolConfig> {
    load_all_tool_configs_with(user_tools_dir().as_deref())
}

/// Injectable-dir variant for tests
pub(crate) fn load_all_tool_configs_with(user_dir: Option<&std::path::Path>) -> Vec<ToolConfig> {
    let mut configs = load_builtin_tool_configs();
    if let Some(dir) = user_dir {
        for user_config in load_user_tool_configs(dir) {
            match configs
                .iter()
                .position(|c| c.tool_name == user_config.tool_name)
            {
                Some(pos) => configs[pos] = user_config,
                None => configs.push(user_config),
            }
        }
    }
    configs
}

fn load_builtin_tool_configs() -> Vec<ToolConfig> {
    BUILTIN_TOOL_CONFIGS
        .iter()
        .filter_map(|(name, json)| match serde_json::from_str(json) {
            Ok(config) => Some(config),
            Err(e) => {
                tracing::warn!(tool = name, error = %e, "Failed to parse builtin tool config");
                None
            }
        })
        .collect()
}

/// Read `*.json` tool configs from a user directory, sorted by filename so
/// duplicate `tool_name`s resolve deterministically (last wins). Malformed or
/// unreadable files are skipped with a warning.
///
/// Only the user-global config dir is ever scanned — never repo-local paths:
/// `command_template` is arbitrary shell executed at launch, so loading tool
/// configs from a checked-out repository would be a supply-chain hazard.
fn load_user_tool_configs(dir: &std::path::Path) -> Vec<ToolConfig> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == TOOL_CONFIG_EXT))
        .collect();
    paths.sort();

    let mut configs: Vec<ToolConfig> = Vec::new();
    for path in paths {
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "Failed to read user tool config");
                continue;
            }
        };
        match serde_json::from_str::<ToolConfig>(&contents) {
            Ok(config) => {
                if let Some(pos) = configs.iter().position(|c| c.tool_name == config.tool_name) {
                    tracing::warn!(
                        path = %path.display(),
                        tool = %config.tool_name,
                        "Duplicate tool_name in user tool configs; later file wins"
                    );
                    configs[pos] = config;
                } else {
                    configs.push(config);
                }
            }
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "Failed to parse user tool config; skipping");
            }
        }
    }
    configs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_json(tool_name: &str, display_name: &str) -> String {
        format!(
            r#"{{
                "tool_name": "{tool_name}",
                "display_name": "{display_name}",
                "version_command": "{tool_name} --version",
                "capabilities": {{ "supports_sessions": false, "supports_headless": true }},
                "model_aliases": ["default"],
                "arg_mapping": {{ "prompt": "", "model": "" }},
                "command_template": "{tool_name} \"$(cat {{{{prompt_file}}}})\""
            }}"#
        )
    }

    #[test]
    fn test_load_all_tool_configs() {
        let configs = load_all_tool_configs_with(None);
        assert_eq!(configs.len(), 3);

        let names: Vec<_> = configs.iter().map(|c| c.tool_name.as_str()).collect();
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"gemini"));
        assert!(names.contains(&"codex"));
    }

    #[test]
    fn test_user_dir_adds_new_tool() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("agy.json"), tool_json("agy", "Agy")).unwrap();

        let configs = load_all_tool_configs_with(Some(dir.path()));
        assert_eq!(configs.len(), 4);
        let agy = configs.iter().find(|c| c.tool_name == "agy").unwrap();
        assert_eq!(agy.display_name(), "Agy");
    }

    #[test]
    fn test_user_config_replaces_builtin_by_tool_name() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("claude.json"),
            tool_json("claude", "My Claude"),
        )
        .unwrap();

        let configs = load_all_tool_configs_with(Some(dir.path()));
        assert_eq!(configs.len(), 3);
        let claude = configs.iter().find(|c| c.tool_name == "claude").unwrap();
        // Full replacement: user's file wins entirely, not a field merge
        assert_eq!(claude.display_name(), "My Claude");
        assert!(claude.min_version.is_none());
    }

    #[test]
    fn test_malformed_user_config_skipped() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("broken.json"), "{ not json").unwrap();
        std::fs::write(dir.path().join("agy.json"), tool_json("agy", "Agy")).unwrap();

        let configs = load_all_tool_configs_with(Some(dir.path()));
        assert_eq!(configs.len(), 4);
        assert!(configs.iter().any(|c| c.tool_name == "agy"));
    }

    #[test]
    fn test_missing_user_dir_ok() {
        let dir = tempfile::TempDir::new().unwrap();
        let missing = dir.path().join("does-not-exist");
        let configs = load_all_tool_configs_with(Some(&missing));
        assert_eq!(configs.len(), 3);
    }

    #[test]
    fn test_non_json_files_ignored() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("README.md"), "# tools").unwrap();

        let configs = load_all_tool_configs_with(Some(dir.path()));
        assert_eq!(configs.len(), 3);
    }

    #[test]
    fn test_detection_defaults_to_which() {
        let configs = load_all_tool_configs();
        let claude = configs.iter().find(|c| c.tool_name == "claude").unwrap();
        assert!(claude.detection.is_none());
        assert_eq!(claude.detection_mode(), DetectionMode::Which);
    }

    #[test]
    fn test_detection_mode_always_parses() {
        let json = r#"{
            "tool_name": "agy",
            "version_command": "agy --version",
            "capabilities": { "supports_sessions": false, "supports_headless": true },
            "model_aliases": ["default"],
            "arg_mapping": { "prompt": "", "model": "" },
            "command_template": "agy \"$(cat {{prompt_file}})\"",
            "detection": { "mode": "always", "health_command": "agy ping" }
        }"#;
        let config: ToolConfig = serde_json::from_str(json).expect("detection config parses");
        assert_eq!(config.detection_mode(), DetectionMode::Always);
        let detection = config.detection.unwrap();
        assert_eq!(detection.health_command.as_deref(), Some("agy ping"));

        let json_no_health = r#"{
            "tool_name": "agy",
            "version_command": "agy --version",
            "capabilities": { "supports_sessions": false, "supports_headless": true },
            "model_aliases": ["default"],
            "arg_mapping": { "prompt": "", "model": "" },
            "command_template": "agy \"$(cat {{prompt_file}})\"",
            "detection": { "mode": "always" }
        }"#;
        let config: ToolConfig = serde_json::from_str(json_no_health).unwrap();
        assert_eq!(config.detection_mode(), DetectionMode::Always);
        assert!(config.detection.unwrap().health_command.is_none());
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

    #[test]
    fn test_skill_directories_claude() {
        let configs = load_all_tool_configs();
        let claude = configs.iter().find(|c| c.tool_name == "claude").unwrap();
        let dirs = claude
            .skill_directories
            .as_ref()
            .expect("claude should have skill_directories");
        assert_eq!(dirs.global, vec!["~/.claude/commands/"]);
        assert_eq!(dirs.project, vec![".claude/commands/"]);
    }

    #[test]
    fn test_skill_directories_codex() {
        let configs = load_all_tool_configs();
        let codex = configs.iter().find(|c| c.tool_name == "codex").unwrap();
        let dirs = codex
            .skill_directories
            .as_ref()
            .expect("codex should have skill_directories");
        assert!(dirs.global.is_empty());
        assert_eq!(dirs.project, vec![".codex/", "AGENTS.md"]);
    }

    #[test]
    fn test_skill_directories_gemini() {
        let configs = load_all_tool_configs();
        let gemini = configs.iter().find(|c| c.tool_name == "gemini").unwrap();
        let dirs = gemini
            .skill_directories
            .as_ref()
            .expect("gemini should have skill_directories");
        assert!(dirs.global.is_empty());
        assert_eq!(dirs.project, vec![".gemini/"]);
    }
}
