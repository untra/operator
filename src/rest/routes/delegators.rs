//! Delegator CRUD endpoints.
//!
//! Manages agent delegator configurations that define named {tool, model}
//! pairings for autonomous ticket launching.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::config::{Config, Delegator, DelegatorLaunchConfig};
use crate::rest::dto::{
    CreateDelegatorRequest, DelegatorLaunchConfigDto, DelegatorResponse, DelegatorsResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// List all configured delegators
#[utoipa::path(
    get,
    path = "/api/v1/delegators",
    tag = "Delegators",
    responses(
        (status = 200, description = "List of delegators", body = DelegatorsResponse)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<DelegatorsResponse> {
    let delegators: Vec<DelegatorResponse> = state
        .config
        .delegators
        .iter()
        .map(delegator_to_response)
        .collect();
    let total = delegators.len();
    Json(DelegatorsResponse { delegators, total })
}

/// Get a single delegator by name
#[utoipa::path(
    get,
    path = "/api/v1/delegators/{name}",
    tag = "Delegators",
    params(
        ("name" = String, Path, description = "Delegator name")
    ),
    responses(
        (status = 200, description = "Delegator details", body = DelegatorResponse),
        (status = 404, description = "Delegator not found")
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    let delegator = state
        .config
        .delegators
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Delegator '{}' not found", name)))?;

    Ok(Json(delegator_to_response(delegator)))
}

/// Create a new delegator
#[utoipa::path(
    post,
    path = "/api/v1/delegators",
    tag = "Delegators",
    request_body = CreateDelegatorRequest,
    responses(
        (status = 200, description = "Delegator created", body = DelegatorResponse),
        (status = 409, description = "Delegator already exists")
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(req): Json<CreateDelegatorRequest>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    // Check for duplicate name
    if state.config.delegators.iter().any(|d| d.name == req.name) {
        return Err(ApiError::Conflict(format!(
            "Delegator '{}' already exists",
            req.name
        )));
    }

    let delegator = Delegator {
        name: req.name,
        llm_tool: req.llm_tool,
        model: req.model,
        display_name: req.display_name,
        model_properties: req.model_properties,
        launch_config: req.launch_config.map(|lc| DelegatorLaunchConfig {
            yolo: lc.yolo,
            permission_mode: lc.permission_mode,
            flags: lc.flags,
        }),
    };

    // Read current config, add delegator, save
    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.delegators.push(delegator.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {}", e)))?;

    Ok(Json(delegator_to_response(&delegator)))
}

/// Delete a delegator by name
#[utoipa::path(
    delete,
    path = "/api/v1/delegators/{name}",
    tag = "Delegators",
    params(
        ("name" = String, Path, description = "Delegator name")
    ),
    responses(
        (status = 200, description = "Delegator deleted", body = DelegatorResponse),
        (status = 404, description = "Delegator not found")
    )
)]
pub async fn delete(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    // Find the delegator first for the response
    let delegator = state
        .config
        .delegators
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Delegator '{}' not found", name)))?;
    let response = delegator_to_response(delegator);

    // Read current config, remove delegator, save
    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.delegators.retain(|d| d.name != name);
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {}", e)))?;

    Ok(Json(response))
}

/// Convert a Delegator config to a DelegatorResponse DTO
fn delegator_to_response(d: &Delegator) -> DelegatorResponse {
    DelegatorResponse {
        name: d.name.clone(),
        llm_tool: d.llm_tool.clone(),
        model: d.model.clone(),
        display_name: d.display_name.clone(),
        model_properties: d.model_properties.clone(),
        launch_config: d.launch_config.as_ref().map(|lc| DelegatorLaunchConfigDto {
            yolo: lc.yolo,
            permission_mode: lc.permission_mode.clone(),
            flags: lc.flags.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_empty() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 0);
        assert!(resp.delegators.is_empty());
    }

    #[tokio::test]
    async fn test_list_with_delegators() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "test-delegator".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Test".to_string()),
            model_properties: std::collections::HashMap::new(),
            launch_config: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 1);
        assert_eq!(resp.delegators[0].name, "test-delegator");
    }

    #[tokio::test]
    async fn test_get_one_not_found() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = get_one(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_one_found() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "my-delegator".to_string(),
            llm_tool: "codex".to_string(),
            model: "gpt-4o".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            launch_config: Some(DelegatorLaunchConfig {
                yolo: true,
                permission_mode: None,
                flags: vec![],
            }),
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = get_one(State(state), Path("my-delegator".to_string())).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.name, "my-delegator");
        assert_eq!(resp.llm_tool, "codex");
        assert!(resp.launch_config.as_ref().unwrap().yolo);
    }
}
