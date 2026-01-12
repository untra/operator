//! Step configuration extraction from template schemas

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::permissions::{ProjectPermissions, ProviderCliArgs, StepPermissions, ToolPattern};
use crate::queue::Ticket;
use crate::templates::{
    schema::{PermissionMode, TemplateSchema},
    TemplateType,
};

/// Step configuration extracted from template schema
#[derive(Debug, Default)]
pub struct StepConfig {
    pub permissions: StepPermissions,
    pub cli_args: ProviderCliArgs,
    pub permission_mode: PermissionMode,
    pub json_schema: Option<serde_json::Value>,
    pub json_schema_file: Option<String>,
}

/// Get step-level configuration from the template schema
pub fn get_step_config(ticket: &Ticket) -> Result<StepConfig> {
    // Try to load template and get step configuration
    let template = TemplateType::from_key(&ticket.ticket_type)
        .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok());

    if let Some(schema) = template {
        // Get the step name (use first step if not specified)
        let step_name = if ticket.step.is_empty() {
            schema.steps.first().map(|s| s.name.clone())
        } else {
            Some(ticket.step.clone())
        };

        if let Some(step_name) = step_name {
            if let Some(step) = schema.get_step(&step_name) {
                let mut permissions = step.permissions.clone().unwrap_or_default();

                // Bridge: Convert allowed_tools to permissions.tools.allow if not explicitly set
                if permissions.tools.allow.is_empty() && !step.allowed_tools.is_empty() {
                    permissions.tools.allow = step
                        .allowed_tools
                        .iter()
                        .filter(|t| *t != "*") // Skip wildcard (allows all tools)
                        .map(ToolPattern::new)
                        .collect();
                }

                return Ok(StepConfig {
                    permissions,
                    cli_args: step.cli_args.clone().unwrap_or_default(),
                    permission_mode: step.permission_mode.clone(),
                    json_schema: step.json_schema.clone(),
                    json_schema_file: step.json_schema_file.clone(),
                });
            }
        }
    }

    // No template or step found, use defaults
    Ok(StepConfig::default())
}

/// Load project-level permission settings from .operator/permissions.json
pub fn load_project_permissions(_config: &Config, project_path: &str) -> Result<StepPermissions> {
    let permissions_path = PathBuf::from(project_path)
        .join(".operator")
        .join("permissions.json");

    if permissions_path.exists() {
        let content = fs::read_to_string(&permissions_path)
            .with_context(|| format!("Failed to read permissions file: {:?}", permissions_path))?;
        let proj_perms: ProjectPermissions = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse permissions file: {:?}", permissions_path))?;
        Ok(proj_perms.base)
    } else {
        // No project permissions file, use empty defaults
        Ok(StepPermissions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{LlmTask, Ticket};
    use std::collections::HashMap;

    /// Helper to create a minimal test ticket
    fn make_test_ticket(ticket_type: &str, step: &str) -> Ticket {
        Ticket {
            id: "TEST-001".to_string(),
            ticket_type: ticket_type.to_string(),
            step: step.to_string(),
            project: "test-project".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20250101-0000".to_string(),
            status: "TODO".to_string(),
            content: String::new(),
            sessions: HashMap::new(),
            llm_task: LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    #[test]
    fn test_allowed_tools_bridged_to_permissions() {
        // FEAT has a "plan" step with allowed_tools: ["Read", "Glob", "Grep", "Write"]
        let ticket = make_test_ticket("FEAT", "plan");
        let config = get_step_config(&ticket).expect("Should get step config");

        // Should have converted allowed_tools to permissions.tools.allow
        assert!(
            !config.permissions.tools.allow.is_empty(),
            "allowed_tools should be bridged to permissions.tools.allow"
        );

        // Verify specific tools are present
        let tool_names: Vec<&str> = config
            .permissions
            .tools
            .allow
            .iter()
            .map(|t| t.tool.as_str())
            .collect();
        assert!(tool_names.contains(&"Read"), "Should contain Read tool");
        assert!(tool_names.contains(&"Glob"), "Should contain Glob tool");
        assert!(tool_names.contains(&"Grep"), "Should contain Grep tool");
        assert!(tool_names.contains(&"Write"), "Should contain Write tool");
    }

    #[test]
    fn test_wildcard_allowed_tools_skipped() {
        // TASK has allowed_tools: ["*"] which should be skipped
        let ticket = make_test_ticket("TASK", "analyze");
        let config = get_step_config(&ticket).expect("Should get step config");

        // Wildcard should be skipped, but other tools in TASK's analyze step should be present
        // Check that no "*" pattern exists
        let has_wildcard = config.permissions.tools.allow.iter().any(|t| t.tool == "*");
        assert!(
            !has_wildcard,
            "Wildcard '*' should be filtered out from permissions"
        );
    }

    #[test]
    fn test_unknown_ticket_type_returns_defaults() {
        let ticket = make_test_ticket("UNKNOWN", "step1");
        let config = get_step_config(&ticket).expect("Should get step config");

        // Should return default config (empty permissions)
        assert!(
            config.permissions.tools.allow.is_empty(),
            "Unknown ticket type should have empty permissions"
        );
    }

    #[test]
    fn test_empty_step_uses_first_step() {
        // Create ticket with empty step - should use first step of FEAT (which is "plan")
        let ticket = make_test_ticket("FEAT", "");
        let config = get_step_config(&ticket).expect("Should get step config");

        // Should have permissions from the first step (plan step has allowed_tools)
        assert!(
            !config.permissions.tools.allow.is_empty(),
            "Empty step should use first step and bridge its allowed_tools"
        );
    }
}
