//! OAuth protected-resource metadata (RFC 9728) for MCP and REST clients.

use crate::rest::state::RequestState as State;
use axum::Json;

use crate::auth::oidc_bearer::{active_connections, protected_resource_metadata};
use crate::rest::dto::auth::ProtectedResourceMetadata;
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// Protected resource metadata
///
/// Names the OIDC issuers that may issue bearer tokens for this server.
/// Absent unless `rest_api.public_url` is set and an OIDC connection is enabled.
#[utoipa::path(
    operation_id = "oauth_protected_resource",
    get,
    path = "/.well-known/oauth-protected-resource",
    tag = "Auth",
    responses(
        (status = 200, description = "Protected resource metadata", body = ProtectedResourceMetadata),
        (status = 404, description = "OIDC discovery is not configured"),
    )
)]
pub async fn protected_resource(
    State(state): State<ApiState>,
) -> Result<Json<ProtectedResourceMetadata>, ApiError> {
    let unavailable = || ApiError::NotFound("OIDC discovery is not configured".into());
    let public = state
        .config()
        .rest_api
        .public_base_url()
        .ok_or_else(unavailable)?;
    let store = state.auth.store.clone();
    let conns = tokio::task::spawn_blocking(move || store.oidc_connections())
        .await
        .map_err(|_| ApiError::Unavailable("Credential storage unavailable".into()))?
        .map_err(|_| ApiError::Unavailable("Credential storage unavailable".into()))?;
    protected_resource_metadata(&public, &active_connections(Some(&public), conns))
        .map(Json)
        .ok_or_else(unavailable)
}
