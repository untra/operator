use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use crate::agents::cmux::{CmuxClient, SystemCmuxClient};
use crate::agents::tmux::{SystemTmuxClient, TmuxClient};
use crate::agents::zellij::{SystemZellijClient, ZellijClient};
use crate::agents::{LaunchOptions, Launcher};
use crate::state::State;
use crate::ui::dashboard::FocusedPanel;
use crate::ui::with_suspended_tui;

use super::{App, AppTerminal};

impl App {
    /// Process pending agent switches triggered by step completion.
    ///
    /// When a step completes and the next step specifies a different agent (delegator),
    /// sync.rs sets the `review_state` to "`switching_agent:{delegator_name`}". This method
    /// detects those markers and performs the actual tmux agent switch.
    pub(super) fn process_agent_switches(&self, state: &mut State) -> Result<()> {
        use crate::agents::agent_switcher::{build_agent_command, AgentSwitcher};

        // Collect agents that need switching (avoid borrowing state during iteration)
        let switches: Vec<_> = state
            .agents
            .iter()
            .filter_map(|agent| {
                let review_state = agent.review_state.as_ref()?;
                let delegator_name = review_state.strip_prefix("switching_agent:")?;
                Some((
                    agent.id.clone(),
                    agent.session_name.clone(),
                    agent.llm_tool.clone(),
                    agent.worktree_path.clone(),
                    agent.project.clone(),
                    delegator_name.to_string(),
                    agent.session_wrapper.clone(),
                ))
            })
            .collect();

        if switches.is_empty() {
            return Ok(());
        }

        for (
            agent_id,
            session_name,
            current_tool,
            worktree_path,
            project,
            delegator_name,
            session_wrapper,
        ) in switches
        {
            let switcher = match session_wrapper.as_deref() {
                Some("zellij") => {
                    let zellij = SystemZellijClient::new();
                    AgentSwitcher::with_zellij(Arc::new(zellij))
                }
                Some("cmux") => {
                    let cmux = SystemCmuxClient::from_config(&self.config.sessions.cmux);
                    AgentSwitcher::with_cmux(Arc::new(cmux))
                }
                _ => AgentSwitcher::new(Arc::clone(&self.tmux_client)),
            };
            let Some(delegator) = self
                .config
                .delegators
                .iter()
                .find(|d| d.name == delegator_name)
            else {
                tracing::warn!(
                    agent_id = %agent_id,
                    delegator = %delegator_name,
                    "Delegator not found in config, clearing switch state"
                );
                state.clear_review_state(&agent_id)?;
                continue;
            };

            let Some(ref session) = session_name else {
                tracing::warn!(
                    agent_id = %agent_id,
                    "No tmux session for agent, cannot switch"
                );
                state.clear_review_state(&agent_id)?;
                continue;
            };

            let current = current_tool.as_deref().unwrap_or("claude");
            let new_command = build_agent_command(delegator);

            match switcher.switch_agent(session, current, &new_command) {
                Ok(()) => {
                    state.update_agent_tool_and_model(
                        &agent_id,
                        &delegator.llm_tool,
                        &delegator.model,
                    )?;
                    state.clear_review_state(&agent_id)?;
                    state.update_agent_status(&agent_id, "running", None)?;

                    // Redeploy skills for the new tool
                    if let Some(ref wt) = worktree_path {
                        let project_path = self.config.projects_path().join(&project);
                        let _ = crate::llm::deploy_skills(
                            &PathBuf::from(wt),
                            &project_path,
                            &[&delegator.llm_tool],
                        );
                    }

                    tracing::info!(
                        agent_id = %agent_id,
                        new_tool = %delegator.llm_tool,
                        new_model = %delegator.model,
                        "Agent switched successfully"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        agent_id = %agent_id,
                        error = %e,
                        "Failed to switch agent"
                    );
                    state.update_agent_status(
                        &agent_id,
                        "awaiting_input",
                        Some(format!("Agent switch failed: {e}")),
                    )?;
                    state.clear_review_state(&agent_id)?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn try_launch(&mut self) -> Result<()> {
        // Check if we can launch
        let state = State::load(&self.config)?;
        let running_count = state.running_agents().len();

        if running_count >= self.config.effective_max_agents() {
            // Could show an error dialog here
            return Ok(());
        }

        if self.dashboard.paused {
            // Could show an error dialog here
            return Ok(());
        }

        // Get selected ticket
        if let Some(ticket) = self.dashboard.selected_ticket().cloned() {
            // Check if project is already busy
            if state.is_project_busy(&ticket.project) {
                // Could show an error dialog here
                return Ok(());
            }

            // Configure dialog with available options from config
            self.confirm_dialog.configure(
                self.config.llm_tools.providers.clone(),
                self.config.projects.clone(),
                self.config.launch.docker.enabled,
                self.config.launch.yolo.enabled,
            );

            // Show confirmation
            self.confirm_dialog.show(ticket);
        }

        Ok(())
    }

    pub(super) async fn launch_confirmed(&mut self) -> Result<()> {
        if let Some(ticket) = self.confirm_dialog.ticket.take() {
            let launcher = Launcher::new(&self.config)?;

            // Build launch options from dialog state
            // Only set project_override if it differs from the ticket's original project
            let project_override = if self.confirm_dialog.is_project_overridden() {
                self.confirm_dialog.selected_project_name().cloned()
            } else {
                None
            };

            let options = LaunchOptions {
                provider: self.confirm_dialog.selected_provider().cloned(),
                docker_mode: self.confirm_dialog.docker_selected,
                yolo_mode: self.confirm_dialog.yolo_selected,
                project_override,
            };

            launcher.launch_with_options(&ticket, options).await?;
            self.confirm_dialog.hide();
            self.refresh_data()?;
        }
        Ok(())
    }

    /// Get the selected session info (name, wrapper, context ref) based on focused panel.
    fn selected_session_info(&self) -> (Option<String>, Option<String>, Option<String>) {
        match self.dashboard.focused {
            FocusedPanel::Agents => {
                // Check if an orphan session is selected
                if let Some(orphan) = self.dashboard.selected_orphan() {
                    (Some(orphan.session_name.clone()), None, None)
                } else {
                    // Otherwise get selected running agent's session
                    self.dashboard
                        .selected_running_agent()
                        .map_or((None, None, None), |a| {
                            (
                                a.session_name.clone(),
                                a.session_wrapper.clone(),
                                a.session_context_ref.clone(),
                            )
                        })
                }
            }
            FocusedPanel::Awaiting => {
                self.dashboard
                    .selected_awaiting_agent()
                    .map_or((None, None, None), |a| {
                        (
                            a.session_name.clone(),
                            a.session_wrapper.clone(),
                            a.session_context_ref.clone(),
                        )
                    })
            }
            _ => (None, None, None),
        }
    }

    pub(super) fn attach_to_session(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        // Get the selected agent or orphan session based on focused panel
        let (session_name, session_wrapper, session_context_ref) = self.selected_session_info();

        let Some(session_name) = session_name else {
            return Ok(());
        };

        // Dispatch based on session wrapper type
        if let Some("cmux") = session_wrapper.as_deref() {
            // For cmux agents, focus the workspace (no TUI suspension needed)
            if let Some(ws_ref) = session_context_ref {
                let cmux = SystemCmuxClient::from_config(&self.config.sessions.cmux);
                tracing::info!(session = %session_name, workspace = %ws_ref, "Focusing cmux workspace");
                if let Err(e) = cmux.focus_workspace(&ws_ref) {
                    tracing::warn!(
                        session = %session_name,
                        error = %e,
                        "Failed to focus cmux workspace"
                    );
                }
            } else {
                tracing::warn!(
                    session = %session_name,
                    "cmux agent has no workspace ref"
                );
            }
        } else if let Some("zellij") = session_wrapper.as_deref() {
            // For zellij agents, focus the tab (no TUI suspension needed)
            let zellij = SystemZellijClient::new();
            tracing::info!(session = %session_name, "Focusing zellij tab");
            if let Err(e) = zellij.focus_tab(&session_name) {
                tracing::warn!(
                    session = %session_name,
                    error = %e,
                    "Failed to focus zellij tab"
                );
            }
        } else {
            // Default: tmux attach
            let tmux: Box<dyn TmuxClient> = if self.config.tmux.config_generated {
                let config_path = self.config.tmux_config_path();
                if config_path.exists() {
                    Box::new(SystemTmuxClient::with_config(config_path))
                } else {
                    Box::new(SystemTmuxClient::new())
                }
            } else {
                Box::new(SystemTmuxClient::new())
            };

            tracing::info!(session = %session_name, "Attaching to tmux session");

            let attach_result =
                with_suspended_tui(terminal, || Ok(tmux.attach_session(&session_name)));

            match attach_result {
                Ok(Ok(())) => {
                    tracing::info!(session = %session_name, "Detached from tmux session");
                }
                Ok(Err(e)) => {
                    let error_str = e.to_string();
                    if error_str.contains("exit code: Some(1)") {
                        tracing::warn!(
                            session = %session_name,
                            "Tmux session not found, showing recovery dialog"
                        );
                        self.show_session_recovery_dialog(&session_name)?;
                    } else {
                        tracing::warn!(
                            session = %session_name,
                            error = %e,
                            "Failed to attach to session"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        session = %session_name,
                        error = %e,
                        "Error during TUI suspension"
                    );
                }
            }
        }

        // Refresh data after returning
        self.refresh_data()?;

        Ok(())
    }

    /// Show session preview for the selected agent
    pub(super) fn show_session_preview(&mut self) -> Result<()> {
        // Only works when agents or awaiting panel is focused
        let agent = match self.dashboard.focused {
            FocusedPanel::Agents => self.dashboard.selected_running_agent().cloned(),
            FocusedPanel::Awaiting => self.dashboard.selected_awaiting_agent().cloned(),
            _ => None,
        };

        let Some(agent) = agent else {
            return Ok(());
        };

        // Check if agent has a session
        let Some(ref session_name) = agent.session_name else {
            self.session_preview.show(
                &agent,
                Err("This agent does not have an attached terminal session.".to_string()),
            );
            return Ok(());
        };

        // Capture the session content, dispatching to the correct wrapper
        let content = if let Some("cmux") = agent.session_wrapper.as_deref() {
            if let Some(ref ws_ref) = agent.session_context_ref {
                let cmux = SystemCmuxClient::from_config(&self.config.sessions.cmux);
                cmux.read_screen(ws_ref, false)
                    .map_err(|e| format!("Failed to capture cmux workspace: {e}"))
            } else {
                Err("cmux agent has no workspace ref".to_string())
            }
        } else if let Some("zellij") = agent.session_wrapper.as_deref() {
            let zellij = SystemZellijClient::new();
            zellij
                .read_screen(session_name)
                .map_err(|e| format!("Failed to capture zellij tab: {e}"))
        } else {
            // Default: tmux capture
            let tmux: Box<dyn TmuxClient> = if self.config.tmux.config_generated {
                let config_path = self.config.tmux_config_path();
                if config_path.exists() {
                    Box::new(SystemTmuxClient::with_config(config_path))
                } else {
                    Box::new(SystemTmuxClient::new())
                }
            } else {
                Box::new(SystemTmuxClient::new())
            };
            tmux.capture_pane(session_name, false)
                .map_err(|e| format!("Failed to capture session: {e}"))
        };

        self.session_preview.show(&agent, content);

        Ok(())
    }
}
