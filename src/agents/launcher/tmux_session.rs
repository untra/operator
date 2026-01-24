//! Tmux session creation and management for agent launches

use std::sync::Arc;

use anyhow::Result;

use crate::agents::tmux::{sanitize_session_name, TmuxClient, TmuxError};
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
use super::SESSION_PREFIX;

/// Launch Claude in a tmux session with specific options
pub fn launch_in_tmux_with_options(
    config: &Config,
    tmux: &Arc<dyn TmuxClient>,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
    options: &LaunchOptions,
) -> Result<String> {
    // Create session name from ticket ID (sanitize for tmux)
    let session_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

    // Check if session already exists
    match tmux.session_exists(&session_name) {
        Ok(true) => {
            anyhow::bail!(
                "Tmux session '{}' already exists. Attach with: tmux attach -t {}",
                session_name,
                session_name
            );
        }
        Err(TmuxError::NotInstalled) => {
            anyhow::bail!(
                "tmux is not installed. Please install tmux to use operator.\n\
                 On macOS: brew install tmux\n\
                 On Ubuntu/Debian: sudo apt install tmux"
            );
        }
        Err(e) => {
            tracing::warn!(error = %e, "Error checking for existing session, proceeding anyway");
        }
        Ok(false) => {} // Good, session doesn't exist
    }

    // Create new tmux session in project directory
    tmux.create_session(&session_name, project_path)
        .map_err(|e| match e {
            TmuxError::NotInstalled => anyhow::anyhow!(
                "tmux is not installed. Please install tmux to use operator.\n\
                 On macOS: brew install tmux\n\
                 On Ubuntu/Debian: sudo apt install tmux"
            ),
            TmuxError::SessionExists(_) => anyhow::anyhow!(
                "Tmux session '{}' already exists. Attach with: tmux attach -t {}",
                session_name,
                session_name
            ),
            _ => anyhow::anyhow!("Failed to create tmux session '{}': {}", session_name, e),
        })?;

    // Wait for the shell to initialize before sending keys
    // Without this delay, send_keys may run before the shell is ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Set up silence monitoring for awaiting input detection
    if let Err(e) = tmux.set_monitor_silence(&session_name, config.agents.silence_threshold as u32)
    {
        tracing::warn!(
            session = %session_name,
            error = %e,
            "Failed to set monitor-silence, awaiting detection may not work"
        );
    }

    // Generate a UUID for the Claude session-id
    let session_uuid = generate_session_uuid();

    // Get the step name (use "initial" if not set)
    let step_name = if ticket.step.is_empty() {
        "initial".to_string()
    } else {
        ticket.step.clone()
    };

    // Store the session UUID in the ticket file (now in in-progress)
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
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "claude".to_string());
        let default_model = get_default_model(config).unwrap_or_else(|| "sonnet".to_string());
        (default_tool, default_model)
    };

    // Build the full prompt using the new interpolation engine
    // Priority: template.prompt (new) > agent_prompt (legacy) > initial_prompt (fallback)
    let full_prompt = if get_template_prompt(&ticket.ticket_type).is_some() {
        // New style: use interpolation engine to combine issuetype prompt + step prompt + ticket
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
        // Legacy: templates with agent_prompt (FEAT, FIX, INV, SPIKE without new prompt field)
        let ticket_path = format!("../.tickets/in-progress/{}", ticket.filename);
        let message = format!(
            "use the {} agent to implement the ticket at {}",
            ticket.ticket_type.to_lowercase(),
            ticket_path
        );
        // Combine prompt and message with separator
        format!("{}\n---\n{}", agent_prompt, message)
    } else {
        // Fallback: templates without any prompt (TASK) - use original detailed prompt
        initial_prompt.to_string()
    };

    // Write prompt to file (avoids newline issues with tmux send-keys)
    let prompt_file = write_prompt_file(config, &session_uuid, &full_prompt)?;

    // Build command using the detected tool's template (with permissions)
    let mut llm_cmd = build_llm_command_with_permissions_for_tool(
        config,
        &tool_name,
        &model,
        &session_uuid,
        &prompt_file,
        Some(ticket),
        Some(project_path),
    )?;

    // Apply YOLO flags if enabled
    if options.yolo_mode {
        llm_cmd = apply_yolo_flags(config, &llm_cmd, &tool_name);
    }

    // Wrap in docker command if docker mode is enabled
    if options.docker_mode {
        llm_cmd = build_docker_command(config, &llm_cmd, project_path)?;
    }

    // Write the command to a shell script file to avoid issues with long commands
    // and special characters when using tmux send-keys
    let command_file = write_command_file(config, &session_uuid, project_path, &llm_cmd)?;

    // Send simple bash command to execute the script (always short, so no buffer needed)
    let bash_cmd = format!("bash {}", command_file.display());
    if let Err(e) = tmux.send_keys(&session_name, &bash_cmd, true) {
        // Clean up the session if we couldn't send the command
        let _ = tmux.kill_session(&session_name);
        anyhow::bail!("Failed to start LLM agent in tmux session: {}", e);
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        launch_mode = %options.launch_mode_string(),
        working_dir = %project_path,
        command_file = %command_file.display(),
        "Launched agent in tmux session"
    );

    Ok(session_name)
}

