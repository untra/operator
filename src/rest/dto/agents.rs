use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

// =============================================================================
// Health/Status DTOs
// =============================================================================

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    /// Top-level directory name of the operator working root (e.g. "acme").
    pub directory_name: String,
    /// Non-reversible fingerprint of the working root's canonical path.
    pub directory_id: String,
}

/// Status response with registry info
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    /// Top-level directory name of the operator working root (e.g. "acme").
    pub directory_name: String,
    /// Non-reversible fingerprint of the working root's canonical path.
    pub directory_id: String,
    pub issuetype_count: usize,
    pub collection_count: usize,
    pub active_collection: String,
}

// =============================================================================
// Kanban Board DTOs
// =============================================================================

/// A ticket card for the kanban board
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanTicketCard {
    /// Ticket ID (e.g., "FEAT-7598")
    pub id: String,
    /// Ticket summary/title
    pub summary: String,
    /// Ticket type: FEAT, FIX, INV, SPIKE
    pub ticket_type: String,
    /// Project name
    pub project: String,
    /// Current status: queued, running, awaiting, completed
    pub status: String,
    /// Current step name
    pub step: String,
    /// Human-readable step name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_display_name: Option<String>,
    /// Priority: P0-critical, P1-high, P2-medium, P3-low
    pub priority: String,
    /// Timestamp for sorting (YYYYMMDD-HHMM format)
    pub timestamp: String,
}

/// Kanban board response with tickets grouped by column
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanBoardResponse {
    /// Tickets in queue (not yet started)
    pub queue: Vec<KanbanTicketCard>,
    /// Tickets currently being worked on
    pub running: Vec<KanbanTicketCard>,
    /// Tickets awaiting review or input
    pub awaiting: Vec<KanbanTicketCard>,
    /// Completed tickets
    pub done: Vec<KanbanTicketCard>,
    /// Total ticket count across all columns
    pub total_count: usize,
    /// ISO 8601 timestamp of last data refresh
    pub last_updated: String,
}

// =============================================================================
// Queue Status DTOs
// =============================================================================

/// Ticket counts by type for queue status
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct QueueByType {
    pub inv: usize,
    pub fix: usize,
    pub feat: usize,
    pub spike: usize,
}

/// Queue status response with ticket counts
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct QueueStatusResponse {
    /// Tickets waiting in queue
    pub queued: usize,
    /// Tickets currently being worked on
    pub in_progress: usize,
    /// Tickets awaiting review or input
    pub awaiting: usize,
    /// Completed tickets (today)
    pub completed: usize,
    /// Breakdown by ticket type
    pub by_type: QueueByType,
}

// =============================================================================
// Active Agents DTOs
// =============================================================================

/// A single active agent
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ActiveAgentResponse {
    /// Agent ID (e.g., "op-gamesvc-001")
    pub id: String,
    /// Associated ticket ID (e.g., "FEAT-042")
    pub ticket_id: String,
    /// Ticket type: FEAT, FIX, INV, SPIKE
    pub ticket_type: String,
    /// Project being worked on
    pub project: String,
    /// Agent status: running, `awaiting_input`, completing
    pub status: String,
    /// Execution mode: autonomous, paired
    pub mode: String,
    /// When the agent started (ISO 8601)
    pub started_at: String,
    /// Current workflow step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
    /// Which session wrapper is in use: "tmux", "vscode", "cmux", or "zellij"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_wrapper: Option<String>,
    /// Session window reference ID (e.g. cmux window, tmux session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_window_ref: Option<String>,
    /// Session context reference (e.g. cmux workspace, zellij session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_context_ref: Option<String>,
    /// Session pane reference (e.g. cmux surface, zellij pane)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_pane_ref: Option<String>,
}

/// Response for active agents list
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ActiveAgentsResponse {
    /// List of active agents
    pub agents: Vec<ActiveAgentResponse>,
    /// Total count of active agents
    pub count: usize,
}

// =============================================================================
// Agent Detail DTOs
// =============================================================================

