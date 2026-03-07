#![allow(dead_code)]

//! Session health monitoring for agent tmux sessions.
//!
//! Periodically checks that agent tmux sessions are still alive and
//! marks agents as orphaned if their sessions have terminated unexpectedly.
//!
//! Uses multi-signal detection for awaiting state:
//! 1. Hook signals (Claude/Gemini) - fastest, most accurate
//! 2. Content pattern detection - checks for idle prompts
//! 3. Tmux silence flag - fallback for all tools

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::artifact_detector::{ArtifactDetector, ArtifactStatus};
use super::cmux::{CmuxClient, SystemCmuxClient};
use super::hooks::HookManager;
use super::idle_detector::IdleDetector;
use super::tmux::{SystemTmuxClient, TmuxClient};
use super::zellij::{SystemZellijClient, ZellijClient};
use crate::config::{Config, SessionWrapperType};
use crate::llm::tool_config::load_all_tool_configs;
use crate::state::{OrphanSession, State};

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
    /// Sessions that have timed out (past `step_timeout`)
    pub timed_out: Vec<String>,
    /// Sessions detected as awaiting input (via hooks, patterns, or silence)
    pub awaiting_input: Vec<String>,
    /// Sessions that resumed from awaiting state (content changed while awaiting)
    pub resumed: Vec<String>,
    /// Sessions where all artifact patterns matched (step produced expected output)
    pub artifact_ready: Vec<String>,
}

/// Session health monitor
pub struct SessionMonitor {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
    cmux: Option<Arc<dyn CmuxClient>>,
    zellij: Option<Arc<dyn ZellijClient>>,
    last_check: Instant,
    check_interval: Duration,
    /// Hook manager for Claude/Gemini hook-based detection
    hook_manager: HookManager,
    /// Idle detector for pattern-based detection
    idle_detector: IdleDetector,
    /// Artifact detector for positive step-completion signals
    artifact_detector: ArtifactDetector,
}

impl SessionMonitor {
    /// Create a new session monitor
    ///
    /// Uses custom tmux config if it has been generated and exists,
    /// matching the socket used by the Launcher.
    /// Also creates a cmux client if the wrapper type is Cmux.
    pub fn new(config: &Config) -> Self {
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

        let cmux: Option<Arc<dyn CmuxClient>> =
            if config.sessions.wrapper == SessionWrapperType::Cmux {
                Some(Arc::new(SystemCmuxClient::from_config(
                    &config.sessions.cmux,
                )))
            } else {
                None
            };

        let zellij: Option<Arc<dyn ZellijClient>> =
            if config.sessions.wrapper == SessionWrapperType::Zellij {
                Some(Arc::new(SystemZellijClient::new()))
            } else {
                None
            };

        // Initialize idle detector from tool configs
        let tool_configs = load_all_tool_configs();
        let idle_detector = IdleDetector::from_tool_configs(&tool_configs);

        Self {
            config: config.clone(),
            tmux,
            cmux,
            zellij,
            last_check: Instant::now(),
            check_interval: Duration::from_secs(config.agents.health_check_interval),
            hook_manager: HookManager::new(),
            idle_detector,
            artifact_detector: ArtifactDetector::new(),
        }
    }

    /// Create a new session monitor with a custom tmux client (for testing)
    pub fn with_tmux_client(config: &Config, tmux: Arc<dyn TmuxClient>) -> Self {
        let tool_configs = load_all_tool_configs();
        let idle_detector = IdleDetector::from_tool_configs(&tool_configs);

        Self {
            config: config.clone(),
            tmux,
            cmux: None,
            zellij: None,
            last_check: Instant::now(),
            check_interval: Duration::from_secs(config.agents.health_check_interval),
            hook_manager: HookManager::new(),
            idle_detector,
            artifact_detector: ArtifactDetector::new(),
        }
    }

