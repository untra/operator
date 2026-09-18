//! MCP SSE transport for JSON-RPC communication.
//!
//! Implements the MCP SSE transport protocol:
//! - `GET /api/v1/mcp/sse` opens an SSE stream and sends the message endpoint URL
//! - `POST /api/v1/mcp/message?sessionId={id}` receives JSON-RPC requests and
//!   sends responses back through the SSE stream

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use super::Host;
use axum::extract::{Query, State};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use crate::mcp::handler::{handle_jsonrpc, JsonRpcRequest};
use crate::mcp::profile_api_base;
use crate::rest::middleware::auth::Authenticated;
use crate::rest::state::{ApiState, McpSession};

/// Query parameters for the message endpoint
#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

/// SSE endpoint - opens an event stream and sends the message endpoint URL
///
/// The client connects here first, receives the message endpoint URL,
/// then sends JSON-RPC requests to that endpoint.
#[utoipa::path(
    get,
    path = "/api/v1/mcp/sse",
    tag = "MCP",
    operation_id = "mcp_sse",
    responses((status = 200, description = "SSE stream carrying the message endpoint and JSON-RPC responses", content_type = "text/event-stream", body = String))
)]
pub async fn sse_handler(
    Host(host): Host,
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::unbounded_channel::<String>();

    // Bind the session to whoever opened it. The session id travels in a URL
    // and is therefore a bearer credential; recording the principal means a
    // leaked id is not enough on its own to drive the session.
    state.mcp_sessions.lock().await.insert(
        session_id.clone(),
        McpSession {
            tx,
            subject: principal.subject.clone(),
            scopes: principal.scopes.clone(),
        },
    );

    // Generated from the configured public URL, not the request `Host` header,
    // which a caller controls and which is plain `http` behind TLS termination.
    let base = profile_api_base(&state, &host);
    let message_url = format!("{base}/mcp/message?sessionId={session_id}");

    let session_id_cleanup = session_id.clone();
    let sessions_cleanup = Arc::clone(&state.mcp_sessions);

    // Build SSE stream: first event is the endpoint URL, then relay messages
    let endpoint_event = tokio_stream::once(Ok::<_, Infallible>(
        Event::default().event("endpoint").data(message_url),
    ));

    let message_stream = UnboundedReceiverStream::new(rx)
        .map(|msg| Ok::<_, Infallible>(Event::default().event("message").data(msg)));

    let combined = endpoint_event.chain(message_stream);

    // Clean up session after 1 hour or when stream ends
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_hours(1)).await;
        sessions_cleanup.lock().await.remove(&session_id_cleanup);
    });

    // Axum's KeepAlive handles keepalive pings automatically
    Sse::new(combined).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}

/// Message endpoint - receives JSON-RPC requests and sends responses via SSE
#[utoipa::path(
    post,
    path = "/api/v1/mcp/message",
    tag = "MCP",
    operation_id = "mcp_message",
    params(("sessionId" = String, Query, description = "MCP SSE session id")),
    request_body = serde_json::Value,
    responses(
        (status = 202, description = "JSON-RPC request accepted for delivery on the SSE stream"),
        (status = 403, description = "Session belongs to another principal"),
        (status = 404, description = "Session not found")
    )
)]
pub async fn message_handler(
    Query(query): Query<MessageQuery>,
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    // Clone the sender and drop the lock before async work.
    let (tx, scopes) = {
        let sessions = state.mcp_sessions.lock().await;
        let Some(session) = sessions.get(&query.session_id) else {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({"error": "Session not found"})),
            );
        };
        // The caller must be the principal that opened this stream. Without
        // this, anyone who learns a session id inherits its authority.
        if session.subject != principal.subject {
            return (
                axum::http::StatusCode::FORBIDDEN,
                Json(json!({"error": "Session belongs to a different principal"})),
            );
        }
        (session.tx.clone(), session.scopes.clone())
    };

    let response = handle_jsonrpc(&request, &state, &scopes).await;

    // Send response through SSE channel
    if let Ok(json_str) = serde_json::to_string(&response) {
        let _ = tx.send(json_str);
    }

    (axum::http::StatusCode::ACCEPTED, Json(json!({})))
}
