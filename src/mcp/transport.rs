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
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use crate::mcp::handler::{handle_jsonrpc, JsonRpcRequest};
use crate::rest::state::ApiState;

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
