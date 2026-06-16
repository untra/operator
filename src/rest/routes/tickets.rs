//! Ticket CRUD endpoints for the REST API.
//!
//! Provides endpoints for fetching ticket details and updating ticket status.
//! These endpoints power the embedded web UI's kanban board and detail drawer.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::queue::creator::TicketCreator;
use crate::queue::{Queue, Ticket};
use crate::rest::dto::{
    CreateAlertRequest, CreateAlertResponse, CreateTicketRequest, CreateTicketResponse,
    TicketDetailResponse, UpdateTicketStatusRequest, UpdateTicketStatusResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::templates::TemplateType;

/// Find a ticket across all directories (queue, in-progress, completed)
pub(crate) fn find_ticket_anywhere(queue: &Queue, ticket_id: &str) -> Result<Ticket, ApiError> {
    // find_ticket searches queue + in-progress
    if let Some(ticket) = queue
        .find_ticket(ticket_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
    {
        return Ok(ticket);
    }

    // Also search completed
    let completed = queue
        .list_completed()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    completed
        .into_iter()
        .find(|t| t.id == ticket_id || t.filename.contains(ticket_id))
        .ok_or_else(|| ApiError::NotFound(format!("Ticket '{ticket_id}' not found")))
}

/// Get full ticket details by ID
///
/// Returns complete ticket data including content, metadata, step history,
/// and session information. Searches queue, in-progress, and completed directories.
#[utoipa::path(
    operation_id = "tickets_get_one",
    get,
    path = "/api/v1/tickets/{id}",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID (e.g., FEAT-7598)")
    ),
    responses(
        (status = 200, description = "Ticket details", body = TicketDetailResponse),
        (status = 404, description = "Ticket not found")
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<TicketDetailResponse>, ApiError> {
    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let ticket = find_ticket_anywhere(&queue, &ticket_id)?;
    let step_display_name = ticket.current_step_display_name();

    Ok(Json(TicketDetailResponse {
        id: ticket.id,
        summary: ticket.summary,
        ticket_type: ticket.ticket_type,
        project: ticket.project,
        status: ticket.status,
        step: ticket.step,
        step_display_name: if step_display_name.is_empty() {
            None
        } else {
            Some(step_display_name)
        },
        priority: ticket.priority,
        timestamp: ticket.timestamp,
        content: ticket.content,
        filename: ticket.filename,
        filepath: ticket.filepath,
        sessions: ticket.sessions,
        step_delegators: ticket.step_delegators,
        worktree_path: ticket.worktree_path,
        branch: ticket.branch,
        external_id: ticket.external_id,
        external_url: ticket.external_url,
        external_provider: ticket.external_provider,
    }))
}

/// Update a ticket's status
///
/// Moves a ticket between queue directories based on the target status.
/// Valid transitions: queued, running, awaiting, done.
#[utoipa::path(
    operation_id = "tickets_update_status",
    put,
    path = "/api/v1/tickets/{id}/status",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID to update")
    ),
    request_body = UpdateTicketStatusRequest,
    responses(
        (status = 200, description = "Ticket status updated", body = UpdateTicketStatusResponse),
        (status = 400, description = "Invalid status value"),
        (status = 404, description = "Ticket not found")
    )
)]
pub async fn update_status(
    State(state): State<ApiState>,
    Path(ticket_id): Path<String>,
    Json(request): Json<UpdateTicketStatusRequest>,
) -> Result<Json<UpdateTicketStatusResponse>, ApiError> {
    let valid_statuses = ["queued", "running", "awaiting", "done"];
    if !valid_statuses.contains(&request.status.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: {}",
            request.status,
            valid_statuses.join(", ")
        )));
    }

    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let ticket = find_ticket_anywhere(&queue, &ticket_id)?;

    let previous_status = ticket.status.clone();
    let target_status = request.status.as_str();

    // Determine target directory
    let tickets_path = state.config.tickets_path();
    let dst_dir = match target_status {
        "queued" => tickets_path.join("queue"),
        "running" | "awaiting" => tickets_path.join("in-progress"),
        "done" => tickets_path.join("completed"),
        _ => unreachable!(),
    };

    let src = std::path::PathBuf::from(&ticket.filepath);
    let dst = dst_dir.join(&ticket.filename);

    // Ensure target directory exists
    std::fs::create_dir_all(&dst_dir)
        .map_err(|e| ApiError::InternalError(format!("Failed to create directory: {e}")))?;

    // Move the file if source and destination differ
    if src != dst {
        std::fs::rename(&src, &dst)
            .map_err(|e| ApiError::InternalError(format!("Failed to move ticket: {e}")))?;
    }

    // Update the status field in the ticket file
    if previous_status != target_status {
        let mut moved_ticket = Ticket::from_file(&dst)
            .map_err(|e| ApiError::InternalError(format!("Failed to reload ticket: {e}")))?;
        moved_ticket
            .update_field("status", target_status)
            .map_err(|e| ApiError::InternalError(format!("Failed to update status field: {e}")))?;
    }

    Ok(Json(UpdateTicketStatusResponse {
        id: ticket.id,
        previous_status,
        status: target_status.to_string(),
        message: format!("Ticket moved to '{target_status}'"),
    }))
}

