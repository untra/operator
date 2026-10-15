use crate::rest::state::RequestState as State;
use axum::extract::{Path, Query};
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::identity::{
    self, CreateOidcConnection, CreateUserKey, IssuedUserKey, OidcConnection, OperatorUser,
    PermissionGrant, UpdateOperatorUser, UserAuthority, UserKey,
};
use crate::auth::scope::Principal;
use crate::licensing::{require_premium, PremiumFeature};
use crate::rest::dto::auth::PrincipalKind;
use crate::rest::error::ApiError;
use crate::rest::middleware::auth::{Authenticated, SESSION_COOKIE};
use crate::rest::state::ApiState;

fn profile_state(
    state: &ApiState,
    profiles: Option<axum::Extension<std::sync::Arc<crate::profiles::ServerProfiles>>>,
    id: uuid::Uuid,
) -> Result<ApiState, ApiError> {
    profiles
        .and_then(|p| p.state(id))
        .or_else(|| (state.config().profile.id == id).then(|| state.clone()))
        .ok_or_else(|| ApiError::NotFound("Unknown profile".into()))
}

const LOGIN_COOKIE: &str = "__Host-operator_oidc";

fn failure(_: impl std::fmt::Display) -> ApiError {
    ApiError::ValidationError(
        "Identity operation rejected; check the supplied values and current permissions".into(),
    )
}

pub fn administrator(principal: &Principal) -> Result<(), ApiError> {
    if principal.kind != PrincipalKind::Session {
        return Err(ApiError::Forbidden(
            "Use an administrator browser session".into(),
        ));
    }
    if principal
        .authority
        .as_ref()
        .is_some_and(|a| !a.user.administrator)
    {
        return Err(ApiError::Forbidden("Administrator access required".into()));
    }
    Ok(())
}

fn owner(principal: &Principal, recent: bool) -> Result<UserAuthority, ApiError> {
    if principal.kind != PrincipalKind::Session {
        return Err(ApiError::Forbidden("Use an OIDC browser session".into()));
    }
    let authority = principal
        .authority
        .clone()
        .ok_or_else(|| ApiError::Forbidden("Sign in with OIDC to manage API keys".into()))?;
    if recent
        && !authority.authenticated_at.is_some_and(|at| {
            (0..=identity::RECENT_LOGIN_SECONDS).contains(&(chrono::Utc::now().timestamp() - at))
        })
    {
        return Err(ApiError::Forbidden(
            "Sign in with OIDC again before issuing a key".into(),
        ));
    }
    Ok(authority)
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(failure)?
        .map_err(|error| {
            if error.is::<identity::IssuanceLimited>() {
                ApiError::RateLimited {
                    retry_after_secs: u64::from(identity::ISSUANCE_WINDOW_SECONDS),
                }
            } else {
                failure(error)
            }
        })
}

fn no_store<T: Serialize>(value: T) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}

#[derive(Serialize, ToSchema, ts_rs::TS)]
#[ts(export)]
pub struct OidcLoginProvider {
    pub id: String,
    pub provider: identity::OidcProvider,
}

#[utoipa::path(get, path = "/api/v1/auth/oidc/providers", tag = "Auth", responses((status = 200, body = Vec<OidcLoginProvider>)))]
pub async fn providers(State(state): State<ApiState>) -> Result<Response, ApiError> {
    let store = state.auth.store.clone();
    let connections = blocking(move || store.oidc_connections()).await?;
    Ok(no_store(
        connections
            .into_iter()
            .filter(|c| !c.disabled)
            .map(|c| OidcLoginProvider {
                id: c.id,
                provider: c.provider,
            })
            .collect::<Vec<_>>(),
    ))
}

