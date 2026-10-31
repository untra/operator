//! Model Context Protocol (MCP) integration for Operator.
//!
//! Provides an MCP server bridge that exposes Operator's REST API as
//! read-only MCP tools. Includes a descriptor endpoint for client discovery,
//! tool definitions, and an SSE transport for JSON-RPC communication.

pub mod args;
pub mod campaigns;
pub mod catalog;
pub mod client_configs;
pub mod control;
pub mod descriptor;
pub mod handler;
pub mod resources;
pub mod stdio;
pub mod tickets;
pub mod tools;
pub mod transport;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

/// Host (`host:port`) extracted from the request, for building the absolute
/// MCP URLs advertised to clients.
pub struct Host(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Host {
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> {
        let host = parts
            .headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
            .or_else(|| parts.uri.authority().map(ToString::to_string))
            .unwrap_or_default();
        std::future::ready(Ok(Host(host)))
    }
}

/// Public base URL for browser device authorization.
///
/// Prefers the configured public URL over the request's `Host` header. The header is attacker-controlled and carries no scheme,
/// so behind TLS termination it yields a `http://` URL a client cannot use. Falling back to it is still correct for a loopback bind.
pub fn public_base_url(state: &crate::rest::state::ApiState, host: &str) -> String {
    state
        .config()
        .rest_api
        .public_base_url()
        .unwrap_or_else(|| format!("http://{host}"))
}

pub fn profile_api_base(state: &crate::rest::state::ApiState, _host: &str) -> String {
    let base = state
        .config()
        .rest_api
        .public_base_url()
        .unwrap_or_default();
    let profile_id = state.config().profile.id;
    if profile_id.is_nil() {
        format!("{base}/api/v1")
    } else {
        format!("{base}/api/v1/profiles/{profile_id}")
    }
}
