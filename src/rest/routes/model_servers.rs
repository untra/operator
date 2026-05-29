//! Model server CRUD endpoints.
//!
//! A model server is a named host that serves models via an inference API
//! (ollama, lmstudio, vllm, any OpenAI-compatible endpoint). Implicit builtin
//! servers (`anthropic-api`, `openai-api`, `google-api`) are returned on list
//! but cannot be created, updated, or deleted.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::config::{implicit_model_server_for_tool, Config, ModelServer};
use crate::rest::dto::{CreateModelServerRequest, ModelServerResponse, ModelServersResponse};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

const IMPLICIT_TOOL_NAMES: &[&str] = &["claude", "codex", "gemini"];

fn server_to_response(s: &ModelServer, user_declared: bool) -> ModelServerResponse {
    ModelServerResponse {
        name: s.name.clone(),
        kind: s.kind.clone(),
        base_url: s.base_url.clone(),
        api_key_env: s.api_key_env.clone(),
        extra_env: s.extra_env.clone(),
        display_name: s.display_name.clone(),
        user_declared,
    }
}

/// List all model servers (user-declared + implicit builtins)
#[utoipa::path(
    operation_id = "model_servers_list",
    get,
    path = "/api/v1/model-servers",
    tag = "ModelServers",
    responses(
        (status = 200, description = "List of model servers", body = ModelServersResponse)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<ModelServersResponse> {
    let mut servers: Vec<ModelServerResponse> = state
        .config
        .model_servers
        .iter()
        .map(|s| server_to_response(s, true))
        .collect();

    for tool in IMPLICIT_TOOL_NAMES {
        let implicit = implicit_model_server_for_tool(tool);
        if !servers.iter().any(|s| s.name == implicit.name) {
            servers.push(server_to_response(&implicit, false));
        }
    }

    let total = servers.len();
    Json(ModelServersResponse { servers, total })
}

/// Get a single model server by name
#[utoipa::path(
    operation_id = "model_servers_get_one",
    get,
    path = "/api/v1/model-servers/{name}",
    tag = "ModelServers",
    params(
        ("name" = String, Path, description = "Model server name")
    ),
    responses(
        (status = 200, description = "Model server details", body = ModelServerResponse),
        (status = 404, description = "Model server not found")
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if let Some(server) = state.config.model_servers.iter().find(|s| s.name == name) {
        return Ok(Json(server_to_response(server, true)));
    }
    for tool in IMPLICIT_TOOL_NAMES {
        let implicit = implicit_model_server_for_tool(tool);
        if implicit.name == name {
            return Ok(Json(server_to_response(&implicit, false)));
        }
    }
    Err(ApiError::NotFound(format!(
        "Model server '{name}' not found"
    )))
}

/// Create a new model server
#[utoipa::path(
    operation_id = "model_servers_create",
    post,
    path = "/api/v1/model-servers",
    tag = "ModelServers",
    request_body = CreateModelServerRequest,
    responses(
        (status = 200, description = "Model server created", body = ModelServerResponse),
        (status = 409, description = "Model server already exists")
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(req): Json<CreateModelServerRequest>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if state
        .config
        .model_servers
        .iter()
        .any(|s| s.name == req.name)
    {
        return Err(ApiError::Conflict(format!(
            "Model server '{}' already exists",
            req.name
        )));
    }
    if IMPLICIT_TOOL_NAMES
        .iter()
        .any(|t| implicit_model_server_for_tool(t).name == req.name)
    {
        return Err(ApiError::Conflict(format!(
            "'{}' is a reserved implicit builtin name",
            req.name
        )));
    }

    let server = ModelServer {
        name: req.name,
        kind: req.kind,
        base_url: req.base_url,
        api_key_env: req.api_key_env,
        extra_env: req.extra_env,
        display_name: req.display_name,
    };

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.model_servers.push(server.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(server_to_response(&server, true)))
}

/// Delete a user-declared model server by name
///
/// Implicit builtin servers cannot be deleted.
#[utoipa::path(
    operation_id = "model_servers_delete",
    delete,
    path = "/api/v1/model-servers/{name}",
    tag = "ModelServers",
    params(
        ("name" = String, Path, description = "Model server name")
    ),
    responses(
        (status = 200, description = "Model server deleted", body = ModelServerResponse),
        (status = 404, description = "Model server not found"),
        (status = 409, description = "Cannot delete implicit builtin server")
    )
)]
pub async fn delete(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if IMPLICIT_TOOL_NAMES
        .iter()
        .any(|t| implicit_model_server_for_tool(t).name == name)
    {
        return Err(ApiError::Conflict(format!(
            "'{name}' is an implicit builtin and cannot be deleted"
        )));
    }

    let server = state
        .config
        .model_servers
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Model server '{name}' not found")))?
        .clone();

    let response = server_to_response(&server, true);

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.model_servers.retain(|s| s.name != name);
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_returns_builtins_when_no_user_servers() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms"));
        let resp = list(State(state)).await;

        // At minimum, one implicit server for each known tool.
        assert!(resp.total >= IMPLICIT_TOOL_NAMES.len());
        assert!(resp.servers.iter().any(|s| s.name == "anthropic-api"));
        assert!(resp.servers.iter().any(|s| s.name == "openai-api"));
        assert!(resp.servers.iter().any(|s| s.name == "google-api"));
        assert!(resp.servers.iter().all(|s| !s.user_declared));
    }

    #[tokio::test]
    async fn test_list_includes_user_declared_servers() {
        let mut config = Config::default();
        config.model_servers.push(ModelServer {
            name: "ollama-local".to_string(),
            kind: "ollama".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: Some("Ollama (local)".to_string()),
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-user"));
        let resp = list(State(state)).await;

        let ollama = resp
            .servers
            .iter()
            .find(|s| s.name == "ollama-local")
            .expect("ollama-local should appear");
        assert!(ollama.user_declared);
        assert_eq!(ollama.kind, "ollama");
    }

    #[tokio::test]
    async fn test_get_one_returns_implicit_builtin() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-get"));
        let resp = get_one(State(state), Path("openai-api".to_string())).await;
        let server = resp.expect("implicit openai-api should resolve").0;
        assert_eq!(server.name, "openai-api");
        assert!(!server.user_declared);
    }

    #[tokio::test]
    async fn test_get_one_404_on_unknown() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-404"));
        let resp = get_one(State(state), Path("nope".to_string())).await;
        assert!(resp.is_err());
    }
}
