use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS, ToSchema)]
#[ts(export)]
pub struct ConnectionsResponse {
    pub version: String,
    pub profile_name: String,
    pub observed_at: DateTime<Utc>,
    /// Externally configured base URL, if present.
    pub public_url: Option<String>,
    pub api_path: String,
    pub mcp_path: String,
    /// Whether this running router mounted the SSE transport.
    pub mcp_http_mounted: bool,
    pub mcp_stdio_advertised: bool,
    pub acp_stdio_advertised: bool,
    pub mcp_connected_clients: usize,
    /// Protected-resource metadata URL when OIDC clients can discover the provider.
    pub oauth_resource_metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS, ToSchema)]
#[ts(export)]
pub struct McpClientStatus {
    /// Display identifier, unrelated to the transport session credential.
    pub id: String,
    pub client_name: Option<String>,
    pub client_version: Option<String>,
    pub initialized: bool,
    pub connected_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}
