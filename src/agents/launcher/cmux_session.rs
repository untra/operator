//! cmux session creation and management for agent launches
//!
//! Parallel to `tmux_session.rs` — provides cmux-specific launch functions
//! that create workspaces/windows and send commands via `CmuxClient`.

use std::sync::Arc;

use anyhow::Result;

use crate::agents::cmux::CmuxClient;
use crate::agents::tmux::sanitize_session_name;
use crate::config::{CmuxPlacementPolicy, Config};
use crate::queue::Ticket;

use super::interpolation::PromptInterpolator;
use super::llm_command::{
    apply_yolo_flags, build_docker_command, build_llm_command_with_permissions_for_tool,
    get_default_model,
};
use super::options::{LaunchOptions, RelaunchOptions};
use super::prompt::{
    generate_session_uuid, get_agent_prompt, get_template_prompt, write_command_file,
    write_prompt_file,
};
use super::SESSION_PREFIX;

/// Result of launching in cmux — includes refs needed for state tracking
#[derive(Debug, Clone)]
pub struct CmuxLaunchResult {
    pub session_name: String,
    pub window_ref: String,
    pub workspace_ref: String,
    pub session_uuid: String,
}

/// Resolve placement policy: returns (`window_ref`, `is_new_window`)
fn resolve_placement(
    cmux: &Arc<dyn CmuxClient>,
    placement: CmuxPlacementPolicy,
) -> Result<(String, bool)> {
    match placement {
        CmuxPlacementPolicy::Workspace => {
            let window_id = cmux
                .active_window_id()
                .map_err(|e| anyhow::anyhow!("Failed to get active window: {e}"))?;
            Ok((window_id, false))
        }
        CmuxPlacementPolicy::Window => {
            let window_id = cmux
                .create_window(None)
                .map_err(|e| anyhow::anyhow!("Failed to create window: {e}"))?;
            Ok((window_id, true))
        }
        CmuxPlacementPolicy::Auto => {
            let count = cmux
                .window_count()
                .map_err(|e| anyhow::anyhow!("Failed to count windows: {e}"))?;
            if count <= 1 {
                let window_id = cmux
                    .active_window_id()
                    .map_err(|e| anyhow::anyhow!("Failed to get active window: {e}"))?;
                Ok((window_id, false))
            } else {
                let window_id = cmux
                    .create_window(None)
                    .map_err(|e| anyhow::anyhow!("Failed to create window: {e}"))?;
                Ok((window_id, true))
            }
        }
    }
}

/// Launch an agent in a cmux workspace with specific options
pub fn launch_in_cmux_with_options(
    config: &Config,
    cmux: &Arc<dyn CmuxClient>,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
    options: &LaunchOptions,
) -> Result<CmuxLaunchResult> {
    // Check cmux is available and we're inside cmux
    cmux.check_available()
        .map_err(|e| anyhow::anyhow!("cmux is not available: {e}"))?;
    cmux.check_in_cmux()
        .map_err(|e| anyhow::anyhow!("Not running inside cmux: {e}"))?;

    // Create session name from ticket ID
    let session_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

    // Resolve placement policy
    let (window_ref, _new_window) = resolve_placement(cmux, config.sessions.cmux.placement)?;

    // Create workspace in the target window
    let workspace_ref = cmux
        .create_workspace(&window_ref, project_path, Some(&session_name))
        .map_err(|e| anyhow::anyhow!("Failed to create cmux workspace: {e}"))?;

    // Generate a UUID for the session
    let session_uuid = generate_session_uuid();

    // Get the step name
    let step_name = if ticket.step.is_empty() {
        "initial".to_string()
    } else {
        ticket.step.clone()
    };

    // Store the session UUID in the ticket file
    let ticket_in_progress_path = config
        .tickets_path()
        .join("in-progress")
        .join(&ticket.filename);
    if ticket_in_progress_path.exists() {
        if let Ok(mut updated_ticket) = Ticket::from_file(&ticket_in_progress_path) {
            if let Err(e) = updated_ticket.set_session_id(&step_name, &session_uuid) {
                tracing::warn!(
                    error = %e,
                    ticket = %ticket.id,
                    step = %step_name,
                    "Failed to store session UUID in ticket"
                );
            }
        }
    }

    // Get the model and tool from options or use defaults
    let (tool_name, model) = if let Some(ref provider) = options.provider {
        (provider.tool.clone(), provider.model.clone())
    } else {
        let default_tool = config
            .llm_tools
            .detected
            .first()
            .map_or_else(|| "claude".to_string(), |t| t.name.clone());
        let default_model = get_default_model(config).unwrap_or_else(|| "sonnet".to_string());
        (default_tool, default_model)
    };

    // Build the full prompt
    let full_prompt = build_full_prompt(config, ticket, project_path, initial_prompt);

    // Write prompt to file
    let prompt_file = write_prompt_file(config, &session_uuid, &full_prompt)?;

    // Build LLM command
    let mut llm_cmd = build_llm_command_with_permissions_for_tool(
        config,
        &tool_name,
        &model,
        &session_uuid,
        &prompt_file,
        Some(ticket),
        Some(project_path),
    )?;

    if options.yolo_mode {
        llm_cmd = apply_yolo_flags(config, &llm_cmd, &tool_name);
    }

    if options.docker_mode {
        llm_cmd = build_docker_command(config, &llm_cmd, project_path)?;
    }

    // Write the command to a shell script file
    let command_file = write_command_file(config, &session_uuid, project_path, &llm_cmd)?;

    // Send the command to the cmux workspace
    let bash_cmd = format!("bash {}\n", command_file.display());
    if let Err(e) = cmux.send_text(&workspace_ref, &bash_cmd) {
        // Clean up workspace on failure
        let _ = cmux.close_workspace(&workspace_ref);
        anyhow::bail!("Failed to start LLM agent in cmux workspace: {e}");
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        workspace = %workspace_ref,
        window = %window_ref,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        launch_mode = %options.launch_mode_string(),
        working_dir = %project_path,
        "Launched agent in cmux workspace"
    );

    Ok(CmuxLaunchResult {
        session_name,
        window_ref,
        workspace_ref,
        session_uuid,
    })
}

