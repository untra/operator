//! LLM command building, docker wrapping, and permission configuration

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{Config, DetectedTool};
use crate::permissions::{PermissionSet, TranslatorManager};
use crate::queue::Ticket;
use crate::templates::schema::PermissionMode;

use super::step_config::{get_step_config, load_project_permissions};

/// Build the LLM command for a specific tool with optional step permissions
pub fn build_llm_command_with_permissions_for_tool(
    config: &Config,
    tool_name: &str,
    model: &str,
    session_id: &str,
    prompt_file: &std::path::Path,
    ticket: Option<&Ticket>,
    project_path: Option<&str>,
) -> Result<String> {
    // Find the specified tool
    let tool = get_detected_tool(config, tool_name).ok_or_else(|| {
        anyhow::anyhow!(
            "LLM tool '{}' not detected. Install it or choose a different provider.",
            tool_name
        )
    })?;

    // Build model flag based on tool's arg_mapping
    let model_flag = format!("--model {} ", model);

    // Generate config flags from permissions
    let config_flags = if let (Some(ticket), Some(project_path)) = (ticket, project_path) {
        generate_config_flags(config, &tool.name, ticket, project_path, session_id)?
    } else {
        String::new()
    };

    // Build command from template
    let cmd = tool
        .command_template
        .replace("{{config_flags}}", &config_flags)
        .replace("{{model_flag}}", &model_flag)
        .replace("{{model}}", model)
        .replace("{{session_id}}", session_id)
        .replace("{{prompt_file}}", &prompt_file.display().to_string());

    Ok(cmd)
}

/// Apply YOLO (auto-accept) flags to the command for the given tool
pub fn apply_yolo_flags(config: &Config, cmd: &str, tool_name: &str) -> String {
    if let Some(tool) = get_detected_tool(config, tool_name) {
        if !tool.yolo_flags.is_empty() {
            // Insert YOLO flags after the tool name
            let yolo_flags_str = tool.yolo_flags.join(" ");
            // Find the tool name in the command and insert flags after it
            if let Some(pos) = cmd.find(tool_name) {
                let insert_pos = pos + tool_name.len();
                let mut result = cmd.to_string();
                result.insert_str(insert_pos, &format!(" {}", yolo_flags_str));
                return result;
            }
        }
    }
    cmd.to_string()
}

/// Build a docker command that wraps the LLM command
pub fn build_docker_command(
    config: &Config,
    inner_cmd: &str,
    project_path: &str,
) -> Result<String> {
    let docker_config = &config.launch.docker;

    if docker_config.image.is_empty() {
        anyhow::bail!(
            "Docker mode is enabled but no image is configured. \
             Set launch.docker.image in your config."
        );
    }

    let mut docker_args = vec![
        "docker".to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "-it".to_string(),
        "-v".to_string(),
        format!("{}:{}:rw", project_path, docker_config.mount_path),
        "-w".to_string(),
        docker_config.mount_path.clone(),
    ];

    // Add environment variables
    for env_var in &docker_config.env_vars {
        docker_args.push("-e".to_string());
        docker_args.push(env_var.clone());
    }

    // Add extra args from config
    for arg in &docker_config.extra_args {
        docker_args.push(arg.clone());
    }

    // Add the image
    docker_args.push(docker_config.image.clone());

    // Add the inner command (use sh -c to handle complex commands)
    docker_args.push("sh".to_string());
    docker_args.push("-c".to_string());
    docker_args.push(inner_cmd.to_string());

    Ok(docker_args.join(" "))
}

/// Generate config flags for the LLM command based on step permissions
fn generate_config_flags(
    config: &Config,
    provider: &str,
    ticket: &Ticket,
    project_path: &str,
    session_id: &str,
) -> Result<String> {
    // Load project permissions
    let project_perms = load_project_permissions(config, project_path)?;

    // Get step configuration from template
    let step_config = get_step_config(ticket)?;

    // Add operator-level directory permissions (.tickets/ for reading ticket files)
    let mut operator_perms = step_config.permissions.clone();
    operator_perms
        .directories
        .allow
        .push(config.tickets_path().to_string_lossy().to_string());

    // Merge permissions (additive)
    let merged = PermissionSet::merge(&project_perms, &operator_perms, &step_config.cli_args);

    // Create session directory for storing configs
    let session_dir = config
        .tickets_path()
        .join("operator")
        .join("sessions")
        .join(&ticket.id);
    fs::create_dir_all(&session_dir)
        .with_context(|| format!("Failed to create session dir: {:?}", session_dir))?;

    // Generate config using translator
    let translator = TranslatorManager::new();
    let generated = translator.generate_config(provider, &merged, &session_dir)?;

    // Save audit info
    let audit_command = format!("Session: {}\nTicket: {}\n", session_id, ticket.id);
    TranslatorManager::save_audit_info(&session_dir, provider, &generated, &audit_command)?;

    // Build CLI flags
    let mut cli_flags = generated.cli_flags;

    // Claude-specific flags
    if provider == "claude" {
        // Add permission mode flag (if not default)
        if step_config.permission_mode != PermissionMode::Default {
            let mode_str = match step_config.permission_mode {
                PermissionMode::Default => "default",
                PermissionMode::Plan => "plan",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::Delegate => "delegate",
            };
            cli_flags.push("--permission-mode".to_string());
            cli_flags.push(mode_str.to_string());
        }

        // Add JSON schema flag for structured output
        // Inline jsonSchema takes precedence over jsonSchemaFile
        if let Some(ref schema) = step_config.json_schema {
            let schema_str =
                serde_json::to_string(schema).context("Failed to serialize JSON schema")?;
            cli_flags.push("--json-schema".to_string());
            cli_flags.push(schema_str);
        } else if let Some(ref schema_file) = step_config.json_schema_file {
            // Resolve path relative to project root
            let schema_path = PathBuf::from(project_path).join(schema_file);
            let schema_content = fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read JSON schema file: {:?}", schema_path))?;
            cli_flags.push("--json-schema".to_string());
            cli_flags.push(schema_content);
        }
    }

    // Format CLI flags as a space-separated string (with trailing space if non-empty)
    if cli_flags.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{} ", cli_flags.join(" ")))
    }
}

/// Get the detected tool for a given provider
fn get_detected_tool<'a>(config: &'a Config, tool_name: &str) -> Option<&'a DetectedTool> {
    config
        .llm_tools
        .detected
        .iter()
        .find(|t| t.name == tool_name)
}

/// Get the model for the first available provider
pub fn get_default_model(config: &Config) -> Option<String> {
    config.llm_tools.providers.first().map(|p| p.model.clone())
}
