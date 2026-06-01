//! REST API for Operator issue type management.
//!
//! Provides HTTP endpoints for listing, viewing, and modifying issue types
//! and collections. Designed to run alongside the TUI or as a standalone server.

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod server;
pub mod state;
#[cfg(feature = "embed-ui")]
pub mod web_ui;

/// Shim exposing the same `EmbeddedUiState` API when the SPA isn't compiled
/// in. Callers can treat the two modules identically without `#[cfg]` blocks.
///
/// `Ready` and `Placeholder` are never constructed in this configuration —
/// `embedded_ui_state()` always returns `Missing` when `embed-ui` is off —
/// but they must exist so call-site `match` arms remain exhaustive across
/// both feature configurations.
#[cfg(not(feature = "embed-ui"))]
pub mod web_ui {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum EmbeddedUiState {
        Ready,
        Placeholder,
        Missing,
    }

    pub fn embedded_ui_state() -> EmbeddedUiState {
        EmbeddedUiState::Missing
    }
}

pub use openapi::ApiDoc;
pub use server::{ApiSessionInfo, RestApiServer, RestApiStatus};
pub use state::ApiState;

/// Default port for the REST API server
#[allow(dead_code)]
pub const DEFAULT_PORT: u16 = 7008;

/// Build the documented API surface as a `utoipa_axum::OpenApiRouter`.
///
/// Every always-on route is mounted here via `routes!`, so mounting a route
/// *is* registering it in the OpenAPI spec — the router and the spec cannot
/// drift. Handlers sharing a path (different HTTP methods) are grouped in a
/// single `routes!` call. Config-gated routes (MCP `sse`/`message`) are NOT
/// documented and are added separately in [`build_router`].
///
/// The base `OpenApi` (info, components/schemas, tags) comes from the
/// [`ApiDoc`] derive; paths and their referenced schemas are collected from the
/// mounted handlers.
fn documented_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        // Health endpoints
        .routes(routes!(routes::health::health))
        .routes(routes!(routes::health::status))
        // Canonical status sections (shared with the TUI / VS Code extension)
        .routes(routes!(routes::sections::list))
        // Issue type endpoints
        .routes(routes!(
            routes::issuetypes::list,
            routes::issuetypes::create
        ))
        .routes(routes!(
            routes::issuetypes::get_one,
            routes::issuetypes::update,
            routes::issuetypes::delete
        ))
        // Step endpoints
        .routes(routes!(routes::steps::list))
        .routes(routes!(routes::steps::get_one, routes::steps::update))
        // Collection endpoints
        .routes(routes!(routes::collections::list))
        .routes(routes!(routes::collections::get_active))
        .routes(routes!(routes::collections::get_one))
        .routes(routes!(routes::collections::activate))
        // Queue endpoints
        .routes(routes!(routes::queue::kanban))
        .routes(routes!(routes::queue::status))
        .routes(routes!(routes::queue::pause))
        .routes(routes!(routes::queue::resume))
        .routes(routes!(routes::queue::sync))
        .routes(routes!(routes::queue::sync_collection))
        // Agent endpoints
        .routes(routes!(routes::agents::active))
        .routes(routes!(routes::agents::get_detail))
        .routes(routes!(routes::agents::approve_review))
        .routes(routes!(routes::agents::reject_review))
        // Project endpoints
        .routes(routes!(routes::projects::list))
        .routes(routes!(routes::projects::assess))
        // Ticket endpoints
        .routes(routes!(routes::tickets::get_one))
        .routes(routes!(routes::tickets::update_status))
        // Launch endpoints
        .routes(routes!(routes::launch::launch_ticket))
        // Workflow export endpoint
        .routes(routes!(routes::workflow::export))
        // Workflow preview endpoint (issue type -> graph, no ticket)
        .routes(routes!(routes::workflow::preview))
        // Step completion endpoint (for opr8r wrapper)
        .routes(routes!(routes::launch::complete_step))
        // Kanban provider endpoints
        .routes(routes!(routes::kanban::provider_catalog))
        .routes(routes!(routes::kanban::external_issue_types))
        .routes(routes!(routes::kanban::sync_issue_types))
        // Kanban onboarding endpoints (validate, list projects, write config, set env)
        .routes(routes!(routes::kanban_onboarding::validate_credentials))
        .routes(routes!(routes::kanban_onboarding::list_projects))
        .routes(routes!(routes::kanban_onboarding::write_config))
        .routes(routes!(routes::kanban_onboarding::set_session_env))
        // Skills endpoint
        .routes(routes!(routes::skills::list))
        // LLM tools endpoints
        .routes(routes!(routes::llm_tools::list))
        .routes(routes!(
            routes::llm_tools::get_default,
            routes::llm_tools::set_default
        ))
        // Delegator endpoints. `from-tool` is a distinct static path; axum 0.7
        // prefers static segments over `{name}`, so ordering is not required for
        // correctness, but the routes stay grouped by path for clarity.
        .routes(routes!(
            routes::delegators::list,
            routes::delegators::create
        ))
        .routes(routes!(routes::delegators::create_from_tool))
        .routes(routes!(
            routes::delegators::get_one,
            routes::delegators::update,
            routes::delegators::delete
        ))
        // Configuration endpoints
        .routes(routes!(
            routes::configuration::get_config,
            routes::configuration::update_config
        ))
        // Model server endpoints
        .routes(routes!(
            routes::model_servers::list,
            routes::model_servers::create
        ))
        .routes(routes!(
            routes::model_servers::get_one,
            routes::model_servers::delete
        ))
        // MCP descriptor — always mounted so non-HTTP MCP clients can still
        // discover the stdio entrypoint.
        .routes(routes!(crate::mcp::descriptor::descriptor))
}

