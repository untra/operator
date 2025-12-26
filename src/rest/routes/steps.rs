//! Step management endpoints.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::issuetypes::schema::IssueTypeSource;
use crate::rest::dto::{StepResponse, UpdateStepRequest};
use crate::rest::error::{ApiError, ErrorResponse};
use crate::rest::state::ApiState;
use crate::templates::schema::{PermissionMode, StepOutput};

/// List all steps for an issue type
#[utoipa::path(
    get,
    path = "/api/v1/issuetypes/{key}/steps",
    tag = "Steps",
    params(
        ("key" = String, Path, description = "Issue type key")
    ),
    responses(
        (status = 200, description = "List of steps", body = Vec<StepResponse>),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn list(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> Result<Json<Vec<StepResponse>>, ApiError> {
    let registry = state.registry.read().await;
    let issue_type = registry
        .get(&key.to_uppercase())
        .ok_or_else(|| ApiError::NotFound(format!("Issue type '{}' not found", key)))?;

    let steps: Vec<StepResponse> = issue_type.steps.iter().map(StepResponse::from).collect();
    Ok(Json(steps))
}

/// Get a single step by name
#[utoipa::path(
    get,
    path = "/api/v1/issuetypes/{key}/steps/{step_name}",
    tag = "Steps",
    params(
        ("key" = String, Path, description = "Issue type key"),
        ("step_name" = String, Path, description = "Step name")
    ),
    responses(
        (status = 200, description = "Step details", body = StepResponse),
        (status = 404, description = "Issue type or step not found", body = ErrorResponse)
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path((key, step_name)): Path<(String, String)>,
) -> Result<Json<StepResponse>, ApiError> {
    let registry = state.registry.read().await;
    let issue_type = registry
        .get(&key.to_uppercase())
        .ok_or_else(|| ApiError::NotFound(format!("Issue type '{}' not found", key)))?;

    let step = issue_type.get_step(&step_name).ok_or_else(|| {
        ApiError::NotFound(format!("Step '{}' not found in '{}'", step_name, key))
    })?;

    Ok(Json(StepResponse::from(step)))
}

/// Update a step
#[utoipa::path(
    put,
    path = "/api/v1/issuetypes/{key}/steps/{step_name}",
    tag = "Steps",
    params(
        ("key" = String, Path, description = "Issue type key"),
        ("step_name" = String, Path, description = "Step name")
    ),
    request_body = UpdateStepRequest,
    responses(
        (status = 200, description = "Step updated", body = StepResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Cannot modify builtin type", body = ErrorResponse),
        (status = 404, description = "Issue type or step not found", body = ErrorResponse)
    )
)]
pub async fn update(
    State(state): State<ApiState>,
    Path((key, step_name)): Path<(String, String)>,
    Json(request): Json<UpdateStepRequest>,
) -> Result<Json<StepResponse>, ApiError> {
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
            "Cannot modify steps in builtin issue type '{}'",
            key
        )));
    }

    // Find and update the step
    let step = issue_type
        .steps
        .iter_mut()
        .find(|s| s.name == step_name)
        .ok_or_else(|| {
            ApiError::NotFound(format!("Step '{}' not found in '{}'", step_name, key))
        })?;

    // Apply updates
    if let Some(display_name) = request.display_name {
        step.display_name = Some(display_name);
    }
    if let Some(prompt) = request.prompt {
        step.prompt = prompt;
    }
    if let Some(outputs) = request.outputs {
        step.outputs = outputs
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
            .collect();
    }
    if let Some(allowed_tools) = request.allowed_tools {
        step.allowed_tools = allowed_tools;
    }
    if let Some(requires_review) = request.requires_review {
        step.requires_review = requires_review;
    }
    if let Some(next_step) = request.next_step {
        step.next_step = Some(next_step);
    }
    if let Some(permission_mode) = request.permission_mode {
        step.permission_mode = match permission_mode.as_str() {
            "plan" => PermissionMode::Plan,
            "acceptEdits" => PermissionMode::AcceptEdits,
            "delegate" => PermissionMode::Delegate,
            _ => PermissionMode::Default,
        };
    }

    // Get updated step response before validation
    let updated_step = issue_type
        .get_step(&step_name)
        .ok_or_else(|| ApiError::InternalError("Step disappeared after update".to_string()))?
        .clone();

    // Validate the entire issue type
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
        .register(issue_type)
        .map_err(|e| ApiError::InternalError(format!("Failed to update issue type: {}", e)))?;

    Ok(Json(StepResponse::from(&updated_step)))
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
    async fn test_list_steps() {
        let state = make_state();
        let result = list(State(state), Path("FEAT".to_string())).await;
        assert!(result.is_ok());
        let steps = result.unwrap();
        assert!(!steps.0.is_empty());
    }

    #[tokio::test]
    async fn test_list_steps_not_found() {
        let state = make_state();
        let result = list(State(state), Path("NOTEXIST".to_string())).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_step_not_found() {
        let state = make_state();
        let result = get_one(
            State(state),
            Path(("FEAT".to_string(), "nonexistent".to_string())),
        )
        .await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }
}
