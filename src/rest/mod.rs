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
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod dto;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod server;
pub mod state;
#[cfg(feature = "embed-ui")]
pub mod web_ui;

pub use openapi::ApiDoc;
pub use server::{ApiSessionInfo, RestApiServer, RestApiStatus};
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

    let mcp_enabled = state.config.mcp.http_enabled;

    let mut router = Router::new()
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
        .route("/api/v1/queue/pause", post(routes::queue::pause))
        .route("/api/v1/queue/resume", post(routes::queue::resume))
        .route("/api/v1/queue/sync", post(routes::queue::sync))
        .route(
            "/api/v1/queue/sync/:provider/:project_key",
            post(routes::queue::sync_collection),
        )
        // Agent endpoints
        .route("/api/v1/agents/active", get(routes::agents::active))
        .route("/api/v1/agents/:agent_id", get(routes::agents::get_detail))
        .route(
            "/api/v1/agents/:agent_id/approve",
            post(routes::agents::approve_review),
        )
        .route(
            "/api/v1/agents/:agent_id/reject",
            post(routes::agents::reject_review),
        )
        // Project endpoints
        .route("/api/v1/projects", get(routes::projects::list))
        .route(
            "/api/v1/projects/:name/assess",
            post(routes::projects::assess),
        )
        // Ticket endpoints
        .route("/api/v1/tickets/:id", get(routes::tickets::get_one))
        .route(
            "/api/v1/tickets/:id/status",
            put(routes::tickets::update_status),
        )
        // Launch endpoints
        .route(
            "/api/v1/tickets/:id/launch",
            post(routes::launch::launch_ticket),
        )
        // Step completion endpoint (for opr8r wrapper)
        .route(
            "/api/v1/tickets/:id/steps/:step/complete",
            post(routes::launch::complete_step),
        )
        // Kanban provider endpoints
        .route(
            "/api/v1/kanban/:provider/:project_key/issuetypes",
            get(routes::kanban::external_issue_types),
        )
        .route(
            "/api/v1/kanban/:provider/:project_key/issuetypes/sync",
            post(routes::kanban::sync_issue_types),
        )
        // Kanban onboarding endpoints (validate, list projects, write config, set env)
        .route(
            "/api/v1/kanban/validate",
            post(routes::kanban_onboarding::validate_credentials),
        )
        .route(
            "/api/v1/kanban/projects",
            post(routes::kanban_onboarding::list_projects),
        )
        .route(
            "/api/v1/kanban/config",
            put(routes::kanban_onboarding::write_config),
        )
        .route(
            "/api/v1/kanban/session-env",
            post(routes::kanban_onboarding::set_session_env),
        )
        // Skills endpoint
        .route("/api/v1/skills", get(routes::skills::list))
        // LLM tools endpoints
        .route("/api/v1/llm-tools", get(routes::llm_tools::list))
        .route(
            "/api/v1/llm-tools/default",
            get(routes::llm_tools::get_default).put(routes::llm_tools::set_default),
        )
        // Delegator endpoints
        .route("/api/v1/delegators", get(routes::delegators::list))
        .route("/api/v1/delegators", post(routes::delegators::create))
        // from-tool must be registered before :name to avoid path capture
        .route(
            "/api/v1/delegators/from-tool",
            post(routes::delegators::create_from_tool),
        )
        .route("/api/v1/delegators/:name", get(routes::delegators::get_one))
        .route("/api/v1/delegators/:name", put(routes::delegators::update))
        .route(
            "/api/v1/delegators/:name",
            delete(routes::delegators::delete),
        )
        // Configuration endpoints
        .route(
            "/api/v1/configuration",
            get(routes::configuration::get_config).put(routes::configuration::update_config),
        )
        // Model server endpoints
        .route("/api/v1/model-servers", get(routes::model_servers::list))
        .route("/api/v1/model-servers", post(routes::model_servers::create))
        .route(
            "/api/v1/model-servers/:name",
            get(routes::model_servers::get_one),
        )
        .route(
            "/api/v1/model-servers/:name",
            delete(routes::model_servers::delete),
        );

    // MCP endpoints — gated by [mcp].http_enabled. The descriptor stays mounted
    // so non-HTTP MCP clients can still discover the stdio entrypoint.
    router = router.route(
        "/api/v1/mcp/descriptor",
        get(crate::mcp::descriptor::descriptor),
    );
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
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

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
