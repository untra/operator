//! MCP descriptor endpoint for client discovery.
//!
//! Returns metadata needed to build a VS Code MCP deep link,
//! including server name, transport URL, and version.

use axum::extract::Host;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// MCP server descriptor for client discovery
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct McpDescriptorResponse {
    /// Server name used in MCP registration (e.g. "operator")
    pub server_name: String,
    /// Unique server identifier (e.g. "operator-mcp")
    pub server_id: String,
    /// Server version from Cargo.toml
    pub version: String,
    /// Full URL of the MCP SSE transport endpoint
    pub transport_url: String,
    /// Human-readable label for the server
    pub label: String,
    /// URL of the OpenAPI spec for reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openapi_url: Option<String>,
}

/// MCP descriptor endpoint
///
/// Returns metadata for building a VS Code MCP deep link.
/// The transport URL is derived from the request Host header
/// so it reflects the actual running port.
#[utoipa::path(
    get,
    path = "/api/v1/mcp/descriptor",
    tag = "MCP",
    responses(
        (status = 200, description = "MCP server descriptor", body = McpDescriptorResponse)
    )
)]
pub async fn descriptor(Host(host): Host) -> Json<McpDescriptorResponse> {
    let base = format!("http://{host}");

    Json(McpDescriptorResponse {
        server_name: "operator".to_string(),
        server_id: "operator-mcp".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        transport_url: format!("{base}/api/v1/mcp/sse"),
        label: "Operator MCP Server".to_string(),
        openapi_url: Some(format!("{base}/api-docs/openapi.json")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_descriptor_response() {
        let resp = descriptor(Host("localhost:7008".to_string())).await;

        assert_eq!(resp.server_name, "operator");
        assert_eq!(resp.server_id, "operator-mcp");
        assert_eq!(resp.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(resp.transport_url, "http://localhost:7008/api/v1/mcp/sse");
        assert_eq!(resp.label, "Operator MCP Server");
        assert_eq!(
            resp.openapi_url,
            Some("http://localhost:7008/api-docs/openapi.json".to_string())
        );
    }

    #[tokio::test]
    async fn test_descriptor_custom_port() {
        let resp = descriptor(Host("localhost:9999".to_string())).await;

        assert_eq!(resp.transport_url, "http://localhost:9999/api/v1/mcp/sse");
        assert_eq!(
            resp.openapi_url,
            Some("http://localhost:9999/api-docs/openapi.json".to_string())
        );
    }
}
