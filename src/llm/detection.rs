//! LLM tool detection logic
//!
//! Detects available LLM CLI tools by checking binary existence
//! and loading configurations from embedded JSON files.

use std::cmp::Ordering;
use std::process::Command;

use crate::config::{DetectedTool, LlmProvider, LlmToolsConfig, ToolCapabilities};

use super::tool_config::{load_all_tool_configs, DetectionMode, ToolConfig};

/// Detect all available LLM CLI tools and build the config from scratch
#[allow(dead_code)] // Used via binary, not reachable from lib.rs
pub fn detect_all_tools() -> LlmToolsConfig {
    refresh_tool_detection(&LlmToolsConfig::default())
}

/// Rebuild detection state from the currently loaded tool configs, preserving
/// user prefs. Cached entries keep their probed fields (`path`, `version` — no
/// process spawns); config-sourced fields are re-derived so config edits and
/// runtime-loaded tools take effect every startup. Tools whose config no longer
/// exists are dropped; new configs are probed fresh.
#[allow(dead_code)] // Used via binary, not reachable from lib.rs
pub fn refresh_tool_detection(existing: &LlmToolsConfig) -> LlmToolsConfig {
    refresh_with_configs(existing, &load_all_tool_configs())
}

fn refresh_with_configs(existing: &LlmToolsConfig, configs: &[ToolConfig]) -> LlmToolsConfig {
    let mut detected = Vec::new();
    let mut providers = Vec::new();

    for config in configs {
        let tool = match existing
            .detected
            .iter()
            .find(|t| t.name == config.tool_name)
        {
            Some(cached) => Some(refresh_cached_tool(cached, config)),
            None => detect_tool(config),
        };
        if let Some(tool) = tool {
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
        default_tool: existing.default_tool.clone(),
        default_model: existing.default_model.clone(),
        skill_directory_overrides: existing.skill_directory_overrides.clone(),
    }
}

/// Re-derive config-sourced fields on a cached tool, keeping its probed
/// `path`/`version` (no version re-spawn). Health is always recomputed — a
/// cached `health_ok` is never trusted — so an uninstalled binary or a newly
/// failing health command demotes the tool on the next startup. Also repairs
/// partial entries written by external detectors (e.g. the VS Code extension
/// caches only name/path/version).
fn refresh_cached_tool(cached: &DetectedTool, config: &ToolConfig) -> DetectedTool {
    let version_ok = match &config.min_version {
        Some(min_ver) => check_version_meets_minimum(&cached.version, min_ver),
        None => true,
    };
    let presence_verified = config.detection_mode() == DetectionMode::Which
        && get_binary_path(&config.tool_name).is_some();
    DetectedTool {
        name: config.tool_name.clone(),
        path: cached.path.clone(),
        version: cached.version.clone(),
        min_version: config.min_version.clone(),
        version_ok,
        model_aliases: config.model_aliases.clone(),
        command_template: config.command_template.clone(),
        capabilities: ToolCapabilities {
            supports_sessions: config.capabilities.supports_sessions,
            supports_headless: config.capabilities.supports_headless,
        },
        yolo_flags: config.yolo_flags.clone(),
        health_ok: check_health(config, presence_verified),
    }
}

/// Detect a single tool from its config
fn detect_tool(config: &ToolConfig) -> Option<DetectedTool> {
    let mode = config.detection_mode();
    let path = match mode {
        DetectionMode::Which => get_binary_path(&config.tool_name)?,
        DetectionMode::Always => config.tool_name.clone(),
    };
    // The which gate above succeeding *is* the presence proof.
    let presence_verified = mode == DetectionMode::Which;
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
        health_ok: check_health(config, presence_verified),
    })
}

/// Decide whether a tool is healthy. Health is earned, never assumed: which-mode
/// tools bank the verified presence of their binary, always-mode tools have
/// nothing locally verifiable and need a passing `health_command`. An unhealthy
/// tool stays detected (and visible) but cannot be launched locally.
fn check_health(config: &ToolConfig, presence_verified: bool) -> bool {
    let cmd = config
        .detection
        .as_ref()
        .and_then(|d| d.health_command.as_deref());

    let healthy = match (config.detection_mode(), cmd) {
        (DetectionMode::Which, None) => presence_verified,
        (DetectionMode::Which, Some(c)) => presence_verified && run_health_command(c),
        (DetectionMode::Always, None) => false,
        (DetectionMode::Always, Some(c)) => run_health_command(c),
    };

    if !healthy {
        tracing::warn!(
            tool = %config.tool_name,
            command = cmd.unwrap_or("none"),
            presence_verified,
            "Tool failed its health check; detected but not launchable locally"
        );
    }
    healthy
}

