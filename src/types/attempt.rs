//! Step execution types - unified from vibe-kanban TaskAttempt + Operator AgentState
//!
//! In Operator's model:
//! - A Ticket has an IssueType which defines Steps
//! - Each Step can have multiple StepAttempts (retries, different approaches)
//! - Each StepAttempt runs in an isolated git worktree with its own branch
//! - Each StepAttempt contains Sessions (conversational continuity)
//! - Each Session spawns ExecutionProcesses (setup, agent, cleanup)

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;
use uuid::Uuid;

/// An isolated execution attempt for a workflow step
///
/// This is the Operator equivalent of vibe-kanban's TaskAttempt/Workspace.
/// Each attempt runs in its own git worktree with a dedicated branch,
/// allowing for retry/reset without affecting the main codebase.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct StepAttempt {
    /// Unique identifier for this attempt
    pub id: Uuid,

    /// Parent ticket ID (e.g., "FEAT-1234")
    pub ticket_id: String,

    /// Which step this attempt is executing (e.g., "plan", "implement", "test")
    pub step_name: String,

    /// Project name this attempt belongs to
    pub project: String,

    // ─────────────────────────────────────────────────────────────────────
    // Git isolation (from vibe-kanban TaskAttempt)
    // ─────────────────────────────────────────────────────────────────────
    /// Git branch for this attempt (e.g., "feat/abc123-add-login")
    pub branch: String,

    /// Target branch to merge into (e.g., "main")
    pub target_branch: String,

    /// Path to isolated git worktree (if using worktree isolation)
    #[serde(default)]
    #[ts(type = "string | null")]
    pub worktree_path: Option<PathBuf>,

    /// Whether the worktree has been cleaned up
    #[serde(default)]
    pub worktree_deleted: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Execution configuration
    // ─────────────────────────────────────────────────────────────────────
    /// Which LLM tool is executing (e.g., "claude", "gemini", "codex")
    pub executor: String,

    /// Executor model override (e.g., "claude-sonnet-4-20250514")
    #[serde(default)]
    pub executor_model: Option<String>,

    /// Current session ID for conversational continuity
    #[serde(default)]
    pub session_id: Option<Uuid>,

    /// Launch mode: "default", "yolo", "docker", "docker-yolo"
    #[serde(default)]
    pub launch_mode: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // Status tracking
    // ─────────────────────────────────────────────────────────────────────
    /// Current status of this attempt
    pub status: AttemptStatus,

    /// Whether this attempt requires human pairing (SPIKE/INV modes)
    #[serde(default)]
    pub paired: bool,

    /// Tmux session name (for Operator's terminal-based execution)
    #[serde(default)]
    pub tmux_session: Option<String>,

    /// Hash of last captured terminal content (for change detection)
    #[serde(default)]
    pub content_hash: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // PR tracking
    // ─────────────────────────────────────────────────────────────────────
    /// Pull request URL if created
    #[serde(default)]
    pub pr_url: Option<String>,

    /// PR number for API tracking
    #[serde(default)]
    pub pr_number: Option<u64>,

    /// GitHub repo in format "owner/repo"
    #[serde(default)]
    pub github_repo: Option<String>,

    /// Current PR status ("open", "approved", "changes_requested", "merged", "closed")
    #[serde(default)]
    pub pr_status: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // Timestamps
    // ─────────────────────────────────────────────────────────────────────
    /// When setup completed (worktree created, scripts run)
    #[serde(default)]
    #[ts(type = "string | null")]
    pub setup_completed_at: Option<DateTime<Utc>>,

    /// Last activity timestamp
    #[ts(type = "string")]
    pub last_activity: DateTime<Utc>,

    /// Last time terminal content changed (for hung detection)
    #[serde(default)]
    #[ts(type = "string | null")]
    pub last_content_change: Option<DateTime<Utc>>,

    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Status of a step attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum AttemptStatus {
    /// Created but not started
    #[default]
    Pending,
    /// Running setup script
    Setting,
    /// Agent actively executing
    Running,
    /// Awaiting human input (for paired modes)
    AwaitingInput,
    /// Completed, awaiting review
    InReview,
    /// Successfully finished
    Completed,
    /// Error occurred
    Failed,
    /// User cancelled
    Cancelled,
    /// Session died unexpectedly
    Orphaned,
}

