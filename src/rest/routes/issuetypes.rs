//! Issue type CRUD endpoints.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::issuetypes::schema::IssueTypeSource;
use crate::rest::dto::{
    CreateIssueTypeRequest, IssueTypeResponse, IssueTypeSummary, UpdateIssueTypeRequest,
};
use crate::rest::error::{ApiError, ErrorResponse};
use crate::rest::state::ApiState;

/// List all issue types
#[utoipa::path(
    get,
    path = "/api/v1/issuetypes",
    tag = "Issue Types",
    responses(
        (status = 200, description = "List of all issue types", body = Vec<IssueTypeSummary>)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<Vec<IssueTypeSummary>> {
    let registry = state.registry.read().await;
    let types: Vec<IssueTypeSummary> = registry.all_types().map(IssueTypeSummary::from).collect();
    Json(types)
}

/// Get a single issue type by key
#[utoipa::path(
    get,
    path = "/api/v1/issuetypes/{key}",
    tag = "Issue Types",
    params(
        ("key" = String, Path, description = "Issue type key (e.g., FEAT, FIX)")
    ),
    responses(
        (status = 200, description = "Issue type details", body = IssueTypeResponse),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> Result<Json<IssueTypeResponse>, ApiError> {
    let registry = state.registry.read().await;
    let issue_type = registry
        .get(&key.to_uppercase())
        .ok_or_else(|| ApiError::NotFound(format!("Issue type '{}' not found", key)))?;

    Ok(Json(IssueTypeResponse::from(issue_type)))
}

/// Create a new issue type
#[utoipa::path(
    post,
    path = "/api/v1/issuetypes",
    tag = "Issue Types",
    request_body = CreateIssueTypeRequest,
    responses(
        (status = 200, description = "Issue type created", body = IssueTypeResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Issue type already exists", body = ErrorResponse)
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(request): Json<CreateIssueTypeRequest>,
) -> Result<Json<IssueTypeResponse>, ApiError> {
    let issue_type = request.into_issue_type();

    // Validate the issue type
    issue_type.validate().map_err(|errors| {
        let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        ApiError::ValidationError(msgs.join("; "))
    })?;

    // Check if key already exists
    {
        let registry = state.registry.read().await;
        if registry.get(&issue_type.key).is_some() {
            return Err(ApiError::Conflict(format!(
                "Issue type '{}' already exists",
                issue_type.key
            )));
        }
    }

    // Persist to filesystem
    state.ensure_issuetypes_dir().await?;
    let filepath = state
        .issuetypes_path()
        .join(format!("{}.json", issue_type.key));
    let json = issue_type.to_json()?;
    tokio::fs::write(&filepath, json).await?;

    // Register in memory
    let mut registry = state.registry.write().await;
    registry
        .register(issue_type.clone())
        .map_err(|e| ApiError::InternalError(format!("Failed to register issue type: {}", e)))?;

    Ok(Json(IssueTypeResponse::from(&issue_type)))
}

/// Update an existing issue type
#[utoipa::path(
    put,
    path = "/api/v1/issuetypes/{key}",
    tag = "Issue Types",
    params(
        ("key" = String, Path, description = "Issue type key")
    ),
    request_body = UpdateIssueTypeRequest,
    responses(
        (status = 200, description = "Issue type updated", body = IssueTypeResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Cannot modify builtin type", body = ErrorResponse),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn update(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    Json(request): Json<UpdateIssueTypeRequest>,
) -> Result<Json<IssueTypeResponse>, ApiError> {
    let key = key.to_uppercase();

    // Get existing issue type
    let mut issue_type = {
        let registry = state.registry.read().await;
        registry
            .get(&key)
            .ok_or_else(|| ApiError::NotFound(format!("Issue type '{}' not found", key)))?
            .clone()
    };

    // Check if it's a builtin
    if matches!(issue_type.source, IssueTypeSource::Builtin) {
        return Err(ApiError::BuiltinReadOnly(format!(
            "Cannot modify builtin issue type '{}'",
            key
        )));
    }

    // Apply updates
    if let Some(name) = request.name {
        issue_type.name = name;
    }
    if let Some(description) = request.description {
        issue_type.description = description;
    }
    if let Some(mode) = request.mode {
        issue_type.mode = if mode == "paired" {
            crate::templates::schema::ExecutionMode::Paired
        } else {
            crate::templates::schema::ExecutionMode::Autonomous
        };
    }
    if let Some(glyph) = request.glyph {
        issue_type.glyph = glyph;
    }
    if let Some(color) = request.color {
        issue_type.color = Some(color);
    }
    if let Some(project_required) = request.project_required {
        issue_type.project_required = project_required;
    }
    if let Some(fields) = request.fields {
        issue_type.fields = fields.into_iter().map(|f| f.into()).collect();
    }
    if let Some(steps) = request.steps {
        issue_type.steps = steps.into_iter().map(|s| s.into()).collect();
    }

    // Validate updated issue type
    issue_type.validate().map_err(|errors| {
        let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        ApiError::ValidationError(msgs.join("; "))
    })?;

    // Persist to filesystem
    let filepath = state.issuetypes_path().join(format!("{}.json", key));
    let json = issue_type.to_json()?;
    tokio::fs::write(&filepath, json).await?;

    // Update in memory
    let mut registry = state.registry.write().await;
    registry
        .register(issue_type.clone())
        .map_err(|e| ApiError::InternalError(format!("Failed to update issue type: {}", e)))?;

    Ok(Json(IssueTypeResponse::from(&issue_type)))
}

/// Delete an issue type
#[utoipa::path(
    delete,
    path = "/api/v1/issuetypes/{key}",
    tag = "Issue Types",
    params(
        ("key" = String, Path, description = "Issue type key")
    ),
    responses(
        (status = 200, description = "Issue type deleted"),
        (status = 403, description = "Cannot delete builtin type", body = ErrorResponse),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn delete(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key = key.to_uppercase();

    // Check if it exists and is not builtin
    {
        let registry = state.registry.read().await;
        let issue_type = registry
            .get(&key)
            .ok_or_else(|| ApiError::NotFound(format!("Issue type '{}' not found", key)))?;

        if matches!(issue_type.source, IssueTypeSource::Builtin) {
            return Err(ApiError::BuiltinReadOnly(format!(
                "Cannot delete builtin issue type '{}'",
                key
            )));
        }
    }

    // Delete from filesystem
    let filepath = state.issuetypes_path().join(format!("{}.json", key));
    if filepath.exists() {
        tokio::fs::remove_file(&filepath).await?;
    }

    // Note: We can't remove from registry without exposing a remove method,
    // but the file is deleted so it won't be loaded on next restart.
    // For full removal, user would need to restart the API server.

    Ok(Json(serde_json::json!({
        "deleted": key,
        "message": "Issue type deleted. Restart API for full removal from memory."
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test"))
    }

    #[tokio::test]
    async fn test_list() {
        let state = make_state();
        let resp = list(State(state)).await;
        assert!(!resp.0.is_empty());
    }

    #[tokio::test]
    async fn test_get_one_exists() {
        let state = make_state();
        let result = get_one(State(state), Path("FEAT".to_string())).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.key, "FEAT");
    }

    #[tokio::test]
    async fn test_get_one_not_found() {
        let state = make_state();
        let result = get_one(State(state), Path("NOTEXIST".to_string())).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }
}
