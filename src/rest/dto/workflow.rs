//! DTOs for the workflow-export endpoint.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::workflow_gen::ExportedWorkflow;

/// Response for exporting a ticket to a Claude dynamic workflow (`.js`).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WorkflowExportResponse {
    /// The ticket the workflow was generated from.
    pub ticket_id: String,
    /// The issue type key that supplied the step structure.
    pub issuetype_key: String,
    /// Suggested filename for saving the workflow (`<ticket-id>.workflow.js`).
    pub suggested_filename: String,
    /// The generated `.js` workflow source.
    pub contents: String,
}

impl From<ExportedWorkflow> for WorkflowExportResponse {
    fn from(e: ExportedWorkflow) -> Self {
        Self {
            ticket_id: e.ticket_id,
            issuetype_key: e.issuetype_key,
            suggested_filename: e.suggested_filename,
            contents: e.contents,
        }
    }
}
