//! Paired endpoint machines: list, inspect, update, revoke, and the uplink.
//!
//! The durable record lives in the auth store; liveness and the freshest
//! capabilities come from the in-memory [`EndpointRegistry`] while connected.
//!
//! [`EndpointRegistry`]: crate::endpoint::registry::EndpointRegistry

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::Path;
use axum::response::Response;
use axum::Json;
use chrono::Utc;

use crate::auth::audit::AuditAction;
use crate::auth::scope::Principal;
use crate::auth::store::{EndpointRecord, ENDPOINT_NAME_MAX_CHARS};
use crate::endpoint::protocol::Capabilities;
use crate::endpoint::registry::REVOKED;
use crate::rest::dto::auth::PrincipalKind;
use crate::rest::dto::{EndpointDetail, EndpointSummary, EndpointsResponse, UpdateEndpointRequest};
use crate::rest::error::ApiError;
use crate::rest::middleware::auth::Authenticated;
use crate::rest::state::ApiState;
use crate::rest::state::RequestState as State;

async fn blocking<T, F>(f: F) -> Result<T, ApiError>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| ApiError::InternalError(format!("endpoint task failed: {e}")))?
        .map_err(|e| ApiError::InternalError(e.to_string()))
}

fn not_found(id: &str) -> ApiError {
    ApiError::NotFound(format!("no endpoint {id}"))
}

fn stored_capabilities(record: &EndpointRecord) -> Option<Capabilities> {
    record
        .capabilities_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
}

fn detail(state: &ApiState, record: EndpointRecord) -> EndpointDetail {
    let live = state.endpoints.snapshot(&record.id);
    let capabilities = live
        .as_ref()
        .and_then(|s| s.capabilities.clone())
        .or_else(|| stored_capabilities(&record));
    let last_seen_at = live
        .as_ref()
        .and_then(|s| s.last_heartbeat)
        .or(record.last_seen_at);
    EndpointDetail {
        summary: EndpointSummary {
            liveness: state.endpoints.liveness(&record.id, Utc::now()),
            gpu_count: capabilities.as_ref().map_or(0, |c| c.gpus.len()),
            id: record.id,
            name: record.name,
            created_at: record.created_at,
            last_seen_at,
            launch_enabled: record.launch_enabled,
        },
        capabilities_stale: live.is_some_and(|s| s.capabilities_stale),
        capabilities,
    }
}

async fn live_record(state: &ApiState, id: String) -> Result<EndpointRecord, ApiError> {
    let store = state.auth.store.clone();
    let lookup = id.clone();
    blocking(move || store.get_endpoint(&lookup))
        .await?
        .filter(|record| record.revoked_at.is_none())
        .ok_or_else(|| not_found(&id))
}

/// List paired endpoints
#[utoipa::path(
    operation_id = "endpoints_list",
    get,
    path = "/api/v1/endpoints",
    tag = "Endpoints",
    responses((status = 200, description = "Paired, unrevoked endpoints", body = EndpointsResponse))
)]
pub async fn list(State(state): State<ApiState>) -> Result<Json<EndpointsResponse>, ApiError> {
    let store = state.auth.store.clone();
    let records = blocking(move || store.list_endpoints()).await?;
    let endpoints: Vec<EndpointSummary> = records
        .into_iter()
        .filter(|record| record.revoked_at.is_none())
        .map(|record| detail(&state, record).summary)
        .collect();
    let total = endpoints.len();
    Ok(Json(EndpointsResponse { endpoints, total }))
}

/// Get one endpoint
#[utoipa::path(
    operation_id = "endpoints_get_one",
    get,
    path = "/api/v1/endpoints/{id}",
    tag = "Endpoints",
    params(("id" = String, Path, description = "Endpoint id")),
    responses(
        (status = 200, description = "Endpoint detail", body = EndpointDetail),
        (status = 404, description = "Unknown or revoked endpoint"),
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<EndpointDetail>, ApiError> {
    let record = live_record(&state, id).await?;
    Ok(Json(detail(&state, record)))
}

/// Rename an endpoint or toggle launching on it
#[utoipa::path(
    operation_id = "endpoints_update",
    patch,
    path = "/api/v1/endpoints/{id}",
    tag = "Endpoints",
    params(("id" = String, Path, description = "Endpoint id")),
    request_body = UpdateEndpointRequest,
    responses(
        (status = 200, description = "Updated endpoint", body = EndpointDetail),
        (status = 400, description = "Invalid name"),
        (status = 404, description = "Unknown or revoked endpoint"),
    )
)]
pub async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEndpointRequest>,
) -> Result<Json<EndpointDetail>, ApiError> {
    let name = match req.name.as_deref().map(str::trim) {
        Some("") => {
            return Err(ApiError::ValidationError(
                "endpoint name must not be blank".into(),
            ))
        }
        Some(name) if name.chars().count() > ENDPOINT_NAME_MAX_CHARS => {
            return Err(ApiError::ValidationError(format!(
                "endpoint name must be at most {ENDPOINT_NAME_MAX_CHARS} characters"
            )))
        }
        other => other.map(str::to_string),
    };
    let store = state.auth.store.clone();
    let target = id.clone();
    let launch_enabled = req.launch_enabled;
    let changed =
        blocking(move || store.update_endpoint(&target, name.as_deref(), launch_enabled)).await?;
    if !changed {
        return Err(not_found(&id));
    }
    let record = live_record(&state, id).await?;
    Ok(Json(detail(&state, record)))
}

