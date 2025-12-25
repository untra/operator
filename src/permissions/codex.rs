//! Codex-specific permission translation
//!
//! Codex uses a TOML config file with sections for tools and MCP servers.

use std::collections::HashMap;

use super::translator::PermissionTranslator;
use super::PermissionSet;

/// Translator for OpenAI Codex CLI
pub struct CodexTranslator;

impl CodexTranslator {
    /// Map generic tool names to Codex's tool names
    fn map_tool_name(tool: &str) -> String {
        match tool {
            "Bash" => "exec".to_string(),
            "Read" => "read_file".to_string(),
            "Write" => "write_file".to_string(),
            "Edit" => "apply_patch".to_string(),
            "Glob" => "glob".to_string(),
            "Grep" => "grep".to_string(),
            other => other.to_lowercase(),
        }
    }

    /// Escape a string for TOML
    fn escape_toml_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    /// Format patterns as TOML array
    fn format_toml_array(patterns: &[String]) -> String {
        if patterns.is_empty() {
            "[]".to_string()
        } else {
            let items: Vec<String> = patterns
                .iter()
                .map(|p| format!("\"{}\"", Self::escape_toml_string(p)))
                .collect();
            format!("[{}]", items.join(", "))
        }
    }
}

impl PermissionTranslator for CodexTranslator {
    fn provider_name(&self) -> &str {
        "codex"
    }

    fn generate_cli_flags(&self, _permissions: &PermissionSet) -> Vec<String> {
        // Codex uses config file, not CLI flags for permissions
        // The --config-dir flag is added by TranslatorManager
        Vec::new()
    }

    fn generate_config_content(&self, permissions: &PermissionSet) -> Option<String> {
        let mut toml_content = String::new();

        // MCP servers
        for server in &permissions.mcp_enable {
            toml_content.push_str(&format!(
                "[mcp_servers.\"{}\"]\nenabled = true\n\n",
                Self::escape_toml_string(server)
            ));
        }

        for server in &permissions.mcp_disable {
            toml_content.push_str(&format!(
                "[mcp_servers.\"{}\"]\nenabled = false\n\n",
                Self::escape_toml_string(server)
            ));
        }

        // Group tool patterns by tool name
        let mut tools_by_name: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();

        // Process tool allow patterns
        for pattern in &permissions.tools_allow {
            let tool_name = Self::map_tool_name(&pattern.tool);
            let entry = tools_by_name.entry(tool_name).or_default();
            if let Some(p) = &pattern.pattern {
                entry.0.push(p.clone());
            }
        }

        // Process tool deny patterns
        for pattern in &permissions.tools_deny {
            let tool_name = Self::map_tool_name(&pattern.tool);
            let entry = tools_by_name.entry(tool_name).or_default();
            if let Some(p) = &pattern.pattern {
                entry.1.push(p.clone());
            }
        }

        // Add directory permissions to read_file and write_file tools
        if !permissions.directories_allow.is_empty() || !permissions.directories_deny.is_empty() {
            let read_entry = tools_by_name.entry("read_file".to_string()).or_default();
            read_entry.0.extend(permissions.directories_allow.clone());
            read_entry.1.extend(permissions.directories_deny.clone());

            let write_entry = tools_by_name.entry("write_file".to_string()).or_default();
            write_entry.0.extend(permissions.directories_allow.clone());
            write_entry.1.extend(permissions.directories_deny.clone());

            let patch_entry = tools_by_name.entry("apply_patch".to_string()).or_default();
            patch_entry.0.extend(permissions.directories_allow.clone());
            patch_entry.1.extend(permissions.directories_deny.clone());
        }

        // Generate tool sections
        for (tool_name, (allow, deny)) in &tools_by_name {
            if allow.is_empty() && deny.is_empty() {
                continue;
            }

            toml_content.push_str(&format!("[tools.\"{}\"]\n", tool_name));

            if !allow.is_empty() {
                toml_content.push_str(&format!(
                    "allow_patterns = {}\n",
                    Self::format_toml_array(allow)
                ));
            }

            if !deny.is_empty() {
                toml_content.push_str(&format!(
                    "deny_patterns = {}\n",
                    Self::format_toml_array(deny)
                ));
            }

            toml_content.push('\n');
        }

        // Custom flags - serialize as top-level TOML keys
        for (k, v) in &permissions.custom_flags.codex {
            let value_str = match v {
                serde_json::Value::String(s) => format!("\"{}\"", Self::escape_toml_string(s)),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    let items: Vec<String> = arr
                        .iter()
                        .filter_map(|v| match v {
                            serde_json::Value::String(s) => {
                                Some(format!("\"{}\"", Self::escape_toml_string(s)))
                            }
                            _ => None,
                        })
                        .collect();
                    format!("[{}]", items.join(", "))
                }
                _ => continue, // Skip complex values
            };
            toml_content.push_str(&format!("{} = {}\n", k, value_str));
        }

