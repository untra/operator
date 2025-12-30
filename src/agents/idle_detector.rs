//! Idle state detection via content pattern matching
//!
//! This module provides pattern-based detection of when an LLM CLI tool
//! is idle (waiting for input) vs actively working. It uses tool-specific
//! regex patterns from the tool configuration.

use crate::llm::tool_config::{IdleDetectionConfig, ToolConfig};
use regex::Regex;
use std::collections::HashMap;

/// Compiled patterns for a single tool
#[derive(Debug)]
struct CompiledPatterns {
    /// Patterns that indicate idle/awaiting state (e.g., prompt chars)
    idle: Vec<Regex>,
    /// Patterns that indicate active processing (spinners, status messages)
    activity: Vec<Regex>,
}

/// Detector for idle/awaiting state based on terminal content patterns
#[derive(Debug)]
pub struct IdleDetector {
    /// Compiled patterns per tool
    tool_patterns: HashMap<String, CompiledPatterns>,
}

impl Default for IdleDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl IdleDetector {
    /// Create a new empty IdleDetector
    pub fn new() -> Self {
        Self {
            tool_patterns: HashMap::new(),
        }
    }

    /// Create an IdleDetector from tool configurations
    pub fn from_tool_configs(configs: &[ToolConfig]) -> Self {
        let mut detector = Self::new();

        for config in configs {
            if let Some(ref idle_config) = config.idle_detection {
                detector.add_tool_patterns(&config.tool_name, idle_config);
            }
        }

        detector
    }

