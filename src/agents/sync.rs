#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! Ticket-session synchronization for keeping ticket metadata in sync with tmux sessions.
//!
//! This module provides periodic and manual synchronization between:
//! - Ticket files (in .tickets/in-progress/)
//! - Agent state (in state.json)
//! - Tmux sessions (op-{ticket-id})

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;

use super::monitor::{HealthCheckResult, SessionMonitor};
use super::tmux::TmuxClient;
use crate::config::Config;
use crate::queue::{Queue, Ticket};
use crate::state::State;

/// Result of a sync cycle
#[derive(Debug, Default)]
pub struct SyncResult {
    /// Number of tickets synced
    pub synced: usize,
    /// Tickets moved to AWAITING status
    pub moved_to_awaiting: Vec<String>,
    /// Tickets that timed out
    pub timed_out: Vec<String>,
    /// Tickets that completed their step
    pub completed: Vec<String>,
    /// Errors encountered during sync
    pub errors: Vec<String>,
}

/// Action taken for a single ticket during sync
#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    /// No changes needed
    NoChange,
    /// Status was updated
    UpdatedStatus(String),
    /// Ticket moved to AWAITING
    MovedToAwaiting,
    /// Step completed, ready for next step
    StepCompleted,
    /// Step timed out
    TimedOut,
    /// Session is hung (no content change)
    Hung,
}

/// Ticket-session synchronizer
pub struct TicketSessionSync {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
    last_sync: Instant,
    sync_interval: Duration,
}

impl TicketSessionSync {
    /// Create a new sync manager
    pub fn new(config: &Config, tmux: Arc<dyn TmuxClient>) -> Self {
        Self {
            config: config.clone(),
            tmux,
            last_sync: Instant::now()
                .checked_sub(Duration::from_secs(config.agents.sync_interval))
                .unwrap_or_else(Instant::now),
            sync_interval: Duration::from_secs(config.agents.sync_interval),
        }
    }

    /// Check if it's time to run a sync
    pub fn should_sync(&self) -> bool {
        self.last_sync.elapsed() >= self.sync_interval
    }

    /// Force a sync now (resets the timer)
    pub fn force_sync(&mut self) {
        // Set last_sync to a time in the past to trigger immediate sync
        self.last_sync = Instant::now()
            .checked_sub(self.sync_interval)
            .unwrap_or_else(Instant::now);
    }

    /// Sync all in-progress tickets with their sessions
    pub fn sync_all(
        &mut self,
        state: &mut State,
        queue: &Queue,
        health_result: &HealthCheckResult,
    ) -> Result<SyncResult> {
        self.last_sync = Instant::now();
        let mut result = SyncResult::default();

        // Get all in-progress tickets
        let tickets = queue.list_in_progress()?;

        for mut ticket in tickets {
            // Find the corresponding agent
            if let Some(agent) = state.agent_by_ticket(&ticket.id) {
                let agent_id = agent.id.clone();
                let session_name = agent.session_name.clone().unwrap_or_default();

                // Determine the sync action based on health check results
                let action = self.determine_action(&ticket, &session_name, health_result);

                match action {
                    SyncAction::NoChange => {}
                    SyncAction::MovedToAwaiting => {
                        // Update agent status
                        state.update_agent_status(&agent_id, "awaiting_input", None)?;

                        // Add history entry to ticket
                        let step_display = ticket.current_step_display_name();
                        if let Err(e) = ticket.add_awaiting_entry(&step_display) {
                            result.errors.push(format!(
                                "Failed to add history entry for {}: {}",
                                ticket.id, e
                            ));
                        }

                        // Reset silence flag after handling
                        let _ = self.tmux.reset_silence_flag(&session_name);

                        result.moved_to_awaiting.push(ticket.id.clone());
                        tracing::info!(
                            ticket_id = %ticket.id,
                            step = %step_display,
                            "Ticket moved to AWAITING"
                        );
                    }
                    SyncAction::TimedOut => {
                        // Timeout is treated as AWAITING with timeout note
                        state.update_agent_status(
                            &agent_id,
                            "awaiting_input",
                            Some("Step timed out".to_string()),
                        )?;

                        // Add timeout entry to history
                        let step_display = ticket.current_step_display_name();
                        if let Err(e) = ticket.append_history(&format!(
                            "- **{}** - Step \"{}\" timed out after {} minutes",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            step_display,
                            self.config.agents.step_timeout / 60
                        )) {
                            result.errors.push(format!(
                                "Failed to add timeout entry for {}: {}",
                                ticket.id, e
                            ));
                        }

                        result.timed_out.push(ticket.id.clone());
                        tracing::warn!(
                            ticket_id = %ticket.id,
                            step = %step_display,
                            "Step timed out"
                        );
                    }
                    SyncAction::UpdatedStatus(new_status) => {
                        state.update_agent_status(&agent_id, &new_status, None)?;
                    }
                    SyncAction::StepCompleted => {
                        // This would be handled by the agent detecting completion
                        // and advancing to the next step
                        result.completed.push(ticket.id.clone());
                    }
                    SyncAction::Hung => {
                        // Session is hung but not necessarily awaiting input
                        tracing::warn!(
                            ticket_id = %ticket.id,
                            "Session appears hung (no content changes)"
                        );
                    }
                }

                result.synced += 1;
            }
        }

        Ok(result)
    }

