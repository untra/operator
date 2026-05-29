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
use crate::rest::dto::WorkflowExportResponse;
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
    let exported = crate::workflow_gen::export_workflow_for_ticket(&ticket, &registry, None)
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

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
}
