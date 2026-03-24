use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::notifications::NotificationEvent;
use crate::queue::Queue;
use crate::state::State;

use super::App;

impl App {
    pub(super) fn refresh_data(&mut self) -> Result<()> {
        // Load queue
        let queue = Queue::new(&self.config)?;
        let tickets = queue.list_by_priority()?;
        self.dashboard.update_queue(tickets);

        // Load state
        let state = State::load(&self.config)?;
        self.dashboard.paused = state.paused;

        // Update agents
        let agents: Vec<_> = state.agents.clone();
        self.dashboard.update_agents(agents);

        // Update completed
        let completed: Vec<_> = state
            .recent_completions(self.config.ui.completed_history_hours)
            .into_iter()
            .cloned()
            .collect();
        self.dashboard.update_completed(completed);

        // Update Backstage server status
        self.backstage_server.refresh_status();
        self.dashboard
            .update_backstage_status(self.backstage_server.status());

        Ok(())
    }

    /// Reconcile state with actual tmux sessions on startup
    pub(super) fn reconcile_sessions(&self) -> Result<()> {
        let result = self.session_monitor.reconcile_on_startup()?;

        if result.active > 0 {
            tracing::info!(
                active = result.active,
                "Found active agent sessions from previous run"
            );
        }

        if !result.orphaned.is_empty() {
            tracing::warn!(
                orphaned = result.orphaned.len(),
                "Found orphaned agents (sessions no longer exist)"
            );

            // Notify about orphaned sessions
            for session in &result.orphaned {
                self.notification_service
                    .notify_sync(NotificationEvent::AgentSessionLost {
                        session_name: session.clone(),
                    });
            }
        }

        if !result.stale_sessions.is_empty() {
            tracing::warn!(
                stale = result.stale_sessions.len(),
                "Found stale tmux sessions with no matching agent"
            );

            // Auto-cleanup stale sessions
            let killed = self
                .session_monitor
                .cleanup_stale_sessions(&result.stale_sessions)?;
            if killed > 0 {
                tracing::info!(killed = killed, "Cleaned up stale tmux sessions");
            }
        }

        Ok(())
    }

    /// Run session health checks and handle orphaned sessions
    pub(super) fn run_health_checks(&mut self) -> Result<()> {
        // Only check if it's time
        if !self.session_monitor.should_check() {
            return Ok(());
        }

        let result = self.session_monitor.check_health(&HashMap::new())?;

        // Send notifications for orphaned sessions
        if !result.orphaned.is_empty() {
            tracing::warn!(
                orphaned = result.orphaned.len(),
                "Detected orphaned agent sessions"
            );

            for session in &result.orphaned {
                self.notification_service
                    .notify_sync(NotificationEvent::AgentSessionLost {
                        session_name: session.clone(),
                    });
            }
        }

        // Log content changes at debug level
        if !result.changed.is_empty() {
            tracing::debug!(
                changed = result.changed.len(),
                "Agent sessions with content changes"
            );
        }

        // Detect and update orphan sessions for display
        if let Ok(orphans) = self.session_monitor.detect_orphan_sessions() {
            self.dashboard.update_orphan_sessions(orphans);
        }

        Ok(())
    }

    /// Run periodic ticket-session sync
    pub(super) fn run_periodic_sync(&mut self) -> Result<()> {
        if !self.ticket_sync.should_sync() {
            return Ok(());
        }

        self.execute_sync()
    }

    /// Run manual sync (triggered by 'S' key)
    pub(super) fn run_manual_sync(&mut self) -> Result<()> {
        self.ticket_sync.force_sync();
        self.execute_sync()
    }

    /// Execute the sync and handle results
    pub(super) fn execute_sync(&mut self) -> Result<()> {
        let mut state = State::load(&self.config)?;
        let queue = Queue::new(&self.config)?;

        // Build artifact context: agent_id → (worktree_path, artifact_patterns)
        let mut artifact_context: HashMap<String, (PathBuf, Vec<String>)> = HashMap::new();
        let in_progress = queue.list_in_progress().unwrap_or_default();
        for ticket in &in_progress {
            if let Some(agent) = state.agent_by_ticket(&ticket.id) {
                if let Some(ref wt) = agent.worktree_path {
                    if let Some(step_schema) = ticket.current_step_schema() {
                        if step_schema.has_artifact_patterns() {
                            artifact_context.insert(
                                agent.id.clone(),
                                (PathBuf::from(wt), step_schema.artifact_patterns.clone()),
                            );
                        }
                    }
                }
            }
        }

        // Run health check to get current session states
        let health_result = self.session_monitor.check_health(&artifact_context)?;

        // Run the sync
        let result = self
            .ticket_sync
            .sync_all(&mut state, &queue, &health_result)?;

        // Log results
        if result.synced > 0 {
            tracing::debug!(
                synced = result.synced,
                awaiting = result.moved_to_awaiting.len(),
                timed_out = result.timed_out.len(),
                "Ticket-session sync completed"
            );
        }

        // Handle pending agent switches from step completions
        self.process_agent_switches(&mut state)?;

        // Send notifications for tickets that moved to awaiting
        for ticket_id in &result.moved_to_awaiting {
            self.notification_service
                .notify_sync(NotificationEvent::AgentAwaitingInput {
                    project: String::new(), // Project unknown in this context
                    ticket_type: String::new(),
                    ticket_id: ticket_id.clone(),
                    reason: "The agent is waiting for user input.".to_string(),
                });
        }

        for ticket_id in &result.timed_out {
            self.notification_service
                .notify_sync(NotificationEvent::AgentAwaitingInput {
                    project: String::new(),
                    ticket_type: String::new(),
                    ticket_id: ticket_id.clone(),
                    reason: "The agent step has timed out and is now awaiting input.".to_string(),
                });
        }

        // Log any errors
        for error in &result.errors {
            tracing::warn!("Sync error: {}", error);
        }

        Ok(())
    }
}
