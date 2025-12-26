//! # Deferred Module: Provider Permission Translation
//!
//! **Status**: Complete implementation, not yet integrated into main application
//!
//! **Purpose**: Translate abstract, provider-agnostic permissions into provider-specific
//! configurations for Claude, Gemini, and Codex LLM tools.
//!
//! **Integration Point**: `agents/launcher.rs` - when generating agent configs
//!
//! **Milestone**: TBD - When multi-provider support is prioritized
//!
//! ## Architecture
//!
//! The module provides a unified permission model:
//!
//! - [`StepPermissions`]: Abstract permissions for a workflow step
//! - [`ToolPermissions`]: Tool-level allow/deny lists
//! - [`DirectoryPermissions`]: File system access controls
//! - [`McpServerPermissions`]: MCP server enable/disable
//! - [`PermissionSet`]: Merged project + step permissions
//!
//! ## Provider Translators
//!
//! - [`ClaudeTranslator`]: Translates to Claude's `--allowedTools` format
//! - [`GeminiTranslator`]: Translates to Gemini's tool configuration
//! - [`CodexTranslator`]: Translates to Codex's permission model
//! - [`TranslatorManager`]: Factory for provider-specific translators
//!
//! ## Usage When Integrated
//!
//! ```rust,ignore
//! use crate::permissions::{TranslatorManager, PermissionSet};
//!
//! let translator = TranslatorManager::for_provider(tool.provider());
//! let merged = PermissionSet::merge(&project_perms, &step_perms, &cli_args);
//! let cli_flags = translator.to_cli_args(&merged);
//! ```

#![allow(dead_code)] // DEFERRED: See module docs for integration plan

mod claude;
mod codex;
mod gemini;
mod translator;

pub use claude::ClaudeTranslator;
pub use codex::CodexTranslator;
pub use gemini::GeminiTranslator;
pub use translator::TranslatorManager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider-agnostic tool pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolPattern {
    /// Tool name: Read, Write, Edit, Bash, Glob, Grep, WebFetch, etc.
    pub tool: String,
    /// Optional pattern for tool arguments (e.g., "cargo test:*" for Bash)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl ToolPattern {
    /// Create a new tool pattern with just a tool name
    pub fn new(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            pattern: None,
        }
    }

    /// Create a new tool pattern with a tool name and argument pattern
    pub fn with_pattern(tool: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            pattern: Some(pattern.into()),
        }
    }
}

/// Tool-level permissions (allow/deny lists)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ToolPermissions {
    /// Tools/patterns to allow
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<ToolPattern>,
    /// Tools/patterns to deny
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<ToolPattern>,
}

/// Directory-level permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DirectoryPermissions {
    /// Additional directories to allow access to (glob patterns)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    /// Directories to deny access to (glob patterns)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

/// MCP server permissions (server-level enable/disable only)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct McpServerPermissions {
    /// MCP servers to enable for this step
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enable: Vec<String>,
    /// MCP servers to disable for this step
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disable: Vec<String>,
}

/// Per-provider custom configuration flags
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CustomFlags {
    /// Claude-specific configuration flags
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub claude: HashMap<String, serde_json::Value>,
    /// Gemini-specific configuration flags
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub gemini: HashMap<String, serde_json::Value>,
    /// Codex-specific configuration flags
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub codex: HashMap<String, serde_json::Value>,
}

/// Complete permission set for a step (as defined in issuetype schema)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StepPermissions {
    /// Tool-level allow/deny lists
    #[serde(default, skip_serializing_if = "is_default")]
    pub tools: ToolPermissions,
    /// Directory-level allow/deny lists
    #[serde(default, skip_serializing_if = "is_default")]
    pub directories: DirectoryPermissions,
    /// MCP server enable/disable configuration
    #[serde(default, skip_serializing_if = "is_default")]
    pub mcp_servers: McpServerPermissions,
    /// Per-provider custom configuration flags
    #[serde(default, skip_serializing_if = "is_default")]
    pub custom_flags: CustomFlags,
}

