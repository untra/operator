//! Kanban onboarding REST endpoints.
//!
//! Thin wrappers around `services::kanban_onboarding` — each handler
//! deserializes its DTO, delegates to the service, and serializes the
//! response. Business logic lives in the service module.

use axum::extract::State;
use axum::Json;

use crate::rest::dto::{
    ListKanbanProjectsRequest, ListKanbanProjectsResponse, SetKanbanSessionEnvRequest,
    SetKanbanSessionEnvResponse, ValidateKanbanCredentialsRequest,
    ValidateKanbanCredentialsResponse, WriteKanbanConfigRequest, WriteKanbanConfigResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::services::kanban_onboarding;

/// POST /`api/v1/kanban/validate`
///
/// Validate credentials against the live provider API without persisting
/// anything. Auth failures return `valid: false` with an `error` string
/// rather than a 4xx/5xx status so clients can display errors inline.
pub async fn validate_credentials(
    State(_state): State<ApiState>,
    Json(req): Json<ValidateKanbanCredentialsRequest>,
) -> Result<Json<ValidateKanbanCredentialsResponse>, ApiError> {
    let resp = kanban_onboarding::validate_credentials(req).await?;
    Ok(Json(resp))
}

/// POST /`api/v1/kanban/projects`
///
/// List available projects/teams for the given provider using ephemeral
/// credentials. No persistence side effects.
pub async fn list_projects(
    State(_state): State<ApiState>,
    Json(req): Json<ListKanbanProjectsRequest>,
) -> Result<Json<ListKanbanProjectsResponse>, ApiError> {
    let resp = kanban_onboarding::list_projects(req).await?;
    Ok(Json(resp))
}

/// PUT /`api/v1/kanban/config`
///
/// Write or upsert a kanban provider+project section into `config.toml`.
/// Does NOT receive the actual secret — only the env var name (`api_key_env`).
pub async fn write_config(
    State(_state): State<ApiState>,
    Json(req): Json<WriteKanbanConfigRequest>,
) -> Result<Json<WriteKanbanConfigResponse>, ApiError> {
    // Pass `None` so the service uses the production config path.
    let resp = kanban_onboarding::write_config(req, None)?;
    Ok(Json(resp))
}

/// POST /`api/v1/kanban/session-env`
///
/// Set kanban env vars on the server process for the current session so
/// subsequent `from_config()` calls find the API key. Returns a
/// `shell_export_block` with placeholder values for the client to display.
pub async fn set_session_env(
    State(_state): State<ApiState>,
    Json(req): Json<SetKanbanSessionEnvRequest>,
) -> Result<Json<SetKanbanSessionEnvResponse>, ApiError> {
    let resp = kanban_onboarding::set_session_env(req);
    Ok(Json(resp))
}