/// Full details for a single agent
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct AgentDetailResponse {
    /// Agent ID (UUID)
    pub id: String,
    /// Associated ticket ID (e.g., "FEAT-042")
    pub ticket_id: String,
    /// Ticket type: FEAT, FIX, INV, SPIKE
    pub ticket_type: String,
    /// Project being worked on
    pub project: String,
    /// Agent status: running, `awaiting_input`, completing, orphaned
    pub status: String,
    /// When the agent started (ISO 8601)
    pub started_at: String,
    /// Last activity timestamp (ISO 8601)
    pub last_activity: String,
    /// Current workflow step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
    /// LLM tool used (e.g., "claude", "gemini", "codex")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_tool: Option<String>,
    /// LLM model alias (e.g., "opus", "sonnet", "gpt-4o")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,
    /// Launch mode: "default", "yolo", "docker", "docker-yolo"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<String>,
    /// PR URL if created during "pr" step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    /// Last known PR status ("open", "approved", "`changes_requested`", "merged", "closed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_status: Option<String>,
    /// Which session wrapper is in use: "tmux", "vscode", "cmux", or "zellij"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_wrapper: Option<String>,
    /// Review state for `awaiting_input` agents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_state: Option<String>,
    /// Completed steps for this ticket
    pub completed_steps: Vec<String>,
    /// Path to the git worktree for this ticket
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    /// Whether this is a paired (interactive) agent
    pub paired: bool,
}

// =============================================================================
// Ticket Launch DTOs
// =============================================================================

/// Request to launch a ticket
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LaunchTicketRequest {
    /// Named delegator to use (takes precedence over provider/model)
    #[serde(default)]
    pub delegator: Option<String>,
    /// LLM provider to use (e.g., "claude") — legacy fallback when no delegator
    #[serde(default)]
    pub provider: Option<String>,
    /// Model to use (e.g., "sonnet", "opus") — legacy fallback when no delegator
    #[serde(default)]
    pub model: Option<String>,
    /// Ad-hoc model server to target (e.g. "ollama-local") — legacy fallback when
    /// no delegator. Injects the server's base URL / API key env at spawn.
    #[serde(default)]
    pub model_server: Option<String>,
    /// Run in YOLO mode (auto-accept all prompts)
    #[serde(default)]
    pub yolo_mode: bool,
    /// Session wrapper type: "vscode", "tmux", "cmux", "terminal"
    #[serde(default)]
    pub wrapper: Option<String>,
    /// Feedback for relaunch (what went wrong on previous attempt)
    #[serde(default)]
    pub retry_reason: Option<String>,
    /// Existing session ID to resume (for continuing from where it left off)
    #[serde(default)]
    pub resume_session_id: Option<String>,
}

