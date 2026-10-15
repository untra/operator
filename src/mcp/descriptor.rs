//! MCP descriptor endpoint for client discovery.
//!
//! Returns metadata needed to register operator with an MCP-capable client.
//! Includes both the SSE transport URL (for network clients like the
//! vscode-extension) and, optionally, the stdio entrypoint command so
//! clients can spawn `operator mcp` as a subprocess instead.

use super::Host;
use crate::rest::state::RequestState as State;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::rest::state::ApiState;

/// Stdio entrypoint advertised in the descriptor when
/// `[mcp].stdio_advertised = true`. Clients use this to spawn operator
/// as an MCP subprocess instead of (or alongside) the SSE transport.
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StdioCommand {
    /// Absolute path to the operator binary (the same binary serving this descriptor)
    pub command: String,
    /// Args to pass: typically `["mcp"]`
    pub args: Vec<String>,
    /// Working directory the client should set when spawning. Defaults to the
    /// operator process's current working directory.
    pub cwd: String,
}

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
    /// SSE endpoint, relative unless a public URL is configured.
    pub transport_url: String,
    /// Whether the running API has mounted the SSE transport.
    pub http_available: bool,
    /// Human-readable label for the server
    pub label: String,
    /// URL of the OpenAPI spec for reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openapi_url: Option<String>,
    /// Stdio transport entrypoint. Present when `[mcp].stdio_advertised = true`.
    /// Clients may spawn this as a subprocess instead of using `transport_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdio: Option<StdioCommand>,
}

/// MCP descriptor endpoint
///
/// Returns metadata for registering operator with an MCP-capable client.
/// Transport URLs use the configured public base or relative API paths.
#[utoipa::path(
    operation_id = "mcp_descriptor",
    get,
    path = "/api/v1/mcp/descriptor",
    tag = "MCP",
    responses(
        (status = 200, description = "MCP server descriptor", body = McpDescriptorResponse)
    )
)]
pub async fn descriptor(
    State(state): State<ApiState>,
    Host(host): Host,
) -> Json<McpDescriptorResponse> {
    let base = state
        .config()
        .rest_api
        .public_base_url()
        .unwrap_or_default();
    let profile_api = super::profile_api_base(&state, &host);
    let config = state.config();

    let stdio = if config.mcp.stdio_advertised {
        let command = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_else(|| "operator".to_string());
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_default();
        let args = if config.profile.id.is_nil() {
            vec!["mcp".to_string()]
        } else {
            vec![
                "--profile".to_string(),
                config.profile.name.clone(),
                "mcp".to_string(),
            ]
        };
        Some(StdioCommand { command, args, cwd })
    } else {
        None
    };

    Json(McpDescriptorResponse {
        server_name: "operator".to_string(),
        server_id: "operator-mcp".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        transport_url: format!("{profile_api}/mcp/sse"),
        http_available: state.mcp_http_mounted,
        label: "Operator MCP Server".to_string(),
        openapi_url: Some(format!("{base}/api-docs/openapi.json")),
        stdio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn state_with_stdio(advertised: bool) -> ApiState {
        let mut config = Config::default();
        config.mcp.stdio_advertised = advertised;
        ApiState::in_memory(config)
    }

    #[tokio::test]
    async fn test_descriptor_response() {
        let state = state_with_stdio(true);
        let resp = descriptor(State(state), Host("localhost:7008".to_string())).await;

        assert_eq!(resp.server_name, "operator");
        assert_eq!(resp.server_id, "operator-mcp");
        assert_eq!(resp.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(resp.transport_url, "/api/v1/mcp/sse");
        assert_eq!(resp.label, "Operator MCP Server");
        assert_eq!(resp.openapi_url, Some("/api-docs/openapi.json".to_string()));
    }

    #[tokio::test]
    async fn test_descriptor_custom_port() {
        let state = state_with_stdio(true);
        let resp = descriptor(State(state), Host("localhost:9999".to_string())).await;

        assert_eq!(resp.transport_url, "/api/v1/mcp/sse");
        assert_eq!(resp.openapi_url, Some("/api-docs/openapi.json".to_string()));
    }

    #[tokio::test]
    async fn descriptor_scopes_transports_to_the_configuration() {
        let mut config = Config::default();
        config.profile.id = uuid::Uuid::new_v4();
        config.profile.name = "team-one".to_string();
        config.mcp.stdio_advertised = true;
        let profile_id = config.profile.id;
        let state = ApiState::in_memory(config);

        let response = descriptor(State(state), Host("localhost:7008".to_string())).await;

        assert_eq!(
            response.transport_url,
            format!("/api/v1/profiles/{profile_id}/mcp/sse")
        );
        assert_eq!(
            response.stdio.as_ref().unwrap().args.as_slice(),
            ["--profile", "team-one", "mcp"]
        );
    }

    #[tokio::test]
    async fn test_descriptor_stdio_present_when_advertised() {
        let state = state_with_stdio(true);
        let resp = descriptor(State(state), Host("localhost:7008".to_string())).await;

        let stdio = resp.stdio.as_ref().expect("stdio should be present");
        assert_eq!(stdio.args, vec!["mcp".to_string()]);
        assert!(
            !stdio.command.is_empty(),
            "command path should be populated from current_exe"
        );
    }

    #[tokio::test]
    async fn test_descriptor_stdio_absent_when_disabled() {
        let state = state_with_stdio(false);
        let resp = descriptor(State(state), Host("localhost:7008".to_string())).await;
        assert!(resp.stdio.is_none());
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_descriptor_public_url_preserves_https_prefix_and_availability() {
        let mut config = Config::default();
        config.rest_api.public_url = Some("https://operator.test/prefix/".into());
        config.mcp.http_enabled = false;
        let response = descriptor(
            State(ApiState::in_memory(config)),
            Host("internal:7008".into()),
        )
        .await;
        assert_eq!(
            response.transport_url,
            "https://operator.test/prefix/api/v1/mcp/sse"
        );
        assert!(!response.http_available);
    }

    #[tokio::test]
    async fn test_descriptor_without_public_url_uses_relative_transport() {
        let state = ApiState::in_memory(Config::default());
        let response = descriptor(State(state), Host("internal:7008".into())).await;
        assert_eq!(response.transport_url, "/api/v1/mcp/sse");
    }
}
