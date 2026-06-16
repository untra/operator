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

/// Request to create a new ticket from a template.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateTicketRequest {
    /// Template type key (feature, fix, spike, investigation, task).
    pub template: String,
    /// Project the ticket targets (filled into the template's `project` value).
    #[serde(default)]
    pub project: Option<String>,
    /// One-line summary (filled into the template's `summary` value).
    #[serde(default)]
    pub summary: Option<String>,
    /// Additional Handlebars values for the template. Explicit `project`/
    /// `summary` fields take precedence over the same keys here.
    #[serde(default)]
    pub values: HashMap<String, String>,
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