/// The canonical OpenAPI spec for the documented API surface.
///
/// Built from [`documented_router`] so it always reflects the mounted routes.
/// Config-gated MCP transport routes are omitted (they carry no
/// `#[utoipa::path]` and only ever exist when `[mcp].http_enabled`).
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    documented_router().split_for_parts().1
}

/// Build the API router with all routes
pub fn build_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mcp_enabled = state.config.mcp.http_enabled;

    let (mut router, api) = documented_router().split_for_parts();

    // MCP transport endpoints — gated by [mcp].http_enabled and intentionally
    // undocumented (no OpenAPI schema for the SSE/JSON-RPC transport).
    if mcp_enabled {
        router = router
            .route("/api/v1/mcp/sse", get(crate::mcp::transport::sse_handler))
            .route(
                "/api/v1/mcp/message",
                post(crate::mcp::transport::message_handler),
            );
    }

    let router = router
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors)
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    #[cfg(feature = "embed-ui")]
    let router = router.fallback(web_ui::spa_handler);

    router
}

/// Start the REST API server (standalone mode with session file and logging)
pub async fn serve(state: ApiState, port: u16) -> Result<()> {
    let tickets_path = state.tickets_path.clone();
    let app = build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("REST API listening on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    // Write session file for client discovery
    write_session_file(&tickets_path, port)?;

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Clean up session file on shutdown
    remove_session_file(&tickets_path);

    Ok(())
}

/// Write API session file for client discovery (standalone mode)
fn write_session_file(tickets_path: &std::path::Path, port: u16) -> Result<()> {
    let operator_dir = tickets_path.join("operator");
    std::fs::create_dir_all(&operator_dir)?;

    let session_file = operator_dir.join("api-session.json");
    let session = ApiSessionInfo {
        port,
        pid: std::process::id(),
        started_at: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let json = serde_json::to_string_pretty(&session)?;
    std::fs::write(&session_file, json)?;

    println!("Session file: {}", session_file.display());
    Ok(())
}

/// Remove API session file on shutdown (standalone mode)
fn remove_session_file(tickets_path: &std::path::Path) {
    let session_file = tickets_path.join("operator").join("api-session.json");
    if session_file.exists() {
        if let Err(e) = std::fs::remove_file(&session_file) {
            tracing::warn!(error = %e, "Failed to remove API session file");
        } else {
            println!("Cleaned up session file");
        }
    }
}

/// Shutdown signal handler for graceful termination
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            println!("\nReceived Ctrl+C, shutting down...");
        },
        () = terminate => {
            println!("\nReceived terminate signal, shutting down...");
        },
    }
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