    /// Add patterns for a specific tool
    pub fn add_tool_patterns(&mut self, tool_name: &str, config: &IdleDetectionConfig) {
        let idle_patterns: Vec<Regex> = config
            .idle_patterns
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!(
                        tool = tool_name,
                        pattern = p,
                        error = %e,
                        "Failed to compile idle pattern"
                    );
                    None
                }
            })
            .collect();

        let activity_patterns: Vec<Regex> = config
            .activity_patterns
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!(
                        tool = tool_name,
                        pattern = p,
                        error = %e,
                        "Failed to compile activity pattern"
                    );
                    None
                }
            })
            .collect();

        if !idle_patterns.is_empty() || !activity_patterns.is_empty() {
            self.tool_patterns.insert(
                tool_name.to_string(),
                CompiledPatterns {
                    idle: idle_patterns,
                    activity: activity_patterns,
                },
            );
        }
    }

    /// Check if the content indicates an idle/awaiting state
    ///
    /// Returns true if:
    /// - No activity patterns are found in recent output
    /// - At least one idle pattern is found
    ///
    /// Activity patterns take precedence - if any activity is detected,
    /// the tool is considered working regardless of idle patterns.
    pub fn is_idle(&self, tool_name: &str, content: &str) -> bool {
        let patterns = match self.tool_patterns.get(tool_name) {
            Some(p) => p,
            None => return false, // No patterns configured, can't determine
        };

        // Get last N lines for analysis (focus on recent output)
        let last_lines: Vec<&str> = content.lines().rev().take(10).collect();

        // Check for activity indicators first (takes precedence)
        for line in &last_lines {
            for pattern in &patterns.activity {
                if pattern.is_match(line) {
                    return false; // Tool is actively working
                }
            }
        }

        // Check for idle prompt pattern in the very last few lines
        let prompt_lines: Vec<&str> = content.lines().rev().take(3).collect();
        for line in &prompt_lines {
            for pattern in &patterns.idle {
                if pattern.is_match(line) {
                    return true; // Tool is idle/awaiting input
                }
            }
        }

        false // Can't determine - assume not idle
    }

    /// Check if the content shows active processing
    ///
    /// This is the inverse check - returns true if activity indicators are present.
    /// Used for resume detection.
    pub fn is_active(&self, tool_name: &str, content: &str) -> bool {
        let patterns = match self.tool_patterns.get(tool_name) {
            Some(p) => p,
            None => return false,
        };

        let last_lines: Vec<&str> = content.lines().rev().take(10).collect();

        for line in &last_lines {
            for pattern in &patterns.activity {
                if pattern.is_match(line) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if patterns are configured for a tool
    pub fn has_patterns_for(&self, tool_name: &str) -> bool {
        self.tool_patterns.contains_key(tool_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::tool_config::IdleDetectionConfig;

    fn create_test_config() -> IdleDetectionConfig {
        IdleDetectionConfig {
            idle_patterns: vec![r"^>\s*$".to_string(), r"^❯\s*$".to_string()],
            activity_patterns: vec![
                "⠋".to_string(),
                "⠙".to_string(),
                "Thinking".to_string(),
                "Working".to_string(),
            ],
            hook_config: None,
        }
    }

    #[test]
    fn test_idle_detector_new() {
        let detector = IdleDetector::new();
        assert!(detector.tool_patterns.is_empty());
    }

    #[test]
    fn test_add_tool_patterns() {
        let mut detector = IdleDetector::new();
        let config = create_test_config();

        detector.add_tool_patterns("claude", &config);

        assert!(detector.has_patterns_for("claude"));
        assert!(!detector.has_patterns_for("codex"));
    }

    #[test]
    fn test_is_idle_with_prompt() {
        let mut detector = IdleDetector::new();
        detector.add_tool_patterns("claude", &create_test_config());

        // Content ending with prompt
        let content = "Some output\nMore output\n> ";
        assert!(detector.is_idle("claude", content));

        // Content with chevron prompt
        let content = "Some output\n❯ ";
        assert!(detector.is_idle("claude", content));
    }

    #[test]
    fn test_is_idle_with_activity() {
        let mut detector = IdleDetector::new();
        detector.add_tool_patterns("claude", &create_test_config());

        // Content with activity spinner - should NOT be idle
        let content = "Some output\n⠋ Processing...\n> ";
        assert!(!detector.is_idle("claude", content));

        // Content with "Thinking" message
        let content = "Some output\nThinking about your request...\n> ";
        assert!(!detector.is_idle("claude", content));
    }

    #[test]
    fn test_is_idle_no_prompt() {
        let mut detector = IdleDetector::new();
        detector.add_tool_patterns("claude", &create_test_config());

        // Content without prompt at end
        let content = "Some output\nMore output";
        assert!(!detector.is_idle("claude", content));
    }

    #[test]
    fn test_is_idle_unknown_tool() {
        let detector = IdleDetector::new();

        // Unknown tool should return false (can't determine)
        let content = "Some output\n> ";
        assert!(!detector.is_idle("unknown", content));
    }

    #[test]
    fn test_is_active() {
        let mut detector = IdleDetector::new();
        detector.add_tool_patterns("claude", &create_test_config());

        // Content with activity
        let content = "⠋ Working on it...";
        assert!(detector.is_active("claude", content));

        // Content without activity
        let content = "Done!\n> ";
        assert!(!detector.is_active("claude", content));
    }

    #[test]
    fn test_activity_takes_precedence() {
        let mut detector = IdleDetector::new();
        detector.add_tool_patterns("claude", &create_test_config());

        // Even with prompt visible, activity should indicate not idle
        // This simulates the case where prompt is visible but spinner is also showing
        let content = "Output\nWorking on your request\n> ";
        assert!(!detector.is_idle("claude", content));
    }

    #[test]
    fn test_multiline_content() {
        let mut detector = IdleDetector::new();
        detector.add_tool_patterns("claude", &create_test_config());

        // Long content with prompt at end
        let content = r#"
This is a long response from the AI.
It contains multiple lines.
And some code:
```rust
fn main() {
    println!("Hello!");
}
```
Here's the final output.
> "#;

        assert!(detector.is_idle("claude", content));
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let mut detector = IdleDetector::new();
        let config = IdleDetectionConfig {
            idle_patterns: vec![r"[invalid".to_string(), r"^>\s*$".to_string()],
            activity_patterns: vec![],
            hook_config: None,
        };

        // Should not panic, just skip invalid patterns
        detector.add_tool_patterns("test", &config);

        // Valid pattern should still work
        let content = "> ";
        assert!(detector.is_idle("test", content));
    }
}
