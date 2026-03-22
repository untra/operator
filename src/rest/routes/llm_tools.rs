//! LLM tools endpoint.
//!
//! Returns detected CLI tools with their model aliases, capabilities,
//! and version information from the operator configuration.

use axum::extract::State;
use axum::Json;

use crate::rest::dto::LlmToolsResponse;
use crate::rest::state::ApiState;

/// List detected LLM tools with model aliases
#[utoipa::path(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DetectedTool};
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
            capabilities: Default::default(),
            yolo_flags: vec![],
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 1);
        assert_eq!(resp.tools[0].name, "claude");
        assert_eq!(resp.tools[0].model_aliases.len(), 3);
    }
}