#[utoipa::path(get, path = "/api/v1/auth/oidc/connections", tag = "Auth", responses((status = 200, body = Vec<OidcConnection>)))]
pub async fn connections(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Response, ApiError> {
    administrator(&principal)?;
    let store = state.auth.store.clone();
    Ok(no_store(blocking(move || store.oidc_connections()).await?))
}

#[utoipa::path(post, path = "/api/v1/auth/oidc/connections", tag = "Auth", request_body = CreateOidcConnection, responses((status = 200, body = String)))]
pub async fn create_connection(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Json(request): Json<CreateOidcConnection>,
) -> Result<Response, ApiError> {
    administrator(&principal)?;
    require_premium(&state.config(), PremiumFeature::OidcConnections)?;
    crate::auth::oidc::validate_url(&request.issuer).map_err(failure)?;
    let redirect = crate::auth::oidc::validate_url(&request.redirect_uri).map_err(failure)?;
    let base = state
        .config()
        .rest_api
        .public_base_url()
        .ok_or_else(|| {
            ApiError::ValidationError("Configure the public HTTPS URL before OIDC".into())
        })?
        .clone();
    let expected = format!("{}/api/v1/auth/oidc/callback", base.trim_end_matches('/'));
    if redirect.as_str() != expected {
        return Err(ApiError::ValidationError(format!(
            "Redirect URI must be {expected}"
        )));
    }
    if request.client_id.is_empty() || request.client_id.len() > 128 {
        return Err(ApiError::ValidationError("Invalid client ID".into()));
    }
    let store = state.auth.store.clone();
    Ok(no_store(
        blocking(move || store.add_oidc_connection(&principal.subject, &request)).await?,
    ))
}

#[utoipa::path(delete, path = "/api/v1/auth/oidc/connections/{id}", tag = "Auth", params(("id" = String, Path)), responses((status = 200)))]
pub async fn disable_connection(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    administrator(&principal)?;
    let store = state.auth.store.clone();
    blocking(move || store.disable_oidc_connection(&principal.subject, &id)).await?;
    Ok(no_store(()))
}

#[utoipa::path(get, path = "/api/v1/auth/oidc/login/{id}", tag = "Auth", params(("id" = String, Path)), responses((status = 303)))]
pub async fn login(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let store = state.auth.store.clone();
    let connections = blocking(move || store.oidc_connections()).await?;
    let connection = connections
        .into_iter()
        .find(|c| c.id == id && !c.disabled)
        .ok_or_else(|| ApiError::NotFound("Unknown connection".into()))?;
    let cookie = crate::auth::secret::generate_secret().map_err(failure)?;
    let url = crate::auth::oidc::start(&state.auth.store, connection, &cookie)
        .await
        .map_err(failure)?;
    Ok((
        [
            (
                header::SET_COOKIE,
                format!(
                    "{LOGIN_COOKIE}={cookie}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=300"
                ),
            ),
            (header::CACHE_CONTROL, "no-store".into()),
        ],
        Redirect::to(&url),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

#[utoipa::path(get, path = "/api/v1/auth/oidc/callback", tag = "Auth", responses((status = 303)))]
pub async fn callback(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, ApiError> {
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            h.split(';')
                .filter_map(|p| p.trim().split_once('='))
                .find(|(k, _)| *k == LOGIN_COOKIE)
                .map(|(_, v)| v)
        })
        .ok_or_else(|| ApiError::Unauthorized("Missing login cookie".into()))?;
    let (session, _, _) =
        crate::auth::oidc::complete(&state.auth.store, &query.state, cookie, query.code)
            .await
            .map_err(|_| {
                ApiError::Unauthorized(
                    "OIDC sign-in failed or the account awaits administrator approval".into(),
                )
            })?;
    let mut response = Redirect::to("/#/connections").into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response.headers_mut().append(
        header::SET_COOKIE,
        format!(
            "{SESSION_COOKIE}={session}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=43200"
        )
        .parse()
        .map_err(failure)?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        format!("{LOGIN_COOKIE}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0")
            .parse()
            .map_err(failure)?,
    );
    Ok(response)
}

#[utoipa::path(get, path = "/api/v1/auth/users", tag = "Auth", responses((status = 200, body = Vec<OperatorUser>)))]
pub async fn users(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Query(page): Query<KeyPage>,
) -> Result<Response, ApiError> {
    administrator(&principal)?;
    let store = state.auth.store.clone();
    Ok(no_store(
        blocking(move || store.operator_users(page.offset)).await?,
    ))
}

#[utoipa::path(put, path = "/api/v1/auth/users/{id}", tag = "Auth", params(("id" = String, Path)), request_body = UpdateOperatorUser, responses((status = 200)))]
pub async fn update_user(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
    profiles: Option<axum::Extension<std::sync::Arc<crate::profiles::ServerProfiles>>>,
    Json(request): Json<UpdateOperatorUser>,
) -> Result<Response, ApiError> {
    let state = profile_state(&state, profiles, request.grants.profile_id)?;
    administrator(&principal)?;
    let store = state.auth.store.clone();
    let actor = principal.subject;
    let premium = require_premium(&state.config(), PremiumFeature::OidcConnections).is_ok();
    if request
        .grants
        .projects
        .iter()
        .any(|p| !state.config().projects.contains(p))
    {
        return Err(ApiError::ValidationError("Unknown project".into()));
    }
    blocking(move || store.update_operator_user(&actor, &id, &request, premium)).await?;
    Ok(no_store(()))
}

#[derive(Serialize, ToSchema, ts_rs::TS)]
#[ts(export)]
pub struct CapabilityResponse {
    pub grants: PermissionGrant,
    pub administration_grants: Option<PermissionGrant>,
    pub oidc_user: bool,
    pub administrator: bool,
    pub can_issue: bool,
    pub connection_id: Option<String>,
}

#[utoipa::path(get, path = "/api/v1/auth/capabilities", tag = "Auth", responses((status = 200, body = CapabilityResponse)))]
pub async fn capabilities(
    State(state): State<ApiState>,
    Authenticated(mut principal): Authenticated,
    Query(page): Query<KeyPage>,
    profiles: Option<axum::Extension<std::sync::Arc<crate::profiles::ServerProfiles>>>,
) -> Result<Response, ApiError> {
    let state = profile_state(
        &state,
        profiles,
        page.profile_id.unwrap_or(state.config().profile.id),
    )?;
    if let Some(existing) = principal.authority.as_ref() {
        let mut authority = state
            .auth
            .store
            .user_authority(&existing.user.id, state.config().profile.id)
            .map_err(failure)?
            .ok_or_else(|| ApiError::Unauthorized("Account disabled".into()))?;
        authority.authenticated_at = existing.authenticated_at;
        principal.authority = Some(authority);
    }
    let authority = principal.authority.as_ref();
    Ok(no_store(CapabilityResponse {
        grants: authority.map_or_else(
            || PermissionGrant {
                profile_id: state.config().profile.id,
                instance_wide: true,
                capabilities: identity::capabilities(),
                projects: state.config().projects.clone(),
            },
            |a| a.grants.clone(),
        ),
        administration_grants: administrator(&principal).is_ok().then(|| PermissionGrant {
            profile_id: state.config().profile.id,
            instance_wide: true,
            capabilities: identity::capabilities(),
            projects: state.config().projects.clone(),
        }),
        oidc_user: authority.is_some(),
        administrator: authority.is_none_or(|a| a.user.administrator)
            && principal.kind == PrincipalKind::Session,
        can_issue: owner(&principal, true).is_ok()
            && require_premium(&state.config(), PremiumFeature::ApiKeys).is_ok(),
        connection_id: authority.map(|a| a.user.connection_id.clone()),
    }))
}

#[derive(Default, Deserialize)]
pub struct KeyPage {
    #[serde(default)]
    offset: u32,
    profile_id: Option<uuid::Uuid>,
}

#[derive(Serialize, ToSchema, ts_rs::TS)]
#[ts(export)]
pub struct UserKeyList {
    pub keys: Vec<UserKey>,
}

#[utoipa::path(get, path = "/api/v1/auth/keys", tag = "Auth", responses((status = 200, body = UserKeyList)))]
pub async fn keys(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Query(page): Query<KeyPage>,
) -> Result<Response, ApiError> {
    let owner = owner(&principal, false)?;
    let store = state.auth.store.clone();
    let profile = page.profile_id.unwrap_or(state.config().profile.id);
    Ok(no_store(UserKeyList {
        keys: blocking(move || store.user_keys(&owner.user.id, profile, page.offset)).await?,
    }))
}

#[utoipa::path(post, path = "/api/v1/auth/keys", tag = "Auth", request_body = CreateUserKey, responses((status = 200, body = IssuedUserKey)))]
pub async fn create_key(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    profiles: Option<axum::Extension<std::sync::Arc<crate::profiles::ServerProfiles>>>,
    Json(request): Json<CreateUserKey>,
) -> Result<Response, ApiError> {
    let state = profile_state(&state, profiles, request.grants.profile_id)?;
    let owner = owner(&principal, true)?;
    require_premium(&state.config(), PremiumFeature::ApiKeys)?;
    let store = state.auth.store.clone();
    Ok(no_store(
        blocking(move || store.create_user_key(&owner, &request, None)).await?,
    ))
}

#[utoipa::path(post, path = "/api/v1/auth/keys/{id}/rotate", tag = "Auth", params(("id" = String, Path)), request_body = CreateUserKey, responses((status = 200, body = IssuedUserKey)))]
pub async fn rotate_key(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
    profiles: Option<axum::Extension<std::sync::Arc<crate::profiles::ServerProfiles>>>,
    Json(request): Json<CreateUserKey>,
) -> Result<Response, ApiError> {
    let state = profile_state(&state, profiles, request.grants.profile_id)?;
    let owner = owner(&principal, true)?;
    require_premium(&state.config(), PremiumFeature::ApiKeys)?;
    let store = state.auth.store.clone();
    Ok(no_store(
        blocking(move || store.create_user_key(&owner, &request, Some(&id))).await?,
    ))
}

#[utoipa::path(delete, path = "/api/v1/auth/keys/{id}", tag = "Auth", params(("id" = String, Path)), responses((status = 200)))]
pub async fn revoke_key(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    if principal.kind != PrincipalKind::Session {
        return Err(ApiError::Forbidden("Use a browser session".into()));
    }
    let admin = administrator(&principal).is_ok();
    let store = state.auth.store.clone();
    let revoked = blocking(move || store.revoke_user_key(&principal.subject, &id, admin)).await?;
    if !revoked {
        return Err(ApiError::NotFound("Unknown key".into()));
    }
    Ok(no_store(()))
}

pub async fn exchange_key(
    state: &ApiState,
    secret: String,
) -> Result<Json<crate::rest::dto::auth::TokenResponse>, ApiError> {
    let store = state.auth.store.clone();
    let (authority, expires) = blocking(move || store.exchange_user_key(&secret))
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid, expired, or revoked key".into()))?;
    let now = chrono::Utc::now();
    let mut claims = crate::auth::tokens::api_claims(
        &authority.user.id,
        &[],
        now,
        uuid::Uuid::new_v4().to_string(),
    );
    claims.key_id = authority.key_id;
    claims.profile_id = Some(authority.grants.profile_id);
    claims.exp = claims.exp.min(expires.timestamp());
    let access_token = state.auth.signing_key.sign(&claims).map_err(failure)?;
    Ok(Json(crate::rest::dto::auth::TokenResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: u64::try_from(claims.exp - now.timestamp()).unwrap_or(0),
        refresh_token: None,
        scopes: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_key_http_scopes_profile_projects_and_revocation() {
        let config = crate::config::Config {
            projects: vec!["allowed".into(), "hidden".into()],
            ..Default::default()
        };
        let state = ApiState::in_memory(config);
        let (_, mut owner, mut request) =
            crate::auth::identity::tests::fixture_in(state.auth.store.clone());
        request.grants.profile_id = state.config().profile.id;
        request.grants.projects = vec!["allowed".into()];
        request.grants.capabilities = vec!["projects:read".into()];
        state
            .auth
            .store
            .update_operator_user(
                "admin",
                &owner.user.id,
                &UpdateOperatorUser {
                    enabled: true,
                    administrator: true,
                    grants: request.grants.clone(),
                },
                true,
            )
            .unwrap();
        owner = state
            .auth
            .store
            .user_authority(&owner.user.id, request.grants.profile_id)
            .unwrap()
            .unwrap();
        let issued = state
            .auth
            .store
            .create_user_key(&owner, &request, None)
            .unwrap();
        let token = exchange_key(&state, issued.secret)
            .await
            .unwrap()
            .0
            .access_token;
        let app = crate::rest::build_profile_router(state.clone());
        for (method, path, expected) in [
            ("GET", "/api/v1/projects", StatusCode::OK),
            ("GET", "/api/v1/agents", StatusCode::FORBIDDEN),
            ("POST", "/api/v1/auth/keys", StatusCode::FORBIDDEN),
            ("POST", "/api/v1/auth/device/approve", StatusCode::FORBIDDEN),
            ("POST", "/otlp/v1/logs", StatusCode::FORBIDDEN),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "{method} {path}");
            if expected == StatusCode::OK {
                let body = response.into_body().collect().await.unwrap().to_bytes();
                let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(value.as_array().unwrap().len(), 1);
                assert_eq!(value[0]["project_name"], "allowed");
            }
        }
        state
            .auth
            .store
            .revoke_user_key(&owner.user.id, &issued.key.summary.id, false)
            .unwrap();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