    /// Determine what sync action to take for a ticket based on health check results
    fn determine_action(
        &self,
        ticket: &Ticket,
        session_name: &str,
        health_result: &HealthCheckResult,
    ) -> SyncAction {
        // Check if timed out (takes priority)
        if health_result.timed_out.iter().any(|s| s == session_name) {
            return SyncAction::TimedOut;
        }

        // Check if awaiting input (silence detected)
        if health_result
            .awaiting_input
            .iter()
            .any(|s| s == session_name)
        {
            return SyncAction::MovedToAwaiting;
        }

        // Check if orphaned
        if health_result.orphaned.iter().any(|s| s == session_name) {
            return SyncAction::UpdatedStatus("orphaned".to_string());
        }

        SyncAction::NoChange
    }

    /// Sync a single ticket (useful for manual sync of specific ticket)
    pub fn sync_ticket(
        &self,
        ticket: &mut Ticket,
        state: &mut State,
        health_result: &HealthCheckResult,
    ) -> Result<SyncAction> {
        if let Some(agent) = state.agent_by_ticket(&ticket.id) {
            let agent_id = agent.id.clone();
            let session_name = agent.session_name.clone().unwrap_or_default();

            let action = self.determine_action(ticket, &session_name, health_result);

            match &action {
                SyncAction::MovedToAwaiting => {
                    state.update_agent_status(&agent_id, "awaiting_input", None)?;
                    let step_display = ticket.current_step_display_name();
                    ticket.add_awaiting_entry(&step_display)?;
                    let _ = self.tmux.reset_silence_flag(&session_name);
                }
                SyncAction::TimedOut => {
                    state.update_agent_status(
                        &agent_id,
                        "awaiting_input",
                        Some("Step timed out".to_string()),
                    )?;
                    let step_display = ticket.current_step_display_name();
                    ticket.append_history(&format!(
                        "- **{}** - Step \"{}\" timed out after {} minutes",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        step_display,
                        self.config.agents.step_timeout / 60
                    ))?;
                }
                SyncAction::UpdatedStatus(status) => {
                    state.update_agent_status(&agent_id, status, None)?;
                }
                _ => {}
            }

            return Ok(action);
        }

        Ok(SyncAction::NoChange)
    }

    /// Get time until next scheduled sync
    pub fn time_until_next_sync(&self) -> Duration {
        let elapsed = self.last_sync.elapsed();
        if elapsed >= self.sync_interval {
            Duration::ZERO
        } else {
            self.sync_interval - elapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::tmux::MockTmuxClient;
    use crate::config::PathsConfig;
    use tempfile::TempDir;

    fn make_test_config(temp_dir: &TempDir) -> Config {
        let projects_path = temp_dir.path().join("projects");
        let tickets_path = temp_dir.path().join("tickets");
        let state_path = temp_dir.path().join("state");
        std::fs::create_dir_all(&projects_path).unwrap();
        std::fs::create_dir_all(tickets_path.join("queue")).unwrap();
        std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
        std::fs::create_dir_all(&state_path).unwrap();

        let mut config = Config {
            paths: PathsConfig {
                tickets: tickets_path.to_string_lossy().to_string(),
                projects: projects_path.to_string_lossy().to_string(),
                state: state_path.to_string_lossy().to_string(),
            },
            ..Default::default()
        };
        config.agents.sync_interval = 1;
        config.agents.step_timeout = 60;
        config
    }

    #[test]
    fn test_should_sync_initially() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);
        // Should be ready to sync immediately after creation
        assert!(sync.should_sync());
    }

    #[test]
    fn test_determine_action_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        // Empty health result means no action needed
        let health = HealthCheckResult::default();

        // Create a minimal ticket for testing
        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-123".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "plan".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
        };

        let action = sync.determine_action(&ticket, "op-FEAT-123", &health);
        assert_eq!(action, SyncAction::NoChange);
    }

    #[test]
    fn test_determine_action_awaiting() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        health.awaiting_input.push("op-FEAT-123".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-123".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "plan".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
        };

        let action = sync.determine_action(&ticket, "op-FEAT-123", &health);
        assert_eq!(action, SyncAction::MovedToAwaiting);
    }

    #[test]
    fn test_determine_action_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        health.timed_out.push("op-FEAT-456".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-456".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "implement".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
        };

        let action = sync.determine_action(&ticket, "op-FEAT-456", &health);
        assert_eq!(action, SyncAction::TimedOut);
    }

    #[test]
    fn test_timeout_takes_priority_over_awaiting() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        // Both timeout and awaiting for same session
        health.timed_out.push("op-FEAT-789".to_string());
        health.awaiting_input.push("op-FEAT-789".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-789".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "test".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
        };

        let action = sync.determine_action(&ticket, "op-FEAT-789", &health);
        // Timeout should take priority
        assert_eq!(action, SyncAction::TimedOut);
    }
}