/// Create a new ticket from a template and write it to the queue.
///
/// Reuses the same [`TicketCreator`] the CLI (`operator create`) and MCP
/// (`operator_create_ticket`) use, so a ticket created over HTTP is identical to
/// one created on any other surface. Powers the AGNT `operator-create-ticket`
/// node.
#[utoipa::path(
    operation_id = "tickets_create",
    post,
    path = "/api/v1/tickets",
    tag = "Tickets",
    request_body = CreateTicketRequest,
    responses(
        (status = 200, description = "Ticket created", body = CreateTicketResponse),
        (status = 400, description = "Unknown template type", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(request): Json<CreateTicketRequest>,
) -> Result<Json<CreateTicketResponse>, ApiError> {
    let CreateTicketRequest {
        template,
        project,
        summary,
        mut values,
    } = request;

    let template_type = TemplateType::from_key(&template)
        .ok_or_else(|| ApiError::BadRequest(format!("Unknown template type: {template}")))?;

    // Explicit fields take precedence over the same keys in `values`.
    if let Some(p) = project {
        values.insert("project".to_string(), p);
    }
    if let Some(s) = summary {
        values.insert("summary".to_string(), s);
    }

    let (ticket, path) = create_ticket_from_values(&state, template_type, values).await?;

    Ok(Json(CreateTicketResponse {
        id: ticket.id,
        filename: ticket.filename,
        path: path.to_string_lossy().into_owned(),
    }))
}

/// Shared ticket-creation core for the REST surface: overlays caller `values`
/// on top of the template's generated defaults (so the ticket gets a real id,
/// branch, etc.), writes it to the queue via the embedded-template
/// [`TicketCreator`], and reloads it. Used by both `create` and `create_alert`.
async fn create_ticket_from_values(
    state: &ApiState,
    template_type: TemplateType,
    values: HashMap<String, String>,
) -> Result<(Ticket, std::path::PathBuf), ApiError> {
    let config = (*state.config).clone();
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf, String> {
        let creator = TicketCreator::new(&config);
        let project = values.get("project").cloned().unwrap_or_default();
        // Start from generated defaults (id, created, branch, step, status),
        // then overlay the caller's values so explicit fields win.
        let mut merged = creator.generate_default_values(template_type, &project);
        merged.extend(values);
        creator
            .create_ticket_headless(template_type, &merged)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))?
    .map_err(ApiError::InternalError)?;

    let ticket = Ticket::from_file(&path).map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok((ticket, path))
}

/// Raise an external alert as an investigation (INV) ticket.
///
/// Creates an investigation through the same embedded-template path as
/// [`create`], folding the alert's `source`/`severity` into the ticket values.
/// Powers the AGNT `operator-alert` node.
#[utoipa::path(
    operation_id = "alerts_create",
    post,
    path = "/api/v1/alerts",
    tag = "Tickets",
    request_body = CreateAlertRequest,
    responses(
        (status = 200, description = "Investigation created", body = CreateAlertResponse),
        (status = 500, description = "Failed to create investigation", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn create_alert(
    State(state): State<ApiState>,
    Json(request): Json<CreateAlertRequest>,
) -> Result<Json<CreateAlertResponse>, ApiError> {
    let CreateAlertRequest {
        source,
        message,
        severity,
        project,
    } = request;

    let mut values = HashMap::new();
    values.insert("summary".to_string(), message);
    values.insert("source".to_string(), source);
    values.insert("severity".to_string(), severity);
    if let Some(p) = project {
        values.insert("project".to_string(), p);
    }

    let (ticket, _path) =
        create_ticket_from_values(&state, TemplateType::Investigation, values).await?;

    Ok(Json(CreateAlertResponse {
        id: ticket.id,
        filename: ticket.filename,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test-tickets"))
    }

    fn make_state_in(dir: &std::path::Path) -> ApiState {
        let mut config = Config::default();
        config.paths.tickets = dir.to_string_lossy().into_owned();
        ApiState::new(config, dir.to_path_buf())
    }

    #[test]
    fn test_valid_statuses() {
        let valid = ["queued", "running", "awaiting", "done"];
        for s in &valid {
            assert!(valid.contains(s));
        }
        assert!(!valid.contains(&"invalid"));
    }

    #[tokio::test]
    async fn test_get_ticket_not_found() {
        let state = make_state();
        let result = get_one(State(state), Path("NONEXISTENT-999".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_status_invalid() {
        let state = make_state();
        let request = UpdateTicketStatusRequest {
            status: "invalid".to_string(),
        };
        let result = update_status(State(state), Path("FEAT-001".to_string()), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_unknown_template_is_bad_request() {
        let state = make_state();
        let request = CreateTicketRequest {
            template: "not-a-template".to_string(),
            project: None,
            summary: None,
            values: std::collections::HashMap::new(),
        };
        let result = create(State(state), Json(request)).await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_create_ticket_writes_to_queue() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let request = CreateTicketRequest {
            template: "feat".to_string(),
            project: Some("gamesvc".to_string()),
            summary: Some("Add pagination".to_string()),
            values: std::collections::HashMap::new(),
        };
        let resp = create(State(state), Json(request))
            .await
            .expect("create should succeed")
            .0;
        assert!(!resp.id.is_empty(), "created ticket has an id");
        assert!(resp.filename.contains(".md"), "filename: {}", resp.filename);
        assert!(
            std::path::Path::new(&resp.path).exists(),
            "ticket file written to {}",
            resp.path
        );
    }

    #[tokio::test]
    async fn test_create_alert_creates_investigation() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let request = CreateAlertRequest {
            source: "pagerduty".to_string(),
            message: "500 errors in backend".to_string(),
            severity: "S1".to_string(),
            project: Some("gamesvc".to_string()),
        };
        let resp = create_alert(State(state), Json(request))
            .await
            .expect("alert should create an investigation")
            .0;
        assert!(!resp.id.is_empty(), "investigation has an id");
        assert!(resp.filename.contains(".md"), "filename: {}", resp.filename);
    }
}
