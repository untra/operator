#![allow(dead_code)]

//! Session health monitoring for agent tmux sessions.
//!
//! Periodically checks that agent tmux sessions are still alive and
//! marks agents as orphaned if their sessions have terminated unexpectedly.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::tmux::{SystemTmuxClient, TmuxClient};
use crate::config::Config;
use crate::state::State;

/// Result of a health check cycle
#[derive(Debug, Default)]
pub struct HealthCheckResult {
    /// Number of sessions checked
    pub checked: usize,
    /// Sessions that were found alive
    pub alive: usize,
    /// Sessions that were marked as orphaned
    pub orphaned: Vec<String>,
    /// Sessions with content changes
    pub changed: Vec<String>,
    /// Sessions that have timed out (past step_timeout)
    pub timed_out: Vec<String>,
    /// Sessions detected as awaiting input (silence flag set)
    pub awaiting_input: Vec<String>,
}

/// Session health monitor
pub struct SessionMonitor {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
    last_check: Instant,
    check_interval: Duration,
}

impl SessionMonitor {
    /// Create a new session monitor with the default tmux client
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            tmux: Arc::new(SystemTmuxClient::new()),
            last_check: Instant::now(),
            check_interval: Duration::from_secs(config.agents.health_check_interval),
        }
    }

    /// Create a new session monitor with a custom tmux client (for testing)
    pub fn with_tmux_client(config: &Config, tmux: Arc<dyn TmuxClient>) -> Self {
        Self {
            config: config.clone(),
            tmux,
            last_check: Instant::now(),
            check_interval: Duration::from_secs(config.agents.health_check_interval),
        }
    }

    /// Check if it's time to run a health check
    pub fn should_check(&self) -> bool {
        self.last_check.elapsed() >= self.check_interval
    }

    /// Run a health check on all active agent sessions
    pub fn check_health(&mut self) -> Result<HealthCheckResult> {
        self.last_check = Instant::now();

        let mut result = HealthCheckResult::default();
        let mut state = State::load(&self.config)?;

        // Get all agents that should have sessions
        let agents_with_sessions: Vec<_> = state
            .agents_with_sessions()
            .iter()
            .map(|a| (a.id.clone(), a.session_name.clone().unwrap_or_default()))
            .collect();

        result.checked = agents_with_sessions.len();

        // Get all active operator sessions from tmux
        let active_sessions: HashSet<String> = self
            .tmux
            .list_sessions(Some("op-"))
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.name)
            .collect();

        // Check each agent
        for (agent_id, session_name) in agents_with_sessions {
            if session_name.is_empty() {
                continue;
            }

            if active_sessions.contains(&session_name) {
                result.alive += 1;

                // Check for content changes
                if let Ok(content) = self.tmux.capture_pane(&session_name, false) {
                    let hash = hash_content(&content);
                    if let Ok(changed) = state.update_agent_content_hash(&agent_id, &hash) {
                        if changed {
                            result.changed.push(session_name.clone());
                            // Content changed means session is active, record the change
                            let _ = state.record_content_change(&agent_id);
                            tracing::debug!(
                                agent_id = %agent_id,
                                session = %session_name,
                                "Session content changed"
                            );
                        }
                    }
                }

                // Check for silence flag (awaiting input detection)
                if let Ok(is_silent) = self.tmux.check_silence_flag(&session_name) {
                    if is_silent {
                        result.awaiting_input.push(session_name.clone());
                        tracing::info!(
                            agent_id = %agent_id,
                            session = %session_name,
                            "Session is silent (awaiting input)"
                        );
                    }
                }

                // Check for step timeout
                if state.is_step_timed_out(&agent_id, self.config.agents.step_timeout) {
                    result.timed_out.push(session_name.clone());
                    tracing::warn!(
                        agent_id = %agent_id,
                        session = %session_name,
                        timeout_secs = self.config.agents.step_timeout,
                        "Step has timed out"
                    );
                }
            } else {
                // Session is gone - mark as orphaned
                tracing::warn!(
                    agent_id = %agent_id,
                    session = %session_name,
                    "Session not found, marking agent as orphaned"
                );

                state.mark_agent_orphaned(&agent_id)?;
                result.orphaned.push(session_name);
            }
        }

        Ok(result)
    }

    /// Force an immediate health check (resets the timer)
    pub fn force_check(&mut self) -> Result<HealthCheckResult> {
        self.check_health()
    }

    /// Get the time until the next scheduled check
    pub fn time_until_next_check(&self) -> Duration {
        let elapsed = self.last_check.elapsed();
        if elapsed >= self.check_interval {
            Duration::ZERO
        } else {
            self.check_interval - elapsed
        }
    }
}