/// Arbitrary CLI arguments per provider
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProviderCliArgs {
    /// CLI arguments for Claude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude: Vec<String>,
    /// CLI arguments for Gemini
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gemini: Vec<String>,
    /// CLI arguments for Codex
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codex: Vec<String>,
}

/// Merged permission set (project + step permissions combined)
#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    /// Final tool allow list (combined from project + step)
    pub tools_allow: Vec<ToolPattern>,
    /// Final tool deny list (combined from project + step)
    pub tools_deny: Vec<ToolPattern>,
    /// Final directory allow list (combined from project + step)
    pub directories_allow: Vec<String>,
    /// Final directory deny list (combined from project + step)
    pub directories_deny: Vec<String>,
    /// MCP servers to enable
    pub mcp_enable: Vec<String>,
    /// MCP servers to disable
    pub mcp_disable: Vec<String>,
    /// Provider-specific custom flags (step overrides project for same key)
    pub custom_flags: CustomFlags,
    /// Arbitrary CLI arguments per provider
    pub cli_args: ProviderCliArgs,
}

impl PermissionSet {
    /// Merge step permissions additively with project permissions
    ///
    /// Both allow and deny lists are concatenated (additive merge).
    /// For custom_flags, step values override project values for the same key.
    pub fn merge(
        project: &StepPermissions,
        step: &StepPermissions,
        step_cli_args: &ProviderCliArgs,
    ) -> Self {
        Self {
            // Additive: combine both allow lists
            tools_allow: [project.tools.allow.clone(), step.tools.allow.clone()].concat(),
            // Additive: combine both deny lists
            tools_deny: [project.tools.deny.clone(), step.tools.deny.clone()].concat(),
            // Additive: combine directory allows
            directories_allow: [
                project.directories.allow.clone(),
                step.directories.allow.clone(),
            ]
            .concat(),
            // Additive: combine directory denies
            directories_deny: [
                project.directories.deny.clone(),
                step.directories.deny.clone(),
            ]
            .concat(),
            // Additive: combine MCP enables
            mcp_enable: [
                project.mcp_servers.enable.clone(),
                step.mcp_servers.enable.clone(),
            ]
            .concat(),
            // Additive: combine MCP disables
            mcp_disable: [
                project.mcp_servers.disable.clone(),
                step.mcp_servers.disable.clone(),
            ]
            .concat(),
            // Step flags override project flags for the same key
            custom_flags: CustomFlags {
                claude: merge_flags(&project.custom_flags.claude, &step.custom_flags.claude),
                gemini: merge_flags(&project.custom_flags.gemini, &step.custom_flags.gemini),
                codex: merge_flags(&project.custom_flags.codex, &step.custom_flags.codex),
            },
            cli_args: step_cli_args.clone(),
        }
    }

    /// Create a PermissionSet from just step permissions (no project permissions)
    pub fn from_step(step: &StepPermissions, cli_args: &ProviderCliArgs) -> Self {
        Self::merge(&StepPermissions::default(), step, cli_args)
    }
}

/// Helper function to merge flag hashmaps (overlay values override base values)
fn merge_flags(
    base: &HashMap<String, serde_json::Value>,
    overlay: &HashMap<String, serde_json::Value>,
) -> HashMap<String, serde_json::Value> {
    let mut result = base.clone();
    for (k, v) in overlay {
        result.insert(k.clone(), v.clone());
    }
    result
}

/// Helper to check if a value is default (for skip_serializing_if)
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == T::default()
}

