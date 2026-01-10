//! Agent launcher for starting Claude agents in tmux sessions
//!
//! This module handles launching agents with appropriate permissions,
//! prompt generation, and session management.

#![allow(dead_code)]

pub mod interpolation;
mod llm_command;
mod options;
mod prompt;
mod step_config;
mod tmux_session;
pub mod worktree_setup;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::agents::tmux::{sanitize_session_name, SystemTmuxClient, TmuxClient, TmuxError};
use crate::config::Config;
use crate::notifications;
use crate::queue::{Queue, Ticket};
use crate::state::State;

pub use options::{LaunchOptions, RelaunchOptions};
use prompt::generate_prompt;
use tmux_session::{launch_in_tmux_with_options, launch_in_tmux_with_relaunch_options};
use worktree_setup::setup_worktree_for_ticket;

use self::interpolation::PromptInterpolator;
use self::llm_command::{
    apply_yolo_flags, build_docker_command, build_llm_command_with_permissions_for_tool,
    get_default_model,
};
use self::prompt::{
    generate_session_uuid, get_agent_prompt, get_template_prompt, write_prompt_file,
};

/// Session name prefix for operator-managed tmux sessions
pub const SESSION_PREFIX: &str = "op-";

/// Result of preparing a launch without executing it
///
/// Contains all the information needed to launch an agent in any wrapper
/// (tmux, VS Code terminal, or standalone terminal).
#[derive(Debug, Clone)]
pub struct PreparedLaunch {
    /// Agent ID assigned to this launch
    pub agent_id: String,
    /// Ticket ID being launched
    pub ticket_id: String,
    /// Working directory (worktree path or project path)
    pub working_directory: PathBuf,
    /// Command to execute in the terminal
    pub command: String,
    /// Terminal/session name (e.g., "op-FEAT-123")
    pub terminal_name: String,
    /// Session UUID for the LLM tool
    pub session_id: String,
    /// Whether a worktree was created
    pub worktree_created: bool,
    /// Branch name (if worktree was created)
    pub branch: Option<String>,
}

/// Minimum required tmux version
pub const MIN_TMUX_VERSION: (u32, u32) = (2, 1);

pub struct Launcher {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
}

impl Launcher {
    /// Create a new Launcher with the system tmux client
    ///
    /// Uses custom tmux config if it has been generated and exists.
    pub fn new(config: &Config) -> Result<Self> {
        // Use custom tmux config if it exists
        let tmux: Arc<dyn TmuxClient> = if config.tmux.config_generated {
            let config_path = config.tmux_config_path();
            if config_path.exists() {
                Arc::new(SystemTmuxClient::with_config(config_path))
            } else {
                Arc::new(SystemTmuxClient::new())
            }
        } else {
            Arc::new(SystemTmuxClient::new())
        };

        Ok(Self {
            config: config.clone(),
            tmux,
        })
    }