/// Response from launching a ticket
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LaunchTicketResponse {
    /// Agent ID assigned to this launch
    pub agent_id: String,
    /// Ticket ID that was launched
    pub ticket_id: String,
    /// Working directory (worktree if created, else project path)
    pub working_directory: String,
    /// Command to execute in terminal
    pub command: String,
    /// Terminal name to use (same value as `tmux_session_name`)
    pub terminal_name: String,
    /// Tmux session name for attaching (same value as `terminal_name`, kept for backward compat)
    pub tmux_session_name: String,
    /// Which session wrapper was used: "tmux", "vscode", or "cmux"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_wrapper: Option<String>,
    /// Session window reference ID (e.g. cmux window, tmux session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_window_ref: Option<String>,
    /// Session context reference (e.g. cmux workspace, zellij session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_context_ref: Option<String>,
    /// Session UUID for the LLM tool
    pub session_id: String,
    /// Whether a worktree was created
    pub worktree_created: bool,
    /// Branch name (if worktree was created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

// =============================================================================
// OperatorOutput DTOs (structured agent output)
// =============================================================================

/// Standardized agent output for progress tracking and step transitions.
///
/// Agents output a status block in their response which is parsed into this structure.
/// Used for progress tracking, loop detection, and intelligent step transitions.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS, Default)]
#[ts(export)]
pub struct OperatorOutput {
    /// Current work status: `in_progress`, complete, blocked, failed
    pub status: String,
    /// Agent signals done with step (true) or more work remains (false)
    pub exit_signal: bool,
    /// Agent's confidence in completion (0-100%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    /// Number of files changed this iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_modified: Option<u32>,
    /// Test suite status: passing, failing, skipped, `not_run`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests_status: Option<String>,
    /// Number of errors encountered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_count: Option<u32>,
    /// Number of sub-tasks completed this iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_completed: Option<u32>,
    /// Estimated remaining sub-tasks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_remaining: Option<u32>,
    /// Brief description of work done (max 500 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Suggested next action (max 200 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
    /// Issues preventing progress (signals intervention needed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockers: Option<Vec<String>>,
}

// =============================================================================
// Step Completion DTOs (for opr8r wrapper)
// =============================================================================

/// Request to report step completion (from opr8r wrapper)
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StepCompleteRequest {
    /// Exit code from the LLM command
    pub exit_code: i32,
    /// Whether output validation passed (if schema was specified)
    #[serde(default = "default_true")]
    pub output_valid: bool,
    /// List of validation errors (if `output_valid` is false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema_errors: Option<Vec<String>>,
    /// Session ID from the LLM session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Duration of the step in seconds
    pub duration_secs: u64,
    /// Sample of the output (first N chars for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_sample: Option<String>,
    /// Structured output from agent (parsed `OPERATOR_STATUS` block)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<OperatorOutput>,
}

fn default_true() -> bool {
    true
}

/// Response from step completion endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StepCompleteResponse {
    /// Status of the step: "completed", "`awaiting_review`", "failed", "iterate"
    pub status: String,
    /// Information about the next step (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<NextStepInfo>,
    /// Whether to automatically proceed to the next step
    pub auto_proceed: bool,
    /// Command to execute for the next step (opr8r wrapped)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_command: Option<String>,

    // Analysis results from OperatorOutput processing
    /// Whether `OperatorOutput` was successfully parsed from agent output
    #[serde(default)]
    pub output_valid: bool,
    /// Agent has more work (`exit_signal=false`) - indicates iteration needed
    #[serde(default)]
    pub should_iterate: bool,
    /// How many times this step has run (for circuit breaker)
    #[serde(default)]
    pub iteration_count: u32,
    /// Circuit breaker state: closed (normal), `half_open` (monitoring), open (halted)
    #[serde(default = "default_circuit_closed")]
    pub circuit_state: String,

    // Context piped from agent output for next step
    /// Summary from previous step's `OperatorOutput`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_summary: Option<String>,
    /// Recommendation from previous step's `OperatorOutput`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_recommendation: Option<String>,
    /// Cumulative files modified across iterations
    #[serde(default)]
    pub cumulative_files_modified: u32,
    /// Cumulative errors across iterations
    #[serde(default)]
    pub cumulative_errors: u32,
}

fn default_circuit_closed() -> String {
    "closed".to_string()
}