impl std::fmt::Display for AttemptStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttemptStatus::Pending => write!(f, "pending"),
            AttemptStatus::Setting => write!(f, "setting"),
            AttemptStatus::Running => write!(f, "running"),
            AttemptStatus::AwaitingInput => write!(f, "awaiting_input"),
            AttemptStatus::InReview => write!(f, "in_review"),
            AttemptStatus::Completed => write!(f, "completed"),
            AttemptStatus::Failed => write!(f, "failed"),
            AttemptStatus::Cancelled => write!(f, "cancelled"),
            AttemptStatus::Orphaned => write!(f, "orphaned"),
        }
    }
}

/// An individual execution process within an attempt
///
/// Each attempt can spawn multiple processes: setup script, coding agent, cleanup.
/// This maps to vibe-kanban's ExecutionProcess concept.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct ExecutionProcess {
    /// Unique identifier
    pub id: Uuid,

    /// Parent attempt ID
    pub attempt_id: Uuid,

    /// Why this process was spawned
    pub run_reason: RunReason,

    /// Action chain configuration (for sequential execution)
    #[serde(default)]
    pub executor_action: Option<serde_json::Value>,

    // ─────────────────────────────────────────────────────────────────────
    // Git state tracking
    // ─────────────────────────────────────────────────────────────────────
    /// Git HEAD before execution started
    #[serde(default)]
    pub before_head_commit: Option<String>,

    /// Git HEAD after execution completed
    #[serde(default)]
    pub after_head_commit: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // Process state
    // ─────────────────────────────────────────────────────────────────────
    /// Current process status
    pub status: ProcessStatus,

    /// Process exit code (if completed)
    #[serde(default)]
    pub exit_code: Option<i32>,

    /// Whether this process is excluded from the timeline (e.g., after retry)
    #[serde(default)]
    pub dropped: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Timestamps
    // ─────────────────────────────────────────────────────────────────────
    #[ts(type = "string")]
    pub started_at: DateTime<Utc>,

    #[serde(default)]
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Why an execution process was spawned
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum RunReason {
    /// Running a setup script before agent execution
    SetupScript,
    /// Running the AI coding agent
    CodingAgent,
    /// Running cleanup after agent completion
    Cleanup,
    /// Follow-up execution after user feedback
    FollowUp,
}

/// Process execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ProcessStatus {
    /// Process is currently running
    #[default]
    Running,
    /// Process completed successfully
    Completed,
    /// Process failed with error
    Failed,
    /// Process was manually killed
    Killed,
}

/// A conversational session within an attempt
///
/// Sessions provide continuity for follow-up interactions.
/// This merges Operator's tmux session tracking with vibe-kanban's agent session concept.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct Session {
    /// Unique identifier
    pub id: Uuid,

    /// Parent attempt ID
    pub attempt_id: Uuid,

    // ─────────────────────────────────────────────────────────────────────
    // Operator's tmux tracking
    // ─────────────────────────────────────────────────────────────────────
    /// Tmux session name for terminal-based execution
    #[serde(default)]
    pub tmux_session_name: Option<String>,

    /// Hash of terminal content for change detection
    #[serde(default)]
    pub content_hash: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // vibe-kanban's agent session tracking
    // ─────────────────────────────────────────────────────────────────────
    /// Agent's internal session ID (e.g., Claude's conversation ID)
    #[serde(default)]
    pub agent_session_id: Option<String>,

    /// Summary extracted from message store
    #[serde(default)]
    pub summary: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // Timestamps
    // ─────────────────────────────────────────────────────────────────────
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