/// Result of startup reconciliation
#[derive(Debug, Default)]
pub struct ReconciliationResult {
    /// Agents in state whose sessions are still running
    pub active: usize,
    /// Agents marked as orphaned (session gone)
    pub orphaned: Vec<String>,
    /// Tmux sessions with no matching agent (stale)
    pub stale_sessions: Vec<String>,
}

impl SessionMonitor {
    /// Reconcile state with actual tmux sessions on startup
    ///
    /// This should be called once when the app starts to detect:
    /// - Agents whose sessions have died while operator wasn't running
    /// - Stale tmux sessions that have no corresponding agent
    pub fn reconcile_on_startup(&self) -> Result<ReconciliationResult> {
        let mut result = ReconciliationResult::default();
        let mut state = State::load(&self.config)?;

        // Get all active operator sessions from tmux
        let active_sessions: HashSet<String> = self
            .tmux
            .list_sessions(Some("op-"))
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.name)
            .collect();

        // Get all agents that should have sessions
        let agents_with_sessions: Vec<_> = state
            .agents_with_sessions()
            .iter()
            .map(|a| (a.id.clone(), a.session_name.clone().unwrap_or_default()))
            .collect();

        let known_session_names: HashSet<String> = agents_with_sessions
            .iter()
            .map(|(_, name)| name.clone())
            .collect();

        // Check each agent's session
        for (agent_id, session_name) in agents_with_sessions {
            if session_name.is_empty() {
                continue;
            }

            if active_sessions.contains(&session_name) {
                result.active += 1;
            } else {
                // Session is gone - mark as orphaned
                tracing::warn!(
                    agent_id = %agent_id,
                    session = %session_name,
                    "Agent session not found on startup, marking as orphaned"
                );
                state.mark_agent_orphaned(&agent_id)?;
                result.orphaned.push(session_name);
            }
        }

        // Find stale sessions (tmux sessions with no matching agent)
        for session_name in &active_sessions {
            if !known_session_names.contains(session_name) {
                tracing::warn!(
                    session = %session_name,
                    "Found stale tmux session with no matching agent"
                );
                result.stale_sessions.push(session_name.clone());
            }
        }

        Ok(result)
    }

    /// Kill stale tmux sessions that have no matching agent
    pub fn cleanup_stale_sessions(&self, sessions: &[String]) -> Result<usize> {
        let mut killed = 0;
        for session in sessions {
            match self.tmux.kill_session(session) {
                Ok(()) => {
                    tracing::info!(session = %session, "Killed stale tmux session");
                    killed += 1;
                }
                Err(e) => {
                    tracing::warn!(session = %session, error = %e, "Failed to kill stale session");
                }
            }
        }
        Ok(killed)
    }
}

