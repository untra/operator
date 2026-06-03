//! Workflow export endpoint for the REST API.
//!
//! Renders a ticket (against its issue type) into a Claude Code dynamic
//! workflow `.js`. This is the HTTP face of the same shared code path the CLI
//! and TUI use in-process (`workflow_gen::export_workflow_for_ticket`), so the
//! web UI and VS Code extension produce identical output.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::queue::Queue;
use crate::rest::dto::{WorkflowExportResponse, WorkflowPreviewResponse};
use crate::rest::error::ApiError;
use crate::rest::routes::tickets::find_ticket_anywhere;
use crate::rest::state::ApiState;

/// Export a ticket to a Claude dynamic workflow.
///
/// Resolves the ticket (searching queue, in-progress, and completed), looks up
/// its issue type in the registry, and returns the rendered `.js` plus a
/// suggested filename.
#[utoipa::path(
    operation_id = "workflow_export",
    post,
    path = "/api/v1/tickets/{id}/workflow-export",
    tag = "Workflow",
    params(
        ("id" = String, Path, description = "Ticket ID (e.g., FEAT-7598)")
    ),
    responses(
        (status = 200, description = "Generated workflow", body = WorkflowExportResponse),
        (status = 404, description = "Ticket or issue type not found", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn export(
    State(state): State<ApiState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<WorkflowExportResponse>, ApiError> {
    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;
    let ticket = find_ticket_anywhere(&queue, &ticket_id)?;

    let registry = state.registry.read().await;
    let exported =
        crate::workflow_gen::export_workflow_for_ticket(&ticket, &registry, None, &state.config)
            .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(exported.into()))
}

/// Preview the workflow for an issue type (no ticket required).
///
/// Renders the issue type's step structure into a Claude dynamic workflow using
/// placeholder field values. Used by the UI to visualize an issue type's
/// workflow shape.
#[utoipa::path(
    operation_id = "workflow_preview",
    get,
    path = "/api/v1/issuetypes/{key}/workflow-preview",
    tag = "Workflow",
    params(
        ("key" = String, Path, description = "Issue type key (e.g., FEAT, FIX)")
    ),
    responses(
        (status = 200, description = "Generated preview workflow", body = WorkflowPreviewResponse),
        (status = 404, description = "Issue type not found", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn preview(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> Result<Json<WorkflowPreviewResponse>, ApiError> {
    let registry = state.registry.read().await;
    let issuetype = registry
        .get(&key.to_uppercase())
        .ok_or_else(|| ApiError::NotFound(format!("Issue type '{key}' not found")))?;

    let exported = crate::workflow_gen::export_workflow_for_issuetype(issuetype)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(exported.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        ApiState::new(
            Config::default(),
            PathBuf::from("/tmp/test-tickets-workflow"),
        )
    }

    #[tokio::test]
    async fn export_unknown_ticket_is_not_found() {
        let state = make_state();
        let result = export(State(state), Path("NONEXISTENT-999".to_string())).await;
        assert!(result.is_err(), "unknown ticket should error");
    }

    #[tokio::test]
    async fn preview_known_issuetype_returns_workflow() {
        let state = make_state();
        // Lower-case key exercises the to_uppercase() normalization.
        let result = preview(State(state), Path("feat".to_string())).await;
        let body = result.expect("FEAT preview should resolve").0;
        assert_eq!(body.issuetype_key, "FEAT");
        assert!(body.contents.contains("export const meta"));
    }

    #[tokio::test]
    async fn preview_unknown_issuetype_is_not_found() {
        let state = make_state();
        let result = preview(State(state), Path("NOPE".to_string())).await;
        assert!(result.is_err(), "unknown issue type should error");
    }
}