/// Revoke an endpoint
#[utoipa::path(
    operation_id = "endpoints_revoke",
    delete,
    path = "/api/v1/endpoints/{id}",
    tag = "Endpoints",
    params(("id" = String, Path, description = "Endpoint id")),
    responses(
        (status = 204, description = "Revoked; its uplink is closed"),
        (status = 404, description = "Unknown or already revoked endpoint"),
    )
)]
pub async fn revoke(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, ApiError> {
    let store = state.auth.store.clone();
    let target = id.clone();
    let revoked = blocking(move || {
        let revoked = store.revoke_endpoint(&target, "revoked by admin")?;
        if revoked {
            store.audit(AuditAction::EndpointRevoked, Some(&target), true)?;
        }
        Ok(revoked)
    })
    .await?;
    if !revoked {
        return Err(not_found(&id));
    }
    state.endpoints.close(&id, REVOKED);
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// The endpoint id an uplink may open as: only a store-checked endpoint principal.
fn uplink_endpoint_id(principal: &Principal) -> Result<String, ApiError> {
    match (&principal.kind, &principal.endpoint_id) {
        (PrincipalKind::Endpoint, Some(id)) => Ok(id.clone()),
        _ => Err(ApiError::Forbidden(
            "the uplink is only for paired endpoints".into(),
        )),
    }
}

/// Open the endpoint uplink (WebSocket)
#[utoipa::path(
    operation_id = "endpoints_uplink",
    get,
    path = "/api/v1/endpoints/uplink",
    tag = "Endpoints",
    responses(
        (status = 101, description = "Switching to the endpoint uplink WebSocket protocol"),
        (status = 403, description = "Caller is not a paired endpoint"),
    )
)]
pub async fn uplink(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let endpoint_id = uplink_endpoint_id(&principal)?;
    Ok(ws.on_upgrade(move |socket| crate::endpoint::uplink::run(socket, state, endpoint_id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::tokens::ENDPOINT_CLIENT_ID;
    use crate::config::Config;
    use crate::endpoint::protocol::{Capabilities, GpuInfo, GpuVendor};
    use crate::endpoint::registry::{Liveness, Outbound, OUTBOUND_BUFFER};
    use chrono::Utc;
    use tokio::sync::mpsc;

    fn fresh() -> (tempfile::TempDir, ApiState) {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = dir.path().to_string_lossy().into_owned();
        let state = ApiState::new(config, dir.path().join("tickets"));
        (dir, state)
    }

    fn pair(state: &ApiState) -> String {
        state
            .auth
            .store
            .create_endpoint_family(ENDPOINT_CLIENT_ID)
            .unwrap()
            .0
    }

    fn caps() -> Capabilities {
        Capabilities {
            hash: "h1".into(),
            hostname: "GAMING-PC".into(),
            os: "windows".into(),
            arch: "x86_64".into(),
            gpus: vec![GpuInfo {
                name: "RTX 3090".into(),
                vendor: GpuVendor::Nvidia,
                vram_total_mb: 24_576,
                vram_used_mb: None,
                driver: None,
            }],
            llm_tools: vec![],
            model_servers: vec![],
            projects: vec![],
            launch_enabled: true,
        }
    }

    fn connect_live(state: &ApiState, id: &str) -> mpsc::Receiver<Outbound> {
        let (tx, rx) = mpsc::channel(OUTBOUND_BUFFER);
        let generation = state.endpoints.connect(id, tx);
        assert!(state
            .endpoints
            .set_capabilities(id, generation, caps(), Utc::now()));
        rx
    }

    #[tokio::test]
    async fn list_reports_live_endpoints_and_hides_revoked_ones() {
        let (_dir, state) = fresh();
        assert_eq!(list(State(state.clone())).await.unwrap().0.total, 0);

        let live = pair(&state);
        let _rx = connect_live(&state, &live);
        let offline = pair(&state);
        let revoked = pair(&state);
        state.auth.store.revoke_endpoint(&revoked, "test").unwrap();

        let body = list(State(state.clone())).await.unwrap().0;
        assert_eq!(body.total, 2);
        let by_id = |id: &str| body.endpoints.iter().find(|e| e.id == id).unwrap().clone();
        assert_eq!(by_id(&live).liveness, Liveness::Live);
        assert_eq!(by_id(&live).gpu_count, 1);
        assert_eq!(by_id(&offline).liveness, Liveness::Offline);
        assert_eq!(by_id(&offline).gpu_count, 0);
        assert!(body.endpoints.iter().all(|e| e.id != revoked));
    }

    #[tokio::test]
    async fn detail_prefers_live_capabilities_and_falls_back_to_stored() {
        let (_dir, state) = fresh();
        let id = pair(&state);
        let detail = get_one(State(state.clone()), Path(id.clone()))
            .await
            .unwrap()
            .0;
        assert!(detail.capabilities.is_none());

        let rx = connect_live(&state, &id);
        let detail = get_one(State(state.clone()), Path(id.clone()))
            .await
            .unwrap()
            .0;
        assert_eq!(detail.capabilities, Some(caps()));
        assert!(!detail.capabilities_stale);
        drop(rx);

        let json = serde_json::to_string(&caps()).unwrap();
        state
            .auth
            .store
            .record_endpoint_capabilities(&id, &json, "h1", Utc::now())
            .unwrap();
        let mut restarted = state.clone();
        restarted.endpoints = std::sync::Arc::default();
        let detail = get_one(State(restarted), Path(id)).await.unwrap().0;
        assert_eq!(
            detail.capabilities,
            Some(caps()),
            "from the store after restart"
        );
        assert_eq!(detail.summary.liveness, Liveness::Offline);
    }

    #[tokio::test]
    async fn unknown_and_revoked_endpoints_are_not_found() {
        let (_dir, state) = fresh();
        assert!(matches!(
            get_one(State(state.clone()), Path("nope".into())).await,
            Err(ApiError::NotFound(_))
        ));
        let id = pair(&state);
        state.auth.store.revoke_endpoint(&id, "test").unwrap();
        assert!(matches!(
            get_one(State(state.clone()), Path(id)).await,
            Err(ApiError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn update_renames_and_toggles_launch_with_validation() {
        let (_dir, state) = fresh();
        let id = pair(&state);
        let updated = update(
            State(state.clone()),
            Path(id.clone()),
            Json(UpdateEndpointRequest {
                name: Some("  rig  ".into()),
                launch_enabled: Some(true),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(updated.summary.name, "rig");
        assert!(updated.summary.launch_enabled);

        for bad in ["   ".to_string(), "x".repeat(ENDPOINT_NAME_MAX_CHARS + 1)] {
            let result = update(
                State(state.clone()),
                Path(id.clone()),
                Json(UpdateEndpointRequest {
                    name: Some(bad),
                    launch_enabled: None,
                }),
            )
            .await;
            assert!(matches!(result, Err(ApiError::ValidationError(_))));
        }

        let result = update(
            State(state.clone()),
            Path("nope".into()),
            Json(UpdateEndpointRequest {
                name: None,
                launch_enabled: Some(false),
            }),
        )
        .await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }

    #[tokio::test]
    async fn revoke_closes_the_uplink_and_ends_the_credential() {
        let (_dir, state) = fresh();
        let id = pair(&state);
        let mut rx = connect_live(&state, &id);

        revoke(State(state.clone()), Path(id.clone()))
            .await
            .unwrap();
        assert_eq!(rx.try_recv().unwrap(), Outbound::Close(REVOKED));
        assert!(state.auth.store.endpoint_authority(&id).unwrap().is_none());
        assert!(matches!(
            revoke(State(state.clone()), Path(id)).await,
            Err(ApiError::NotFound(_))
        ));
    }

    #[test]
    fn uplink_requires_an_endpoint_bound_principal() {
        let mut admin = crate::auth::scope::Principal::local("admin");
        assert!(uplink_endpoint_id(&admin).is_err());
        admin.endpoint_id = Some("ep".into());
        assert!(
            uplink_endpoint_id(&admin).is_err(),
            "kind must be endpoint too"
        );
        admin.kind = crate::rest::dto::auth::PrincipalKind::Endpoint;
        assert_eq!(uplink_endpoint_id(&admin).unwrap(), "ep");
    }
}
