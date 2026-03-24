use anyhow::Result;

use crate::agents::{LaunchOptions, Launcher, RelaunchOptions};
use crate::notifications::NotificationEvent;
use crate::queue::Queue;
use crate::state::State;
use crate::ui::SessionRecoverySelection;

use super::App;

impl App {
    /// Show the session recovery dialog for a dead tmux session
    pub(super) fn show_session_recovery_dialog(&mut self, session_name: &str) -> Result<()> {
        // Find the agent by session name
        let state = State::load(&self.config)?;
        let agent = state.agent_by_session(session_name);

        let Some(agent) = agent else {
            tracing::warn!(session = %session_name, "No agent found for session");
            return Ok(());
        };

        let ticket_id = agent.ticket_id.clone();
        let current_step = agent.current_step.clone();

        // Load the ticket to get session data
        let queue = Queue::new(&self.config)?;
        let ticket = queue.get_in_progress_ticket(&ticket_id)?;

        let Some(ticket) = ticket else {
            tracing::warn!(ticket = %ticket_id, "Ticket not found in in-progress");
            return Ok(());
        };

        // Get the step name (current_step from agent or step from ticket)
        let step = current_step.unwrap_or_else(|| ticket.step.clone());
        let step = if step.is_empty() {
            "initial".to_string()
        } else {
            step
        };

        // Look up Claude session ID for this step
        let claude_session_id = ticket.get_session_id(&step).cloned();

        // Show the recovery dialog
        self.session_recovery_dialog.show(
            ticket.id,
            session_name.to_string(),
            step,
            claude_session_id,
        );

        Ok(())
    }

    /// Handle a session recovery dialog selection
    pub(super) async fn handle_session_recovery(
        &mut self,
        selection: SessionRecoverySelection,
    ) -> Result<()> {
        let ticket_id = self.session_recovery_dialog.ticket_id.clone();
        let session_name = self.session_recovery_dialog.session_name.clone();
        let claude_session_id = self.session_recovery_dialog.claude_session_id.clone();

        self.session_recovery_dialog.hide();

        match selection {
            SessionRecoverySelection::ResumeSession => {
                // Relaunch with resume flag
                self.relaunch_ticket(&ticket_id, &session_name, claude_session_id)
                    .await?;
            }
            SessionRecoverySelection::StartFresh => {
                // Relaunch without resume flag
                self.relaunch_ticket(&ticket_id, &session_name, None)
                    .await?;
            }
            SessionRecoverySelection::ReturnToQueue => {
                // Move ticket back to queue, remove agent from state
                self.return_ticket_to_queue(&ticket_id, &session_name)?;
            }
            SessionRecoverySelection::Cancel => {
                // Do nothing, dialog already hidden
            }
        }

        self.refresh_data()?;
        Ok(())
    }

    /// Relaunch a ticket with optional session resume
    pub(super) async fn relaunch_ticket(
        &mut self,
        ticket_id: &str,
        old_session_name: &str,
        resume_session_id: Option<String>,
    ) -> Result<()> {
        // Load ticket from in-progress
        let queue = Queue::new(&self.config)?;
        let ticket = queue
            .get_in_progress_ticket(ticket_id)?
            .ok_or_else(|| anyhow::anyhow!("Ticket not found: {ticket_id}"))?;

        // Remove old agent state
        let mut state = State::load(&self.config)?;
        state.remove_agent_by_session(old_session_name)?;

        // Relaunch with the launcher
        let launcher = Launcher::new(&self.config)?;
        let options = RelaunchOptions {
            launch_options: LaunchOptions::default(),
            resume_session_id,
            retry_reason: None,
        };

        launcher.relaunch(&ticket, options).await?;

        Ok(())
    }

    /// Return a ticket to the queue and clean up agent state
    pub(super) fn return_ticket_to_queue(
        &mut self,
        ticket_id: &str,
        session_name: &str,
    ) -> Result<()> {
        // Load ticket
        let queue = Queue::new(&self.config)?;
        let ticket = queue
            .get_in_progress_ticket(ticket_id)?
            .ok_or_else(|| anyhow::anyhow!("Ticket not found: {ticket_id}"))?;

        // Move ticket back to queue
        queue.return_to_queue(&ticket)?;

        // Remove agent from state
        let mut state = State::load(&self.config)?;
        state.remove_agent_by_session(session_name)?;

        // Send notification
        self.notification_service
            .notify_sync(NotificationEvent::TicketReturned {
                project: ticket.project.clone(),
                ticket_id: ticket.id.clone(),
                summary: ticket.summary,
            });

        Ok(())
    }
}
