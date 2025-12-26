//! Health check and status endpoints.

use axum::{extract::State, Json};

use crate::rest::dto::{HealthResponse, StatusResponse};
use crate::rest::state::ApiState;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get service status with registry info
#[utoipa::path(
    get,
    path = "/api/v1/status",
    tag = "Health",
    responses(
        (status = 200, description = "Service status with registry info", body = StatusResponse)
    )
)]
pub async fn status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let registry = state.registry.read().await;

    Json(StatusResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
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
        let resp = health().await;
        assert_eq!(resp.status, "ok");
        assert!(!resp.version.is_empty());
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
    }
}
