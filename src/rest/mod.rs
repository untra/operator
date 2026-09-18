//! REST API for Operator issue type management.
//!
//! Provides HTTP endpoints for listing, viewing, and modifying issue types
//! and collections. Designed to run alongside the TUI or as a standalone server.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

pub mod directory;
pub mod dto;
pub mod error;
pub mod middleware;
pub mod openapi;
mod profile_router;
pub mod routes;
pub mod server;
pub mod state;
#[cfg(feature = "embed-ui")]
pub mod web_ui;

/// Shim exposing the same `EmbeddedUiState` API when the SPA isn't compiled
/// in. Callers can treat the two modules identically without `#[cfg]` blocks.
///
/// `Ready` and `Placeholder` are never constructed in this configuration -
/// `embedded_ui_state()` always returns `Missing` when `embed-ui` is off -
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
pub use server::{ApiSessionInfo, ExternalApiProbe, RestApiServer, RestApiStatus};
pub use state::ApiState;

/// Default port for the REST API server
#[allow(dead_code)]
pub const DEFAULT_PORT: u16 = 7008;

/// Probe and authentication routes.
///
/// Split out of [`documented_router`] both to keep that function legible and
/// because this is the security-relevant subset: every route here either needs
/// no credential or manages one. Merged in, so it still self-registers in the
/// OpenAPI spec exactly like the rest.
fn auth_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        // Kubernetes probes - public, and deliberately metadata-free.
        .routes(routes!(routes::probes::livez))
        .routes(routes!(routes::probes::readyz))
        // Obtaining a credential.
        .routes(routes!(
            routes::auth::bootstrap_status,
            routes::auth::bootstrap_submit
        ))
        .routes(routes!(routes::auth::login))
        .routes(routes!(routes::auth::forgot_password))
        .routes(routes!(routes::auth::reset_password))
        .routes(routes!(routes::auth::device_code))
        .routes(routes!(routes::auth::token))
        // Managing credentials (authenticated).
        .routes(routes!(routes::auth::logout))
        .routes(routes!(routes::auth::current_session))
        .routes(routes!(routes::auth::csrf_token))
        .routes(routes!(routes::auth::list_sessions))
        .routes(routes!(routes::auth::revoke_session))
        .routes(routes!(routes::auth::device_approve))
        .routes(routes!(
            routes::auth::list_access_keys,
            routes::auth::create_access_key
        ))
        .routes(routes!(routes::auth::revoke_access_key))
}

/// Configuration registry, licence and remote-target routes.
///
/// Split out of [`documented_router`] for the same reason as [`auth_router`]:
/// this is the entitlement-relevant subset, where every mutation either changes
/// what the installation is licensed for or what it can execute on.
fn premium_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        // Configuration registry.
        .routes(routes!(routes::profiles::list, routes::profiles::create))
        .routes(routes!(routes::profiles::get_one, routes::profiles::rename))
        // Licence.
        .routes(routes!(
            routes::license::get,
            routes::license::install,
            routes::license::remove
        ))
        // Remote targets.
        .routes(routes!(routes::targets::list, routes::targets::create))
        .routes(routes!(routes::targets::update, routes::targets::remove))
        .routes(routes!(routes::targets::probe))
}

fn first_run_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(routes::setup::status))
        .routes(routes!(routes::setup::steps))
        .routes(routes!(routes::setup::collections))
        .routes(routes!(routes::setup::initialize))
        .routes(routes!(routes::git_onboarding::providers))
        .routes(routes!(routes::git_onboarding::validate))
        .routes(routes!(routes::git_onboarding::write_config))
        .routes(routes!(routes::git_onboarding::set_session_env))
}