/// Launch in cmux with relaunch options (supports resume from existing session)
pub fn launch_in_cmux_with_relaunch_options(
    config: &Config,
    cmux: &Arc<dyn CmuxClient>,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
    options: &RelaunchOptions,
) -> Result<CmuxLaunchResult> {
    // Check cmux is available and we're inside cmux
    cmux.check_available()
        .map_err(|e| anyhow::anyhow!("cmux is not available: {e}"))?;
    cmux.check_in_cmux()
        .map_err(|e| anyhow::anyhow!("Not running inside cmux: {e}"))?;

    let session_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

    // Resolve placement policy
    let (window_ref, _new_window) = resolve_placement(cmux, config.sessions.cmux.placement)?;

    // Create workspace
    let workspace_ref = cmux
        .create_workspace(&window_ref, project_path, Some(&session_name))
        .map_err(|e| anyhow::anyhow!("Failed to create cmux workspace: {e}"))?;

    let step_name = if ticket.step.is_empty() {
        "initial".to_string()
    } else {
        ticket.step.clone()
    };

    // Determine session UUID and prompt file based on resume mode
    let (session_uuid, prompt_file, is_resume) =
        if let Some(ref resume_id) = options.resume_session_id {
            let prompts_dir = config.tickets_path().join("operator").join("prompts");
            let existing_prompt_file = prompts_dir.join(format!("{resume_id}.txt"));

            if existing_prompt_file.exists() {
                (resume_id.clone(), existing_prompt_file, true)
            } else {
                tracing::warn!(
                    resume_id = %resume_id,
                    "Resume prompt file not found, starting fresh"
                );
                let new_uuid = generate_session_uuid();
                let new_prompt_file = write_prompt_file(config, &new_uuid, initial_prompt)?;
                (new_uuid, new_prompt_file, false)
            }
        } else {
            let new_uuid = generate_session_uuid();
            let full_prompt = build_full_prompt(config, ticket, project_path, initial_prompt);
            let new_prompt_file = write_prompt_file(config, &new_uuid, &full_prompt)?;

            // Store the session UUID in the ticket file
            let ticket_in_progress_path = config
                .tickets_path()
                .join("in-progress")
                .join(&ticket.filename);
            if ticket_in_progress_path.exists() {
                if let Ok(mut updated_ticket) = Ticket::from_file(&ticket_in_progress_path) {
                    if let Err(e) = updated_ticket.set_session_id(&step_name, &new_uuid) {
                        tracing::warn!(
                            error = %e,
                            ticket = %ticket.id,
                            step = %step_name,
                            "Failed to store session UUID in ticket"
                        );
                    }
                }
            }

            (new_uuid, new_prompt_file, false)
        };

    // Get the model and tool
    let (tool_name, model) = if let Some(ref provider) = options.launch_options.provider {
        (provider.tool.clone(), provider.model.clone())
    } else {
        let default_tool = config
            .llm_tools
            .detected
            .first()
            .map_or_else(|| "claude".to_string(), |t| t.name.clone());
        let default_model = get_default_model(config).unwrap_or_else(|| "sonnet".to_string());
        (default_tool, default_model)
    };

    // Build LLM command
    let mut llm_cmd = build_llm_command_with_permissions_for_tool(
        config,
        &tool_name,
        &model,
        &session_uuid,
        &prompt_file,
        Some(ticket),
        Some(project_path),
    )?;

    if is_resume {
        if let Some(pos) = llm_cmd.find(&tool_name) {
            let insert_pos = pos + tool_name.len();
            llm_cmd.insert_str(insert_pos, &format!(" --resume {session_uuid}"));
        }
    }

    if options.launch_options.yolo_mode {
        llm_cmd = apply_yolo_flags(config, &llm_cmd, &tool_name);
    }

    if options.launch_options.docker_mode {
        llm_cmd = build_docker_command(config, &llm_cmd, project_path)?;
    }

    // Write and send command
    let command_file = write_command_file(config, &session_uuid, project_path, &llm_cmd)?;
    let bash_cmd = format!("bash {}\n", command_file.display());
    if let Err(e) = cmux.send_text(&workspace_ref, &bash_cmd) {
        let _ = cmux.close_workspace(&workspace_ref);
        anyhow::bail!("Failed to start LLM agent in cmux workspace: {e}");
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        workspace = %workspace_ref,
        window = %window_ref,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        is_resume = %is_resume,
        launch_mode = %options.launch_options.launch_mode_string(),
        working_dir = %project_path,
        "Relaunched agent in cmux workspace"
    );

    Ok(CmuxLaunchResult {
        session_name,
        window_ref,
        workspace_ref,
        session_uuid,
    })
}

