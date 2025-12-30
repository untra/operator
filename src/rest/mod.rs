//! REST API for Operator issue type management.
//!
//! Provides HTTP endpoints for listing, viewing, and modifying issue types
//! and collections. Designed to run alongside the TUI or as a standalone server.

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod server;
pub mod state;

pub use openapi::ApiDoc;
pub use server::{RestApiServer, RestApiStatus};
pub use state::ApiState;

/// Default port for the REST API server
#[allow(dead_code)]
pub const DEFAULT_PORT: u16 = 7008;

/// Build the API router with all routes
pub fn build_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health endpoints
        .route("/api/v1/health", get(routes::health::health))
        .route("/api/v1/status", get(routes::health::status))
        // Issue type endpoints
        .route("/api/v1/issuetypes", get(routes::issuetypes::list))
        .route("/api/v1/issuetypes", post(routes::issuetypes::create))
        .route("/api/v1/issuetypes/:key", get(routes::issuetypes::get_one))
        .route("/api/v1/issuetypes/:key", put(routes::issuetypes::update))
        .route(
            "/api/v1/issuetypes/:key",
            delete(routes::issuetypes::delete),
        )
        // Step endpoints
        .route("/api/v1/issuetypes/:key/steps", get(routes::steps::list))
        .route(
            "/api/v1/issuetypes/:key/steps/:step_name",
            get(routes::steps::get_one),
        )
        .route(
            "/api/v1/issuetypes/:key/steps/:step_name",
            put(routes::steps::update),
        )
        // Collection endpoints
        .route("/api/v1/collections", get(routes::collections::list))
        .route(
            "/api/v1/collections/active",
            get(routes::collections::get_active),
        )
        .route(
            "/api/v1/collections/:name",
            get(routes::collections::get_one),
        )
        .route(
            "/api/v1/collections/:name/activate",
            put(routes::collections::activate),
        )
        // Queue endpoints
        .route("/api/v1/queue/kanban", get(routes::queue::kanban))
        .route("/api/v1/queue/status", get(routes::queue::status))
        // Agent endpoints
        .route("/api/v1/agents/active", get(routes::agents::active))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Start the REST API server
pub async fn serve(state: ApiState, port: u16) -> Result<()> {
    let app = build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("REST API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_build_router() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));
        let _router = build_router(state);
        // Router builds without panicking
    }
}
