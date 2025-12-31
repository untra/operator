#![allow(dead_code)] // Active module - core state management, some fields reserved for future persistence

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use ts_rs::TS;
use uuid::Uuid;

use crate::config::Config;
use crate::types::llm_stats::ProjectLlmStats;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct State {
    pub paused: bool,
    pub agents: Vec<AgentState>,
    pub completed: Vec<CompletedTicket>,

    /// Per-project LLM usage statistics
    #[serde(default)]
    pub project_llm_stats: HashMap<String, ProjectLlmStats>,

    /// Per-project issue type collection preferences (project_name -> collection_name)
    #[serde(default)]
    pub project_collection_prefs: HashMap<String, String>,

    #[serde(skip)]
    #[ts(skip)]
    state_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct AgentState {
    pub id: String,
    pub ticket_id: String,
    pub ticket_type: String,
    pub project: String,
    pub status: String, // "running", "awaiting_input", "completing", "orphaned"
    #[ts(type = "string")]
    pub started_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub last_activity: DateTime<Utc>,
    pub last_message: Option<String>,
    pub paired: bool, // true for SPIKE/INV
    /// The tmux session name for this agent (for recovery)
    #[serde(default)]
    pub session_name: Option<String>,
    /// Hash of the last captured pane content (for change detection)
    #[serde(default)]
    pub content_hash: Option<String>,
    /// Current step in the ticket workflow (e.g., "plan", "implement", "test")
    #[serde(default)]
    pub current_step: Option<String>,
    /// When the current step started (for timeout detection)
    #[serde(default)]
    #[ts(type = "string | null")]
    pub step_started_at: Option<DateTime<Utc>>,
    /// Last time content changed in the session (for hung detection)
    #[serde(default)]
    #[ts(type = "string | null")]
    pub last_content_change: Option<DateTime<Utc>>,
    /// PR URL if created during "pr" step
    #[serde(default)]
    pub pr_url: Option<String>,
    /// PR number for GitHub API tracking
    #[serde(default)]
    pub pr_number: Option<u64>,
    /// GitHub repo in format "owner/repo"
    #[serde(default)]
    pub github_repo: Option<String>,
    /// Last known PR status ("open", "approved", "changes_requested", "merged", "closed")
    #[serde(default)]
    pub pr_status: Option<String>,
    /// Completed steps for this ticket
    #[serde(default)]
    pub completed_steps: Vec<String>,
    /// LLM tool used (e.g., "claude", "gemini", "codex")
    #[serde(default)]
    pub llm_tool: Option<String>,
    /// Launch mode: "default", "yolo", "docker", "docker-yolo"
    #[serde(default)]
    pub launch_mode: Option<String>,
    /// Review state for awaiting_input agents
    /// Values: "pending_plan", "pending_visual", "pending_pr_creation", "pending_pr_merge"
    #[serde(default)]
    pub review_state: Option<String>,
    /// Server process ID for visual review cleanup (if applicable)
    #[serde(default)]
    pub dev_server_pid: Option<u32>,
    /// Path to the git worktree for this ticket (per-ticket isolation)
    #[serde(default)]
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct CompletedTicket {
    pub ticket_id: String,
    pub ticket_type: String,
    pub project: String,
    pub summary: String,
    #[ts(type = "string")]
    pub completed_at: DateTime<Utc>,
    pub pr_url: Option<String>,
    pub output_tickets: Vec<String>,
}

/// Represents a tmux session with op-* prefix that has no matching agent in state.
/// These are "orphan" sessions that exist but are not tracked.
#[derive(Debug, Clone)]
pub struct OrphanSession {
    pub session_name: String,
    pub created: Option<String>,
    pub attached: bool,
}

impl State {
    pub fn load(config: &Config) -> Result<Self> {
        let state_path = config.state_path();
        fs::create_dir_all(&state_path).context("Failed to create state directory")?;

        let state_file = state_path.join("state.json");

        if state_file.exists() {
            let contents = fs::read_to_string(&state_file).context("Failed to read state file")?;
            let mut state: State =
                serde_json::from_str(&contents).context("Failed to parse state file")?;
            state.state_path = state_path;
            Ok(state)
        } else {
            Ok(Self {
                paused: false,
                agents: Vec::new(),
                completed: Vec::new(),
                project_llm_stats: HashMap::new(),
                project_collection_prefs: HashMap::new(),
                state_path,
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let state_file = self.state_path.join("state.json");
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(state_file, contents)?;
        Ok(())
    }

    pub fn running_agents(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| a.status == "running" || a.status == "awaiting_input")
            .collect()
    }

    pub fn stalled_agents(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| a.status == "awaiting_input")
            .collect()
    }

    pub fn set_paused(&mut self, paused: bool) -> Result<()> {
        self.paused = paused;
        self.save()
    }

    pub fn add_agent(
        &mut self,
        ticket_id: String,
        ticket_type: String,
        project: String,
        paired: bool,
    ) -> Result<String> {
        self.add_agent_with_options(ticket_id, ticket_type, project, paired, None, None)
    }

    /// Add an agent with launch options (llm_tool and launch_mode)
    pub fn add_agent_with_options(
        &mut self,
        ticket_id: String,
        ticket_type: String,
        project: String,
        paired: bool,
        llm_tool: Option<String>,
        launch_mode: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        self.agents.push(AgentState {
            id: id.clone(),
            ticket_id,
            ticket_type,
            project,
            status: "running".to_string(),
            started_at: now,
            last_activity: now,
            last_message: None,
            paired,
            session_name: None,
            content_hash: None,
            current_step: None,
            step_started_at: None,
            last_content_change: Some(now),
            pr_url: None,
            pr_number: None,
            github_repo: None,
            pr_status: None,
            completed_steps: Vec::new(),
            llm_tool,
            launch_mode,
            review_state: None,
            dev_server_pid: None,
            worktree_path: None,
        });

        self.save()?;
        Ok(id)
    }

    pub fn update_agent_status(
        &mut self,
        agent_id: &str,
        status: &str,
        message: Option<String>,
    ) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.status = status.to_string();
            agent.last_activity = Utc::now();
            if message.is_some() {
                agent.last_message = message;
            }
        }
        self.save()
    }

    pub fn complete_agent(
        &mut self,
        agent_id: &str,
        summary: String,
        pr_url: Option<String>,
        output_tickets: Vec<String>,
    ) -> Result<()> {
        if let Some(pos) = self.agents.iter().position(|a| a.id == agent_id) {
            let agent = self.agents.remove(pos);

            self.completed.push(CompletedTicket {
                ticket_id: agent.ticket_id,
                ticket_type: agent.ticket_type,
                project: agent.project,
                summary,
                completed_at: Utc::now(),
                pr_url,
                output_tickets,
            });

            // Keep only recent completions (last 100)
            if self.completed.len() > 100 {
                self.completed.remove(0);
            }
        }
        self.save()
    }

    pub fn remove_agent(&mut self, agent_id: &str) -> Result<()> {
        self.agents.retain(|a| a.id != agent_id);
        self.save()
    }

    pub fn recent_completions(&self, hours: u64) -> Vec<&CompletedTicket> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        self.completed
            .iter()
            .filter(|c| c.completed_at > cutoff)
            .collect()
    }

    pub fn is_project_busy(&self, project: &str) -> bool {
        self.agents
            .iter()
            .any(|a| a.project == project && a.status == "running")
    }

    /// Update the tmux session name for an agent
    pub fn update_agent_session(&mut self, agent_id: &str, session_name: &str) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.session_name = Some(session_name.to_string());
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Update the worktree path for an agent (for per-ticket isolation)
    pub fn update_agent_worktree_path(
        &mut self,
        agent_id: &str,
        worktree_path: &str,
    ) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.worktree_path = Some(worktree_path.to_string());
        }
        self.save()
    }

    /// Update the content hash for an agent (for change detection)
    pub fn update_agent_content_hash(&mut self, agent_id: &str, hash: &str) -> Result<bool> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            let changed = agent.content_hash.as_ref() != Some(&hash.to_string());
            if changed {
                agent.content_hash = Some(hash.to_string());
                agent.last_activity = Utc::now();
                self.save()?;
            }
            return Ok(changed);
        }
        Ok(false)
    }

    /// Get an agent by its tmux session name
    pub fn agent_by_session(&self, session_name: &str) -> Option<&AgentState> {
        self.agents
            .iter()
            .find(|a| a.session_name.as_ref() == Some(&session_name.to_string()))
    }

    /// Get a mutable agent by its tmux session name
    pub fn agent_by_session_mut(&mut self, session_name: &str) -> Option<&mut AgentState> {
        self.agents
            .iter_mut()
            .find(|a| a.session_name.as_ref() == Some(&session_name.to_string()))
    }

    /// Mark an agent as orphaned (session died unexpectedly)
    pub fn mark_agent_orphaned(&mut self, agent_id: &str) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.status = "orphaned".to_string();
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Get all agents that have session names (for health checking)
    pub fn agents_with_sessions(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| a.session_name.is_some() && a.status != "orphaned")
            .collect()
    }

    /// Get all orphaned agents
    pub fn orphaned_agents(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| a.status == "orphaned")
            .collect()
    }

    /// Remove agent by tmux session name (for session recovery cleanup)
    pub fn remove_agent_by_session(&mut self, session_name: &str) -> Result<Option<AgentState>> {
        let pos = self
            .agents
            .iter()
            .position(|a| a.session_name.as_ref() == Some(&session_name.to_string()));

        if let Some(pos) = pos {
            let agent = self.agents.remove(pos);
            self.save()?;
            return Ok(Some(agent));
        }
        Ok(None)
    }

    /// Update the current step for an agent (resets step timer)
    pub fn update_agent_step(&mut self, agent_id: &str, step: &str) -> Result<()> {
        let now = Utc::now();
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.current_step = Some(step.to_string());
            agent.step_started_at = Some(now);
            agent.last_activity = now;
            agent.last_content_change = Some(now);
        }
        self.save()
    }

    /// Record that content changed in the session (updates last_content_change)
    pub fn record_content_change(&mut self, agent_id: &str) -> Result<()> {
        let now = Utc::now();
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.last_content_change = Some(now);
            agent.last_activity = now;
        }
        self.save()
    }

    /// Check if an agent's step has timed out
    pub fn is_step_timed_out(&self, agent_id: &str, timeout_secs: u64) -> bool {
        if let Some(agent) = self.agents.iter().find(|a| a.id == agent_id) {
            if let Some(step_started) = agent.step_started_at {
                let elapsed = Utc::now().signed_duration_since(step_started);
                return elapsed.num_seconds() > timeout_secs as i64;
            }
        }
        false
    }

    /// Get an agent by its ticket ID
    pub fn agent_by_ticket(&self, ticket_id: &str) -> Option<&AgentState> {
        self.agents.iter().find(|a| a.ticket_id == ticket_id)
    }

    /// Get a mutable agent by its ticket ID
    pub fn agent_by_ticket_mut(&mut self, ticket_id: &str) -> Option<&mut AgentState> {
        self.agents.iter_mut().find(|a| a.ticket_id == ticket_id)
    }

    /// Record step completion and add to completed_steps list
    pub fn complete_step(&mut self, agent_id: &str, step: &str) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            if !agent.completed_steps.contains(&step.to_string()) {
                agent.completed_steps.push(step.to_string());
            }
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Update PR information for an agent
    pub fn update_agent_pr(
        &mut self,
        agent_id: &str,
        pr_url: &str,
        pr_number: u64,
        github_repo: &str,
    ) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.pr_url = Some(pr_url.to_string());
            agent.pr_number = Some(pr_number);
            agent.github_repo = Some(github_repo.to_string());
            agent.pr_status = Some("open".to_string());
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Update PR status for an agent
    pub fn update_pr_status(&mut self, agent_id: &str, status: &str) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.pr_status = Some(status.to_string());
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Get all agents that are waiting for PR approval
    pub fn agents_awaiting_pr_approval(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| {
                a.pr_number.is_some()
                    && a.pr_status
                        .as_ref()
                        .map(|s| s == "open" || s == "pending")
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Get all agents with active PRs (for status polling)
    pub fn agents_with_prs(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| a.pr_number.is_some() && a.github_repo.is_some())
            .collect()
    }

    /// Set the review state for an agent (used when entering awaiting_input with a review type)
    pub fn set_agent_review_state(&mut self, agent_id: &str, review_state: &str) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.review_state = Some(review_state.to_string());
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Clear the review state for an agent (used when resuming from awaiting_input)
    pub fn clear_review_state(&mut self, agent_id: &str) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.review_state = None;
            agent.dev_server_pid = None;
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Set the dev server PID for visual review cleanup
    pub fn set_agent_dev_server_pid(&mut self, agent_id: &str, pid: u32) -> Result<()> {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.dev_server_pid = Some(pid);
            agent.last_activity = Utc::now();
        }
        self.save()
    }

    /// Get all agents awaiting plan review
    pub fn agents_awaiting_plan_review(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| {
                a.status == "awaiting_input"
                    && a.review_state
                        .as_ref()
                        .map(|s| s == "pending_plan")
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Get all agents awaiting visual review
    pub fn agents_awaiting_visual_review(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| {
                a.status == "awaiting_input"
                    && a.review_state
                        .as_ref()
                        .map(|s| s == "pending_visual")
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Get all agents awaiting PR merge
    pub fn agents_awaiting_pr_merge(&self) -> Vec<&AgentState> {
        self.agents
            .iter()
            .filter(|a| {
                a.status == "awaiting_input"
                    && a.review_state
                        .as_ref()
                        .map(|s| s == "pending_pr_merge")
                        .unwrap_or(false)
            })
            .collect()
    }

    // ─── LLM Stats Methods ────────────────────────────────────────────────────

    /// Complete an agent and record LLM usage statistics
    pub fn complete_agent_with_stats(
        &mut self,
        agent_id: &str,
        summary: String,
        pr_url: Option<String>,
        output_tickets: Vec<String>,
        success: bool,
    ) -> Result<()> {
        if let Some(pos) = self.agents.iter().position(|a| a.id == agent_id) {
            let agent = &self.agents[pos];

            // Calculate duration
            let duration_secs = (Utc::now() - agent.started_at).num_seconds() as u64;

            // Update LLM stats
            if let Some(ref tool) = agent.llm_tool {
                // Extract model from launch_mode or use "default"
                let model = agent.launch_mode.as_deref().unwrap_or("default");

                let stats = self
                    .project_llm_stats
                    .entry(agent.project.clone())
                    .or_insert_with(|| ProjectLlmStats::new(&agent.project));

                stats.record_completion(tool, model, success, duration_secs);
            }

            // Continue with normal completion
            let agent = self.agents.remove(pos);
            self.completed.push(CompletedTicket {
                ticket_id: agent.ticket_id,
                ticket_type: agent.ticket_type,
                project: agent.project,
                summary,
                completed_at: Utc::now(),
                pr_url,
                output_tickets,
            });

            if self.completed.len() > 100 {
                self.completed.remove(0);
            }
        }
        self.save()
    }

    /// Get LLM stats for a project
    pub fn get_project_llm_stats(&self, project: &str) -> Option<&ProjectLlmStats> {
        self.project_llm_stats.get(project)
    }

    /// Get mutable LLM stats for a project
    pub fn get_project_llm_stats_mut(&mut self, project: &str) -> Option<&mut ProjectLlmStats> {
        self.project_llm_stats.get_mut(project)
    }

    /// Set preferred LLM for a project
    pub fn set_project_preferred_llm(
        &mut self,
        project: &str,
        tool: Option<String>,
        model: Option<String>,
    ) -> Result<()> {
        let stats = self
            .project_llm_stats
            .entry(project.to_string())
            .or_insert_with(|| ProjectLlmStats::new(project));

        stats.preferred_tool = tool;
        stats.preferred_model = model;
        stats.updated_at = Utc::now();

        self.save()
    }

    /// Get preferred LLM tool for a project (user override or most used)
    pub fn get_preferred_llm_tool(&self, project: &str) -> Option<&str> {
        self.project_llm_stats.get(project).and_then(|stats| {
            stats
                .preferred_tool
                .as_deref()
                .or_else(|| stats.most_used_tool())
        })
    }

    /// Get preferred model for a project and tool
    pub fn get_preferred_model(&self, project: &str, tool: &str) -> Option<&str> {
        self.project_llm_stats.get(project).and_then(|stats| {
            stats
                .preferred_model
                .as_deref()
                .or_else(|| stats.most_used_model(tool))
        })
    }

    /// Get all projects with LLM stats
    pub fn projects_with_stats(&self) -> Vec<&str> {
        self.project_llm_stats.keys().map(|s| s.as_str()).collect()
    }

    // ─── Collection Preferences Methods ──────────────────────────────────────────

    /// Get the preferred collection for a project
    pub fn get_project_collection(&self, project: &str) -> Option<&str> {
        self.project_collection_prefs
            .get(project)
            .map(|s| s.as_str())
    }

    /// Set the preferred collection for a project
    pub fn set_project_collection(&mut self, project: &str, collection: &str) -> Result<()> {
        self.project_collection_prefs
            .insert(project.to_string(), collection.to_string());
        self.save()
    }

    /// Clear the project collection preference (use global default)
    pub fn clear_project_collection(&mut self, project: &str) -> Result<()> {
        self.project_collection_prefs.remove(project);
        self.save()
    }

    /// Get all projects with collection preferences
    pub fn projects_with_collection_prefs(&self) -> Vec<(&str, &str)> {
        self.project_collection_prefs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(temp_dir: &TempDir) -> Config {
        let state_path = temp_dir.path().to_path_buf();
        let mut config = Config::default();
        config.paths.state = state_path.to_string_lossy().to_string();
        config
    }

    // ─── Load/Save Tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_state_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        let state = State::load(&config).unwrap();

        assert!(!state.paused);
        assert!(state.agents.is_empty());
        assert!(state.completed.is_empty());
        assert!(state.project_llm_stats.is_empty());
    }

    #[test]
    fn test_state_load_corrupted_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Create a corrupted state file
        let state_path = temp_dir.path().join("state.json");
        fs::write(&state_path, "{ invalid json }").unwrap();

        let result = State::load(&config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to parse state file"));
    }

    #[test]
    fn test_state_load_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Create an empty state file
        let state_path = temp_dir.path().join("state.json");
        fs::write(&state_path, "").unwrap();

        let result = State::load(&config);

        assert!(result.is_err());
    }

    #[test]
    fn test_state_save_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Create state with some data
        let mut state = State::load(&config).unwrap();
        state.paused = true;
        state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        // Save and reload
        let state2 = State::load(&config).unwrap();

        assert!(state2.paused);
        assert_eq!(state2.agents.len(), 1);
        assert_eq!(state2.agents[0].ticket_id, "FEAT-001");
    }

    // ─── Agent Add/Remove Tests ──────────────────────────────────────────────────

    #[test]
    fn test_add_agent_generates_uuid() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        // UUID v4 format: 8-4-4-4-12 hex chars
        assert_eq!(id.len(), 36);
        assert!(id.chars().filter(|c| *c == '-').count() == 4);
    }

    #[test]
    fn test_add_agent_saves_immediately() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        // Read file directly to verify it was saved
        let state_file = temp_dir.path().join("state.json");
        let contents = fs::read_to_string(&state_file).unwrap();
        assert!(contents.contains("FEAT-001"));
    }

    #[test]
    fn test_add_agent_with_options() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        state
            .add_agent_with_options(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
                Some("claude".to_string()),
                Some("yolo".to_string()),
            )
            .unwrap();

        assert_eq!(state.agents[0].llm_tool, Some("claude".to_string()));
        assert_eq!(state.agents[0].launch_mode, Some("yolo".to_string()));
    }

    #[test]
    fn test_remove_agent_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        assert_eq!(state.agents.len(), 1);

        state.remove_agent(&id).unwrap();

        assert!(state.agents.is_empty());
    }

    #[test]
    fn test_remove_agent_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        // Should not error on nonexistent agent
        let result = state.remove_agent("nonexistent-id");
        assert!(result.is_ok());
    }

    // ─── Agent Status Tests ──────────────────────────────────────────────────────

    #[test]
    fn test_update_agent_status() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        let original_activity = state.agents[0].last_activity;

        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(10));

        state
            .update_agent_status(&id, "awaiting_input", Some("Needs review".to_string()))
            .unwrap();

        assert_eq!(state.agents[0].status, "awaiting_input");
        assert_eq!(
            state.agents[0].last_message,
            Some("Needs review".to_string())
        );
        assert!(state.agents[0].last_activity > original_activity);
    }

    #[test]
    fn test_update_agent_status_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        // Should not error on nonexistent agent
        let result = state.update_agent_status("nonexistent-id", "running", None);
        assert!(result.is_ok());
    }

    // ─── Agent Query Tests ───────────────────────────────────────────────────────

    #[test]
    fn test_agent_by_ticket_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        let agent = state.agent_by_ticket("FEAT-001");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().ticket_type, "FEAT");
    }

    #[test]
    fn test_agent_by_ticket_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let state = State::load(&config).unwrap();

        let agent = state.agent_by_ticket("NONEXISTENT");
        assert!(agent.is_none());
    }

    #[test]
    fn test_is_project_busy_running() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        // Agent starts with status "running"
        assert!(state.is_project_busy("test-project"));
        assert!(!state.is_project_busy("other-project"));
    }

    #[test]
    fn test_is_project_busy_awaiting_input() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        state
            .update_agent_status(&id, "awaiting_input", None)
            .unwrap();

        // is_project_busy only checks for "running" status
        assert!(!state.is_project_busy("test-project"));
    }

    // ─── Step Completion Tests ───────────────────────────────────────────────────

    #[test]
    fn test_complete_step_adds_to_list() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        state.complete_step(&id, "plan").unwrap();

        assert_eq!(state.agents[0].completed_steps, vec!["plan".to_string()]);
    }

    #[test]
    fn test_complete_step_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        state.complete_step(&id, "plan").unwrap();
        state.complete_step(&id, "plan").unwrap(); // Duplicate

        // Should only have one entry
        assert_eq!(state.agents[0].completed_steps.len(), 1);
    }

    // ─── Agent Completion Tests ──────────────────────────────────────────────────

    #[test]
    fn test_complete_agent_moves_to_completed() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        let id = state
            .add_agent(
                "FEAT-001".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();

        state
            .complete_agent(
                &id,
                "Completed successfully".to_string(),
                Some("https://github.com/test/pr/1".to_string()),
                vec![],
            )
            .unwrap();

        assert!(state.agents.is_empty());
        assert_eq!(state.completed.len(), 1);
        assert_eq!(state.completed[0].ticket_id, "FEAT-001");
        assert_eq!(state.completed[0].summary, "Completed successfully");
        assert_eq!(
            state.completed[0].pr_url,
            Some("https://github.com/test/pr/1".to_string())
        );
    }

    #[test]
    fn test_completed_tickets_fifo_eviction() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let mut state = State::load(&config).unwrap();

        // Add and complete 101 agents
        for i in 0..101 {
            let id = state
                .add_agent(
                    format!("FEAT-{:03}", i),
                    "FEAT".to_string(),
                    "test".to_string(),
                    false,
                )
                .unwrap();

            state
                .complete_agent(&id, format!("Summary {}", i), None, vec![])
                .unwrap();
        }

        // Should have max 100 completed tickets
        assert_eq!(state.completed.len(), 100);
        // First ticket (FEAT-000) should be evicted
        assert_eq!(state.completed[0].ticket_id, "FEAT-001");
        // Last ticket should be present
        assert_eq!(state.completed[99].ticket_id, "FEAT-100");
    }
}