/// Build the full prompt from template, agent prompt, or initial prompt
fn build_full_prompt(
    config: &Config,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
) -> String {
    if get_template_prompt(&ticket.ticket_type).is_some() {
        let interpolator = PromptInterpolator::new();
        match interpolator.build_launch_prompt(config, ticket, project_path) {
            Ok(prompt) => prompt,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    ticket = %ticket.id,
                    "Failed to build interpolated prompt, falling back to initial prompt"
                );
                initial_prompt.to_string()
            }
        }
    } else if let Some(agent_prompt) = get_agent_prompt(&ticket.ticket_type) {
        let ticket_path = format!("../.tickets/in-progress/{}", ticket.filename);
        let message = format!(
            "use the {} agent to implement the ticket at {}",
            ticket.ticket_type.to_lowercase(),
            ticket_path
        );
        format!("{agent_prompt}\n---\n{message}")
    } else {
        initial_prompt.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::cmux::{CmuxClient, MockCmuxClient};

    #[test]
    fn test_resolve_placement_auto_single_window() {
        let client: Arc<dyn CmuxClient> = Arc::new(MockCmuxClient::new());
        let (window_ref, new) = resolve_placement(&client, CmuxPlacementPolicy::Auto).unwrap();
        assert_eq!(window_ref, "win-1");
        assert!(!new);
    }

    #[test]
    fn test_resolve_placement_auto_multi_window() {
        let mock = MockCmuxClient::new();
        mock.add_window("Second");
        let client: Arc<dyn CmuxClient> = Arc::new(mock);
        let (window_ref, new) = resolve_placement(&client, CmuxPlacementPolicy::Auto).unwrap();
        assert_ne!(window_ref, "win-1"); // New window created
        assert!(new);
    }

    #[test]
    fn test_resolve_placement_workspace_always_active() {
        let mock = MockCmuxClient::new();
        mock.add_window("Second");
        let client: Arc<dyn CmuxClient> = Arc::new(mock);
        let (window_ref, new) = resolve_placement(&client, CmuxPlacementPolicy::Workspace).unwrap();
        assert_eq!(window_ref, "win-1"); // Active window
        assert!(!new);
    }

    #[test]
    fn test_resolve_placement_window_always_new() {
        let client: Arc<dyn CmuxClient> = Arc::new(MockCmuxClient::new());
        let (window_ref, new) = resolve_placement(&client, CmuxPlacementPolicy::Window).unwrap();
        assert_ne!(window_ref, "win-1"); // New window
        assert!(new);
    }
}
