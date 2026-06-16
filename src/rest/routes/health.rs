//! Health check and status endpoints.

use axum::{extract::State, Json};

use crate::rest::directory::directory_identity;
use crate::rest::dto::{HealthResponse, StatusResponse};
use crate::rest::state::ApiState;

/// Health check endpoint
#[utoipa::path(
    operation_id = "health_check",
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health(State(state): State<ApiState>) -> Json<HealthResponse> {
    let (directory_name, directory_id) = directory_identity(&state.tickets_path);
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        directory_name,
        directory_id,
    })
}

/// Get service status with registry info
#[utoipa::path(
    operation_id = "health_status",
    get,
    path = "/api/v1/status",
    tag = "Health",
    responses(
        (status = 200, description = "Service status with registry info", body = StatusResponse)
    )
)]
pub async fn status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let (directory_name, directory_id) = directory_identity(&state.tickets_path);
    let registry = state.registry.read().await;

    Json(StatusResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        directory_name,
        directory_id,
        issuetype_count: registry.type_count(),
        collection_count: registry.collection_count(),
        active_collection: registry.active_collection_name().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health() {
        use crate::config::Config;
        use std::path::PathBuf;

        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/home/acme/.tickets"));

        let resp = health(State(state)).await;
        assert_eq!(resp.status, "ok");
        assert!(!resp.version.is_empty());
        // Directory identity is derived from the working root (parent of .tickets).
        assert_eq!(resp.directory_name, "acme");
        assert_eq!(resp.directory_id.len(), 12);
    }

    #[tokio::test]
    async fn test_status() {
        use crate::config::Config;
        use std::path::PathBuf;

        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = status(State(state)).await;
        assert_eq!(resp.status, "ok");
        assert!(resp.issuetype_count >= 5); // At least builtins
        assert_eq!(resp.directory_id.len(), 12);
    }
}