/// Build the documented API surface as a `utoipa_axum::OpenApiRouter`.
///
/// Every always-on route is mounted here via `routes!`, so mounting a route
/// *is* registering it in the OpenAPI spec - the router and the spec cannot
/// drift. Handlers sharing a path (different HTTP methods) are grouped in a
/// single `routes!` call. Config-gated routes (MCP `sse`/`message`) are NOT
/// documented and are added separately in [`build_router`].
///
/// The base `OpenApi` (info, components/schemas, tags) comes from the
/// [`ApiDoc`] derive; paths and their referenced schemas are collected from the
/// mounted handlers.
fn documented_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(auth_router())
        .merge(premium_router())
        // Health endpoints
        .routes(routes!(routes::health::health))
        .routes(routes!(routes::health::status))
        // Canonical status sections (shared with the TUI / VS Code extension)
        .routes(routes!(routes::sections::list))
        // Vertical integration catalog + support status
        .routes(routes!(routes::integrations::catalog))
        // First-run setup
        .merge(first_run_router())
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
        // The native Operator workflow document, byte-shaped like a hosted
        // collection's `<KEY>.json` so every surface renders the same graph.
        .routes(routes!(routes::issuetypes::get_document))
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
        .routes(routes!(routes::agents::focus_session))
        // Project endpoints
        .routes(routes!(routes::projects::list))
        .routes(routes!(routes::projects::assess))
        // Ticket endpoints
        .routes(routes!(routes::tickets::get_one))
        .routes(routes!(routes::tickets::create))
        .routes(routes!(routes::tickets::update_status))
        // External alert -> investigation
        .routes(routes!(routes::tickets::create_alert))
        // Launch endpoints
        .routes(routes!(routes::launch::launch_ticket))
        // Workflow export endpoint
        .routes(routes!(routes::workflow::export))
        // Workflow preview endpoint (issue type -> graph, no ticket)
        .routes(routes!(routes::workflow::preview))
        // Workflow export formats discovery endpoint
        .routes(routes!(routes::workflow::formats))
        // Step completion endpoint (for opr8r wrapper)
        .routes(routes!(routes::launch::complete_step))
        // Kanban provider endpoints
        .routes(routes!(routes::kanban::provider_catalog))
        .routes(routes!(routes::kanban::external_issue_types))
        .routes(routes!(routes::kanban::project_statuses))
        .routes(routes!(routes::kanban::sync_issue_types))
        // Kanban onboarding endpoints (validate, list projects/statuses, write config, set env)
        .routes(routes!(routes::kanban_onboarding::validate_credentials))
        .routes(routes!(routes::kanban_onboarding::list_projects))
        .routes(routes!(routes::kanban_onboarding::list_statuses))
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
        // AgentProfile interchange (import is a distinct static path; export is a
        // static suffix on the `{name}` param path - neither collides with CRUD).
        .routes(routes!(routes::delegators::import_profile))
        .routes(routes!(routes::delegators::export_profile))
        .routes(routes!(
            routes::delegators::get_one,
            routes::delegators::update,
            routes::delegators::delete
        ))
        // Configuration endpoints
        .routes(routes!(
            routes::configuration::get_config,
            routes::configuration::patch_config
        ))
        .routes(routes!(routes::configuration::execution_targets))
        // Model server endpoints
        .routes(routes!(
            routes::model_servers::list,
            routes::model_servers::create
        ))
        .routes(routes!(routes::model_servers::kinds))
        .routes(routes!(routes::model_servers::kind_models))
        .routes(routes!(routes::model_servers::models))
        .routes(routes!(
            routes::model_servers::get_one,
            routes::model_servers::update,
            routes::model_servers::delete
        ))
        // MCP descriptor - always mounted so non-HTTP MCP clients can still
        // discover the stdio entrypoint.
        .routes(routes!(crate::mcp::descriptor::descriptor))
}

/// The canonical OpenAPI spec for the documented API surface.
///
/// Built from [`documented_router`] so it always reflects the mounted routes.
/// Config-gated MCP transport routes remain in the contract so clients can
/// discover their wire format even when a particular deployment disables them.
///
/// The `info.version` is stamped here from `CARGO_PKG_VERSION` - the compiled
/// release version that CI writes into `Cargo.toml`/`VERSION` on every release.
/// This is the single source of version truth for *every* consumer (served
/// swagger-ui, generated `docs/schemas/openapi.json`, and `ApiDoc::json/yaml`),
/// so the spec version always matches `/api/v1/health` and the published release.
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    let mut spec = documented_router().split_for_parts().1;
    spec.info.version = env!("CARGO_PKG_VERSION").to_string();
    openapi::apply_contract_metadata(spec)
}

