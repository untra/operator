use anyhow::Result;

use crate::ui::dashboard::FocusedPanel;

use super::App;

impl App {
    /// Handle review approval for the selected agent
    ///
    /// Only works for agents in `awaiting_input` with a `review_state` of `pending_plan` or `pending_visual`.
    /// Creates a signal file to trigger resume in the next sync cycle.
    pub(super) fn handle_review_approval(&mut self) -> Result<()> {
        // Only works when agents or awaiting panel is focused
        let agent = match self.dashboard.focused {
            FocusedPanel::Agents => self.dashboard.selected_running_agent().cloned(),
            FocusedPanel::Awaiting => self.dashboard.selected_awaiting_agent().cloned(),
            _ => None,
        };

        let Some(agent) = agent else {
            return Ok(());
        };

        // Only process if agent has a review state that can be approved
        if let Some("pending_plan" | "pending_visual") = agent.review_state.as_deref() {
            // Write signal file to trigger resume
            if let Some(ref session_name) = agent.session_name {
                let signal_file = format!("/tmp/operator-detach-{session_name}.signal");
                std::fs::write(&signal_file, "approved")?;

                tracing::info!(
                    agent_id = %agent.id,
                    session = %session_name,
                    review_state = ?agent.review_state,
                    "Review approved - signal file written"
                );
            }
        } else {
            // No review state or non-approvable state - ignore
        }

        Ok(())
    }

    /// Handle review rejection for the selected agent
    ///
    /// Only works for agents in `awaiting_input` with a `review_state` of `pending_plan` or `pending_visual`.
    /// For now, this just logs the rejection. A full implementation would show a dialog
    /// for entering a rejection reason and possibly restart the step.
    pub(super) fn handle_review_rejection(&mut self) -> Result<()> {
        // Only works when agents or awaiting panel is focused
        let agent = match self.dashboard.focused {
            FocusedPanel::Agents => self.dashboard.selected_running_agent().cloned(),
            FocusedPanel::Awaiting => self.dashboard.selected_awaiting_agent().cloned(),
            _ => None,
        };

        let Some(agent) = agent else {
            return Ok(());
        };

        // Only process if agent has a review state that can be rejected
        if let Some("pending_plan" | "pending_visual") = agent.review_state.as_deref() {
            // TODO: Show rejection dialog for entering reason
            // For now, just log the rejection
            tracing::info!(
                agent_id = %agent.id,
                review_state = ?agent.review_state,
                "Review rejected (rejection dialog not yet implemented)"
            );
        } else {
            // No review state or non-rejectable state - ignore
        }

        Ok(())
    }
}
