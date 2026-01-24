//! Data Transfer Objects for the REST API.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

// Note: ToSchema is derived on all DTOs for OpenAPI documentation generation

use crate::issuetypes::schema::IssueTypeSource;
use crate::issuetypes::{IssueType, IssueTypeCollection};
use crate::templates::schema::{
    ExecutionMode, FieldSchema, FieldType, PermissionMode, StepOutput, StepSchema,
};

// =============================================================================
// Issue Type DTOs
// =============================================================================

/// Response for a single issue type
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct IssueTypeResponse {
    pub key: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    pub glyph: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub project_required: bool,
    pub source: String,
    pub fields: Vec<FieldResponse>,
    pub steps: Vec<StepResponse>,
}

impl From<&IssueType> for IssueTypeResponse {
    fn from(it: &IssueType) -> Self {
        Self {
            key: it.key.clone(),
            name: it.name.clone(),
            description: it.description.clone(),
            mode: match it.mode {
                ExecutionMode::Autonomous => "autonomous".to_string(),
                ExecutionMode::Paired => "paired".to_string(),
            },
            glyph: it.glyph.clone(),
            color: it.color.clone(),
            project_required: it.project_required,
            source: it.source_display(),
            fields: it.fields.iter().map(FieldResponse::from).collect(),
            steps: it.steps.iter().map(StepResponse::from).collect(),
        }
    }
}

/// Summary response for listing issue types
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct IssueTypeSummary {
    pub key: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    pub glyph: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub color: Option<String>,
    pub source: String,
    pub step_count: usize,
}

impl From<&IssueType> for IssueTypeSummary {
    fn from(it: &IssueType) -> Self {
        Self {
            key: it.key.clone(),
            name: it.name.clone(),
            description: it.description.clone(),
            mode: match it.mode {
                ExecutionMode::Autonomous => "autonomous".to_string(),
                ExecutionMode::Paired => "paired".to_string(),
            },
            glyph: it.glyph.clone(),
            color: it.color.clone(),
            source: it.source_display(),
            step_count: it.steps.len(),
        }
    }
}

/// Request to create a new issue type
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateIssueTypeRequest {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    pub glyph: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default = "default_true")]
    pub project_required: bool,
    #[serde(default)]
    pub fields: Vec<CreateFieldRequest>,
    pub steps: Vec<CreateStepRequest>,
}

fn default_mode() -> String {
    "autonomous".to_string()
}

fn default_true() -> bool {
    true
}

impl CreateIssueTypeRequest {
    /// Convert request to IssueType
    pub fn into_issue_type(self) -> IssueType {
        IssueType {
            key: self.key.to_uppercase(),
            name: self.name,
            description: self.description,
            mode: if self.mode == "paired" {
                ExecutionMode::Paired
            } else {
                ExecutionMode::Autonomous
            },
            glyph: self.glyph,
            color: self.color,
            project_required: self.project_required,
            fields: self.fields.into_iter().map(|f| f.into()).collect(),
            steps: self.steps.into_iter().map(|s| s.into()).collect(),
            agent_prompt: None,
            source: IssueTypeSource::User,
            external_id: None,
        }
    }
}

/// Request to update an issue type
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateIssueTypeRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub glyph: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub project_required: Option<bool>,
    #[serde(default)]
    pub fields: Option<Vec<CreateFieldRequest>>,
    #[serde(default)]
    pub steps: Option<Vec<CreateStepRequest>>,
}

// =============================================================================
// Field DTOs
// =============================================================================

/// Response for a field
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct FieldResponse {
    pub name: String,
    pub description: String,
    pub field_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    pub user_editable: bool,
}