/// Build the API router with all routes
/// Build the CORS layer from configuration.
///
/// Replaces a blanket `allow_origin(Any)`, which let any website on the
/// internet call this API. `Any` is also incompatible with credentials: a
/// browser refuses to send cookies to a wildcard origin, so the permissive
/// version could not have supported an authenticated dashboard anyway.
///
/// An empty `cors_origins` means **same-origin only** - no `Access-Control-Allow-Origin`
/// is emitted, the same-origin dashboard still works, and no other site can
/// read a response.
fn cors_layer(config: &crate::config::Config) -> CorsLayer {
    let origins: Vec<axum::http::HeaderValue> = config
        .rest_api
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    if origins.is_empty() {
        return CorsLayer::new();
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers(vec![
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static(crate::rest::middleware::auth::CSRF_HEADER),
        ])
        // Required for the dashboard's session cookie to be sent at all.
        .allow_credentials(true)
}

pub fn build_router(state: ApiState) -> Router {
    let profiles = crate::profiles::ServerProfiles::open(state)
        .expect("configuration registry must be available");
    let default = profiles.default_state();
    profile_router::mount(build_profile_router(default), profiles)
}

pub(crate) fn build_profile_router(state: ApiState) -> Router {
    let config = state.config();
    let cors = cors_layer(&config);

    let mcp_enabled = config.mcp.http_enabled;

    let (mut router, _api) = documented_router().split_for_parts();

    // MCP transport endpoints are gated by [mcp].http_enabled.
    if mcp_enabled {
        router = router
            .route("/api/v1/mcp/sse", get(crate::mcp::transport::sse_handler))
            .route(
                "/api/v1/mcp/message",
                post(crate::mcp::transport::message_handler),
            );
    }

    // Swagger UI and its spec are merged in here, and the SPA fallback is
    // registered here, so that the auth layer below covers both. Order matters: `Router::fallback` registered *after* `.layer`
    let router =
        router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi_spec()));

    #[cfg(feature = "embed-ui")]
    let router = router.fallback(web_ui::spa_handler);

    router
        // Authorization runs over the composed router: documented routes, the config-gated MCP transport, Swagger, and the SPA fallback.
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::authorize,
        ))
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors)
        .with_state(state)
}

/// Start the REST API server (standalone mode with session file and logging)
pub async fn serve(state: ApiState, port: u16) -> Result<()> {
    let tickets_path = state.tickets_path.clone();
    let state_path = state.config().auth_state_path();
    let profile_id = state.config().profile.id;
    let host_ip = state.config().rest_api.host_ip();
    let app = build_router(state.clone());
    let addr = SocketAddr::new(host_ip, port);

    tracing::info!("REST API listening on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    // Write session file for client discovery
    write_session_file(&tickets_path, &state_path, port, profile_id)?;

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(state.clone()))
        .await?;

    // Clean up session file on shutdown
    remove_session_file(&tickets_path);

    Ok(())
}

