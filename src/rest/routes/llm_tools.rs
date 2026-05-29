//! LLM tools endpoint.
//!
//! Returns detected CLI tools with their model aliases, capabilities,
//! and version information from the operator configuration.

use axum::extract::State;
use axum::Json;

use crate::config::Config;
use crate::rest::dto::{DefaultLlmResponse, LlmToolsResponse, SetDefaultLlmRequest};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// List detected LLM tools with model aliases
#[utoipa::path(
    operation_id = "llm_tools_list",
    get,
    path = "/api/v1/llm-tools",
    tag = "LLM Tools",
    responses(
        (status = 200, description = "List of detected LLM tools", body = LlmToolsResponse)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<LlmToolsResponse> {
    let tools = state.config.llm_tools.detected.clone();
    let total = tools.len();
    Json(LlmToolsResponse { tools, total })
}

/// Get the current default LLM tool and model
#[utoipa::path(
    operation_id = "llm_tools_get_default",
    get,
    path = "/api/v1/llm-tools/default",
    tag = "LLM Tools",
    responses(
        (status = 200, description = "Current default LLM", body = DefaultLlmResponse)
    )
)]
pub async fn get_default(State(state): State<ApiState>) -> Json<DefaultLlmResponse> {
    Json(DefaultLlmResponse {
        tool: state
            .config
            .llm_tools
            .default_tool
            .clone()
            .unwrap_or_default(),
        model: state
            .config
            .llm_tools
            .default_model
            .clone()
            .unwrap_or_default(),
    })
}

/// Set the global default LLM tool and model
#[utoipa::path(
    operation_id = "llm_tools_set_default",
    put,
    path = "/api/v1/llm-tools/default",
    tag = "LLM Tools",
    request_body = SetDefaultLlmRequest,
    responses(
        (status = 200, description = "Default LLM set", body = DefaultLlmResponse),
        (status = 404, description = "Tool not detected")
    )
)]
pub async fn set_default(
    State(state): State<ApiState>,
    Json(req): Json<SetDefaultLlmRequest>,
) -> Result<Json<DefaultLlmResponse>, ApiError> {
    if !state
        .config
        .llm_tools
        .detected
        .iter()
        .any(|t| t.name == req.tool)
    {
        return Err(ApiError::NotFound(format!(
            "Tool '{}' not detected",
            req.tool
        )));
    }

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.llm_tools.default_tool = Some(req.tool.clone());
    config.llm_tools.default_model = Some(req.model.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(DefaultLlmResponse {
        tool: req.tool,
        model: req.model,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DetectedTool, ToolCapabilities};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_empty() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 0);
        assert!(resp.tools.is_empty());
    }

    #[tokio::test]
    async fn test_list_with_tools() {
        let mut config = Config::default();
        config.llm_tools.detected.push(DetectedTool {
            name: "claude".to_string(),
            path: "/usr/local/bin/claude".to_string(),
            version: "2.5.0".to_string(),
            min_version: Some("2.1.0".to_string()),
            version_ok: true,
            model_aliases: vec![
                "opus".to_string(),
                "sonnet".to_string(),
                "haiku".to_string(),
            ],
            command_template: String::new(),
            capabilities: ToolCapabilities::default(),
            yolo_flags: vec![],
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 1);
        assert_eq!(resp.tools[0].name, "claude");
        assert_eq!(resp.tools[0].model_aliases.len(), 3);
    }
}