/// Compute a SHA256 hash of content for change detection
fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
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

        let mut config = Config::default();
        config.paths = PathsConfig {
            tickets: tickets_path.to_string_lossy().to_string(),
            projects: projects_path.to_string_lossy().to_string(),
            state: state_path.to_string_lossy().to_string(),
        };
        config.agents.health_check_interval = 1; // 1 second for testing
        config
    }

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("Hello, World!");
        let hash2 = hash_content("Hello, World!");
        let hash3 = hash_content("Different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_should_check_initially_true() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = make_test_config(&temp_dir);
        config.agents.health_check_interval = 0; // Immediate

        let monitor = SessionMonitor::new(&config);
        assert!(monitor.should_check());
    }

    #[test]
    fn test_health_check_no_agents() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health().unwrap();

        assert_eq!(result.checked, 0);
        assert_eq!(result.alive, 0);
        assert!(result.orphaned.is_empty());
    }

    #[test]
    fn test_health_check_finds_alive_session() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a session
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-123".to_string(),
                "TASK".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-TASK-123")
            .unwrap();

        // Create mock with matching session
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-TASK-123", "/tmp");
        mock.set_session_content("op-TASK-123", "Claude is working...");

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health().unwrap();

        assert_eq!(result.checked, 1);
        assert_eq!(result.alive, 1);
        assert!(result.orphaned.is_empty());
    }

    #[test]
    fn test_health_check_detects_orphan() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a session that doesn't exist
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-456".to_string(),
                "TASK".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-TASK-456")
            .unwrap();

        // Create mock WITHOUT the session (simulating it died)
        let mock = Arc::new(MockTmuxClient::new());
        // Note: NOT adding the session

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health().unwrap();

        assert_eq!(result.checked, 1);
        assert_eq!(result.alive, 0);
        assert_eq!(result.orphaned.len(), 1);
        assert_eq!(result.orphaned[0], "op-TASK-456");

        // Verify agent was marked as orphaned
        let state = State::load(&config).unwrap();
        let orphaned = state.orphaned_agents();
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0].id, agent_id);
    }

    #[test]
    fn test_health_check_detects_content_change() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a session and initial content hash
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-789".to_string(),
                "TASK".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-TASK-789")
            .unwrap();
        state
            .update_agent_content_hash(&agent_id, &hash_content("Initial content"))
            .unwrap();

        // Create mock with different content
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-TASK-789", "/tmp");
        mock.set_session_content("op-TASK-789", "New different content!");

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health().unwrap();

        assert_eq!(result.checked, 1);
        assert_eq!(result.alive, 1);
        assert_eq!(result.changed.len(), 1);
        assert_eq!(result.changed[0], "op-TASK-789");
    }

    #[test]
    fn test_time_until_next_check() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = make_test_config(&temp_dir);
        config.agents.health_check_interval = 60; // 60 seconds

        let monitor = SessionMonitor::new(&config);
        let time_until = monitor.time_until_next_check();

        // Should be close to 60 seconds (allow some slack for test execution)
        assert!(time_until.as_secs() <= 60);
        assert!(time_until.as_secs() >= 59);
    }

    #[test]
    fn test_reconcile_no_agents_no_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.reconcile_on_startup().unwrap();

        assert_eq!(result.active, 0);
        assert!(result.orphaned.is_empty());
        assert!(result.stale_sessions.is_empty());
    }

    #[test]
    fn test_reconcile_finds_active_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a session
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-100".to_string(),
                "TASK".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-TASK-100")
            .unwrap();

        // Create mock with matching session
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-TASK-100", "/tmp");

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.reconcile_on_startup().unwrap();

        assert_eq!(result.active, 1);
        assert!(result.orphaned.is_empty());
        assert!(result.stale_sessions.is_empty());
    }

    #[test]
    fn test_reconcile_detects_orphans() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a session that's gone
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-200".to_string(),
                "TASK".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-TASK-200")
            .unwrap();

        // Create mock WITHOUT the session
        let mock = Arc::new(MockTmuxClient::new());

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.reconcile_on_startup().unwrap();

        assert_eq!(result.active, 0);
        assert_eq!(result.orphaned.len(), 1);
        assert_eq!(result.orphaned[0], "op-TASK-200");
        assert!(result.stale_sessions.is_empty());

        // Verify agent is marked as orphaned
        let state = State::load(&config).unwrap();
        let orphaned = state.orphaned_agents();
        assert_eq!(orphaned.len(), 1);
    }

    #[test]
    fn test_reconcile_detects_stale_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create mock with a session that has no agent
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-UNKNOWN-999", "/tmp");

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.reconcile_on_startup().unwrap();

        assert_eq!(result.active, 0);
        assert!(result.orphaned.is_empty());
        assert_eq!(result.stale_sessions.len(), 1);
        assert_eq!(result.stale_sessions[0], "op-UNKNOWN-999");
    }

    #[test]
    fn test_cleanup_stale_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-STALE-1", "/tmp");
        mock.add_session("op-STALE-2", "/tmp");

        let monitor = SessionMonitor::with_tmux_client(&config, mock.clone());

        // Verify sessions exist
        assert!(mock.session_exists("op-STALE-1").unwrap());
        assert!(mock.session_exists("op-STALE-2").unwrap());

        // Cleanup
        let killed = monitor
            .cleanup_stale_sessions(&["op-STALE-1".to_string(), "op-STALE-2".to_string()])
            .unwrap();

        assert_eq!(killed, 2);
        assert!(!mock.session_exists("op-STALE-1").unwrap());
        assert!(!mock.session_exists("op-STALE-2").unwrap());
    }
}