    /// Capture content for an agent, dispatching to the correct wrapper.
    /// For cmux agents, reads from the session context (workspace); for tmux, captures pane.
    fn capture_agent_content(
        &self,
        session_name: &str,
        session_wrapper: Option<&str>,
        session_context_ref: Option<&str>,
    ) -> Option<String> {
        match session_wrapper {
            Some("cmux") => {
                if let (Some(cmux), Some(ws_ref)) = (&self.cmux, session_context_ref) {
                    cmux.read_screen(ws_ref, false).ok()
                } else {
                    None
                }
            }
            Some("zellij") => {
                if let Some(ref zellij) = self.zellij {
                    zellij.read_screen(session_name).ok()
                } else {
                    None
                }
            }
            _ => {
                // Default to tmux (backward compat)
                self.tmux.capture_pane(session_name, false).ok()
            }
        }
    }

    /// Check if it's time to run a health check
    pub fn should_check(&self) -> bool {
        self.last_check.elapsed() >= self.check_interval
    }

    /// Run a health check on all active agent sessions
    ///
    /// Uses multi-signal detection for awaiting state:
    /// 1. Hook signals (Claude/Gemini) - fastest, most accurate
    /// 2. Content pattern detection - checks for idle prompts
    /// 3. Tmux silence flag - fallback for all tools
    ///
    /// Also detects resume: content changed while in awaiting status.
    ///
    /// `artifact_context` maps `agent_id` → (`worktree_path`, `artifact_patterns`) for
    /// positive completion detection. When an agent is idle AND its artifacts exist,
    /// the session is added to `artifact_ready` instead of just `awaiting_input`.
    pub fn check_health(
        &mut self,
        artifact_context: &HashMap<String, (PathBuf, Vec<String>)>,
    ) -> Result<HealthCheckResult> {
        self.last_check = Instant::now();

        let mut result = HealthCheckResult::default();
        let mut state = State::load(&self.config)?;

        // Get all agents that should have sessions, including their status, tool, and wrapper info
        let agents_with_sessions: Vec<_> = state
            .agents_with_sessions()
            .iter()
            .map(|a| {
                (
                    a.id.clone(),
                    a.session_name.clone().unwrap_or_default(),
                    a.status.clone(),
                    a.llm_tool.clone(),
                    a.session_wrapper.clone(),
                    a.session_context_ref.clone(),
                )
            })
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
        for (
            agent_id,
            session_name,
            agent_status,
            llm_tool,
            session_wrapper,
            session_context_ref,
        ) in agents_with_sessions
        {
            if session_name.is_empty() {
                continue;
            }

            // For cmux/zellij agents, check aliveness differently
            let is_cmux = session_wrapper.as_deref() == Some("cmux");
            let is_zellij = session_wrapper.as_deref() == Some("zellij");

            // For tmux agents, check against active tmux sessions
            // For cmux agents, assume alive if we have a workspace ref (cmux doesn't have list-sessions in the same way)
            // For zellij agents, check if the tab still exists
            let is_alive = if is_cmux {
                session_context_ref.is_some()
            } else if is_zellij {
                // Check if the zellij tab still exists
                if let Some(ref zellij) = self.zellij {
                    zellij
                        .list_tab_names()
                        .map(|tabs| tabs.iter().any(|t| t == &session_name))
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                active_sessions.contains(&session_name)
            };

            if is_alive {
                result.alive += 1;

                // Track if this session is detected as awaiting (avoid duplicate detection)
                let mut detected_awaiting = false;

                // 1. Check hook signal first (fastest, most accurate for Claude/Gemini)
                if let Some(signal) = self.hook_manager.check_hook_signal(&agent_id) {
                    if signal.event == "stop" {
                        detected_awaiting = true;
                        result.awaiting_input.push(session_name.clone());
                        tracing::info!(
                            agent_id = %agent_id,
                            session = %session_name,
                            "Hook signal detected: agent stopped (awaiting input)"
                        );
                    }
                }

                // Capture content for pattern detection and change tracking
                // Dispatches to correct wrapper based on session_wrapper field
                if let Some(content) = self.capture_agent_content(
                    &session_name,
                    session_wrapper.as_deref(),
                    session_context_ref.as_deref(),
                ) {
                    let hash = hash_content(&content);
                    let content_changed = state
                        .update_agent_content_hash(&agent_id, &hash)
                        .unwrap_or(false);

                    if content_changed {
                        result.changed.push(session_name.clone());
                        let _ = state.record_content_change(&agent_id);
                        tracing::debug!(
                            agent_id = %agent_id,
                            session = %session_name,
                            "Session content changed"
                        );

                        // RESUME DETECTION: If agent was awaiting and content changed, it resumed
                        if agent_status == "awaiting_input" {
                            result.resumed.push(session_name.clone());
                            tracing::info!(
                                agent_id = %agent_id,
                                session = %session_name,
                                "Agent resumed from awaiting state (content changed)"
                            );
                            // Clear any hook signal since agent is now active
                            let _ = self.hook_manager.clear_signal(&agent_id);
                            continue; // Skip awaiting detection for this agent
                        }
                    }

                    // 2. Pattern-based idle detection (if not already detected via hook)
                    if !detected_awaiting {
                        if let Some(ref tool_name) = llm_tool {
                            if self.idle_detector.is_idle(tool_name, &content) {
                                detected_awaiting = true;
                                result.awaiting_input.push(session_name.clone());
                                tracing::info!(
                                    agent_id = %agent_id,
                                    session = %session_name,
                                    tool = %tool_name,
                                    "Pattern detection: agent is idle (awaiting input)"
                                );
                            }
                        }
                    }
                }

                // 3. Fallback: Silence flag check (tmux only — cmux/zellij don't have silence monitoring)
                if !detected_awaiting && !is_cmux && !is_zellij {
                    if let Ok(is_silent) = self.tmux.check_silence_flag(&session_name) {
                        if is_silent {
                            result.awaiting_input.push(session_name.clone());
                            tracing::info!(
                                agent_id = %agent_id,
                                session = %session_name,
                                "Silence flag: agent is silent (awaiting input)"
                            );
                        }
                    }
                }

                // Check artifacts for positive completion signal
                if detected_awaiting {
                    if let Some((worktree, patterns)) = artifact_context.get(&agent_id) {
                        if !patterns.is_empty()
                            && self.artifact_detector.check_artifacts(worktree, patterns)
                                == ArtifactStatus::Ready
                        {
                            result.artifact_ready.push(session_name.clone());
                            tracing::info!(
                                agent_id = %agent_id,
                                session = %session_name,
                                "Artifact detection: all expected outputs found"
                            );
                        }
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
    pub fn force_check(
        &mut self,
        artifact_context: &HashMap<String, (PathBuf, Vec<String>)>,
    ) -> Result<HealthCheckResult> {
        self.check_health(artifact_context)
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

    /// Detect orphan tmux sessions (op-* sessions with no matching agent in state).
    ///
    /// Returns a list of `OrphanSession` structs representing tmux sessions that
    /// have the operator prefix but are not tracked by any agent in state.
    /// Unlike `reconcile_on_startup`, this does not modify state - it's purely
    /// for display purposes.
    pub fn detect_orphan_sessions(&self) -> Result<Vec<OrphanSession>> {
        let state = State::load(&self.config)?;

        // Get all op-* sessions from tmux
        let active_sessions = self.tmux.list_sessions(Some("op-")).unwrap_or_default();

        // Get session names from tracked agents (excluding orphaned agents)
        let known_sessions: HashSet<String> = state
            .agents_with_sessions()
            .iter()
            .filter_map(|a| a.session_name.clone())
            .collect();

        // Return sessions that exist in tmux but have no matching agent
        let orphans: Vec<OrphanSession> = active_sessions
            .into_iter()
            .filter(|s| !known_sessions.contains(&s.name))
            .map(|s| OrphanSession {
                session_name: s.name,
                created: s.created,
                attached: s.attached,
            })
            .collect();

        Ok(orphans)
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

        let mut config = Config {
            paths: PathsConfig {
                tickets: tickets_path.to_string_lossy().to_string(),
                projects: projects_path.to_string_lossy().to_string(),
                state: state_path.to_string_lossy().to_string(),
                worktrees: state_path.join("worktrees").to_string_lossy().to_string(),
            },
            ..Default::default()
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
        let result = monitor.check_health(&HashMap::new()).unwrap();

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
        let result = monitor.check_health(&HashMap::new()).unwrap();

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
        let result = monitor.check_health(&HashMap::new()).unwrap();

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
        let result = monitor.check_health(&HashMap::new()).unwrap();

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

    #[test]
    fn test_detect_orphan_sessions_none_when_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // No agents, no sessions
        let mock = Arc::new(MockTmuxClient::new());

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let orphans = monitor.detect_orphan_sessions().unwrap();

        assert!(orphans.is_empty());
    }

    #[test]
    fn test_detect_orphan_sessions_finds_orphans() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // No agents in state, but sessions exist in tmux
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-ORPHAN-001", "/tmp");
        mock.add_session("op-ORPHAN-002", "/tmp");
        mock.add_session("other-session", "/tmp"); // Non-operator session, should be ignored

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let orphans = monitor.detect_orphan_sessions().unwrap();

        // Should find only the op-* sessions (2 orphans)
        assert_eq!(orphans.len(), 2);
        assert!(orphans.iter().any(|o| o.session_name == "op-ORPHAN-001"));
        assert!(orphans.iter().any(|o| o.session_name == "op-ORPHAN-002"));
    }

    #[test]
    fn test_detect_orphan_sessions_none_when_matched() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a matching session
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "FEAT-100".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-100")
            .unwrap();

        // Mock has the matching session
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-FEAT-100", "/tmp");

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let orphans = monitor.detect_orphan_sessions().unwrap();

        // No orphans - session matches agent
        assert!(orphans.is_empty());
    }

    #[test]
    fn test_detect_orphan_sessions_ignores_non_operator_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Only non-operator sessions in tmux
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("my-personal-session", "/tmp");
        mock.add_session("work-stuff", "/tmp");

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let orphans = monitor.detect_orphan_sessions().unwrap();

        // No orphans - only op-* prefix sessions are tracked
        assert!(orphans.is_empty());
    }

    #[test]
    fn test_detect_orphan_sessions_mixed() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create an agent with a session
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "FEAT-200".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-200")
            .unwrap();

        // Mock has the matching session plus an orphan
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-FEAT-200", "/tmp"); // Matched
        mock.add_session("op-ORPHAN-777", "/tmp"); // Orphan

        let monitor = SessionMonitor::with_tmux_client(&config, mock);
        let orphans = monitor.detect_orphan_sessions().unwrap();

        // Only the unmatched session is an orphan
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].session_name, "op-ORPHAN-777");
    }

    /// Helper: write a hook signal file to trigger idle detection in check_health
    fn write_hook_signal(agent_id: &str) -> PathBuf {
        use crate::agents::hooks::HookSignal;

        let signal_dir = PathBuf::from("/tmp/operator-signals");
        std::fs::create_dir_all(&signal_dir).unwrap();
        let signal = HookSignal {
            event: "stop".to_string(),
            timestamp: 1234567890,
            session_id: agent_id.to_string(),
        };
        let signal_path = signal_dir.join(format!("{}.signal", agent_id));
        std::fs::write(&signal_path, serde_json::to_string(&signal).unwrap()).unwrap();
        signal_path
    }

    #[test]
    fn test_health_check_artifact_ready_when_idle_and_artifacts_exist() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create agent with session
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "FEAT-ART-1".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-ART-1")
            .unwrap();

        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-FEAT-ART-1", "/tmp");
        mock.set_session_content("op-FEAT-ART-1", "Working...");

        // Write hook signal to trigger idle detection
        let signal_path = write_hook_signal(&agent_id);

        // Create artifact files in a temp worktree
        let worktree = TempDir::new().unwrap();
        let plans_dir = worktree.path().join(".tickets").join("plans");
        std::fs::create_dir_all(&plans_dir).unwrap();
        std::fs::write(plans_dir.join("FEAT-ART-1.md"), "# Plan").unwrap();

        // Build artifact context
        let mut artifact_context: HashMap<String, (PathBuf, Vec<String>)> = HashMap::new();
        artifact_context.insert(
            agent_id.clone(),
            (
                worktree.path().to_path_buf(),
                vec![".tickets/plans/*.md".to_string()],
            ),
        );

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health(&artifact_context).unwrap();

        // Cleanup
        let _ = std::fs::remove_file(&signal_path);

        // Agent should be detected as idle AND artifacts found
        assert!(
            result.awaiting_input.contains(&"op-FEAT-ART-1".to_string()),
            "Session should be in awaiting_input"
        );
        assert!(
            result.artifact_ready.contains(&"op-FEAT-ART-1".to_string()),
            "Session should be in artifact_ready"
        );
    }

    #[test]
    fn test_health_check_no_artifact_ready_when_idle_but_artifacts_missing() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "FEAT-ART-2".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-ART-2")
            .unwrap();

        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-FEAT-ART-2", "/tmp");
        mock.set_session_content("op-FEAT-ART-2", "Working...");

        // Write hook signal to trigger idle detection
        let signal_path = write_hook_signal(&agent_id);

        // Empty worktree — no artifact files
        let worktree = TempDir::new().unwrap();

        let mut artifact_context: HashMap<String, (PathBuf, Vec<String>)> = HashMap::new();
        artifact_context.insert(
            agent_id.clone(),
            (
                worktree.path().to_path_buf(),
                vec![".tickets/plans/*.md".to_string()],
            ),
        );

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health(&artifact_context).unwrap();

        let _ = std::fs::remove_file(&signal_path);

        // Agent is idle but artifacts are missing
        assert!(
            result.awaiting_input.contains(&"op-FEAT-ART-2".to_string()),
            "Session should be in awaiting_input"
        );
        assert!(
            result.artifact_ready.is_empty(),
            "artifact_ready should be empty when artifacts are missing"
        );
    }

