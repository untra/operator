//! MCP SSE transport for JSON-RPC communication.
//!
//! Implements the MCP SSE transport protocol:
//! - `GET /api/v1/mcp/sse` opens an SSE stream and sends the message endpoint URL
//! - `POST /api/v1/mcp/message?sessionId={id}` receives JSON-RPC requests and
//!   sends responses back through the SSE stream

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Host, Query, State};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use crate::mcp::tools;
use crate::rest::state::ApiState;

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// Query parameters for the message endpoint
#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

/// SSE endpoint — opens an event stream and sends the message endpoint URL
///
/// The client connects here first, receives the message endpoint URL,
/// then sends JSON-RPC requests to that endpoint.
pub async fn sse_handler(
    Host(host): Host,
    State(state): State<ApiState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::unbounded_channel::<String>();

    // Register session
    state
        .mcp_sessions
        .lock()
        .await
        .insert(session_id.clone(), tx);

    let message_url = format!("http://{host}/api/v1/mcp/message?sessionId={session_id}");

    let session_id_cleanup = session_id.clone();
    let sessions_cleanup = state.mcp_sessions.clone();

    // Build SSE stream: first event is the endpoint URL, then relay messages
    let endpoint_event = tokio_stream::once(Ok::<_, Infallible>(
        Event::default().event("endpoint").data(message_url),
    ));

    let message_stream = UnboundedReceiverStream::new(rx)
        .map(|msg| Ok::<_, Infallible>(Event::default().event("message").data(msg)));

    let combined = endpoint_event.chain(message_stream);

    // Clean up session after 1 hour or when stream ends
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        sessions_cleanup.lock().await.remove(&session_id_cleanup);
    });

    // Axum's KeepAlive handles keepalive pings automatically
    Sse::new(combined).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}

/// Message endpoint — receives JSON-RPC requests and sends responses via SSE
pub async fn message_handler(
    Query(query): Query<MessageQuery>,
    State(state): State<ApiState>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    // Clone the sender and drop the lock before async work
    let tx = {
        let sessions = state.mcp_sessions.lock().await;
        let Some(tx) = sessions.get(&query.session_id) else {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({"error": "Session not found"})),
            );
        };
        tx.clone()
    };

    let response = handle_jsonrpc(&request, &state).await;

    // Send response through SSE channel
    if let Ok(json_str) = serde_json::to_string(&response) {
        let _ = tx.send(json_str);
    }

    (axum::http::StatusCode::ACCEPTED, Json(json!({})))
}

/// Handle a JSON-RPC request and return the response
async fn handle_jsonrpc(request: &JsonRpcRequest, state: &ApiState) -> JsonRpcResponse {
    let id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
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
        assert_eq!(tools_arr.len(), 7);

        // Verify first tool has expected shape
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

        // Parse the text content to verify it's valid health JSON
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