    /// Create a new Launcher with a custom tmux client (for testing)
    pub fn with_tmux_client(config: &Config, tmux: Arc<dyn TmuxClient>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            tmux,
        })
    }

    /// Check if tmux is available and meets minimum version requirements
    pub fn check_tmux(&self) -> Result<(), TmuxError> {
        let version = self.tmux.check_available()?;

        if !version.meets_minimum(MIN_TMUX_VERSION.0, MIN_TMUX_VERSION.1) {
            return Err(TmuxError::VersionTooOld(
                version.raw,
                format!("{}.{}", MIN_TMUX_VERSION.0, MIN_TMUX_VERSION.1),
            ));
        }

        tracing::info!(
            version = %version.raw,
            "tmux is available"
        );

        Ok(())
    }

    /// Launch a Claude agent in a tmux session for the given ticket
    pub async fn launch(&self, ticket: &Ticket) -> Result<String> {
        self.launch_with_options(ticket, LaunchOptions::default())
            .await
    }

    /// Launch an agent with specific launch options
    pub async fn launch_with_options(
        &self,
        ticket: &Ticket,
        options: LaunchOptions,
    ) -> Result<String> {
        // Clone ticket so we can update worktree info
        let mut ticket = ticket.clone();

        // Move ticket to in-progress
        let queue = Queue::new(&self.config)?;
        queue.claim_ticket(&ticket)?;

        // Get project path (use override if provided)
        let project_path = if let Some(ref override_project) = options.project_override {
            PathBuf::from(self.get_project_path_for(override_project)?)
        } else {
            PathBuf::from(self.get_project_path(&ticket)?)
        };

        // Setup worktree for per-ticket isolation (if project is a git repo)
        let working_dir = setup_worktree_for_ticket(&self.config, &mut ticket, &project_path)
            .await
            .context("Failed to setup worktree for ticket")?;

        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Generate the initial prompt for the agent
        let initial_prompt = generate_prompt(&self.config, &ticket);

        // Launch in tmux session (using worktree as working directory)
        let session_name = launch_in_tmux_with_options(
            &self.config,
            &self.tmux,
            &ticket,
            &working_dir_str,
            &initial_prompt,
            &options,
        )?;

        // Determine tool name from options or default
        let llm_tool = options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with launch options
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &session_name)?;

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        // Send notification
        if self.config.notifications.enabled && self.config.notifications.on_agent_start {
            let mode_suffix = match (options.docker_mode, options.yolo_mode) {
                (true, true) => " [docker-yolo]",
                (true, false) => " [docker]",
                (false, true) => " [yolo]",
                (false, false) => "",
            };
            let worktree_suffix = if ticket.worktree_path.is_some() {
                " [worktree]"
            } else {
                ""
            };
            // TODO: Migrate to NotificationService when Launcher has access to it
            #[allow(deprecated)]
            notifications::send(
                "Agent Started",
                &format!(
                    "{} - {} (tmux: {}){}{}",
                    ticket.project, ticket.ticket_type, session_name, mode_suffix, worktree_suffix
                ),
                &ticket.summary,
                self.config.notifications.sound,
            )?;
        }

        Ok(agent_id)
    }

    /// Prepare a launch without executing it
    ///
    /// This method does everything needed to launch an agent (claim ticket,
    /// setup worktree, generate prompt, build command, register agent) but
    /// returns the command and details instead of executing in tmux.
    ///
    /// Use this for launching via VS Code terminals or other wrappers.
    pub async fn prepare_launch(
        &self,
        ticket: &Ticket,
        options: LaunchOptions,
    ) -> Result<PreparedLaunch> {
        // Clone ticket so we can update worktree info
        let mut ticket = ticket.clone();

        // Move ticket to in-progress
        let queue = Queue::new(&self.config)?;
        queue.claim_ticket(&ticket)?;

        // Get project path (use override if provided)
        let project_path = if let Some(ref override_project) = options.project_override {
            PathBuf::from(self.get_project_path_for(override_project)?)
        } else {
            PathBuf::from(self.get_project_path(&ticket)?)
        };

        // Setup worktree for per-ticket isolation (if project is a git repo)
        let working_dir = setup_worktree_for_ticket(&self.config, &mut ticket, &project_path)
            .await
            .context("Failed to setup worktree for ticket")?;

        let worktree_created = ticket.worktree_path.is_some();
        let branch = ticket.branch.clone();
        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Generate terminal/session name
        let terminal_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

        // Generate session UUID
        let session_uuid = generate_session_uuid();

        // Get the step name (use "initial" if not set)
        let step_name = if ticket.step.is_empty() {
            "initial".to_string()
        } else {
            ticket.step.clone()
        };

        // Store the session UUID in the ticket file (now in in-progress)
        let ticket_in_progress_path = self
            .config
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
            let default_tool = self
                .config
                .llm_tools
                .detected
                .first()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "claude".to_string());
            let default_model =
                get_default_model(&self.config).unwrap_or_else(|| "sonnet".to_string());
            (default_tool, default_model)
        };

        // Build the full prompt using the interpolation engine
        let initial_prompt = generate_prompt(&self.config, &ticket);
        let full_prompt = if get_template_prompt(&ticket.ticket_type).is_some() {
            let interpolator = PromptInterpolator::new();
            match interpolator.build_launch_prompt(&self.config, &ticket, &working_dir_str) {
                Ok(prompt) => prompt,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        ticket = %ticket.id,
                        "Failed to build interpolated prompt, falling back to initial prompt"
                    );
                    initial_prompt.clone()
                }
            }
        } else if let Some(agent_prompt) = get_agent_prompt(&ticket.ticket_type) {
            let ticket_path = format!("../.tickets/in-progress/{}", ticket.filename);
            let message = format!(
                "use the {} agent to implement the ticket at {}",
                ticket.ticket_type.to_lowercase(),
                ticket_path
            );
            format!("{}\n---\n{}", agent_prompt, message)
        } else {
            initial_prompt
        };

        // Write prompt to file
        let prompt_file = write_prompt_file(&self.config, &session_uuid, &full_prompt)?;

        // Build command using the detected tool's template (with permissions)
        let mut llm_cmd = build_llm_command_with_permissions_for_tool(
            &self.config,
            &tool_name,
            &model,
            &session_uuid,
            &prompt_file,
            Some(&ticket),
            Some(&working_dir_str),
        )?;

        // Apply YOLO flags if enabled
        if options.yolo_mode {
            llm_cmd = apply_yolo_flags(&self.config, &llm_cmd, &tool_name);
        }

        // Wrap in docker command if docker mode is enabled
        if options.docker_mode {
            llm_cmd = build_docker_command(&self.config, &llm_cmd, &working_dir_str)?;
        }

        // Determine tool name from options or default
        let llm_tool = options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with launch
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &terminal_name)?;

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        tracing::info!(
            terminal = %terminal_name,
            session_uuid = %session_uuid,
            project = %ticket.project,
            ticket = %ticket.id,
            step = %step_name,
            tool = %tool_name,
            launch_mode = %options.launch_mode_string(),
            working_dir = %working_dir_str,
            "Prepared agent launch"
        );

        Ok(PreparedLaunch {
            agent_id,
            ticket_id: ticket.id.clone(),
            working_directory: working_dir,
            command: llm_cmd,
            terminal_name,
            session_id: session_uuid,
            worktree_created,
            branch,
        })
    }

    /// Relaunch an existing in-progress ticket (does NOT claim from queue)
    ///
    /// Used when a tmux session died but the ticket is still in progress.
    /// Can optionally resume from an existing Claude session ID.
    pub async fn relaunch(&self, ticket: &Ticket, options: RelaunchOptions) -> Result<String> {
        // Clone ticket so we can update worktree info if needed
        let mut ticket = ticket.clone();

        // Get working directory (use existing worktree, or setup new one)
        let working_dir = if let Some(ref worktree_path) = ticket.worktree_path {
            let path = PathBuf::from(worktree_path);
            if path.exists() {
                // Reuse existing worktree
                path
            } else {
                // Worktree was deleted, recreate it
                let project_path = PathBuf::from(self.get_project_path(&ticket)?);
                setup_worktree_for_ticket(&self.config, &mut ticket, &project_path)
                    .await
                    .context("Failed to recreate worktree for ticket")?
            }
        } else {
            // No worktree yet, try to create one
            let project_path = PathBuf::from(self.get_project_path(&ticket)?);
            setup_worktree_for_ticket(&self.config, &mut ticket, &project_path)
                .await
                .context("Failed to setup worktree for ticket")?
        };

        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Generate the initial prompt for the agent
        let initial_prompt = generate_prompt(&self.config, &ticket);

        // Launch in tmux session with resume support
        let session_name = launch_in_tmux_with_relaunch_options(
            &self.config,
            &self.tmux,
            &ticket,
            &working_dir_str,
            &initial_prompt,
            &options,
        )?;

        // Determine tool name from options or default
        let llm_tool = options
            .launch_options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with new agent
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &session_name)?;

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        // Send notification
        if self.config.notifications.enabled && self.config.notifications.on_agent_start {
            let mode_suffix = if options.resume_session_id.is_some() {
                " [resumed]"
            } else {
                " [restarted]"
            };
            let worktree_suffix = if ticket.worktree_path.is_some() {
                " [worktree]"
            } else {
                ""
            };
            // TODO: Migrate to NotificationService when Launcher has access to it
            #[allow(deprecated)]
            notifications::send(
                "Agent Relaunched",
                &format!(
                    "{} - {} (tmux: {}){}{}",
                    ticket.project, ticket.ticket_type, session_name, mode_suffix, worktree_suffix
                ),
                &ticket.summary,
                self.config.notifications.sound,
            )?;
        }

        Ok(agent_id)
    }

    fn get_project_path(&self, ticket: &Ticket) -> Result<String> {
        self.get_project_path_for(&ticket.project)
    }

    /// Get project path for a given project name
    fn get_project_path_for(&self, project: &str) -> Result<String> {
        let projects_root = self.config.projects_path();

        let project_path = if project == "global" {
            // Global tickets use the root directory
            projects_root
        } else {
            projects_root.join(project)
        };

        if !project_path.exists() {
            anyhow::bail!("Project path does not exist: {:?}", project_path);
        }

        Ok(project_path.to_string_lossy().to_string())
    }

    /// List all operator tmux sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        match self.tmux.list_sessions(Some(SESSION_PREFIX)) {
            Ok(sessions) => Ok(sessions.into_iter().map(|s| s.name).collect()),
            Err(TmuxError::NotInstalled) => {
                tracing::warn!("tmux not installed, returning empty session list");
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to list tmux sessions");
                Ok(Vec::new())
            }
        }
    }

    /// Kill a specific operator tmux session
    pub fn kill_session(&self, session_name: &str) -> Result<()> {
        self.tmux
            .kill_session(session_name)
            .context("Failed to kill tmux session")?;
        Ok(())
    }

    /// Capture the current content of a session's pane
    pub fn capture_session_content(&self, session_name: &str) -> Result<String> {
        self.tmux
            .capture_pane(session_name, false)
            .context("Failed to capture pane content")
    }

    /// Check if a session is still alive
    pub fn session_alive(&self, session_name: &str) -> bool {
        matches!(self.tmux.session_exists(session_name), Ok(true))
    }

    /// Attach to a tmux session (returns the command to run)
    pub fn attach_command(session_name: &str) -> String {
        format!("tmux attach -t {}", session_name)
    }
}