impl StepAttempt {
    /// Create a new step attempt
    pub fn new(
        ticket_id: String,
        step_name: String,
        project: String,
        branch: String,
        target_branch: String,
        executor: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            ticket_id,
            step_name,
            project,
            branch,
            target_branch,
            worktree_path: None,
            worktree_deleted: false,
            executor,
            executor_model: None,
            session_id: None,
            launch_mode: None,
            status: AttemptStatus::Pending,
            paired: false,
            tmux_session: None,
            content_hash: None,
            pr_url: None,
            pr_number: None,
            github_repo: None,
            pr_status: None,
            setup_completed_at: None,
            last_activity: now,
            last_content_change: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this attempt is in an active state
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            AttemptStatus::Pending
                | AttemptStatus::Setting
                | AttemptStatus::Running
                | AttemptStatus::AwaitingInput
        )
    }

    /// Check if this attempt is complete (success or failure)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            AttemptStatus::Completed
                | AttemptStatus::Failed
                | AttemptStatus::Cancelled
                | AttemptStatus::Orphaned
        )
    }
}

impl ExecutionProcess {
    /// Create a new execution process
    pub fn new(attempt_id: Uuid, run_reason: RunReason) -> Self {
        Self {
            id: Uuid::new_v4(),
            attempt_id,
            run_reason,
            executor_action: None,
            before_head_commit: None,
            after_head_commit: None,
            status: ProcessStatus::Running,
            exit_code: None,
            dropped: false,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

impl Session {
    /// Create a new session
    pub fn new(attempt_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            attempt_id,
            tmux_session_name: None,
            content_hash: None,
            agent_session_id: None,
            summary: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_attempt_new() {
        let attempt = StepAttempt::new(
            "FEAT-1234".to_string(),
            "implement".to_string(),
            "my-project".to_string(),
            "feat/abc123-login".to_string(),
            "main".to_string(),
            "claude".to_string(),
        );

        assert_eq!(attempt.ticket_id, "FEAT-1234");
        assert_eq!(attempt.step_name, "implement");
        assert_eq!(attempt.status, AttemptStatus::Pending);
        assert!(attempt.is_active());
        assert!(!attempt.is_terminal());
    }

    #[test]
    fn test_attempt_status_transitions() {
        let mut attempt = StepAttempt::new(
            "FIX-001".to_string(),
            "plan".to_string(),
            "proj".to_string(),
            "fix/001".to_string(),
            "main".to_string(),
            "claude".to_string(),
        );

        assert!(attempt.is_active());

        attempt.status = AttemptStatus::Running;
        assert!(attempt.is_active());

        attempt.status = AttemptStatus::Completed;
        assert!(attempt.is_terminal());
        assert!(!attempt.is_active());
    }

    #[test]
    fn test_execution_process_new() {
        let attempt_id = Uuid::new_v4();
        let process = ExecutionProcess::new(attempt_id, RunReason::CodingAgent);

        assert_eq!(process.attempt_id, attempt_id);
        assert_eq!(process.run_reason, RunReason::CodingAgent);
        assert_eq!(process.status, ProcessStatus::Running);
        assert!(!process.dropped);
    }

    #[test]
    fn test_session_new() {
        let attempt_id = Uuid::new_v4();
        let session = Session::new(attempt_id);

        assert_eq!(session.attempt_id, attempt_id);
        assert!(session.tmux_session_name.is_none());
        assert!(session.agent_session_id.is_none());
    }

    #[test]
    fn test_attempt_status_display() {
        assert_eq!(AttemptStatus::Running.to_string(), "running");
        assert_eq!(AttemptStatus::AwaitingInput.to_string(), "awaiting_input");
        assert_eq!(AttemptStatus::InReview.to_string(), "in_review");
    }
}
