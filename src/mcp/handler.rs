//! Transport-agnostic JSON-RPC handler for MCP.
//!
//! Both the HTTP/SSE transport (`transport.rs`) and the stdio transport
//! (`stdio.rs`) dispatch through `handle_jsonrpc`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mcp::tools;
use crate::rest::state::ApiState;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

pub async fn handle_jsonrpc(request: &JsonRpcRequest, state: &ApiState) -> JsonRpcResponse {
    let id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": { "subscribe": false, "listChanged": false }
                },
                "serverInfo": {
                    "name": "operator",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        },

        "notifications/initialized" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        },

        "tools/list" => {
            let tool_defs = tools::all_tool_definitions();
            let tools_json: Vec<Value> = tool_defs
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema
                    })
                })
                .collect();

            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "tools": tools_json })),
                error: None,
            }
        }

        "tools/call" => {
            let tool_name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            match tools::execute_tool(tool_name, arguments, state).await {
                Ok(result) => {
                    let text = serde_json::to_string_pretty(&result).unwrap_or_default();
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [{
                                "type": "text",
                                "text": text
                            }]
                        })),
                        error: None,
                    }
                }
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: e,
                    }),
                },
            }
        }

        "resources/list" => {
            let resources = crate::mcp::resources::list_resources(state)
                .await
                .unwrap_or_default();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "resources": resources })),
                error: None,
            }
        }

        "resources/read" => {
            let uri = request
                .params
                .get("uri")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match crate::mcp::resources::read_resource(uri, state).await {
                Ok(contents) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "text/markdown",
                            "text": contents
                        }]
                    })),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: e,
                    }),
                },
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn test_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test"))
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let state = test_state();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: json!({}),
        };

        let response = handle_jsonrpc(&request, &state).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(1));
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["capabilities"]["resources"].is_object());
        assert_eq!(result["serverInfo"]["name"], "operator");
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let state = test_state();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: json!({}),
        };

        let response = handle_jsonrpc(&request, &state).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools_arr = result["tools"].as_array().unwrap();
        assert_eq!(tools_arr.len(), 12);

        let first = &tools_arr[0];
        assert!(first.get("name").is_some());
        assert!(first.get("description").is_some());
        assert!(first.get("inputSchema").is_some());
    }

    #[tokio::test]
    async fn test_handle_tools_call_health() {
        let state = test_state();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: json!({
                "name": "operator_health",
                "arguments": {}
            }),
        };

        let response = handle_jsonrpc(&request, &state).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");

        let text = content[0]["text"].as_str().unwrap();
        let health: Value = serde_json::from_str(text).unwrap();
        assert_eq!(health["status"], "ok");
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown() {
        let state = test_state();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: json!({
                "name": "nonexistent",
                "arguments": {}
            }),
        };

        let response = handle_jsonrpc(&request, &state).await;

        assert!(response.error.is_some());
        assert!(response.error.unwrap().message.contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let state = test_state();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(5)),
            method: "unknown/method".to_string(),
            params: json!({}),
        };

        let response = handle_jsonrpc(&request, &state).await;

        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_notifications_initialized() {
        let state = test_state();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(6)),
            method: "notifications/initialized".to_string(),
            params: json!({}),
        };

        let response = handle_jsonrpc(&request, &state).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }
}
