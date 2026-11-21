//! DTOs for ticket creation and external alerts.
//!
//! These power external automation surfaces (e.g. the AGNT `operator-plugin`),
//! which create tickets and raise investigations over HTTP rather than via the
//! MCP server or CLI.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Request to create a new ticket. Every surface (web, TUI, VS Code, CLI,
/// MCP) sends this same shape to `POST /api/v1/tickets`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateTicketRequest {
    /// Issue type key from the registry (e.g. `FEAT`, `FIX`, or a custom
    /// collection type). `template` is accepted as a legacy alias.
    #[serde(alias = "template")]
    pub issue_type: String,
    /// Project the ticket targets. Required when the issue type sets
    /// `project_required`.
    #[serde(default)]
    pub project: Option<String>,
    /// One-line summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// Values for the issue type's user-editable fields, keyed by field name.
    /// Explicit `project`/`summary` take precedence over the same keys here.
    #[serde(default)]
    pub values: HashMap<String, String>,
    /// Push the new ticket to the mapped kanban board. `None` pushes whenever
    /// the project maps to a bidirectional kanban project.
    #[serde(default)]
    pub sync_to_kanban: Option<bool>,
}

/// Response after creating a ticket.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateTicketResponse {
    /// The created ticket's id (e.g. `FEAT-1234`).
    pub id: String,
    /// The ticket filename written to the queue.
    pub filename: String,
    /// The absolute path the ticket was written to.
    pub path: String,
    /// What happened when mirroring the ticket to its kanban board. A failed
    /// push never fails the create: the ticket exists locally regardless.
    pub kanban: KanbanPushOutcome,
}

/// The kanban project a new ticket would be pushed to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanTarget {
    /// Provider name (`jira`, `linear`, `github`, `gitea`).
    pub provider: String,
    /// Provider project / team key.
    pub project_key: String,
}

/// Result of pushing a newly created ticket to its kanban board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum KanbanPushOutcome {
    /// The ticket's project maps to no bidirectional kanban project.
    NotConfigured,
    /// A target exists but the push was not attempted.
    Skipped { reason: String },
    /// Pushed; the external ids were written back into the ticket.
    Pushed {
        provider: String,
        project_key: String,
        external_id: String,
        external_url: Option<String>,
    },
    /// The provider rejected or could not be reached.
    Failed { provider: String, message: String },
}

/// A value that failed issue type field validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct FieldError {
    /// Field name (`project`, `summary`, or an issue type field).
    pub field: String,
    /// Human-readable reason.
    pub message: String,
}

/// Dry run of `POST /api/v1/tickets`: what would be written, and where it
/// would sync, without allocating an id or touching the queue.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct TicketPreviewResponse {
    /// Rendered markdown, with a placeholder id. Empty when `errors` is not.
    pub markdown: String,
    /// The kanban project the ticket would be pushed to, if any.
    pub kanban_target: Option<KanbanTarget>,
    /// Validation errors; the create would be rejected while non-empty.
    pub errors: Vec<FieldError>,
}

/// Request to raise an external alert as an investigation ticket.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateAlertRequest {
    /// Where the alert came from (e.g. `pagerduty`, `sentry`).
    pub source: String,
    /// The alert message / summary.
    pub message: String,
    /// Severity label (e.g. `S1`, `S2`).
    pub severity: String,
    /// Optional project the investigation targets.
    #[serde(default)]
    pub project: Option<String>,
}

/// Response after raising an alert.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateAlertResponse {
    /// The created investigation ticket's id.
    pub id: String,
    /// The investigation ticket filename written to the queue.
    pub filename: String,
}