/// Information about the next step in the workflow
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct NextStepInfo {
    /// Step name
    pub name: String,
    /// Display name for the step
    pub display_name: String,
    /// Review type: "none", "plan", "visual", "pr"
    pub review_type: String,
    /// Prompt template for the step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

// =============================================================================
// Queue Control DTOs
// =============================================================================

/// Response for queue pause/resume operations
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct QueueControlResponse {
    /// Whether the queue is currently paused
    pub paused: bool,
    /// Human-readable message about the operation
    pub message: String,
}

/// Response for kanban sync operations
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanSyncResponse {
    /// Ticket IDs that were created
    pub created: Vec<String>,
    /// Ticket IDs that were skipped (already exist)
    pub skipped: Vec<String>,
    /// Error messages for failed syncs
    pub errors: Vec<String>,
    /// Total count of issues processed
    pub total_processed: usize,
}

// =============================================================================
// Agent Review DTOs
// =============================================================================

/// Response for agent review operations (approve/reject)
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ReviewResponse {
    /// Agent ID that was reviewed
    pub agent_id: String,
    /// Review status: "approved" or "rejected"
    pub status: String,
    /// Human-readable message about the operation
    pub message: String,
}

/// Request to reject an agent's review
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct RejectReviewRequest {
    /// Reason for rejection (feedback for the agent)
    pub reason: String,
}

// =============================================================================
// Ticket Detail DTOs
// =============================================================================

/// Full ticket details including content and metadata
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct TicketDetailResponse {
    /// Ticket ID (e.g., "FEAT-7598")
    pub id: String,
    /// Ticket summary/title
    pub summary: String,
    /// Ticket type: FEAT, FIX, INV, SPIKE
    pub ticket_type: String,
    /// Project name
    pub project: String,
    /// Current status: queued, running, awaiting, completed
    pub status: String,
    /// Current step name
    pub step: String,
    /// Human-readable step name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_display_name: Option<String>,
    /// Priority: P0-critical, P1-high, P2-medium, P3-low
    pub priority: String,
    /// Timestamp (YYYYMMDD-HHMM format)
    pub timestamp: String,
    /// Full markdown content of the ticket
    pub content: String,
    /// Ticket filename
    pub filename: String,
    /// Full filesystem path
    pub filepath: String,
    /// Session IDs per step (`step_name` -> `session_uuid`)
    pub sessions: std::collections::HashMap<String, String>,
    /// Delegator used per step (`step_name` -> `delegator_name`)
    pub step_delegators: std::collections::HashMap<String, String>,
    /// Path to git worktree (if created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    /// Git branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// External issue ID from kanban provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// URL to the issue in the external provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    /// Provider name (e.g., "jira", "linear")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_provider: Option<String>,
}

/// Request to update a ticket's status
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateTicketStatusRequest {
    /// Target status: queued, running, awaiting, done
    pub status: String,
}

/// Response from updating a ticket's status
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateTicketStatusResponse {
    /// Ticket ID
    pub id: String,
    /// Previous status before the update
    pub previous_status: String,
    /// New status after the update
    pub status: String,
    /// Human-readable message
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_roundtrip_preserves_fields() {
        let resp = HealthResponse {
            status: "ok".to_string(),
            version: "0.2.2".to_string(),
            directory_name: "acme".to_string(),
            directory_id: "abc123".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: HealthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.version, "0.2.2");
        assert_eq!(parsed.directory_name, "acme");
        assert_eq!(parsed.directory_id, "abc123");
    }

    #[test]
    fn test_kanban_ticket_card_step_display_name_absent_when_none() {
        let card = KanbanTicketCard {
            id: "FEAT-1".to_string(),
            summary: "Add thing".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "gamesvc".to_string(),
            status: "queued".to_string(),
            step: "execute".to_string(),
            step_display_name: None,
            priority: "P2-medium".to_string(),
            timestamp: "20260616-1200".to_string(),
        };
        let json = serde_json::to_string(&card).unwrap();
        assert!(!json.contains("step_display_name"));
    }

    #[test]
    fn test_kanban_ticket_card_step_display_name_present_when_set() {
        let card = KanbanTicketCard {
            id: "FEAT-1".to_string(),
            summary: "Add thing".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "gamesvc".to_string(),
            status: "queued".to_string(),
            step: "execute".to_string(),
            step_display_name: Some("Execute".to_string()),
            priority: "P2-medium".to_string(),
            timestamp: "20260616-1200".to_string(),
        };
        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("\"step_display_name\":\"Execute\""));
    }

    #[test]
    fn test_queue_status_response_nests_by_type_counts() {
        let resp = QueueStatusResponse {
            queued: 3,
            in_progress: 1,
            awaiting: 2,
            completed: 7,
            by_type: QueueByType {
                inv: 1,
                fix: 1,
                feat: 1,
                spike: 0,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"by_type\":{"));
        let parsed: QueueStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.by_type.inv, 1);
        assert_eq!(parsed.by_type.spike, 0);
        assert_eq!(parsed.completed, 7);
    }

    #[test]
    fn test_active_agent_response_session_fields_absent_when_none() {
        let agent = ActiveAgentResponse {
            id: "op-1".to_string(),
            ticket_id: "FEAT-1".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "gamesvc".to_string(),
            status: "running".to_string(),
            mode: "autonomous".to_string(),
            started_at: "2026-06-16T12:00:00Z".to_string(),
            current_step: None,
            session_wrapper: None,
            session_window_ref: None,
            session_context_ref: None,
            session_pane_ref: None,
        };
        let json = serde_json::to_string(&agent).unwrap();
        assert!(!json.contains("current_step"));
        assert!(!json.contains("session_wrapper"));
        assert!(!json.contains("session_window_ref"));
        assert!(!json.contains("session_context_ref"));
        assert!(!json.contains("session_pane_ref"));
    }

    #[test]
    fn test_agent_detail_response_optional_fields_absent_when_none() {
        let detail = AgentDetailResponse {
            id: "uuid-1".to_string(),
            ticket_id: "FEAT-1".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "gamesvc".to_string(),
            status: "running".to_string(),
            started_at: "2026-06-16T12:00:00Z".to_string(),
            last_activity: "2026-06-16T12:05:00Z".to_string(),
            current_step: None,
            llm_tool: None,
            llm_model: None,
            launch_mode: None,
            pr_url: None,
            pr_status: None,
            session_wrapper: None,
            review_state: None,
            completed_steps: vec![],
            worktree_path: None,
            paired: false,
        };
        let json = serde_json::to_string(&detail).unwrap();
        // skip_serializing_if optionals are omitted...
        assert!(!json.contains("pr_url"));
        assert!(!json.contains("worktree_path"));
        // ...but non-optional fields (incl. empty Vec) remain in the shape.
        assert!(json.contains("\"completed_steps\":[]"));
        assert!(json.contains("\"paired\":false"));
    }

    #[test]
    fn test_launch_ticket_request_minimal_json_applies_defaults() {
        // Every field is #[serde(default)]; an empty object must parse.
        let req: LaunchTicketRequest = serde_json::from_str("{}").unwrap();
        assert!(req.delegator.is_none());
        assert!(req.provider.is_none());
        assert!(req.model.is_none());
        assert!(req.model_server.is_none());
        assert!(!req.yolo_mode);
        assert!(req.wrapper.is_none());
        assert!(req.retry_reason.is_none());
        assert!(req.resume_session_id.is_none());
    }

    #[test]
    fn test_step_complete_request_output_valid_defaults_true_when_absent() {
        // default_true(): output_valid should be true when the JSON omits it.
        let json = r#"{ "exit_code": 0, "duration_secs": 10 }"#;
        let req: StepCompleteRequest = serde_json::from_str(json).unwrap();
        assert!(req.output_valid);
        assert!(req.output.is_none());
    }

    #[test]
    fn test_step_complete_response_circuit_state_defaults_closed_when_absent() {
        // default_circuit_closed(): circuit_state should be "closed" when omitted,
        // and the other #[serde(default)] fields fall back to their zero values.
        let json = r#"{ "status": "completed", "auto_proceed": true }"#;
        let resp: StepCompleteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.circuit_state, "closed");
        assert!(!resp.output_valid);
        assert!(!resp.should_iterate);
        assert_eq!(resp.iteration_count, 0);
        assert_eq!(resp.cumulative_files_modified, 0);
        assert_eq!(resp.cumulative_errors, 0);
    }

    #[test]
    fn test_ticket_detail_response_roundtrip_preserves_maps() {
        let mut sessions = std::collections::HashMap::new();
        sessions.insert("execute".to_string(), "uuid-1".to_string());
        let mut step_delegators = std::collections::HashMap::new();
        step_delegators.insert("execute".to_string(), "claude-opus".to_string());

        let detail = TicketDetailResponse {
            id: "FEAT-1".to_string(),
            summary: "Add thing".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "gamesvc".to_string(),
            status: "running".to_string(),
            step: "execute".to_string(),
            step_display_name: None,
            priority: "P2-medium".to_string(),
            timestamp: "20260616-1200".to_string(),
            content: "# Ticket".to_string(),
            filename: "feat-1.md".to_string(),
            filepath: "/tmp/feat-1.md".to_string(),
            sessions,
            step_delegators,
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        };
        let json = serde_json::to_string(&detail).unwrap();
        let parsed: TicketDetailResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sessions.get("execute").unwrap(), "uuid-1");
        assert_eq!(
            parsed.step_delegators.get("execute").unwrap(),
            "claude-opus"
        );
        assert!(parsed.external_provider.is_none());
    }

    #[test]
    fn test_next_step_info_prompt_absent_when_none() {
        let info = NextStepInfo {
            name: "review".to_string(),
            display_name: "Review".to_string(),
            review_type: "pr".to_string(),
            prompt: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"review_type\":\"pr\""));
        assert!(!json.contains("prompt"));
    }
}
