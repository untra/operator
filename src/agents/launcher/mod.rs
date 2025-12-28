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

#[cfg(test)]
mod tests;

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

/// Session name prefix for operator-managed tmux sessions
pub const SESSION_PREFIX: &str = "op-";

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
        // Move ticket to in-progress
        let queue = Queue::new(&self.config)?;
        queue.claim_ticket(ticket)?;

        // Get project path (use override if provided)
        let project_path = if let Some(ref override_project) = options.project_override {
            self.get_project_path_for(override_project)?
        } else {
            self.get_project_path(ticket)?
        };

        // Generate the initial prompt for the agent
        let initial_prompt = generate_prompt(&self.config, ticket);

        // Launch in tmux session
        let session_name = launch_in_tmux_with_options(
            &self.config,
            &self.tmux,
            ticket,
            &project_path,
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
            notifications::send(
                "Agent Started",
                &format!(
                    "{} - {} (tmux: {}){}",
                    ticket.project, ticket.ticket_type, session_name, mode_suffix
                ),
                &ticket.summary,
                self.config.notifications.sound,
            )?;
        }

        Ok(agent_id)
    }

    /// Relaunch an existing in-progress ticket (does NOT claim from queue)
    ///
    /// Used when a tmux session died but the ticket is still in progress.
    /// Can optionally resume from an existing Claude session ID.
    pub async fn relaunch(&self, ticket: &Ticket, options: RelaunchOptions) -> Result<String> {
        // Get project path (ticket is already in in-progress)
        let project_path = self.get_project_path(ticket)?;

        // Generate the initial prompt for the agent
        let initial_prompt = generate_prompt(&self.config, ticket);

        // Launch in tmux session with resume support
        let session_name = launch_in_tmux_with_relaunch_options(
            &self.config,
            &self.tmux,
            ticket,
            &project_path,
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
            notifications::send(
                "Agent Relaunched",
                &format!(
                    "{} - {} (tmux: {}){}",
                    ticket.project, ticket.ticket_type, session_name, mode_suffix
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