impl From<&FieldSchema> for FieldResponse {
    fn from(f: &FieldSchema) -> Self {
        Self {
            name: f.name.clone(),
            description: f.description.clone(),
            field_type: match f.field_type {
                FieldType::String => "string".to_string(),
                FieldType::Enum => "enum".to_string(),
                FieldType::Bool => "bool".to_string(),
                FieldType::Date => "date".to_string(),
                FieldType::Text => "text".to_string(),
                FieldType::Integer => "integer".to_string(),
            },
            required: f.required,
            default: f.default.clone(),
            options: f.options.clone(),
            placeholder: f.placeholder.clone(),
            max_length: f.max_length,
            user_editable: f.user_editable,
        }
    }
}

/// Request to create a field
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateFieldRequest {
    pub name: String,
    pub description: String,
    #[serde(default = "default_string_type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default = "default_true")]
    pub user_editable: bool,
}

fn default_string_type() -> String {
    "string".to_string()
}

impl From<CreateFieldRequest> for FieldSchema {
    fn from(f: CreateFieldRequest) -> Self {
        Self {
            name: f.name,
            description: f.description,
            field_type: match f.field_type.as_str() {
                "enum" => FieldType::Enum,
                "bool" => FieldType::Bool,
                "date" => FieldType::Date,
                "text" => FieldType::Text,
                _ => FieldType::String,
            },
            required: f.required,
            default: f.default,
            auto: None,
            options: f.options,
            placeholder: f.placeholder,
            max_length: f.max_length,
            display_order: None,
            user_editable: f.user_editable,
        }
    }
}

// =============================================================================
// Step DTOs
// =============================================================================

/// Response for a step
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StepResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub prompt: String,
    pub outputs: Vec<String>,
    pub allowed_tools: Vec<String>,
    /// Type of review required: "none", "plan", "visual", "pr"
    pub review_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<String>,
    pub permission_mode: String,
}

impl From<&StepSchema> for StepResponse {
    fn from(s: &StepSchema) -> Self {
        Self {
            name: s.name.clone(),
            display_name: s.display_name.clone(),
            prompt: s.prompt.clone(),
            outputs: s
                .outputs
                .iter()
                .map(|o| match o {
                    StepOutput::Plan => "plan".to_string(),
                    StepOutput::Code => "code".to_string(),
                    StepOutput::Test => "test".to_string(),
                    StepOutput::Pr => "pr".to_string(),
                    StepOutput::Ticket => "ticket".to_string(),
                    StepOutput::Review => "review".to_string(),
                    StepOutput::Report => "report".to_string(),
                    StepOutput::Documentation => "documentation".to_string(),
                })
                .collect(),
            allowed_tools: s.allowed_tools.clone(),
            review_type: match s.review_type {
                crate::templates::schema::ReviewType::None => "none".to_string(),
                crate::templates::schema::ReviewType::Plan => "plan".to_string(),
                crate::templates::schema::ReviewType::Visual => "visual".to_string(),
                crate::templates::schema::ReviewType::Pr => "pr".to_string(),
            },
            next_step: s.next_step.clone(),
            permission_mode: match s.permission_mode {
                PermissionMode::Default => "default".to_string(),
                PermissionMode::Plan => "plan".to_string(),
                PermissionMode::AcceptEdits => "acceptEdits".to_string(),
                PermissionMode::Delegate => "delegate".to_string(),
            },
        }
    }
}

/// Request to create a step
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateStepRequest {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default = "default_all_tools")]
    pub allowed_tools: Vec<String>,
    /// Type of review required: "none", "plan", "visual", "pr"
    #[serde(default = "default_review_type")]
    pub review_type: String,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
}

fn default_all_tools() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_review_type() -> String {
    "none".to_string()
}

fn default_permission_mode() -> String {
    "default".to_string()
}