/// Write API session file for client discovery (standalone mode)
fn write_session_file(
    tickets_path: &std::path::Path,
    state_path: &std::path::Path,
    port: u16,
    profile_id: uuid::Uuid,
) -> Result<()> {
    let operator_dir = tickets_path.join("operator");
    std::fs::create_dir_all(&operator_dir)?;

    let session_file = operator_dir.join("api-session.json");
    let session = ApiSessionInfo {
        port,
        profile_id,
        pid: std::process::id(),
        started_at: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        state_dir: state_path.to_path_buf(),
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
async fn shutdown_signal(state: state::ApiState) {
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

    if !state.start_draining() {
        return;
    }

    let config = state.config();
    let drain_deadline =
        Instant::now() + Duration::from_secs(u64::from(config.rest_api.shutdown_drain_seconds));
    tracing::info!(
        drain_seconds = config.rest_api.shutdown_drain_seconds,
        "Shutdown drain started"
    );

    loop {
        let active_agents = crate::state::State::load(&config)
            .map(|app_state| {
                app_state.agents.iter().any(|agent| {
                    matches!(
                        agent.status.as_str(),
                        "running" | "awaiting_input" | "completing"
                    )
                })
            })
            .unwrap_or(true);
        if !active_agents && state.in_flight_operations() == 0 {
            break;
        }
        if Instant::now() >= drain_deadline {
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    state.start_stopping();
    state.mcp_sessions.lock().await.clear();
    let cleanup_deadline =
        Instant::now() + Duration::from_secs(u64::from(config.rest_api.shutdown_cleanup_seconds));
    while state.in_flight_operations() != 0 && Instant::now() < cleanup_deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    if state.in_flight_operations() == 0 {
        let cleanup_config = (*config).clone();
        let cleanup =
            tokio::task::spawn_blocking(move || interrupt_agents_for_shutdown(&cleanup_config));
        let remaining = cleanup_deadline.saturating_duration_since(Instant::now());
        match tokio::time::timeout(remaining, cleanup).await {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(error))) => tracing::error!(%error, "Shutdown agent cleanup failed"),
            Ok(Err(error)) => tracing::error!(%error, "Shutdown cleanup task failed"),
            Err(_) => tracing::error!("Shutdown cleanup exceeded its deadline"),
        }
    } else {
        tracing::error!(
            in_flight = state.in_flight_operations(),
            "Skipping agent state cleanup to avoid overwriting in-flight completion writes"
        );
    }
}

fn interrupt_agents_for_shutdown(config: &crate::config::Config) -> Result<()> {
    use crate::state::ShutdownRecovery;

    const INTERRUPTION_MESSAGE: &str = "Interrupted by Operator shutdown; retry explicitly";

    let launcher = match crate::agents::Launcher::new(config) {
        Ok(launcher) => Some(launcher),
        Err(error) => {
            tracing::warn!(%error, "Session controls are unavailable during shutdown");
            None
        }
    };

    // Reread under the write lock: a completion callback may have landed between the drain loop's last poll and here
    crate::state::State::mutate(config, |app_state| {
        for agent in &mut app_state.agents {
            if !matches!(
                agent.status.as_str(),
                "running" | "awaiting_input" | "completing"
            ) {
                continue;
            }
            let configured_remote = agent
                .target_name
                .as_deref()
                .and_then(|name| config.targets.iter().find(|target| target.name == name))
                .is_some_and(|target| {
                    matches!(
                        &target.kind,
                        crate::config::TargetKind::Coder(_) | crate::config::TargetKind::Ssh(_)
                    )
                });
            let remote = agent.remote_host.is_some()
                || configured_remote
                || agent
                    .launch_mode
                    .as_deref()
                    .is_some_and(|mode| mode.starts_with("coder") || mode.starts_with("ssh"));
            if remote {
                agent.shutdown_recovery = Some(ShutdownRecovery::RemoteAwaitingReconciliation);
                agent.last_message =
                    Some("Remote work preserved during Operator shutdown".to_string());
                continue;
            }

            if agent.session_name.is_some() {
                if let Some(launcher) = &launcher {
                    if let Err(error) = launcher.kill_local_agent_session(agent) {
                        tracing::warn!(%error, agent_id = %agent.id, "Failed to stop local session during shutdown");
                    }
                }
            }
            agent.status = "failed".to_string();
            agent.last_message = Some(INTERRUPTION_MESSAGE.to_string());
            agent.shutdown_recovery = Some(ShutdownRecovery::InterruptedLocal);
            agent.last_activity = chrono::Utc::now();
        }
    })
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

    fn shutdown_config(temp: &tempfile::TempDir) -> Config {
        let mut config = Config::default();
        config.paths.state = temp.path().join("state").display().to_string();
        config.paths.tickets = temp.path().join("tickets").display().to_string();
        config.paths.projects = temp.path().join("projects").display().to_string();
        config
    }

    fn seed_agent(config: &Config, ticket: &str, launch_mode: Option<&str>) -> String {
        crate::state::State::mutate(config, |state| {
            state
                .add_agent_with_options(
                    ticket.to_string(),
                    "FEAT".to_string(),
                    "project".to_string(),
                    false,
                    None,
                    launch_mode.map(str::to_string),
                )
                .unwrap()
        })
        .unwrap()
    }

    /// Shutdown bookkeeping must not resurrect an agent that reported completion
    /// while the drain was running. Both writes go through `State::mutate`, so
    /// the later one rereads the earlier one's result.
    #[test]
    fn shutdown_does_not_overwrite_a_completion_recorded_during_the_drain() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = shutdown_config(&temp);
        let finished = seed_agent(&config, "DONE-1", None);
        let still_running = seed_agent(&config, "BUSY-1", None);

        // The callback lands mid-drain, before cleanup runs.
        crate::state::State::mutate(&config, |state| {
            let agent = state
                .agents
                .iter_mut()
                .find(|agent| agent.id == finished)
                .unwrap();
            agent.status = "completed".to_string();
        })
        .unwrap();

        interrupt_agents_for_shutdown(&config).unwrap();

        let state = crate::state::State::load(&config).unwrap();
        let done = state.agents.iter().find(|a| a.id == finished).unwrap();
        assert_eq!(
            done.status, "completed",
            "a completion recorded during the drain must survive shutdown bookkeeping"
        );
        assert_eq!(
            done.shutdown_recovery, None,
            "finished work needs no recovery marker"
        );

        let busy = state.agents.iter().find(|a| a.id == still_running).unwrap();
        assert_eq!(busy.status, "failed");
        assert_eq!(
            busy.shutdown_recovery,
            Some(crate::state::ShutdownRecovery::InterruptedLocal)
        );
    }

    /// Running shutdown bookkeeping twice (a second signal, or a retry) must be
    /// idempotent rather than compounding.
    #[test]
    fn shutdown_bookkeeping_is_idempotent() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = shutdown_config(&temp);
        let id = seed_agent(&config, "LOCAL-2", None);

        interrupt_agents_for_shutdown(&config).unwrap();
        let first = crate::state::State::load(&config).unwrap();
        interrupt_agents_for_shutdown(&config).unwrap();
        let second = crate::state::State::load(&config).unwrap();

        let before = first.agents.iter().find(|a| a.id == id).unwrap();
        let after = second.agents.iter().find(|a| a.id == id).unwrap();
        assert_eq!(before.status, after.status);
        assert_eq!(before.shutdown_recovery, after.shutdown_recovery);
        assert_eq!(second.agents.len(), 1, "no records may be duplicated");
    }

    /// Remote work is left alone on the way down and picked up by startup
    /// reconciliation, which must not resolve it either.
    #[test]
    fn remote_work_survives_shutdown_and_restart_as_unresolved() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = shutdown_config(&temp);
        let id = seed_agent(&config, "REMOTE-2", Some("coder"));

        interrupt_agents_for_shutdown(&config).unwrap();
        crate::startup::recovery::reconcile(&config).unwrap();

        let agent = crate::state::State::load(&config)
            .unwrap()
            .agents
            .into_iter()
            .find(|a| a.id == id)
            .unwrap();
        assert_eq!(
            agent.status, "running",
            "a surviving remote workspace must not be failed or completed by us"
        );
        assert_eq!(
            agent.shutdown_recovery,
            Some(crate::state::ShutdownRecovery::RemoteAwaitingReconciliation)
        );
    }

    #[test]
    fn shutdown_marks_local_work_interrupted_and_preserves_remote_work() {
        let temp = tempfile::TempDir::new().unwrap();
        let mut config = Config::default();
        config.paths.state = temp.path().join("state").display().to_string();
        config.paths.tickets = temp.path().join("tickets").display().to_string();
        config.paths.projects = temp.path().join("projects").display().to_string();

        let mut app_state = crate::state::State::load(&config).unwrap();
        let local_id = app_state
            .add_agent_with_full_options(
                "LOCAL-1".to_string(),
                "FEAT".to_string(),
                "project".to_string(),
                false,
                None,
                Some("default".to_string()),
                None,
            )
            .unwrap();
        let remote_id = app_state
            .add_agent_with_full_options(
                "REMOTE-1".to_string(),
                "FEAT".to_string(),
                "project".to_string(),
                false,
                None,
                Some("coder".to_string()),
                None,
            )
            .unwrap();

        interrupt_agents_for_shutdown(&config).unwrap();
        let app_state = crate::state::State::load(&config).unwrap();
        let local = app_state
            .agents
            .iter()
            .find(|agent| agent.id == local_id)
            .unwrap();
        assert_eq!(local.status, "failed");
        assert_eq!(
            local.shutdown_recovery,
            Some(crate::state::ShutdownRecovery::InterruptedLocal)
        );
        let remote = app_state
            .agents
            .iter()
            .find(|agent| agent.id == remote_id)
            .unwrap();
        assert_eq!(remote.status, "running");
        assert_eq!(
            remote.shutdown_recovery,
            Some(crate::state::ShutdownRecovery::RemoteAwaitingReconciliation)
        );
    }
}
