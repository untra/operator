//! LLM tool detection logic
//!
//! Detects available LLM CLI tools by checking binary existence
//! and loading configurations from embedded JSON files.

use std::cmp::Ordering;
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
                    ..Default::default()
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

    // Check if installed version meets minimum requirement
    let version_ok = match &config.min_version {
        Some(min_ver) => check_version_meets_minimum(&version, min_ver),
        None => true, // No minimum specified = always OK
    };

    if !version_ok {
        tracing::warn!(
            tool = %config.tool_name,
            installed = %version,
            required = config.min_version.as_deref().unwrap_or("none"),
            "Tool version is below minimum required version"
        );
    }

    Some(DetectedTool {
        name: config.tool_name.clone(),
        path,
        version,
        min_version: config.min_version.clone(),
        version_ok,
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

/// Check if installed version meets the minimum required version.
///
/// Extracts semver-like version numbers from strings and compares them.
/// Handles formats like "claude 1.0.34", "1.0.34", "v1.0.34", etc.
fn check_version_meets_minimum(installed: &str, minimum: &str) -> bool {
    let installed_parts = extract_version_parts(installed);
    let minimum_parts = extract_version_parts(minimum);

    match compare_version_parts(&installed_parts, &minimum_parts) {
        Ordering::Less => false,
        Ordering::Equal | Ordering::Greater => true,
    }
}

/// Extract version parts (major, minor, patch) from a version string.
///
/// Handles various formats:
/// - "1.0.34" -> [1, 0, 34]
/// - "v1.0.34" -> [1, 0, 34]
/// - "claude 1.0.34" -> [1, 0, 34]
/// - "claude code v1.0.34 (abc123)" -> [1, 0, 34]
fn extract_version_parts(version_str: &str) -> Vec<u32> {
    // Simple regex-free approach: split and find numeric.numeric pattern
    for word in version_str.split_whitespace() {
        let cleaned = word.trim_start_matches('v').trim_start_matches('V');
        let parts: Vec<&str> = cleaned.split('.').collect();

        if parts.len() >= 2 {
            let parsed: Vec<Option<u32>> = parts.iter().map(|p| p.parse::<u32>().ok()).collect();

            // If at least major.minor are valid numbers
            if parsed.len() >= 2 && parsed[0].is_some() && parsed[1].is_some() {
                return parsed.into_iter().flatten().collect();
            }
        }
    }

    // Fallback: try the whole string as version
    let cleaned = version_str
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V');
    let parts: Vec<&str> = cleaned.split('.').collect();
    parts.iter().filter_map(|p| p.parse::<u32>().ok()).collect()
}

/// Compare two version part vectors.
///
/// Compares element by element, treating missing elements as 0.
fn compare_version_parts(a: &[u32], b: &[u32]) -> Ordering {
    let max_len = a.len().max(b.len());

    for i in 0..max_len {
        let a_part = a.get(i).copied().unwrap_or(0);
        let b_part = b.get(i).copied().unwrap_or(0);

        match a_part.cmp(&b_part) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
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

    #[test]
    fn test_extract_version_parts_simple() {
        assert_eq!(extract_version_parts("1.0.34"), vec![1, 0, 34]);
        assert_eq!(extract_version_parts("2.1.0"), vec![2, 1, 0]);
        assert_eq!(extract_version_parts("0.1.0"), vec![0, 1, 0]);
    }

    #[test]
    fn test_extract_version_parts_with_prefix() {
        assert_eq!(extract_version_parts("v1.0.34"), vec![1, 0, 34]);
        assert_eq!(extract_version_parts("V2.1.0"), vec![2, 1, 0]);
    }

    #[test]
    fn test_extract_version_parts_with_tool_name() {
        assert_eq!(extract_version_parts("claude 1.0.34"), vec![1, 0, 34]);
        assert_eq!(extract_version_parts("claude code v2.1.0"), vec![2, 1, 0]);
        assert_eq!(
            extract_version_parts("gemini cli 0.5.12 (abc123)"),
            vec![0, 5, 12]
        );
    }

    #[test]
    fn test_extract_version_parts_major_minor_only() {
        assert_eq!(extract_version_parts("1.0"), vec![1, 0]);
        assert_eq!(extract_version_parts("v2.5"), vec![2, 5]);
    }

    #[test]
    fn test_compare_version_parts_equal() {
        assert_eq!(
            compare_version_parts(&[1, 0, 34], &[1, 0, 34]),
            Ordering::Equal
        );
        assert_eq!(
            compare_version_parts(&[2, 1, 0], &[2, 1, 0]),
            Ordering::Equal
        );
    }

    #[test]
    fn test_compare_version_parts_greater() {
        assert_eq!(
            compare_version_parts(&[2, 0, 0], &[1, 0, 0]),
            Ordering::Greater
        );
        assert_eq!(
            compare_version_parts(&[1, 1, 0], &[1, 0, 0]),
            Ordering::Greater
        );
        assert_eq!(
            compare_version_parts(&[1, 0, 1], &[1, 0, 0]),
            Ordering::Greater
        );
    }

    #[test]
    fn test_compare_version_parts_less() {
        assert_eq!(
            compare_version_parts(&[1, 0, 0], &[2, 0, 0]),
            Ordering::Less
        );
        assert_eq!(
            compare_version_parts(&[1, 0, 0], &[1, 1, 0]),
            Ordering::Less
        );
        assert_eq!(
            compare_version_parts(&[1, 0, 0], &[1, 0, 1]),
            Ordering::Less
        );
    }

    #[test]
    fn test_compare_version_parts_different_lengths() {
        // 1.0 should equal 1.0.0
        assert_eq!(compare_version_parts(&[1, 0], &[1, 0, 0]), Ordering::Equal);
        // 1.0.1 > 1.0
        assert_eq!(
            compare_version_parts(&[1, 0, 1], &[1, 0]),
            Ordering::Greater
        );
    }

    #[test]
    fn test_check_version_meets_minimum() {
        // Exact match
        assert!(check_version_meets_minimum("1.0.34", "1.0.34"));

        // Greater version
        assert!(check_version_meets_minimum("2.0.0", "1.0.34"));
        assert!(check_version_meets_minimum("1.1.0", "1.0.34"));
        assert!(check_version_meets_minimum("1.0.35", "1.0.34"));

        // Lower version
        assert!(!check_version_meets_minimum("1.0.33", "1.0.34"));
        assert!(!check_version_meets_minimum("0.9.0", "1.0.34"));

        // With tool name prefix
        assert!(check_version_meets_minimum("claude 2.1.0", "2.1.0"));
        assert!(!check_version_meets_minimum("claude 1.0.0", "2.1.0"));
    }
}