impl From<CreateStepRequest> for StepSchema {
    fn from(s: CreateStepRequest) -> Self {
        Self {
            name: s.name,
            display_name: s.display_name,
            prompt: s.prompt,
            outputs: s
                .outputs
                .iter()
                .filter_map(|o| match o.as_str() {
                    "plan" => Some(StepOutput::Plan),
                    "code" => Some(StepOutput::Code),
                    "test" => Some(StepOutput::Test),
                    "pr" => Some(StepOutput::Pr),
                    "ticket" => Some(StepOutput::Ticket),
                    "review" => Some(StepOutput::Review),
                    "report" => Some(StepOutput::Report),
                    "documentation" => Some(StepOutput::Documentation),
                    _ => None,
                })
                .collect(),
            allowed_tools: s.allowed_tools,
            review_type: match s.review_type.as_str() {
                "plan" => crate::templates::schema::ReviewType::Plan,
                "visual" => crate::templates::schema::ReviewType::Visual,
                "pr" => crate::templates::schema::ReviewType::Pr,
                _ => crate::templates::schema::ReviewType::None,
            },
            visual_config: None,
            on_reject: None,
            next_step: s.next_step,
            permissions: None,
            cli_args: None,
            permission_mode: match s.permission_mode.as_str() {
                "plan" => PermissionMode::Plan,
                "acceptEdits" => PermissionMode::AcceptEdits,
                "delegate" => PermissionMode::Delegate,
                _ => PermissionMode::Default,
            },
            json_schema: None,
            json_schema_file: None,
        }
    }
}

/// Request to update a step
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateStepRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub outputs: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
    /// Type of review required: "none", "plan", "visual", "pr"
    #[serde(default)]
    pub review_type: Option<String>,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

// =============================================================================
// Collection DTOs
// =============================================================================

/// Response for a collection
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CollectionResponse {
    pub name: String,
    pub description: String,
    pub types: Vec<String>,
    pub is_active: bool,
}

impl CollectionResponse {
    pub fn from_collection(c: &IssueTypeCollection, is_active: bool) -> Self {
        Self {
            name: c.name.clone(),
            description: c.description.clone(),
            types: c.types.clone(),
            is_active,
        }
    }
}

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
    /// Agent status: running, awaiting_input, completing
    pub status: String,
    /// Execution mode: autonomous, paired
    pub mode: String,
    /// When the agent started (ISO 8601)
    pub started_at: String,
    /// Current workflow step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
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
    /// LLM provider to use (e.g., "claude")
    #[serde(default)]
    pub provider: Option<String>,
    /// Model to use (e.g., "sonnet", "opus")
    #[serde(default)]
    pub model: Option<String>,
    /// Run in YOLO mode (auto-accept all prompts)
    #[serde(default)]
    pub yolo_mode: bool,
    /// Session wrapper type: "vscode", "tmux", "terminal"
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
    /// Terminal name to use (same value as tmux_session_name)
    pub terminal_name: String,
    /// Tmux session name for attaching (same value as terminal_name)
    pub tmux_session_name: String,
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
    /// Current work status: in_progress, complete, blocked, failed
    pub status: String,
    /// Agent signals done with step (true) or more work remains (false)
    pub exit_signal: bool,
    /// Agent's confidence in completion (0-100%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    /// Number of files changed this iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_modified: Option<u32>,
    /// Test suite status: passing, failing, skipped, not_run
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
    /// List of validation errors (if output_valid is false)
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
    /// Structured output from agent (parsed OPERATOR_STATUS block)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<OperatorOutput>,
}

