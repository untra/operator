//! Step configuration extraction from template schemas

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::permissions::{ProjectPermissions, ProviderCliArgs, StepPermissions};
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
                return Ok(StepConfig {
                    permissions: step.permissions.clone().unwrap_or_default(),
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
