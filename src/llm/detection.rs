//! LLM tool detection logic
//!
//! Detects available LLM CLI tools by checking binary existence
//! and loading configurations from embedded JSON files.

use std::process::Command;

use crate::config::{DetectedTool, LlmProvider, LlmToolsConfig, ToolCapabilities};

use super::tool_config::{load_all_tool_configs, ToolConfig};

/// Detect all available LLM CLI tools and build the config
pub fn detect_all_tools() -> LlmToolsConfig {
    let tool_configs = load_all_tool_configs();
    let mut detected = Vec::new();
    let mut providers = Vec::new();

    for config in tool_configs {
        if let Some(tool) = detect_tool(&config) {
            // Build provider pairs from tool + each model alias
            for model in &config.model_aliases {
                providers.push(LlmProvider {
                    tool: tool.name.clone(),
                    model: model.clone(),
                    display_name: Some(format!("{} {}", config.display_name(), capitalize(model))),
                });
            }
            detected.push(tool);
        }
    }

    LlmToolsConfig {
        detected,
        providers,
        detection_complete: true,
    }
}

/// Detect a single tool from its config
fn detect_tool(config: &ToolConfig) -> Option<DetectedTool> {
    // Check if binary exists
    let path = get_binary_path(&config.tool_name)?;
    let version = get_version(&config.version_command).unwrap_or_else(|| "unknown".to_string());

    Some(DetectedTool {
        name: config.tool_name.clone(),
        path,
        version,
        model_aliases: config.model_aliases.clone(),
        command_template: config.command_template.clone(),
        capabilities: ToolCapabilities {
            supports_sessions: config.capabilities.supports_sessions,
            supports_headless: config.capabilities.supports_headless,
        },
        yolo_flags: config.yolo_flags.clone(),
    })
}

/// Get binary path using `which`
fn get_binary_path(tool_name: &str) -> Option<String> {
    Command::new("which")
        .arg(tool_name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Get version by running the version command
fn get_version(version_command: &str) -> Option<String> {
    let parts: Vec<&str> = version_command.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Capitalize the first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("opus"), "Opus");
        assert_eq!(capitalize("sonnet"), "Sonnet");
        assert_eq!(capitalize("haiku"), "Haiku");
        assert_eq!(capitalize("gpt-4o"), "Gpt-4o");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_detect_all_tools_structure() {
        let config = detect_all_tools();
        assert!(config.detection_complete);
        // If claude is installed, we should have providers
        // If not, the lists will be empty but that's okay
    }

    #[test]
    fn test_detected_tool_has_required_fields() {
        let config = detect_all_tools();
        for tool in &config.detected {
            assert!(!tool.name.is_empty(), "Tool name should not be empty");
            assert!(!tool.path.is_empty(), "Tool path should not be empty");
            // Version can be "unknown" but not empty
            assert!(!tool.version.is_empty(), "Tool version should not be empty");
            // Model aliases should not be empty
            assert!(
                !tool.model_aliases.is_empty(),
                "Tool should have at least one model alias"
            );
        }
    }

    #[test]
    fn test_providers_match_detected_tools() {
        let config = detect_all_tools();
        // Each provider should reference a detected tool
        for provider in &config.providers {
            let tool_exists = config.detected.iter().any(|t| t.name == provider.tool);
            assert!(
                tool_exists,
                "Provider {} references unknown tool",
                provider.tool
            );
        }
    }

    #[test]
    fn test_provider_display_name_format() {
        let config = detect_all_tools();
        for provider in &config.providers {
            if let Some(display) = &provider.display_name {
                // Display name should contain the tool name or model
                let display_lower = display.to_lowercase();
                assert!(
                    display_lower.contains(&provider.tool)
                        || display_lower.contains(&provider.model),
                    "Display name '{}' should contain tool '{}' or model '{}'",
                    display,
                    provider.tool,
                    provider.model
                );
            }
        }
    }

    #[test]
    fn test_provider_count_matches_detected_tools() {
        let config = detect_all_tools();
        // Total providers should equal sum of model_aliases across detected tools
        let expected_count: usize = config.detected.iter().map(|t| t.model_aliases.len()).sum();
        assert_eq!(
            config.providers.len(),
            expected_count,
            "Provider count should match sum of model aliases"
        );
    }
}
