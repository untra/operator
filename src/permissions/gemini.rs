//! Gemini-specific permission translation
//!
//! Gemini CLI uses a settings.json file for configuration.
//! Tool names need to be mapped to Gemini's naming convention.

use serde_json::json;

use super::translator::PermissionTranslator;
use super::{PermissionSet, ToolPattern};

/// Translator for Google Gemini CLI
pub struct GeminiTranslator;

impl GeminiTranslator {
    /// Map generic tool names to Gemini's tool names
    fn map_tool_name(tool: &str) -> &str {
        match tool {
            "Bash" => "ShellTool",
            "Read" => "ReadFileTool",
            "Write" => "WriteFileTool",
            "Edit" => "EditFileTool",
            "Glob" => "GlobTool",
            "Grep" => "GrepTool",
            "WebFetch" => "WebFetchTool",
            other => other,
        }
    }

    /// Format a ToolPattern into Gemini's permission syntax
    fn format_tool_pattern(pattern: &ToolPattern) -> String {
        let tool_name = Self::map_tool_name(&pattern.tool);
        match &pattern.pattern {
            Some(p) => format!("{}({})", tool_name, p),
            None => tool_name.to_string(),
        }
    }
}

impl PermissionTranslator for GeminiTranslator {
    fn provider_name(&self) -> &str {
        "gemini"
    }

    fn generate_cli_flags(&self, _permissions: &PermissionSet) -> Vec<String> {
        // Gemini uses config file, not CLI flags for permissions
        // The --config-dir flag is added by TranslatorManager
        Vec::new()
    }

    fn generate_config_content(&self, permissions: &PermissionSet) -> Option<String> {
        let mut config = serde_json::Map::new();

        // Core tools (allow list)
        if !permissions.tools_allow.is_empty() {
            let core_tools: Vec<String> = permissions
                .tools_allow
                .iter()
                .map(Self::format_tool_pattern)
                .collect();
            config.insert("coreTools".to_string(), json!(core_tools));
        }

        // Excluded tools (deny list)
        let mut exclude_tools: Vec<String> = permissions
            .tools_deny
            .iter()
            .map(Self::format_tool_pattern)
            .collect();

        // Add directory denies as tool exclusions
        for dir in &permissions.directories_deny {
            exclude_tools.push(format!("ReadFileTool({})", dir));
            exclude_tools.push(format!("WriteFileTool({})", dir));
            exclude_tools.push(format!("EditFileTool({})", dir));
        }

        if !exclude_tools.is_empty() {
            config.insert("excludeTools".to_string(), json!(exclude_tools));
        }

        // Directory permissions
        if !permissions.directories_allow.is_empty() {
            config.insert(
                "includeDirectories".to_string(),
                json!(permissions.directories_allow),
            );
        }

        // MCP servers
        if !permissions.mcp_enable.is_empty() || !permissions.mcp_disable.is_empty() {
            let mut mcp_servers = serde_json::Map::new();

            for server in &permissions.mcp_enable {
                mcp_servers.insert(server.clone(), json!({ "trust": true }));
            }

            for server in &permissions.mcp_disable {
                mcp_servers.insert(server.clone(), json!({ "enabled": false }));
            }

            config.insert("mcpServers".to_string(), json!(mcp_servers));
        }

        // Custom flags from permissions
        for (k, v) in &permissions.custom_flags.gemini {
            config.insert(k.clone(), v.clone());
        }

        if config.is_empty() {
            None
        } else {
            Some(serde_json::to_string_pretty(&config).unwrap_or_default())
        }
    }

    fn config_path(&self) -> Option<&str> {
        Some(".gemini/settings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{
        DirectoryPermissions, McpServerPermissions, ProviderCliArgs, StepPermissions,
        ToolPermissions,
    };

    #[test]
    fn test_map_tool_name() {
        assert_eq!(GeminiTranslator::map_tool_name("Bash"), "ShellTool");
        assert_eq!(GeminiTranslator::map_tool_name("Read"), "ReadFileTool");
        assert_eq!(GeminiTranslator::map_tool_name("Write"), "WriteFileTool");
        assert_eq!(GeminiTranslator::map_tool_name("Unknown"), "Unknown");
    }

    #[test]
    fn test_format_tool_pattern() {
        let pattern = ToolPattern::with_pattern("Bash", "cargo test");
        assert_eq!(
            GeminiTranslator::format_tool_pattern(&pattern),
            "ShellTool(cargo test)"
        );
    }

    #[test]
    fn test_generate_config_content_empty() {
        let translator = GeminiTranslator;
        let permissions = PermissionSet::default();
        assert!(translator.generate_config_content(&permissions).is_none());
    }

    #[test]
    fn test_generate_config_content_with_tools() {
        let translator = GeminiTranslator;
        let step = StepPermissions {
            tools: ToolPermissions {
                allow: vec![ToolPattern::new("Read"), ToolPattern::new("Write")],
                deny: vec![ToolPattern::with_pattern("Bash", "rm")],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let content = translator.generate_config_content(&permissions);

        assert!(content.is_some());
        let json: serde_json::Value = serde_json::from_str(&content.unwrap()).unwrap();

        // Check coreTools
        let core_tools = json["coreTools"].as_array().unwrap();
        assert!(core_tools.contains(&json!("ReadFileTool")));
        assert!(core_tools.contains(&json!("WriteFileTool")));

        // Check excludeTools
        let exclude_tools = json["excludeTools"].as_array().unwrap();
        assert!(exclude_tools.contains(&json!("ShellTool(rm)")));
    }

    #[test]
    fn test_generate_config_content_with_mcp() {
        let translator = GeminiTranslator;
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
        let json: serde_json::Value = serde_json::from_str(&content.unwrap()).unwrap();

        // Check mcpServers
        let mcp_servers = &json["mcpServers"];
        assert_eq!(mcp_servers["memory"]["trust"], json!(true));
        assert_eq!(mcp_servers["filesystem"]["enabled"], json!(false));
    }

    #[test]
    fn test_generate_config_content_with_directories() {
        let translator = GeminiTranslator;
        let step = StepPermissions {
            directories: DirectoryPermissions {
                allow: vec!["../docs/".to_string()],
                deny: vec!["./.env".to_string()],
            },
            ..Default::default()
        };
        let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
        let content = translator.generate_config_content(&permissions);

        assert!(content.is_some());
        let json: serde_json::Value = serde_json::from_str(&content.unwrap()).unwrap();

        // Check includeDirectories
        let include_dirs = json["includeDirectories"].as_array().unwrap();
        assert!(include_dirs.contains(&json!("../docs/")));

        // Check excludeTools for directory denies
        let exclude_tools = json["excludeTools"].as_array().unwrap();
        assert!(exclude_tools.contains(&json!("ReadFileTool(./.env)")));
    }

    #[test]
    fn test_uses_config_file() {
        let translator = GeminiTranslator;
        assert!(!translator.uses_cli_only());
        assert_eq!(translator.config_path(), Some(".gemini/settings.json"));
    }
}
