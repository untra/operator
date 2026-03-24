//! MCP tool definitions and execution.
//!
//! Defines read-only tools that wrap existing REST API route handlers.
//! Each tool calls the handler directly (no internal HTTP round-trip).

use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::rest::routes;
use crate::rest::state::ApiState;

/// Definition of a single MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Returns all available MCP tool definitions
pub fn all_tool_definitions() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "operator_health".to_string(),
            description: "Check Operator API health status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_status".to_string(),
            description: "Get Operator server status including registry info (issue type count, collection count, active collection)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_list_issue_types".to_string(),
            description: "List all available issue types (FEAT, FIX, SPIKE, etc.)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_get_issue_type".to_string(),
            description: "Get detailed information about a specific issue type by key".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Issue type key (e.g., FEAT, FIX, SPIKE)"
                    }
                },
                "required": ["key"]
            }),
        },
        McpToolDefinition {
            name: "operator_list_collections".to_string(),
            description: "List all issue type collections".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_get_collection".to_string(),
            description: "Get detailed information about a specific collection by name".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["name"]
            }),
        },
        McpToolDefinition {
            name: "operator_list_skills".to_string(),
            description: "List all discovered skills across LLM tools".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

/// Execute an MCP tool by name with the given arguments
pub async fn execute_tool(name: &str, args: Value, state: &ApiState) -> Result<Value, String> {
    match name {
        "operator_health" => {
            let resp = routes::health::health().await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_status" => {
            let resp = routes::health::status(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_list_issue_types" => {
            let resp = routes::issuetypes::list(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_get_issue_type" => {
            let key = args
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: key".to_string())?;
            let result =
                routes::issuetypes::get_one(State(state.clone()), Path(key.to_string())).await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(_e) => Err(format!("Issue type '{key}' not found")),
            }
        }
        "operator_list_collections" => {
            let resp = routes::collections::list(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_get_collection" => {
            let name_param = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: name".to_string())?;
            let result =
                routes::collections::get_one(State(state.clone()), Path(name_param.to_string()))
                    .await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(_e) => Err(format!("Collection '{name_param}' not found")),
            }
        }
        "operator_list_skills" => {
            let resp = routes::skills::list(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown tool: {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_all_tool_definitions_count() {
        let tools = all_tool_definitions();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_all_tool_definitions_names() {
        let tools = all_tool_definitions();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"operator_health"));
        assert!(names.contains(&"operator_status"));
        assert!(names.contains(&"operator_list_issue_types"));
        assert!(names.contains(&"operator_get_issue_type"));
        assert!(names.contains(&"operator_list_collections"));
        assert!(names.contains(&"operator_get_collection"));
        assert!(names.contains(&"operator_list_skills"));
    }

    #[test]
    fn test_tool_schemas_are_valid_objects() {
        let tools = all_tool_definitions();
        for tool in &tools {
            assert!(
                tool.input_schema.is_object(),
                "Tool '{}' schema should be an object",
                tool.name
            );
            assert_eq!(
                tool.input_schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Tool '{}' schema type should be 'object'",
                tool.name
            );
        }
    }

    #[test]
    fn test_parameterized_tools_have_required_fields() {
        let tools = all_tool_definitions();

        let get_issue_type = tools
            .iter()
            .find(|t| t.name == "operator_get_issue_type")
            .unwrap();
        let required = get_issue_type.input_schema.get("required").unwrap();
        assert!(required.as_array().unwrap().contains(&json!("key")));

        let get_collection = tools
            .iter()
            .find(|t| t.name == "operator_get_collection")
            .unwrap();
        let required = get_collection.input_schema.get("required").unwrap();
        assert!(required.as_array().unwrap().contains(&json!("name")));
    }

    #[tokio::test]
    async fn test_execute_health() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_health", json!({}), &state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn test_execute_status() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_status", json!({}), &state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn test_execute_list_issue_types() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_list_issue_types", json!({}), &state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_array());
        assert!(value.as_array().unwrap().len() >= 5); // At least builtins
    }

    #[tokio::test]
    async fn test_execute_get_issue_type_missing_key() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_get_issue_type", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required parameter"));
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("nonexistent_tool", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }
}
