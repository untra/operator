//! DTOs for the workflow-export endpoint.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::integrations::{catalog, SupportStatus, Vertical};
use crate::workflow_gen::{ExportedWorkflow, WorkflowFormat};

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

/// Response for a *preview* workflow generated from an issue type alone (no
/// concrete ticket). Used by the UI to visualize an issue type's workflow shape.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WorkflowPreviewResponse {
    /// The issue type key the preview was generated from.
    pub issuetype_key: String,
    /// Suggested filename for saving the preview (`<KEY>.preview.workflow.js`).
    pub suggested_filename: String,
    /// The generated `.js` workflow source (placeholder ticket values).
    pub contents: String,
}

impl From<ExportedWorkflow> for WorkflowPreviewResponse {
    fn from(e: ExportedWorkflow) -> Self {
        Self {
            issuetype_key: e.issuetype_key,
            suggested_filename: e.suggested_filename,
            contents: e.contents,
        }
    }
}

/// One workflow export format operator can emit, for `GET /api/v1/workflow-formats`.
///
/// A projection of [`WorkflowFormat`] joined to its `Workflows` catalog entry —
/// the single source of truth for the format's [`SupportStatus`] and docs. Lets
/// the UIs render a format picker without hardcoding the list.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WorkflowFormatDto {
    /// Stable slug (e.g. "claude", "agnt") — the value the `format` query param takes.
    pub slug: String,
    /// Display label (e.g. "Claude Workflow").
    pub label: String,
    /// File extension of the emitted artifact, no leading dot (e.g. "js", "json").
    pub extension: String,
    /// Official support / maturity status (from the catalog).
    pub status: SupportStatus,
    /// Absolute docs URL, or `null` if undocumented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

/// Project every [`WorkflowFormat`] into a wire DTO, joining each to its
/// `Workflows` catalog entry for status + docs.
pub fn workflow_formats() -> Vec<WorkflowFormatDto> {
    WorkflowFormat::ALL
        .into_iter()
        .map(|f| {
            let entry = catalog::entry_for(Vertical::Workflows, f.slug());
            WorkflowFormatDto {
                slug: f.slug().to_string(),
                label: f.label().to_string(),
                extension: f.extension().to_string(),
                status: entry
                    .as_ref()
                    .map(|e| e.status)
                    .unwrap_or(SupportStatus::Proto),
                docs_url: entry.and_then(|e| e.docs_url()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_formats_projects_every_variant() {
        let dtos = workflow_formats();
        assert_eq!(dtos.len(), WorkflowFormat::ALL.len());
        let claude = dtos.iter().find(|d| d.slug == "claude").unwrap();
        assert_eq!(claude.extension, "js");
        assert_eq!(claude.status, SupportStatus::Ga);
        assert_eq!(
            claude.docs_url.as_deref(),
            Some("https://operator.untra.io/getting-started/workflows/claude/")
        );
        let agnt = dtos.iter().find(|d| d.slug == "agnt").unwrap();
        assert_eq!(agnt.extension, "json");
        assert_eq!(agnt.status, SupportStatus::Alpha);
    }
}
