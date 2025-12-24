#![allow(dead_code)]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub paused: bool,
    pub agents: Vec<AgentState>,
    pub completed: Vec<CompletedTicket>,

    #[serde(skip)]
    state_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub ticket_id: String,
    pub ticket_type: String,
    pub project: String,
    pub status: String, // "running", "awaiting_input", "completing", "orphaned"
    pub started_at: DateTime<Utc>,
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
    pub step_started_at: Option<DateTime<Utc>>,
    /// Last time content changed in the session (for hung detection)
    #[serde(default)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTicket {
    pub ticket_id: String,
    pub ticket_type: String,
    pub project: String,
    pub summary: String,
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
}
