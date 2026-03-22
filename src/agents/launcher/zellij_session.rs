//! Zellij session creation and management for agent launches
//!
//! Parallel to `cmux_session.rs` — provides zellij-specific launch functions
//! that create tabs and send commands via `ZellijClient`.

use std::sync::Arc;

use anyhow::Result;

use crate::agents::tmux::sanitize_session_name;
use crate::agents::zellij::ZellijClient;
use crate::config::Config;
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
/// Result of launching in zellij — includes tab name for state tracking
#[derive(Debug, Clone)]
pub struct ZellijLaunchResult {
    pub session_name: String,
    pub tab_name: String,
    pub session_uuid: String,
}

/// Launch an agent in a zellij tab with specific options
pub fn launch_in_zellij_with_options(
    config: &Config,
    zellij: &Arc<dyn ZellijClient>,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
    options: &LaunchOptions,
) -> Result<ZellijLaunchResult> {
    // Check zellij is available and we're inside zellij
    zellij
        .check_available()
        .map_err(|e| anyhow::anyhow!("zellij is not available: {e}"))?;
    zellij
        .check_in_zellij()
        .map_err(|e| anyhow::anyhow!("Not running inside zellij: {e}"))?;

    // Create session name from ticket ID with project for scannable Zellij tab bar
    let session_name = format!(
        "op:{}:{}",
        sanitize_session_name(&ticket.project),
        sanitize_session_name(&ticket.id)
    );

    // Tab name = session name (1:1 mapping)
    let tab_name = session_name.clone();

    // Create tab in zellij
    zellij
        .create_tab(&tab_name, project_path)
        .map_err(|e| anyhow::anyhow!("Failed to create zellij tab: {e}"))?;

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

    // Send the command to the zellij tab
    let bash_cmd = format!("bash {}\n", command_file.display());
    if let Err(e) = zellij.send_text(&tab_name, &bash_cmd) {
        // Clean up tab on failure
        let _ = zellij.close_tab(&tab_name);
        anyhow::bail!("Failed to start LLM agent in zellij tab: {e}");
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        tab = %tab_name,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        launch_mode = %options.launch_mode_string(),
        working_dir = %project_path,
        "Launched agent in zellij tab"
    );

    Ok(ZellijLaunchResult {
        session_name,
        tab_name,
        session_uuid,
    })
}

/// Launch in zellij with relaunch options (supports resume from existing session)
pub fn launch_in_zellij_with_relaunch_options(
    config: &Config,
    zellij: &Arc<dyn ZellijClient>,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
    options: &RelaunchOptions,
) -> Result<ZellijLaunchResult> {
    // Check zellij is available and we're inside zellij
    zellij
        .check_available()
        .map_err(|e| anyhow::anyhow!("zellij is not available: {e}"))?;
    zellij
        .check_in_zellij()
        .map_err(|e| anyhow::anyhow!("Not running inside zellij: {e}"))?;

    let session_name = format!(
        "op:{}:{}",
        sanitize_session_name(&ticket.project),
        sanitize_session_name(&ticket.id)
    );

    // Tab name = session name (1:1 mapping)
    let tab_name = session_name.clone();

    // Create tab
    zellij
        .create_tab(&tab_name, project_path)
        .map_err(|e| anyhow::anyhow!("Failed to create zellij tab: {e}"))?;

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
    if let Err(e) = zellij.send_text(&tab_name, &bash_cmd) {
        let _ = zellij.close_tab(&tab_name);
        anyhow::bail!("Failed to start LLM agent in zellij tab: {e}");
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        tab = %tab_name,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        is_resume = %is_resume,
        launch_mode = %options.launch_options.launch_mode_string(),
        working_dir = %project_path,
        "Relaunched agent in zellij tab"
    );

    Ok(ZellijLaunchResult {
        session_name,
        tab_name,
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
