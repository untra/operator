//! Kubernetes liveness and readiness probes.
//!
//! These exist as a separate, public pair precisely so `/api/v1/health` does
//! not have to be. That endpoint reports the workspace directory name and a
//! directory identifier — workspace identity, which an unauthenticated probe
//! should not disclose. These two carry no metadata at all: the HTTP status is
//! the entire signal.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::rest::state::ApiState;

/// Liveness probe
///
/// Answers only "is the process serving HTTP". It deliberately does not touch
/// the database: a liveness failure restarts the pod, and restarting will not
/// fix a corrupt database — it would just crash-loop.
#[utoipa::path(
    operation_id = "livez",
    get,
    path = "/livez",
    tag = "Health",
    responses((status = 200, description = "Process is alive"))
)]
pub async fn livez() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// Readiness probe
///
/// Answers "can this instance serve requests", which additionally requires the auth database to be reachable
/// An uninitialized deployment awaiting bootstrap is **ready**
#[utoipa::path(
    operation_id = "readyz",
    get,
    path = "/readyz",
    tag = "Health",
    responses(
        (status = 200, description = "Ready to serve"),
        (status = 503, description = "Not ready"),
    )
)]
pub async fn readyz(State(state): State<ApiState>) -> impl IntoResponse {
    let store = state.auth.store.clone();
    let reachable = tokio::task::spawn_blocking(move || store.bootstrap_state())
        .await
        .is_ok_and(|r| r.is_ok());

    if reachable {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "auth store unavailable")
    }
}