/// Project-level permission settings (read from .operator/permissions.json)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectPermissions {
    /// Base permissions that apply to all steps
    #[serde(default)]
    pub base: StepPermissions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_pattern_new() {
        let pattern = ToolPattern::new("Read");
        assert_eq!(pattern.tool, "Read");
        assert_eq!(pattern.pattern, None);
    }

    #[test]
    fn test_tool_pattern_with_pattern() {
        let pattern = ToolPattern::with_pattern("Bash", "cargo:*");
        assert_eq!(pattern.tool, "Bash");
        assert_eq!(pattern.pattern, Some("cargo:*".to_string()));
    }

    #[test]
    fn test_permission_set_merge_additive() {
        let project = StepPermissions {
            tools: ToolPermissions {
                allow: vec![ToolPattern::new("Read")],
                deny: vec![ToolPattern::with_pattern("Bash", "rm:*")],
            },
            directories: DirectoryPermissions {
                allow: vec!["./docs/".to_string()],
                deny: vec!["./.env".to_string()],
            },
            mcp_servers: McpServerPermissions {
                enable: vec!["memory".to_string()],
                disable: vec![],
            },
            custom_flags: CustomFlags::default(),
        };

        let step = StepPermissions {
            tools: ToolPermissions {
                allow: vec![ToolPattern::new("Write")],
                deny: vec![ToolPattern::with_pattern("Bash", "sudo:*")],
            },
            directories: DirectoryPermissions {
                allow: vec!["./src/".to_string()],
                deny: vec!["./secrets/".to_string()],
            },
            mcp_servers: McpServerPermissions {
                enable: vec![],
                disable: vec!["filesystem".to_string()],
            },
            custom_flags: CustomFlags::default(),
        };

        let cli_args = ProviderCliArgs::default();
        let merged = PermissionSet::merge(&project, &step, &cli_args);

        // Tools allow should have both Read and Write
        assert_eq!(merged.tools_allow.len(), 2);
        assert!(merged.tools_allow.iter().any(|p| p.tool == "Read"));
        assert!(merged.tools_allow.iter().any(|p| p.tool == "Write"));

        // Tools deny should have both rm:* and sudo:*
        assert_eq!(merged.tools_deny.len(), 2);

        // Directories allow should have both docs and src
        assert_eq!(merged.directories_allow.len(), 2);
        assert!(merged.directories_allow.contains(&"./docs/".to_string()));
        assert!(merged.directories_allow.contains(&"./src/".to_string()));

        // Directories deny should have both .env and secrets
        assert_eq!(merged.directories_deny.len(), 2);

        // MCP enable should have memory
        assert_eq!(merged.mcp_enable, vec!["memory".to_string()]);

        // MCP disable should have filesystem
        assert_eq!(merged.mcp_disable, vec!["filesystem".to_string()]);
    }

    #[test]
    fn test_custom_flags_merge_override() {
        let mut project_flags = HashMap::new();
        project_flags.insert("key1".to_string(), serde_json::json!("project_value"));
        project_flags.insert("key2".to_string(), serde_json::json!("project_only"));

        let mut step_flags = HashMap::new();
        step_flags.insert("key1".to_string(), serde_json::json!("step_value"));
        step_flags.insert("key3".to_string(), serde_json::json!("step_only"));

        let project = StepPermissions {
            custom_flags: CustomFlags {
                claude: project_flags,
                ..Default::default()
            },
            ..Default::default()
        };

        let step = StepPermissions {
            custom_flags: CustomFlags {
                claude: step_flags,
                ..Default::default()
            },
            ..Default::default()
        };

        let cli_args = ProviderCliArgs::default();
        let merged = PermissionSet::merge(&project, &step, &cli_args);

        // key1 should have step value (override)
        assert_eq!(
            merged.custom_flags.claude.get("key1"),
            Some(&serde_json::json!("step_value"))
        );

        // key2 should have project value (not overridden)
        assert_eq!(
            merged.custom_flags.claude.get("key2"),
            Some(&serde_json::json!("project_only"))
        );

        // key3 should have step value (new key)
        assert_eq!(
            merged.custom_flags.claude.get("key3"),
            Some(&serde_json::json!("step_only"))
        );
    }

    #[test]
    fn test_step_permissions_serialization() {
        let perms = StepPermissions {
            tools: ToolPermissions {
                allow: vec![ToolPattern::with_pattern("Bash", "cargo:*")],
                deny: vec![],
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&perms).unwrap();
        let parsed: StepPermissions = serde_json::from_str(&json).unwrap();

        assert_eq!(perms, parsed);
    }
}