/// Response from step completion endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StepCompleteResponse {
    /// Status of the step: "completed", "awaiting_review", "failed", "iterate"
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
    /// Whether OperatorOutput was successfully parsed from agent output
    #[serde(default)]
    pub output_valid: bool,
    /// Agent has more work (exit_signal=false) - indicates iteration needed
    #[serde(default)]
    pub should_iterate: bool,
    /// How many times this step has run (for circuit breaker)
    #[serde(default)]
    pub iteration_count: u32,
    /// Circuit breaker state: closed (normal), half_open (monitoring), open (halted)
    #[serde(default = "default_circuit_closed")]
    pub circuit_state: String,

    // Context piped from agent output for next step
    /// Summary from previous step's OperatorOutput
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_summary: Option<String>,
    /// Recommendation from previous step's OperatorOutput
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_issue_type_request_into() {
        let req = CreateIssueTypeRequest {
            key: "test".to_string(),
            name: "Test".to_string(),
            description: "A test type".to_string(),
            mode: "autonomous".to_string(),
            glyph: "T".to_string(),
            color: None,
            project_required: true,
            fields: vec![],
            steps: vec![CreateStepRequest {
                name: "execute".to_string(),
                display_name: None,
                prompt: "Do the thing".to_string(),
                outputs: vec![],
                allowed_tools: vec!["*".to_string()],
                review_type: "none".to_string(),
                next_step: None,
                permission_mode: "default".to_string(),
            }],
        };

        let it = req.into_issue_type();
        assert_eq!(it.key, "TEST"); // Uppercased
        assert_eq!(it.name, "Test");
        assert!(matches!(it.mode, ExecutionMode::Autonomous));
        assert!(matches!(it.source, IssueTypeSource::User));
        assert_eq!(it.steps.len(), 1);
    }

    #[test]
    fn test_issue_type_response_from() {
        let it = IssueType::new_imported(
            "TEST".to_string(),
            "Test".to_string(),
            "A test".to_string(),
            "jira".to_string(),
            "PROJ".to_string(),
            None,
        );

        let resp = IssueTypeResponse::from(&it);
        assert_eq!(resp.key, "TEST");
        assert_eq!(resp.mode, "autonomous");
        assert_eq!(resp.source, "jira/PROJ");
    }

    #[test]
    fn test_operator_output_default() {
        let output = OperatorOutput::default();
        assert_eq!(output.status, "");
        assert!(!output.exit_signal);
        assert!(output.confidence.is_none());
        assert!(output.summary.is_none());
    }

    #[test]
    fn test_operator_output_serialization() {
        let output = OperatorOutput {
            status: "complete".to_string(),
            exit_signal: true,
            confidence: Some(95),
            files_modified: Some(3),
            tests_status: Some("passing".to_string()),
            error_count: Some(0),
            tasks_completed: Some(5),
            tasks_remaining: Some(0),
            summary: Some("Implemented feature".to_string()),
            recommendation: Some("Ready for review".to_string()),
            blockers: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"status\":\"complete\""));
        assert!(json.contains("\"exit_signal\":true"));
        assert!(json.contains("\"confidence\":95"));
        assert!(!json.contains("blockers")); // None fields are skipped
    }

    #[test]
    fn test_operator_output_deserialization() {
        let json = r#"{
            "status": "in_progress",
            "exit_signal": false,
            "confidence": 60,
            "files_modified": 2,
            "tests_status": "failing",
            "summary": "Working on tests"
        }"#;

        let output: OperatorOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.status, "in_progress");
        assert!(!output.exit_signal);
        assert_eq!(output.confidence, Some(60));
        assert_eq!(output.tests_status, Some("failing".to_string()));
    }

    #[test]
    fn test_step_complete_request_with_operator_output() {
        let output = OperatorOutput {
            status: "complete".to_string(),
            exit_signal: true,
            confidence: Some(90),
            ..Default::default()
        };

        let request = StepCompleteRequest {
            exit_code: 0,
            output_valid: true,
            output_schema_errors: None,
            session_id: Some("session-123".to_string()),
            duration_secs: 300,
            output_sample: None,
            output: Some(output),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"exit_code\":0"));
        assert!(json.contains("\"output\":{"));
        assert!(json.contains("\"status\":\"complete\""));
    }

    #[test]
    fn test_step_complete_response_with_analysis_fields() {
        let json = r#"{
            "status": "completed",
            "auto_proceed": true,
            "output_valid": true,
            "should_iterate": false,
            "iteration_count": 1,
            "circuit_state": "closed",
            "previous_summary": "Built feature",
            "cumulative_files_modified": 5,
            "cumulative_errors": 0
        }"#;

        let response: StepCompleteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "completed");
        assert!(response.output_valid);
        assert!(!response.should_iterate);
        assert_eq!(response.iteration_count, 1);
        assert_eq!(response.circuit_state, "closed");
        assert_eq!(response.previous_summary, Some("Built feature".to_string()));
    }
}
