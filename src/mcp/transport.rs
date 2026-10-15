//! MCP SSE transport for JSON-RPC communication.
//!
//! Implements the MCP SSE transport protocol:
//! - `GET /api/v1/mcp/sse` opens an SSE stream and sends the message endpoint URL
//! - `POST /api/v1/mcp/message?sessionId={id}` receives JSON-RPC requests and
//!   sends responses back through the SSE stream

use crate::rest::state::RequestState as State;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use super::Host;
use axum::extract::Query;
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;

use crate::mcp::handler::{handle_jsonrpc, JsonRpcRequest};
use crate::mcp::profile_api_base;
use crate::rest::middleware::auth::Authenticated;
use crate::rest::state::{ApiState, McpSession};

const SESSION_TTL: Duration = Duration::from_hours(1);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);
const DIAGNOSTIC_CLIENT: &str = "operator-connections-check";
const MAX_CLIENT_LABEL_CHARS: usize = 128;
const MAX_QUEUED_MESSAGES: usize = 64;
const MAX_SESSIONS: usize = 1024;
const MAX_SESSIONS_PER_OWNER: usize = 8;

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
) -> Result<
    Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>,
    crate::rest::error::ApiError,
> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<String>(MAX_QUEUED_MESSAGES);

    // Bind the session to whoever opened it. The session id travels in a URL
    // and is therefore a bearer credential; recording the principal means a
    // leaked id is not enough on its own to drive the session.
    let (disconnect_tx, disconnect_rx) = tokio::sync::oneshot::channel::<()>();
    let mut sessions = state.mcp_sessions.lock().await;
    if sessions.len() >= MAX_SESSIONS
        || sessions
            .values()
            .filter(|session| session.subject == principal.subject)
            .count()
            >= MAX_SESSIONS_PER_OWNER
    {
        return Err(crate::rest::error::ApiError::RateLimited {
            retry_after_secs: KEEPALIVE_INTERVAL.as_secs(),
        });
    }
    sessions.insert(
        session_id.clone(),
        McpSession {
            observation_id: uuid::Uuid::new_v4().to_string(),
            tx,
            subject: principal.subject.clone(),
            credential_id: credential_id(&principal),
            authority_grants: principal.authority.as_ref().map(|a| a.grants.clone()),
            initialized: false,
            initialize_received: false,
            diagnostic: false,
            client_name: None,
            client_version: None,
            connected_at: chrono::Utc::now(),
            last_activity_at: chrono::Utc::now(),
        },
    );

    drop(sessions);

    // Generated from the configured public URL, not the request `Host` header,
    // which a caller controls and which is plain `http` behind TLS termination.
    let base = profile_api_base(&state, &host);
    let message_url = if state.config().rest_api.public_base_url().is_some() {
        format!("{base}/mcp/message?sessionId={session_id}")
    } else {
        format!("message?sessionId={session_id}")
    };

    let session_id_cleanup = session_id.clone();
    let sessions_cleanup = Arc::clone(&state.mcp_sessions);

    // Build SSE stream: first event is the endpoint URL, then relay messages
    let endpoint_event = tokio_stream::once(Ok::<_, Infallible>(
        Event::default().event("endpoint").data(message_url),
    ));

    let stream_principal = principal.clone();
    let stream_store = state.auth.store.clone();
    let message_stream = ReceiverStream::new(rx)
        .then(move |msg| {
            let _disconnect = &disconnect_tx;
            let store = stream_store.clone();
            let principal = stream_principal.clone();
            async move {
                credential_active(store, principal)
                    .await
                    .then(|| Ok::<_, Infallible>(Event::default().event("message").data(msg)))
            }
        })
        .take_while(Option::is_some)
        .map(|event| event.expect("active credential"));

    let combined = endpoint_event.chain(message_stream);

    // Clean up session after 1 hour or when stream ends
    tokio::spawn(async move {
        tokio::select! {
            _ = disconnect_rx => {},
            () = tokio::time::sleep(SESSION_TTL) => {},
            () = async {
                loop {
                    tokio::time::sleep(KEEPALIVE_INTERVAL).await;
                    if !credential_active(state.auth.store.clone(), principal.clone()).await { break; }
                }
            } => {},
        }
        sessions_cleanup.lock().await.remove(&session_id_cleanup);
    });

    // Axum's KeepAlive handles keepalive pings automatically
    Ok(Sse::new(combined).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(KEEPALIVE_INTERVAL)
            .text("keepalive"),
    ))
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
) -> Response {
    // Clone the sender and drop the lock before async work.
    let (tx, scopes) = {
        let mut sessions = state.mcp_sessions.lock().await;
        let Some(session) = sessions.get_mut(&query.session_id) else {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({"error": "Session not found"})),
            )
                .into_response();
        };
        // The caller must be the principal that opened this stream. Without
        // this, anyone who learns a session id inherits its authority.
        if session.subject != principal.subject
            || session.credential_id != credential_id(&principal)
            || session.authority_grants != principal.authority.as_ref().map(|a| a.grants.clone())
        {
            return (
                axum::http::StatusCode::FORBIDDEN,
                Json(json!({"error": "Session belongs to a different principal"})),
            )
                .into_response();
        }
        if session.tx.is_closed() {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({"error": "Session disconnected"})),
            )
                .into_response();
        }
        session.last_activity_at = chrono::Utc::now();
        if request.method == "initialize" && request.id.is_some() {
            session.initialize_received = true;
            session.client_name = request.params["clientInfo"]["name"]
                .as_str()
                .map(|name| name.chars().take(MAX_CLIENT_LABEL_CHARS).collect());
            session.client_version = request.params["clientInfo"]["version"]
                .as_str()
                .map(|version| version.chars().take(MAX_CLIENT_LABEL_CHARS).collect());
            session.diagnostic = session.client_name.as_deref() == Some(DIAGNOSTIC_CLIENT);
        }
        if request.method == "notifications/initialized"
            && request.id.is_none()
            && session.initialize_received
        {
            session.initialized = true;
        }
        (session.tx.clone(), principal.scopes.clone())
    };

    if request.id.is_none() {
        return axum::http::StatusCode::ACCEPTED.into_response();
    }
    let response = handle_jsonrpc(&request, &state, &scopes).await;

    // Send response through SSE channel
    if let Ok(json_str) = serde_json::to_string(&response) {
        if tx.try_send(json_str).is_err() {
            state.mcp_sessions.lock().await.remove(&query.session_id);
        }
    }

    axum::http::StatusCode::ACCEPTED.into_response()
}