/// Launch in tmux with relaunch options (supports resume from existing session)
pub fn launch_in_tmux_with_relaunch_options(
    config: &Config,
    tmux: &Arc<dyn TmuxClient>,
    ticket: &Ticket,
    project_path: &str,
    initial_prompt: &str,
    options: &RelaunchOptions,
) -> Result<String> {
    // Create session name from ticket ID (sanitize for tmux)
    let session_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

    // Check if session already exists
    match tmux.session_exists(&session_name) {
        Ok(true) => {
            anyhow::bail!(
                "Tmux session '{}' already exists. Attach with: tmux attach -t {}",
                session_name,
                session_name
            );
        }
        Err(TmuxError::NotInstalled) => {
            anyhow::bail!(
                "tmux is not installed. Please install tmux to use operator.\n\
                 On macOS: brew install tmux\n\
                 On Ubuntu/Debian: sudo apt install tmux"
            );
        }
        Err(e) => {
            tracing::warn!(error = %e, "Error checking for existing session, proceeding anyway");
        }
        Ok(false) => {} // Good, session doesn't exist
    }

    // Create new tmux session in project directory
    tmux.create_session(&session_name, project_path)
        .map_err(|e| match e {
            TmuxError::NotInstalled => anyhow::anyhow!(
                "tmux is not installed. Please install tmux to use operator.\n\
                 On macOS: brew install tmux\n\
                 On Ubuntu/Debian: sudo apt install tmux"
            ),
            TmuxError::SessionExists(_) => anyhow::anyhow!(
                "Tmux session '{}' already exists. Attach with: tmux attach -t {}",
                session_name,
                session_name
            ),
            _ => anyhow::anyhow!("Failed to create tmux session '{}': {}", session_name, e),
        })?;

    // Wait for the shell to initialize before sending keys
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Set up silence monitoring for awaiting input detection
    if let Err(e) = tmux.set_monitor_silence(&session_name, config.agents.silence_threshold as u32)
    {
        tracing::warn!(
            session = %session_name,
            error = %e,
            "Failed to set monitor-silence, awaiting detection may not work"
        );
    }

    // Get the step name (use "initial" if not set)
    let step_name = if ticket.step.is_empty() {
        "initial".to_string()
    } else {
        ticket.step.clone()
    };

    // Determine session UUID and prompt file based on resume mode
    let (session_uuid, prompt_file, is_resume) =
        if let Some(ref resume_id) = options.resume_session_id {
            // Resume mode: use existing session ID and prompt file
            let prompts_dir = config.tickets_path().join("operator").join("prompts");
            let existing_prompt_file = prompts_dir.join(format!("{}.txt", resume_id));

            if existing_prompt_file.exists() {
                (resume_id.clone(), existing_prompt_file, true)
            } else {
                // Prompt file doesn't exist, fall back to fresh start
                tracing::warn!(
                    resume_id = %resume_id,
                    "Resume prompt file not found, starting fresh"
                );
                let new_uuid = generate_session_uuid();
                let new_prompt_file = write_prompt_file(config, &new_uuid, initial_prompt)?;
                (new_uuid, new_prompt_file, false)
            }
        } else {
            // Fresh start: generate new session UUID and prompt
            let new_uuid = generate_session_uuid();

            // Build the full prompt using the new interpolation engine
            // Priority: template.prompt (new) > agent_prompt (legacy) > initial_prompt (fallback)
            let full_prompt = if get_template_prompt(&ticket.ticket_type).is_some() {
                // New style: use interpolation engine
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
                // Legacy: templates with agent_prompt
                let ticket_path = format!("../.tickets/in-progress/{}", ticket.filename);
                let message = format!(
                    "use the {} agent to implement the ticket at {}",
                    ticket.ticket_type.to_lowercase(),
                    ticket_path
                );
                format!("{}\n---\n{}", agent_prompt, message)
            } else {
                // Fallback: templates without any prompt
                initial_prompt.to_string()
            };

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

    // Get the model and tool from options or use defaults
    let (tool_name, model) = if let Some(ref provider) = options.launch_options.provider {
        (provider.tool.clone(), provider.model.clone())
    } else {
        let default_tool = config
            .llm_tools
            .detected
            .first()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "claude".to_string());
        let default_model = get_default_model(config).unwrap_or_else(|| "sonnet".to_string());
        (default_tool, default_model)
    };

    // Build command using the detected tool's template (with permissions)
    let mut llm_cmd = build_llm_command_with_permissions_for_tool(
        config,
        &tool_name,
        &model,
        &session_uuid,
        &prompt_file,
        Some(ticket),
        Some(project_path),
    )?;

    // Add resume flag if resuming
    if is_resume {
        // Insert --resume <session_id> after the tool name
        // Claude CLI: claude --resume <session_id> ...
        if let Some(pos) = llm_cmd.find(&tool_name) {
            let insert_pos = pos + tool_name.len();
            llm_cmd.insert_str(insert_pos, &format!(" --resume {}", session_uuid));
        }
    }

    // Apply YOLO flags if enabled
    if options.launch_options.yolo_mode {
        llm_cmd = apply_yolo_flags(config, &llm_cmd, &tool_name);
    }

    // Wrap in docker command if docker mode is enabled
    if options.launch_options.docker_mode {
        llm_cmd = build_docker_command(config, &llm_cmd, project_path)?;
    }

    // Write the command to a shell script file to avoid issues with long commands
    // and special characters when using tmux send-keys
    let command_file = write_command_file(config, &session_uuid, project_path, &llm_cmd)?;

    // Send simple bash command to execute the script (always short, so no buffer needed)
    let bash_cmd = format!("bash {}", command_file.display());
    if let Err(e) = tmux.send_keys(&session_name, &bash_cmd, true) {
        // Clean up the session if we couldn't send the command
        let _ = tmux.kill_session(&session_name);
        anyhow::bail!("Failed to start LLM agent in tmux session: {}", e);
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        is_resume = %is_resume,
        launch_mode = %options.launch_options.launch_mode_string(),
        working_dir = %project_path,
        command_file = %command_file.display(),
        "Relaunched agent in tmux session"
    );

    Ok(session_name)
}