    #[test]
    fn test_health_check_no_artifact_check_when_not_idle() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "FEAT-ART-3".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-ART-3")
            .unwrap();

        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-FEAT-ART-3", "/tmp");
        mock.set_session_content("op-FEAT-ART-3", "Actively working...");
        // No hook signal, no silence flag — agent is NOT idle

        // Artifacts exist but agent isn't idle
        let worktree = TempDir::new().unwrap();
        let plans_dir = worktree.path().join(".tickets").join("plans");
        std::fs::create_dir_all(&plans_dir).unwrap();
        std::fs::write(plans_dir.join("FEAT-ART-3.md"), "# Plan").unwrap();

        let mut artifact_context: HashMap<String, (PathBuf, Vec<String>)> = HashMap::new();
        artifact_context.insert(
            agent_id.clone(),
            (
                worktree.path().to_path_buf(),
                vec![".tickets/plans/*.md".to_string()],
            ),
        );

        let mut monitor = SessionMonitor::with_tmux_client(&config, mock);
        let result = monitor.check_health(&artifact_context).unwrap();

        // Not idle — artifacts should not be checked
        assert!(
            result.awaiting_input.is_empty(),
            "Session should NOT be in awaiting_input"
        );
        assert!(
            result.artifact_ready.is_empty(),
            "artifact_ready should be empty when agent is not idle"
        );
    }
}