fn credential_id(principal: &crate::auth::scope::Principal) -> Option<String> {
    principal
        .authority
        .as_ref()
        .and_then(|a| a.key_id.as_ref())
        .map(|id| format!("key:{id}"))
        .or_else(|| {
            principal
                .session_id
                .as_ref()
                .map(|id| format!("session:{id}"))
        })
}

async fn credential_active(
    store: crate::auth::store::AuthStore,
    principal: crate::auth::scope::Principal,
) -> bool {
    if principal
        .expires_at
        .is_some_and(|expiry| expiry <= chrono::Utc::now())
    {
        return false;
    }
    if principal.session_id.is_some() {
        return tokio::task::spawn_blocking(move || {
            let profile = principal
                .authority
                .as_ref()
                .map_or_else(uuid::Uuid::nil, |a| a.grants.profile_id);
            store
                .resolve_user_session(principal.clone(), profile)
                .ok()
                .flatten()
                .is_some_and(|current| {
                    current
                        .authority
                        .as_ref()
                        .map(|a| (&a.grants, a.user.administrator))
                        == principal
                            .authority
                            .as_ref()
                            .map(|a| (&a.grants, a.user.administrator))
                })
        })
        .await
        .unwrap_or(false);
    }
    let Some(authority) = principal.authority else {
        return true;
    };
    let Some(id) = authority.key_id.clone() else {
        return true;
    };
    tokio::task::spawn_blocking(move || {
        store
            .key_authority(&id)
            .ok()
            .flatten()
            .is_some_and(|(current, _)| {
                current.user.id == authority.user.id && authority.grants == current.grants
            })
    })
    .await
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::scope::Principal;
    use crate::rest::dto::auth::{PrincipalKind, Scope};
    use axum::body::to_bytes;

    fn principal() -> Principal {
        Principal {
            subject: "test-client".into(),
            kind: PrincipalKind::AccessToken,
            expires_at: None,
            session_id: None,
            ticket_id: None,
            step: None,
            agent_session_id: None,
            authority: None,
            scopes: vec![Scope::Read, Scope::Execute],
        }
    }

    #[tokio::test]
    async fn test_stream_rechecks_key_grants_and_revocation() {
        let (store, owner, request) = crate::auth::identity::tests::fixture_in(
            crate::auth::store::AuthStore::in_memory().unwrap(),
        );
        let key = store.create_user_key(&owner, &request, None).unwrap();
        let (authority, expiry) = store.exchange_user_key(&key.secret).unwrap().unwrap();
        let principal = authority.principal(expiry, None);
        assert!(credential_active(store.clone(), principal.clone()).await);
        let mut grants = request.grants;
        grants.projects.clear();
        store
            .update_operator_user(
                "admin",
                &owner.user.id,
                &crate::auth::identity::UpdateOperatorUser {
                    enabled: true,
                    administrator: false,
                    grants,
                },
                false,
            )
            .unwrap();
        assert!(!credential_active(store.clone(), principal).await);
        let (authority, expiry) = store.key_authority(&key.key.summary.id).unwrap().unwrap();
        let principal = authority.principal(expiry, None);
        assert!(credential_active(store.clone(), principal.clone()).await);
        store
            .revoke_user_key(&owner.user.id, &key.key.summary.id, false)
            .unwrap();
        assert!(!credential_active(store, principal).await);
    }

    #[tokio::test]
    async fn test_stream_closes_after_browser_session_revocation() {
        let (store, owner, _) = crate::auth::identity::tests::fixture_in(
            crate::auth::store::AuthStore::in_memory().unwrap(),
        );
        let (token, _, _) = store
            .oidc_session(&owner.user.id, chrono::Utc::now().timestamp())
            .unwrap();
        let principal = store.authenticate_session(&token).unwrap().unwrap();
        let principal = store
            .resolve_user_session(principal, owner.grants.profile_id)
            .unwrap()
            .unwrap();
        assert!(credential_active(store.clone(), principal.clone()).await);
        store
            .revoke_session(principal.session_id.as_ref().unwrap())
            .unwrap();
        assert!(!credential_active(store, principal).await);
    }

    #[tokio::test]
    async fn test_stream_disconnect_removes_session() {
        let state = ApiState::in_memory(crate::config::Config::default());
        let stream = sse_handler(
            Host("internal:7008".into()),
            State(state.clone()),
            Authenticated(principal()),
        )
        .await;
        assert_eq!(state.mcp_sessions.lock().await.len(), 1);
        drop(stream);
        tokio::time::timeout(Duration::from_secs(1), async {
            while !state.mcp_sessions.lock().await.is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("disconnected sessions must be removed");
    }

    #[tokio::test]
    async fn test_notification_has_no_jsonrpc_response() {
        let state = ApiState::in_memory(crate::config::Config::default());
        let stream = sse_handler(
            Host("internal:7008".into()),
            State(state.clone()),
            Authenticated(principal()),
        )
        .await;
        let session_id = state
            .mcp_sessions
            .lock()
            .await
            .keys()
            .next()
            .unwrap()
            .clone();
        let response = Box::pin(message_handler(
            Query(MessageQuery { session_id }),
            State(state),
            Authenticated(principal()),
            Json(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: None,
                method: "notifications/initialized".into(),
                params: json!({}),
            }),
        ))
        .await
        .into_response();
        assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
        assert!(to_bytes(response.into_body(), 1024)
            .await
            .unwrap()
            .is_empty());
        drop(stream);
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::auth::scope::Principal;
    use crate::rest::routes::connections;
    use http_body_util::BodyExt;

    async fn initialize(state: &ApiState, session_id: &str, client: &str) {
        for (method, id, params) in [
            (
                "initialize",
                Some(json!(1)),
                json!({"clientInfo": {"name": client, "version": "1"}}),
            ),
            ("notifications/initialized", None, json!({})),
        ] {
            let response = Box::pin(message_handler(
                Query(MessageQuery {
                    session_id: session_id.into(),
                }),
                State(state.clone()),
                Authenticated(Principal::local("test")),
                Json(JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    method: method.into(),
                    id,
                    params,
                }),
            ))
            .await;
            assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
        }
    }

    #[tokio::test]
    async fn test_initialized_clients_exclude_diagnostics_and_closed_streams() {
        let state = ApiState::in_memory(crate::config::Config::default());
        let stream = sse_handler(
            Host("internal:7008".into()),
            State(state.clone()),
            Authenticated(Principal::local("test")),
        )
        .await;
        let id = state
            .mcp_sessions
            .lock()
            .await
            .keys()
            .next()
            .unwrap()
            .clone();
        assert_eq!(
            connections::get(State(state.clone()))
                .await
                .0
                .mcp_connected_clients,
            0
        );
        Box::pin(initialize(&state, &id, "Claude Code")).await;
        assert_eq!(
            connections::get(State(state.clone()))
                .await
                .0
                .mcp_connected_clients,
            1
        );
        let clients = connections::clients(State(state.clone())).await.0;
        assert_eq!(clients[0].client_name.as_deref(), Some("Claude Code"));
        assert_ne!(clients[0].id, id);
        Box::pin(initialize(&state, &id, DIAGNOSTIC_CLIENT)).await;
        assert!(connections::clients(State(state.clone()))
            .await
            .0
            .is_empty());
        assert_eq!(
            connections::get(State(state.clone()))
                .await
                .0
                .mcp_connected_clients,
            0
        );
        drop(stream);
    }

    #[tokio::test]
    async fn test_clearing_sessions_closes_streams_during_shutdown() {
        let state = ApiState::in_memory(crate::config::Config::default());
        let stream = sse_handler(
            Host("internal:7008".into()),
            State(state.clone()),
            Authenticated(Principal::local("test")),
        )
        .await;
        let mut body = stream.into_response().into_body();
        body.frame().await.unwrap().unwrap();
        state.mcp_sessions.lock().await.clear();
        let next = tokio::time::timeout(Duration::from_secs(1), body.frame())
            .await
            .expect("shutdown must close the stream");
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn test_sse_endpoint_is_relative_without_public_url() {
        let state = ApiState::in_memory(crate::config::Config::default());
        let stream = sse_handler(
            Host("internal:7008".into()),
            State(state),
            Authenticated(Principal::local("test")),
        )
        .await;
        let mut body = stream.into_response().into_body();
        let frame = body.frame().await.unwrap().unwrap().into_data().unwrap();
        let text = std::str::from_utf8(&frame).unwrap();
        assert!(text.contains("data: message?sessionId="));
        assert!(!text.contains("internal"));
    }
}
