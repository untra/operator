//! Model Context Protocol (MCP) integration for Operator.
//!
//! Provides an MCP server bridge that exposes Operator's REST API as
//! read-only MCP tools. Includes a descriptor endpoint for client discovery,
//! tool definitions, and an SSE transport for JSON-RPC communication.

pub mod client_configs;
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
///
/// Replaces the deprecated `axum_extra::extract::Host` extractor (axum
/// [#3442](https://github.com/tokio-rs/axum/issues/3442)). Reads only the
/// standard `Host` header, falling back to the URI authority for HTTP/2; it
/// deliberately does *not* trust `X-Forwarded-Host`.
pub struct Host(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Host {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let host = parts
            .headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
            .or_else(|| parts.uri.authority().map(ToString::to_string))
            .unwrap_or_default();
        Ok(Host(host))
    }
}
