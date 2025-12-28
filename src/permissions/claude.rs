//! Claude-specific permission translation
//!
//! Claude Code uses CLI arguments for permissions rather than config files.
//! This translator generates `--allowedTools` and `--disallowedTools` flags.

use super::translator::PermissionTranslator;
use super::{PermissionSet, ToolPattern};

/// Translator for Claude Code CLI
pub struct ClaudeTranslator;

impl ClaudeTranslator {
    /// Format a ToolPattern into Claude's permission syntax
    ///
    /// Examples:
    /// - `ToolPattern { tool: "Bash", pattern: Some("cargo:*") }` -> `"Bash(cargo:*)"`
    /// - `ToolPattern { tool: "Read", pattern: None }` -> `"Read"`
    fn format_tool_pattern(pattern: &ToolPattern) -> String {
        match &pattern.pattern {
            Some(p) => format!("{}({})", pattern.tool, p),
            None => pattern.tool.clone(),
        }
    }

    /// Format a directory deny as Read/Write deny patterns
    fn format_directory_deny(dir: &str) -> Vec<String> {
        vec![
            format!("Read({})", dir),
            format!("Write({})", dir),
            format!("Edit({})", dir),
        ]
    }
}

impl PermissionTranslator for ClaudeTranslator {
    fn provider_name(&self) -> &str {
        "claude"
    }

    fn generate_cli_flags(&self, permissions: &PermissionSet) -> Vec<String> {
        let mut flags = Vec::new();

        // Tool allow patterns
        for pattern in &permissions.tools_allow {
            flags.push("--allowedTools".to_string());
            flags.push(Self::format_tool_pattern(pattern));
        }

        // Tool deny patterns
        for pattern in &permissions.tools_deny {
            flags.push("--disallowedTools".to_string());
            flags.push(Self::format_tool_pattern(pattern));
        }

        // Directory allows via --add-dir flag
        for dir in &permissions.directories_allow {
            flags.push("--add-dir".to_string());
            flags.push(dir.clone());
        }

        // Directory denies are converted to Read/Write/Edit denies
        for dir in &permissions.directories_deny {
            for deny_pattern in Self::format_directory_deny(dir) {
                flags.push("--disallowedTools".to_string());
                flags.push(deny_pattern);
            }
        }

        // MCP server configuration
        // Note: Claude CLI may not support MCP flags directly
        // These would need to be in a config file
        // For now, we document this limitation

        flags
    }

    fn generate_config_content(&self, _permissions: &PermissionSet) -> Option<String> {
        // Claude uses CLI args, not config files
        // However, we could optionally generate a settings.local.json for audit purposes
        None
    }

    fn config_path(&self) -> Option<&str> {
        // Claude uses CLI args only
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{
        DirectoryPermissions, ProviderCliArgs, StepPermissions, ToolPermissions,
    };

    #[test]
    fn test_format_tool_pattern_simple() {
        let pattern = ToolPattern::new("Read");
        assert_eq!(ClaudeTranslator::format_tool_pattern(&pattern), "Read");
    }

    #[test]
    fn test_format_tool_pattern_with_pattern() {
        let pattern = ToolPattern::with_pattern("Bash", "cargo:*");
        assert_eq!(
            ClaudeTranslator::format_tool_pattern(&pattern),
            "Bash(cargo:*)"
        );
    }

    #[test]
    fn test_generate_cli_flags_empty() {
        let translator = ClaudeTranslator;
        let permissions = PermissionSet::default();
        let flags = translator.generate_cli_flags(&permissions);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_generate_cli_flags_allows() {
        let translator = ClaudeTranslator;
        let step = StepPermissions {
            tools: ToolPermissions {
                allow: vec![
                    ToolPattern::new("Read"),
                    ToolPattern::with_pattern("Bash", "cargo:*"),
                ],
                deny: vec![],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let flags = translator.generate_cli_flags(&permissions);

        assert_eq!(flags.len(), 4);
        assert_eq!(flags[0], "--allowedTools");
        assert_eq!(flags[1], "Read");
        assert_eq!(flags[2], "--allowedTools");
        assert_eq!(flags[3], "Bash(cargo:*)");
    }

    #[test]
    fn test_generate_cli_flags_denies() {
        let translator = ClaudeTranslator;
        let step = StepPermissions {
            tools: ToolPermissions {
                allow: vec![],
                deny: vec![
                    ToolPattern::with_pattern("Bash", "rm:*"),
                    ToolPattern::with_pattern("Bash", "sudo:*"),
                ],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let flags = translator.generate_cli_flags(&permissions);

        assert_eq!(flags.len(), 4);
        assert_eq!(flags[0], "--disallowedTools");
        assert_eq!(flags[1], "Bash(rm:*)");
        assert_eq!(flags[2], "--disallowedTools");
        assert_eq!(flags[3], "Bash(sudo:*)");
    }

    #[test]
    fn test_generate_cli_flags_directory_denies() {
        let translator = ClaudeTranslator;
        let step = StepPermissions {
            directories: DirectoryPermissions {
                allow: vec![],
                deny: vec!["./.env".to_string()],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let flags = translator.generate_cli_flags(&permissions);

        // Should generate Read, Write, and Edit denies for the directory
        assert_eq!(flags.len(), 6);
        assert!(flags.contains(&"Read(./.env)".to_string()));
        assert!(flags.contains(&"Write(./.env)".to_string()));
        assert!(flags.contains(&"Edit(./.env)".to_string()));
    }

    #[test]
    fn test_generate_cli_flags_directory_allows() {
        let translator = ClaudeTranslator;
        let step = StepPermissions {
            directories: DirectoryPermissions {
                allow: vec!["/path/to/tickets".to_string()],
                deny: vec![],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let flags = translator.generate_cli_flags(&permissions);

        // Should generate --add-dir flag for allowed directory
        assert_eq!(flags.len(), 2);
        assert_eq!(flags[0], "--add-dir");
        assert_eq!(flags[1], "/path/to/tickets");
    }

    #[test]
    fn test_uses_cli_only() {
        let translator = ClaudeTranslator;
        assert!(translator.uses_cli_only());
        assert!(translator.config_path().is_none());
    }
}