fn run_health_command(health_command: &str) -> bool {
    let parts: Vec<&str> = health_command.split_whitespace().collect();
    let Some((program, args)) = parts.split_first() else {
        return true;
    };
    Command::new(program)
        .args(args)
        .output()
        .is_ok_and(|o| o.status.success())
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
    use crate::llm::tool_config::{ArgMapping, DetectionConfig, DetectionMode};

    fn make_tool_config(
        tool_name: &str,
        mode: DetectionMode,
        health_command: Option<&str>,
    ) -> ToolConfig {
        ToolConfig {
            tool_name: tool_name.to_string(),
            display_name: None,
            version_command: format!("{tool_name} --version"),
            min_version: None,
            capabilities: crate::llm::tool_config::ToolCapabilities::default(),
            model_aliases: vec!["default".to_string()],
            arg_mapping: ArgMapping::default(),
            command_template: format!("{tool_name} \"$(cat {{{{prompt_file}}}})\""),
            yolo_flags: vec![],
            idle_detection: None,
            skill_directories: None,
            detection: Some(DetectionConfig {
                mode,
                health_command: health_command.map(String::from),
            }),
        }
    }

    const MISSING_BINARY: &str = "op-test-tool-does-not-exist";

    fn make_cached_tool(name: &str) -> DetectedTool {
        DetectedTool {
            name: name.to_string(),
            path: format!("/usr/bin/{name}"),
            version: "1.0.0".to_string(),
            min_version: None,
            version_ok: true,
            model_aliases: vec!["default".to_string()],
            command_template: format!("{name} \"$(cat {{{{prompt_file}}}})\""),
            capabilities: ToolCapabilities::default(),
            yolo_flags: vec![],
            health_ok: true,
        }
    }

    #[test]
    fn test_refresh_adds_newly_configured_tool() {
        let existing = LlmToolsConfig {
            detected: vec![make_cached_tool("toolx")],
            detection_complete: true,
            ..Default::default()
        };
        let configs = vec![
            make_tool_config("toolx", DetectionMode::Always, None),
            make_tool_config("tooly", DetectionMode::Always, None),
        ];

        let refreshed = refresh_with_configs(&existing, &configs);
        let names: Vec<_> = refreshed.detected.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["toolx", "tooly"]);
        assert!(refreshed.detection_complete);
        let expected: usize = refreshed
            .detected
            .iter()
            .map(|t| t.model_aliases.len())
            .sum();
        assert_eq!(refreshed.providers.len(), expected);
    }

    #[test]
    fn test_refresh_preserves_user_prefs() {
        let mut overrides = std::collections::HashMap::new();
        overrides.insert(
            "toolx".to_string(),
            crate::config::SkillDirectoriesOverride::default(),
        );
        let existing = LlmToolsConfig {
            detected: vec![make_cached_tool("toolx")],
            detection_complete: true,
            default_tool: Some("toolx".to_string()),
            default_model: Some("default".to_string()),
            skill_directory_overrides: overrides,
            ..Default::default()
        };
        let configs = vec![make_tool_config("toolx", DetectionMode::Always, None)];

        let refreshed = refresh_with_configs(&existing, &configs);
        assert_eq!(refreshed.default_tool.as_deref(), Some("toolx"));
        assert_eq!(refreshed.default_model.as_deref(), Some("default"));
        assert!(refreshed.skill_directory_overrides.contains_key("toolx"));
    }

    #[test]
    fn test_refresh_rederives_config_sourced_fields() {
        // A VS Code-written subset entry: probed fields only, config-sourced empty
        let mut cached = make_cached_tool("toolx");
        cached.command_template = String::new();
        cached.model_aliases = vec![];
        let existing = LlmToolsConfig {
            detected: vec![cached],
            detection_complete: true,
            ..Default::default()
        };
        let mut config = make_tool_config("toolx", DetectionMode::Always, None);
        config.min_version = Some("0.5.0".to_string());
        config.model_aliases = vec!["a".to_string(), "b".to_string()];

        let refreshed = refresh_with_configs(&existing, &[config]);
        let tool = &refreshed.detected[0];
        // Probed fields kept, config-sourced fields re-derived
        assert_eq!(tool.path, "/usr/bin/toolx");
        assert_eq!(tool.version, "1.0.0");
        assert_eq!(tool.model_aliases, vec!["a", "b"]);
        assert!(!tool.command_template.is_empty());
        assert_eq!(tool.min_version.as_deref(), Some("0.5.0"));
        assert!(tool.version_ok);
    }

    #[test]
    fn test_refresh_drops_tools_without_config() {
        let existing = LlmToolsConfig {
            detected: vec![make_cached_tool("toolz")],
            detection_complete: true,
            ..Default::default()
        };
        let refreshed = refresh_with_configs(&existing, &[]);
        assert!(refreshed.detected.is_empty());
        assert!(refreshed.providers.is_empty());
    }

    #[test]
    fn test_refresh_reruns_health_command_on_cached_tool() {
        let existing = LlmToolsConfig {
            detected: vec![make_cached_tool("toolx")],
            detection_complete: true,
            ..Default::default()
        };
        let configs = vec![make_tool_config(
            "toolx",
            DetectionMode::Always,
            Some("false"),
        )];

        let refreshed = refresh_with_configs(&existing, &configs);
        assert!(!refreshed.detected[0].health_ok);
    }

    #[test]
    fn test_refresh_which_mode_missing_binary_marks_unhealthy() {
        // A cached tool whose binary has since been uninstalled stays listed
        // but loses its health, so it can no longer be launched.
        let existing = LlmToolsConfig {
            detected: vec![make_cached_tool(MISSING_BINARY)],
            detection_complete: true,
            ..Default::default()
        };
        let configs = vec![make_tool_config(MISSING_BINARY, DetectionMode::Which, None)];

        let refreshed = refresh_with_configs(&existing, &configs);
        assert_eq!(refreshed.detected.len(), 1);
        assert!(!refreshed.detected[0].health_ok);
    }

    #[test]
    fn test_refresh_which_mode_present_binary_is_healthy() {
        let existing = LlmToolsConfig {
            detected: vec![make_cached_tool("true")],
            detection_complete: true,
            ..Default::default()
        };
        let configs = vec![make_tool_config("true", DetectionMode::Which, None)];

        let refreshed = refresh_with_configs(&existing, &configs);
        assert!(refreshed.detected[0].health_ok);
    }

    #[test]
    fn test_detect_tool_mode_always_bypasses_which() {
        let config = make_tool_config(MISSING_BINARY, DetectionMode::Always, None);
        let tool = detect_tool(&config).expect("always mode detects without a local binary");
        assert_eq!(tool.path, MISSING_BINARY);
        assert_eq!(tool.version, "unknown");
    }

    #[test]
    fn test_which_mode_no_health_command_is_healthy() {
        // The which gate itself verifies presence, so no health command is needed.
        let config = make_tool_config("true", DetectionMode::Which, None);
        let tool = detect_tool(&config).expect("`true` is on PATH");
        assert!(tool.health_ok);
    }

    #[test]
    fn test_always_mode_no_health_command_is_unhealthy() {
        // Nothing is verifiable in always mode without a health command.
        let config = make_tool_config(MISSING_BINARY, DetectionMode::Always, None);
        let tool = detect_tool(&config).unwrap();
        assert!(!tool.health_ok);
    }

    #[test]
    fn test_detect_tool_mode_which_missing_binary_is_none() {
        let config = make_tool_config(MISSING_BINARY, DetectionMode::Which, None);
        assert!(detect_tool(&config).is_none());
    }

    #[test]
    fn test_health_command_failure_is_soft() {
        let config = make_tool_config(MISSING_BINARY, DetectionMode::Always, Some("false"));
        let tool = detect_tool(&config).expect("health failure never blocks detection");
        assert!(!tool.health_ok);
    }

    #[test]
    fn test_health_command_success() {
        let config = make_tool_config(MISSING_BINARY, DetectionMode::Always, Some("true"));
        let tool = detect_tool(&config).unwrap();
        assert!(tool.health_ok);
    }

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