        if toml_content.is_empty() {
            None
        } else {
            Some(toml_content)
        }
    }

    fn config_path(&self) -> Option<&str> {
        Some(".codex/config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{
        DirectoryPermissions, McpServerPermissions, ProviderCliArgs, StepPermissions, ToolPattern,
        ToolPermissions,
    };

    #[test]
    fn test_map_tool_name() {
        assert_eq!(CodexTranslator::map_tool_name("Bash"), "exec");
        assert_eq!(CodexTranslator::map_tool_name("Read"), "read_file");
        assert_eq!(CodexTranslator::map_tool_name("Write"), "write_file");
        assert_eq!(CodexTranslator::map_tool_name("Edit"), "apply_patch");
    }

    #[test]
    fn test_format_toml_array() {
        let patterns = vec!["./src/**".to_string(), "./tests/**".to_string()];
        assert_eq!(
            CodexTranslator::format_toml_array(&patterns),
            "[\"./src/**\", \"./tests/**\"]"
        );
    }

    #[test]
    fn test_format_toml_array_empty() {
        let patterns: Vec<String> = vec![];
        assert_eq!(CodexTranslator::format_toml_array(&patterns), "[]");
    }

    #[test]
    fn test_generate_config_content_empty() {
        let translator = CodexTranslator;
        let permissions = PermissionSet::default();
        assert!(translator.generate_config_content(&permissions).is_none());
    }

    #[test]
    fn test_generate_config_content_with_mcp() {
        let translator = CodexTranslator;
        let step = StepPermissions {
            mcp_servers: McpServerPermissions {
                enable: vec!["memory".to_string()],
                disable: vec!["filesystem".to_string()],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let content = translator.generate_config_content(&permissions);

        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("[mcp_servers.\"memory\"]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains("[mcp_servers.\"filesystem\"]"));
        assert!(content.contains("enabled = false"));
    }

    #[test]
    fn test_generate_config_content_with_tools() {
        let translator = CodexTranslator;
        let step = StepPermissions {
            tools: ToolPermissions {
                allow: vec![ToolPattern::with_pattern("Bash", "cargo:*")],
                deny: vec![ToolPattern::with_pattern("Bash", "rm:*")],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let content = translator.generate_config_content(&permissions);

        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("[tools.\"exec\"]"));
        assert!(content.contains("allow_patterns = [\"cargo:*\"]"));
        assert!(content.contains("deny_patterns = [\"rm:*\"]"));
    }

    #[test]
    fn test_generate_config_content_with_directories() {
        let translator = CodexTranslator;
        let step = StepPermissions {
            directories: DirectoryPermissions {
                allow: vec!["./src/**".to_string()],
                deny: vec!["./.env".to_string()],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let content = translator.generate_config_content(&permissions);

        assert!(content.is_some());
        let content = content.unwrap();
        // Should have read_file, write_file, and apply_patch sections
        assert!(content.contains("[tools.\"read_file\"]"));
        assert!(content.contains("allow_patterns"));
        assert!(content.contains("deny_patterns"));
    }

    #[test]
    fn test_uses_config_file() {
        let translator = CodexTranslator;
        assert!(!translator.uses_cli_only());
        assert_eq!(translator.config_path(), Some(".codex/config.toml"));
    }
}
