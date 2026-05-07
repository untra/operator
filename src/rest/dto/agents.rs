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
}

/// Status response with registry info
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
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
