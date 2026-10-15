use crate::rest::dto::connections::{ConnectionsResponse, McpClientStatus};
use crate::rest::state::ApiState;
use crate::rest::state::RequestState as State;
use axum::Json;

#[utoipa::path(get, path = "/api/v1/connections", tag = "Connections", operation_id = "connections_get", responses((status = 200, body = ConnectionsResponse)))]
pub async fn get(State(state): State<ApiState>) -> Json<ConnectionsResponse> {
    let config = state.config();
    let api_path = if config.profile.id.is_nil() {
        "/api/v1".to_string()
    } else {
        format!("/api/v1/profiles/{}", config.profile.id)
    };
    let clients = state
        .mcp_sessions
        .lock()
        .await
        .values()
        .filter(|session| session.is_connected_client())
        .count();
    Json(ConnectionsResponse {
        version: env!("CARGO_PKG_VERSION").into(),
        profile_name: config.profile.name.clone(),
        observed_at: chrono::Utc::now(),
        public_url: config.rest_api.public_base_url(),
        mcp_path: format!("{api_path}/mcp/sse"),
        api_path,
        mcp_http_mounted: state.mcp_http_mounted,
        mcp_stdio_advertised: config.mcp.stdio_advertised,
        acp_stdio_advertised: config.acp.stdio_advertised,
        mcp_connected_clients: clients,
    })
}

#[utoipa::path(get, path = "/api/v1/connections/mcp/clients", tag = "Connections", operation_id = "connections_mcp_clients", responses((status = 200, body = Vec<McpClientStatus>)))]
pub async fn clients(State(state): State<ApiState>) -> Json<Vec<McpClientStatus>> {
    let sessions = state.mcp_sessions.lock().await;
    let mut clients: Vec<_> = sessions
        .values()
        .filter(|session| !session.diagnostic && !session.tx.is_closed())
        .map(|session| McpClientStatus {
            id: session.observation_id.clone(),
            client_name: session.client_name.clone(),
            client_version: session.client_version.clone(),
            initialized: session.initialized,
            connected_at: session.connected_at,
            last_activity_at: session.last_activity_at,
        })
        .collect();
    clients.sort_by_key(|client| client.connected_at);
    Json(clients)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connections_registered_under_distinct_routes() {
        let spec = crate::rest::openapi_spec();
        assert!(spec.paths.paths.contains_key("/api/v1/connections"));
        assert!(spec
            .paths
            .paths
            .contains_key("/api/v1/connections/mcp/clients"));
    }

    #[tokio::test]
    async fn test_connections_disabled_transport_and_profile_paths() {
        let mut config = crate::config::Config::default();
        config.mcp.http_enabled = false;
        config.profile.id = uuid::Uuid::new_v4();
        let id = config.profile.id;
        let state = ApiState::in_memory(config);
        let response = get(State(state)).await.0;
        assert!(!response.mcp_http_mounted);
        assert_eq!(response.mcp_path, format!("/api/v1/profiles/{id}/mcp/sse"));
        assert_eq!(response.mcp_connected_clients, 0);
    }
}
