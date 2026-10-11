//! Authentication endpoints: bootstrap, login, sessions, device flow, keys.
//!
//! Everything here that issues or accepts a credential is rate-limited with
//! persisted backoff, so restarting the process does not reset an attacker's
//! budget.

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;

use crate::auth::store::{
    AuthStore, DevicePollOutcome, RefreshOutcome, ADMIN_SUBJECT, DEVICE_CODE_TTL_SECS,
    DEVICE_POLL_INTERVAL_SECS,
};
use crate::auth::tokens::{api_claims, ACCESS_TOKEN_TTL};
use crate::rest::dto::auth::{
    AccessKeyListResponse, BootstrapState, BootstrapStatusResponse, BootstrapSubmitRequest,
    BootstrapSubmitResponse, CreateAccessKeyRequest, CreateAccessKeyResponse, CsrfTokenResponse,
    CurrentSessionResponse, DeviceApprovalRequest, DeviceApprovalResponse,
    DeviceAuthorizationRequest, DeviceAuthorizationResponse, ForgotPasswordRequest,
    ForgotPasswordResponse, LoginRequest, LoginResponse, LogoutResponse, OAuthErrorCode,
    OAuthErrorResponse, ResetPasswordRequest, ResetPasswordResponse, RevokeAccessKeyResponse,
    Scope, SessionListResponse, TokenRequest, TokenResponse,
};
use crate::rest::error::{ApiError, Rejection};
use crate::rest::middleware::auth::{enforce_backoff, Authenticated, SESSION_COOKIE};
use crate::rest::state::ApiState;

/// Rate-limit bucket names.
const BUCKET_BOOTSTRAP: &str = "bootstrap";
const BUCKET_LOGIN: &str = "login";
const BUCKET_PASSWORD_RESET: &str = "password_reset";
const BUCKET_DEVICE_CODE: &str = "device_code";
const BUCKET_TOKEN: &str = "token";

/// Env var naming a file holding the out-of-band bootstrap password.
///
/// A file rather than a plain env var: an env var is visible in `/proc`, in
/// `docker inspect`, and to every child process Operator spawns - including the
/// agent processes, which is precisely the thing that must not read it.
pub const BOOTSTRAP_PASSWORD_FILE_ENV: &str = "OPERATOR_BOOTSTRAP_PASSWORD_FILE";

/// Read the mounted bootstrap password, if one was supplied.
fn mounted_bootstrap_password() -> Option<String> {
    let path = std::env::var(BOOTSTRAP_PASSWORD_FILE_ENV).ok()?;
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn blocking<T, F>(f: F) -> Result<T, ApiError>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| ApiError::InternalError(format!("auth task failed: {e}")))?
        .map_err(|e| ApiError::InternalError(e.to_string()))
}

fn store(state: &ApiState) -> AuthStore {
    state.auth.store.clone()
}

fn oauth_error(status: StatusCode, code: OAuthErrorCode, message: &str) -> Response {
    (
        status,
        Json(OAuthErrorResponse {
            error: code,
            error_description: Some(message.to_string()),
            error_uri: None,
        }),
    )
        .into_response()
}

async fn record_bucket_failure(state: &ApiState, bucket: &'static str) {
    let s = store(state);
    let _ = blocking(move || s.record_failure(bucket)).await;
}

async fn reject_token(state: &ApiState, code: OAuthErrorCode, message: &str) -> Response {
    record_bucket_failure(state, BUCKET_TOKEN).await;
    oauth_error(StatusCode::BAD_REQUEST, code, message)
}

fn rejected_password(password: &str) -> Option<Response> {
    crate::auth::password::validate_password(password)
        .err()
        .map(|error| ApiError::ValidationError(error.to_string()).into_response())
}

fn valid_client_id(client_id: &str) -> bool {
    !client_id.is_empty()
        && client_id.len() <= crate::rest::dto::auth::MAX_IDENTIFIER_LENGTH
        && client_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn valid_username(username: &str) -> bool {
    let length = username.chars().count();
    length > 0 && length <= crate::rest::dto::auth::MAX_IDENTIFIER_LENGTH
}

// =============================================================================
// Bootstrap
// =============================================================================

/// Bootstrap status
#[utoipa::path(
    operation_id = "auth_bootstrap_status",
    get,
    path = "/api/v1/auth/bootstrap",
    tag = "Auth",
    responses((status = 200, description = "Bootstrap state", body = BootstrapStatusResponse))
)]
pub async fn bootstrap_status(
    State(state): State<ApiState>,
) -> Result<Json<BootstrapStatusResponse>, ApiError> {
    let s = store(&state);
    let bootstrap_state = blocking(move || s.bootstrap_state()).await?;
    Ok(Json(BootstrapStatusResponse {
        state: bootstrap_state,
        requires_temporary_password: mounted_bootstrap_password().is_some(),
    }))
}

/// Claim the admin account
#[utoipa::path(
    operation_id = "auth_bootstrap_submit",
    post,
    path = "/api/v1/auth/bootstrap",
    tag = "Auth",
    request_body = BootstrapSubmitRequest,
    responses(
        (status = 200, description = "Admin account created", body = BootstrapSubmitResponse),
        (status = 409, description = "Already bootstrapped"),
        (status = 429, description = "Too many attempts"),
    )
)]
pub async fn bootstrap_submit(
    State(state): State<ApiState>,
    Json(req): Json<BootstrapSubmitRequest>,
) -> Result<Json<BootstrapSubmitResponse>, Rejection> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_BOOTSTRAP).await {
        return Err(limited.into());
    }

    let s = store(&state);
    let current = blocking(move || s.bootstrap_state()).await?;

    let mounted = mounted_bootstrap_password();

    match current {
        BootstrapState::Complete => Err(ApiError::Conflict(
            "the admin account already exists; use login".to_string(),
        )
        .into()),

        BootstrapState::Uninitialized => {
            // When a bootstrap secret is mounted, it must be presented. That is
            // what closes the race where whoever reaches the endpoint first
            // claims the account.
            if let Some(expected) = mounted.as_deref() {
                let provided = req.temporary_password.as_deref().unwrap_or_default();
                if !crate::auth::local::matches(expected, provided) {
                    let s = store(&state);
                    let _ = blocking(move || {
                        s.record_failure(BUCKET_BOOTSTRAP)?;
                        s.audit("bootstrap", Some("bad temporary password"), false)
                    })
                    .await;
                    return Err(ApiError::Unauthorized(
                        "the temporary password is incorrect".to_string(),
                    )
                    .into());
                }
            }

            if let Some(rejected) = rejected_password(&req.new_password) {
                return Err(rejected.into());
            }

            let password = req.new_password.clone();
            let s = store(&state);
            let created = blocking(move || s.create_admin(&password, false)).await?;

            let s = store(&state);
            if !created {
                let _ = blocking(move || s.audit("bootstrap", Some("lost the race"), false)).await;
                return Err(ApiError::Conflict(
                    "the admin account was created concurrently".to_string(),
                )
                .into());
            }
            let _ = blocking(move || {
                s.clear_rate_limit(BUCKET_BOOTSTRAP)?;
                s.audit("bootstrap", None, true)
            })
            .await;

            Ok(Json(BootstrapSubmitResponse {
                state: BootstrapState::Complete,
                username: ADMIN_SUBJECT.to_string(),
            }))
        }

        BootstrapState::AwaitingPassword => {
            // A temporary password is on the account; replacing it requires
            // proving possession of it.
            let provided = req.temporary_password.clone().unwrap_or_default();
            let s = store(&state);
            let ok = blocking(move || s.verify_admin_password(&provided)).await?;
            if !ok {
                let s = store(&state);
                let _ = blocking(move || {
                    s.record_failure(BUCKET_BOOTSTRAP)?;
                    s.audit("bootstrap", Some("bad temporary password"), false)
                })
                .await;
                return Err(ApiError::Unauthorized(
                    "the temporary password is incorrect".to_string(),
                )
                .into());
            }

            if let Some(rejected) = rejected_password(&req.new_password) {
                return Err(rejected.into());
            }

            let password = req.new_password.clone();
            let s = store(&state);
            blocking(move || {
                s.set_admin_password(&password)?;
                s.clear_rate_limit(BUCKET_BOOTSTRAP)?;
                s.audit("bootstrap", Some("password set"), true)
            })
            .await?;

            Ok(Json(BootstrapSubmitResponse {
                state: BootstrapState::Complete,
                username: ADMIN_SUBJECT.to_string(),
            }))
        }
    }
}

// =============================================================================
// Login / logout / session
// =============================================================================

/// Build the `Set-Cookie` value for a session.
///
/// `__Host-` requires `Secure`, no `Domain`, and `Path=/`; the browser rejects
/// the cookie otherwise. `SameSite=Strict` keeps it off cross-site requests
/// entirely, and `HttpOnly` keeps it away from script.
fn session_cookie(token: &str) -> String {
    format!("{SESSION_COOKIE}={token}; HttpOnly; Secure; SameSite=Strict; Path=/")
}

/// Log in
#[utoipa::path(
    operation_id = "auth_login",
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in", body = LoginResponse),
        (status = 401, description = "Bad username or password"),
        (status = 429, description = "Too many attempts"),
    )
)]
pub async fn login(
    State(state): State<ApiState>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, Rejection> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_LOGIN).await {
        return Err(limited.into());
    }

    if !valid_username(&req.username)
        || req.password.chars().count() > crate::auth::password::MAX_PASSWORD_LENGTH
    {
        return Err(reject_login(&state, "invalid credentials").await.into());
    }

    let username = req.username.clone();
    let password = req.password.clone();
    let s = store(&state);
    let ok = blocking(move || s.verify_credentials(&username, &password)).await?;

    if !ok {
        return Err(reject_login(&state, "invalid credentials").await.into());
    }

    let s = store(&state);
    let (token, csrf, expires_at) = blocking(move || {
        let session = s.create_session()?;
        s.clear_rate_limit(BUCKET_LOGIN)?;
        s.audit("login", None, true)?;
        Ok(session)
    })
    .await?;

    let body = LoginResponse {
        scopes: Scope::ALL.to_vec(),
        expires_at,
        csrf_token: csrf,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, session_cookie(&token))],
        Json(body),
    )
        .into_response())
}

async fn reject_login(state: &ApiState, detail: &'static str) -> Response {
    let s = store(state);
    let _ = blocking(move || {
        s.record_failure(BUCKET_LOGIN)?;
        s.audit("login", Some(detail), false)
    })
    .await;
    ApiError::Unauthorized("incorrect username or password".to_string()).into_response()
}

/// Return recovery guidance without confirming whether the username exists.
#[utoipa::path(
    operation_id = "auth_forgot_password",
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses((status = 200, description = "Recovery guidance", body = ForgotPasswordResponse))
)]
pub async fn forgot_password(
    Json(_req): Json<ForgotPasswordRequest>,
) -> Json<ForgotPasswordResponse> {
    Json(ForgotPasswordResponse {
        message: "If this account exists, an administrator can reset it locally with `operator auth reset-admin-password`.".to_string(),
    })
}

/// Change the password using the current username and password.
#[utoipa::path(
    operation_id = "auth_reset_password",
    post,
    path = "/api/v1/auth/reset-password",
    tag = "Auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password changed", body = ResetPasswordResponse),
        (status = 401, description = "Invalid current credentials"),
        (status = 429, description = "Too many attempts"),
    )
)]
pub async fn reset_password(
    State(state): State<ApiState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, Rejection> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_PASSWORD_RESET).await {
        return Err(limited.into());
    }
    if !valid_username(&req.username)
        || req.current_password.chars().count() > crate::auth::password::MAX_PASSWORD_LENGTH
    {
        return Err(reject_password_reset(&state).await.into());
    }
    crate::auth::password::validate_password(&req.new_password)
        .map_err(|error| ApiError::ValidationError(error.to_string()))?;

    let s = store(&state);
    let changed =
        blocking(move || s.reset_password(&req.username, &req.current_password, &req.new_password))
            .await?;
    if !changed {
        return Err(reject_password_reset(&state).await.into());
    }

    let s = store(&state);
    let _ = blocking(move || {
        s.clear_rate_limit(BUCKET_PASSWORD_RESET)?;
        s.audit(
            "password reset",
            Some("via HTTP with current credentials"),
            true,
        )
    })
    .await;
    Ok(Json(ResetPasswordResponse { changed: true }))
}

async fn reject_password_reset(state: &ApiState) -> Response {
    let s = store(state);
    let _ = blocking(move || {
        s.record_failure(BUCKET_PASSWORD_RESET)?;
        s.audit("password reset", Some("invalid credentials"), false)
    })
    .await;
    ApiError::Unauthorized("incorrect username or password".to_string()).into_response()
}

/// Log out
#[utoipa::path(
    operation_id = "auth_logout",
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses((status = 200, description = "Logged out", body = LogoutResponse))
)]
pub async fn logout(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Response, ApiError> {
    if let Some(session_id) = principal.session_id.clone() {
        let s = store(&state);
        blocking(move || {
            s.revoke_session(&session_id)?;
            s.audit("logout", None, true)
        })
        .await?;
    }

    // Clear the cookie client-side too. The server-side revocation above is
    // what actually ends the session; this only tidies the browser.
    // Attributes must match the cookie being cleared, `Secure` included, or the
    // browser treats this as a different cookie and leaves the original.
    let cleared =
        format!("{SESSION_COOKIE}=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0");

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cleared)],
        Json(LogoutResponse { ended: true }),
    )
        .into_response())
}

/// Current session
#[utoipa::path(
    operation_id = "auth_current_session",
    get,
    path = "/api/v1/auth/session",
    tag = "Auth",
    responses((status = 200, description = "Current principal", body = CurrentSessionResponse))
)]
pub async fn current_session(
    Authenticated(principal): Authenticated,
) -> Json<CurrentSessionResponse> {
    Json(CurrentSessionResponse {
        subject: principal.subject.clone(),
        scopes: principal.scopes.clone(),
        principal_kind: principal.kind,
        expires_at: principal.expires_at,
    })
}

/// Issue a CSRF token for the current session
#[utoipa::path(
    operation_id = "auth_csrf_token",
    get,
    path = "/api/v1/auth/csrf",
    tag = "Auth",
    responses((status = 200, description = "CSRF token", body = CsrfTokenResponse))
)]
pub async fn csrf_token(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<CsrfTokenResponse>, ApiError> {
    // Only a cookie session needs one: a bearer token is never sent ambiently,
    // so there is nothing for a third-party site to forge.
    let Some(session_id) = principal.session_id.clone() else {
        return Err(ApiError::BadRequest(
            "CSRF tokens apply only to cookie-authenticated sessions".to_string(),
        ));
    };

    let s = store(&state);
    let rotated = blocking(move || s.rotate_csrf(&session_id)).await?;
    match rotated {
        Some(csrf_token) => Ok(Json(CsrfTokenResponse { csrf_token })),
        None => Err(ApiError::Unauthorized(
            "session is no longer valid".to_string(),
        )),
    }
}

/// List sessions and devices
#[utoipa::path(
    operation_id = "auth_list_sessions",
    get,
    path = "/api/v1/auth/sessions",
    tag = "Auth",
    responses((status = 200, description = "Sessions and devices", body = SessionListResponse))
)]
pub async fn list_sessions(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<SessionListResponse>, ApiError> {
    let current = principal.session_id.clone();
    let s = store(&state);
    let sessions = blocking(move || s.list_sessions(current.as_deref())).await?;
    let s = store(&state);
    let devices = blocking(move || s.list_devices()).await?;
    Ok(Json(SessionListResponse { sessions, devices }))
}

/// Revoke a session
#[utoipa::path(
    operation_id = "auth_revoke_session",
    delete,
    path = "/api/v1/auth/sessions/{id}",
    tag = "Auth",
    params(("id" = String, Path, description = "Session id")),
    responses((status = 200, description = "Revoked", body = LogoutResponse))
)]
pub async fn revoke_session(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<LogoutResponse>, ApiError> {
    let s = store(&state);
    blocking(move || {
        s.revoke_session(&id)?;
        s.audit("session revoked", None, true)
    })
    .await?;
    Ok(Json(LogoutResponse { ended: true }))
}

// =============================================================================
// Device authorization
// =============================================================================

/// Begin device authorization
#[utoipa::path(
    operation_id = "auth_device_code",
    post,
    path = "/api/v1/auth/device/code",
    tag = "Auth",
    request_body = DeviceAuthorizationRequest,
    responses(
        (status = 200, description = "Device code issued", body = DeviceAuthorizationResponse),
        (status = 429, description = "Too many attempts"),
    )
)]
pub async fn device_code(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(req): Json<DeviceAuthorizationRequest>,
) -> Result<Json<DeviceAuthorizationResponse>, Rejection> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_DEVICE_CODE).await {
        return Err(limited.into());
    }
    if !valid_client_id(&req.client_id) {
        record_bucket_failure(&state, BUCKET_DEVICE_CODE).await;
        return Err(oauth_error(
            StatusCode::BAD_REQUEST,
            OAuthErrorCode::InvalidClient,
            "client_id must contain 1 to 128 letters, numbers, periods, underscores, colons, or hyphens",
        ).into());
    }

    // An IDE client acts as the human admin, so it receives every scope. A
    // client asking for less is honored; asking for more than exists is not.
    let scopes = if req.scopes.is_empty() {
        Scope::ALL.to_vec()
    } else {
        req.scopes.clone()
    };

    let client_id = req.client_id.clone();
    let s = store(&state);
    let (device_code, user_code) = blocking(move || {
        let pair = s.create_device_authorization(&client_id, &scopes)?;
        s.audit("device code issued", None, true)?;
        // A successful issue is not proof of the admin: this endpoint is public.
        s.record_failure(BUCKET_DEVICE_CODE)?;
        Ok(pair)
    })
    .await?;

    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let base = crate::mcp::public_base_url(&state, host);

    Ok(Json(DeviceAuthorizationResponse {
        verification_uri: format!("{base}/#/device"),
        verification_uri_complete: format!("{base}/#/device?user_code={user_code}"),
        device_code,
        user_code,
        expires_in: DEVICE_CODE_TTL_SECS,
        interval: DEVICE_POLL_INTERVAL_SECS,
    }))
}

/// Approve a device
#[utoipa::path(
    operation_id = "auth_device_approve",
    post,
    path = "/api/v1/auth/device/approve",
    tag = "Auth",
    request_body = DeviceApprovalRequest,
    responses(
        (status = 200, description = "Device approved", body = DeviceApprovalResponse),
        (status = 404, description = "Unknown or expired user code"),
    )
)]
pub async fn device_approve(
    State(state): State<ApiState>,
    Json(req): Json<DeviceApprovalRequest>,
) -> Result<Json<DeviceApprovalResponse>, ApiError> {
    let user_code = req.user_code.trim().to_uppercase();
    let s = store(&state);
    let approved = blocking(move || s.approve_device(&user_code)).await?;

    let Some((client_id, scopes)) = approved else {
        return Err(ApiError::NotFound(
            "no pending device authorization for that code".to_string(),
        ));
    };

    let s = store(&state);
    let detail = client_id.clone();
    let _ = blocking(move || s.audit("device approved", Some(&detail), true)).await;

    Ok(Json(DeviceApprovalResponse {
        client_id,
        scopes,
        approved: true,
    }))
}

/// Exchange a credential for an access token
#[utoipa::path(
    operation_id = "auth_token",
    post,
    path = "/api/v1/auth/token",
    tag = "Auth",
    request_body = TokenRequest,
    responses(
        (status = 200, description = "Access token issued", body = TokenResponse),
        (status = 400, description = "OAuth error", body = OAuthErrorResponse),
        (status = 429, description = "Too many attempts"),
    )
)]
pub async fn token(
    State(state): State<ApiState>,
    Json(req): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, Rejection> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_TOKEN).await {
        return Err(limited.into());
    }

    let (scopes, refresh_token) = match req {
        TokenRequest::DeviceCode {
            device_code: code,
            client_id,
        } => {
            if !valid_client_id(&client_id) {
                return Err(reject_token(
                    &state,
                    OAuthErrorCode::InvalidClient,
                    "client_id is invalid",
                )
                .await
                .into());
            }
            let s = store(&state);
            let outcome = blocking(move || s.poll_device(&code, &client_id)).await?;

            match outcome {
                DevicePollOutcome::Approved { client_id, scopes } => {
                    let s = store(&state);
                    let owned = scopes.clone();
                    let refresh =
                        blocking(move || s.create_refresh_family(&client_id, &owned)).await?;
                    (scopes, Some(refresh))
                }
                DevicePollOutcome::Pending => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::AuthorizationPending,
                        "the user has not yet approved this device",
                    )
                    .into())
                }
                DevicePollOutcome::SlowDown => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::SlowDown,
                        "polling faster than the advertised interval",
                    )
                    .into())
                }
                DevicePollOutcome::Denied => {
                    return Err(reject_token(
                        &state,
                        OAuthErrorCode::AccessDenied,
                        "the user declined this device",
                    )
                    .await
                    .into())
                }
                DevicePollOutcome::Expired => {
                    return Err(reject_token(
                        &state,
                        OAuthErrorCode::ExpiredToken,
                        "the device code has expired or was already used",
                    )
                    .await
                    .into())
                }
            }
        }

        TokenRequest::RefreshToken {
            refresh_token: token,
            client_id,
        } => {
            if !valid_client_id(&client_id) {
                return Err(reject_token(
                    &state,
                    OAuthErrorCode::InvalidClient,
                    "client_id is invalid",
                )
                .await
                .into());
            }
            let s = store(&state);
            let outcome = blocking(move || s.redeem_refresh_token(&token, &client_id)).await?;

            match outcome {
                RefreshOutcome::Rotated {
                    refresh_token,
                    scopes,
                } => (scopes, Some(refresh_token)),
                RefreshOutcome::Reused => {
                    let s = store(&state);
                    let _ = blocking(move || {
                        s.record_failure(BUCKET_TOKEN)?;
                        s.audit("refresh token reuse", Some("family revoked"), false)
                    })
                    .await;
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::InvalidGrant,
                        "this refresh token was already used; the token family has been revoked",
                    )
                    .into());
                }
                RefreshOutcome::Invalid => {
                    return Err(reject_token(
                        &state,
                        OAuthErrorCode::InvalidGrant,
                        "the refresh token is invalid, expired, or revoked",
                    )
                    .await
                    .into())
                }
            }
        }

        TokenRequest::AccessKey { access_key: key } => {
            let s = store(&state);
            let scopes = blocking(move || s.redeem_access_key(&key)).await?;
            let Some(scopes) = scopes else {
                let s = store(&state);
                let _ = blocking(move || {
                    s.record_failure(BUCKET_TOKEN)?;
                    s.audit("access key exchange", Some("rejected"), false)
                })
                .await;
                return Err(oauth_error(
                    StatusCode::BAD_REQUEST,
                    OAuthErrorCode::InvalidGrant,
                    "the access key is invalid, expired, or revoked",
                )
                .into());
            };
            // An access key is re-presented on each exchange, so it produces no
            // refresh token - there is nothing to refresh.
            (scopes, None)
        }
    };

    let claims = api_claims(
        ADMIN_SUBJECT,
        &scopes,
        Utc::now(),
        uuid::Uuid::new_v4().to_string(),
    );
    let access_token = state
        .auth
        .signing_key
        .sign(&claims)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let s = store(&state);
    let _ = blocking(move || s.clear_rate_limit(BUCKET_TOKEN)).await;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_TTL.num_seconds().max(0) as u64,
        refresh_token,
        scopes,
    }))
}

/// List access keys
#[utoipa::path(
    operation_id = "auth_list_access_keys",
    get,
    path = "/api/v1/auth/keys",
    tag = "Auth",
    responses((status = 200, description = "Access keys", body = AccessKeyListResponse))
)]
pub async fn list_access_keys(
    State(state): State<ApiState>,
) -> Result<Json<AccessKeyListResponse>, ApiError> {
    let s = store(&state);
    let keys = blocking(move || s.list_access_keys()).await?;
    Ok(Json(AccessKeyListResponse { keys }))
}

/// Create an access key
#[utoipa::path(
    operation_id = "auth_create_access_key",
    post,
    path = "/api/v1/auth/keys",
    tag = "Auth",
    request_body = CreateAccessKeyRequest,
    responses((status = 200, description = "Key created; secret returned once", body = CreateAccessKeyResponse))
)]
pub async fn create_access_key(
    State(state): State<ApiState>,
    Json(req): Json<CreateAccessKeyRequest>,
) -> Result<Json<CreateAccessKeyResponse>, ApiError> {
    let name = req.name.trim();
    if name.is_empty() || name.len() > crate::rest::dto::auth::MAX_IDENTIFIER_LENGTH {
        return Err(ApiError::ValidationError(
            "access key name must contain 1 to 128 characters".to_string(),
        ));
    }
    if req.expires_in_days == 0
        || req.expires_in_days > crate::rest::dto::auth::MAX_ACCESS_KEY_EXPIRY_DAYS
    {
        return Err(ApiError::ValidationError(
            "access key expiry must be between 1 and 365 days".to_string(),
        ));
    }
    let unique_scopes: std::collections::HashSet<_> = req.scopes.iter().collect();
    if unique_scopes.is_empty() || unique_scopes.len() != req.scopes.len() {
        return Err(ApiError::ValidationError(
            "access key scopes must contain 1 to 4 unique values".to_string(),
        ));
    }
    let s = store(&state);
    let name = name.to_string();
    let scopes = req.scopes.clone();
    let days = req.expires_in_days;
    // Validation failures here (no scopes, zero expiry) are the caller's fault,
    // so they must surface as 400 rather than 500.
    let created = tokio::task::spawn_blocking(move || {
        let created = s.create_access_key(&name, &scopes, days)?;
        s.audit("access key created", Some(&name), true)?;
        anyhow::Ok(created)
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("auth task failed: {e}")))?;
    let (key, secret) = created.map_err(|e| ApiError::ValidationError(e.to_string()))?;

    Ok(Json(CreateAccessKeyResponse { key, secret }))
}

/// Revoke an access key
#[utoipa::path(
    operation_id = "auth_revoke_access_key",
    delete,
    path = "/api/v1/auth/keys/{id}",
    tag = "Auth",
    params(("id" = String, Path, description = "Access key id")),
    responses(
        (status = 200, description = "Revoked", body = RevokeAccessKeyResponse),
        (status = 404, description = "Unknown or already revoked"),
    )
)]
pub async fn revoke_access_key(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<RevokeAccessKeyResponse>, ApiError> {
    let s = store(&state);
    let owned = id.clone();
    let revoked = blocking(move || {
        let at = s.revoke_access_key(&owned)?;
        if at.is_some() {
            s.audit("access key revoked", Some(&owned), true)?;
        }
        Ok(at)
    })
    .await?;

    match revoked {
        Some(revoked_at) => Ok(Json(RevokeAccessKeyResponse { id, revoked_at })),
        None => Err(ApiError::NotFound(
            "no active access key with that id".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::{Path, State};
    use axum::http::{header, HeaderMap, Request, StatusCode};
    use axum::response::{IntoResponse, Response};
    use axum::Json;
    use http_body_util::BodyExt;
    use serde::de::DeserializeOwned;
    use tempfile::TempDir;
    use tower::ServiceExt;

    use super::{
        bootstrap_status, bootstrap_submit, create_access_key, csrf_token, current_session,
        device_approve, device_code, forgot_password, list_access_keys, list_sessions, login,
        logout, reset_password, revoke_access_key, revoke_session, token,
        BOOTSTRAP_PASSWORD_FILE_ENV, BUCKET_DEVICE_CODE, BUCKET_LOGIN, BUCKET_PASSWORD_RESET,
        BUCKET_TOKEN,
    };
    use crate::auth::scope::Principal;
    use crate::auth::store::{
        ADMIN_SUBJECT, AUTH_DB_FILENAME, DEVICE_CODE_TTL_SECS, DEVICE_POLL_INTERVAL_SECS,
    };
    use crate::auth::tokens::ACCESS_TOKEN_TTL;
    use crate::config::Config;
    use crate::rest::dto::auth::*;
    use crate::rest::error::{ApiError, Rejection};
    use crate::rest::middleware::auth::{Authenticated, CSRF_HEADER, SESSION_COOKIE};
    use crate::rest::state::ApiState;

    const GOOD: &str = "correct horse battery staple";
    const REPLACEMENT: &str = "replacement horse battery staple";
    const TEMPORARY: &str = "temporary horse battery";
    const SHORT: &str = "elevenchars";

    async fn bootstrap_lock() -> tokio::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await
    }

    struct RestoreEnv {
        previous: Option<String>,
    }

    impl RestoreEnv {
        fn set(value: &str) -> Self {
            let previous = std::env::var(BOOTSTRAP_PASSWORD_FILE_ENV).ok();
            std::env::set_var(BOOTSTRAP_PASSWORD_FILE_ENV, value);
            Self { previous }
        }
    }

    impl Drop for RestoreEnv {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var(BOOTSTRAP_PASSWORD_FILE_ENV, value),
                None => std::env::remove_var(BOOTSTRAP_PASSWORD_FILE_ENV),
            }
        }
    }

    fn state_at(path: &std::path::Path) -> ApiState {
        let mut config = Config::default();
        config.paths.state = path.to_string_lossy().into_owned();
        ApiState::new(config, path.join("tickets"))
    }

    fn fresh() -> (TempDir, ApiState) {
        let dir = TempDir::new().unwrap();
        let state = state_at(dir.path());
        (dir, state)
    }

    fn settle<T: IntoResponse>(result: Result<T, Rejection>) -> Response {
        match result {
            Ok(body) => body.into_response(),
            Err(rejection) => rejection.into_response(),
        }
    }

    fn settle_api<T: IntoResponse>(result: Result<T, ApiError>) -> Response {
        match result {
            Ok(body) => body.into_response(),
            Err(error) => error.into_response(),
        }
    }

    async fn body_json(response: Response) -> (StatusCode, HeaderMap, serde_json::Value) {
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!(
                "response was not json ({error}): {}",
                String::from_utf8_lossy(&bytes)
            )
        });
        (status, headers, value)
    }

    fn parse<T: DeserializeOwned>(value: serde_json::Value) -> T {
        serde_json::from_value(value.clone()).unwrap_or_else(|error| panic!("{error}: {value}"))
    }

    fn retry_after(headers: &HeaderMap) -> u64 {
        headers
            .get(header::RETRY_AFTER)
            .unwrap_or_else(|| panic!("missing Retry-After: {headers:?}"))
            .to_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    fn session_cookie(headers: &HeaderMap) -> String {
        let cookie = headers
            .get(header::SET_COOKIE)
            .unwrap_or_else(|| panic!("missing Set-Cookie: {headers:?}"))
            .to_str()
            .unwrap();
        let token = cookie
            .split(';')
            .next()
            .unwrap()
            .strip_prefix(&format!("{SESSION_COOKIE}="))
            .unwrap_or_else(|| panic!("unexpected cookie: {cookie}"));
        assert!(!token.is_empty());
        assert_eq!(
            cookie,
            format!("{SESSION_COOKIE}={token}; HttpOnly; Secure; SameSite=Strict; Path=/")
        );
        token.to_string()
    }

    fn cleared_cookie() -> String {
        format!("{SESSION_COOKIE}=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0")
    }

    fn rewind_device_poll(dir: &std::path::Path) {
        let conn = rusqlite::Connection::open(dir.join(AUTH_DB_FILENAME)).unwrap();
        let earlier = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let updated = conn
            .execute(
                "UPDATE device_authorization SET last_polled_at = ?1",
                [earlier],
            )
            .unwrap();
        assert_eq!(updated, 1);
    }

    fn widen_persisted_backoff(dir: &std::path::Path, bucket: &str) {
        let conn = rusqlite::Connection::open(dir.join(AUTH_DB_FILENAME)).unwrap();
        let until = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        let updated = conn
            .execute(
                "UPDATE rate_limit SET retry_after = ?1 WHERE bucket = ?2 AND attempts >= 4",
                rusqlite::params![until, bucket],
            )
            .unwrap();
        assert_eq!(updated, 1, "bucket {bucket} was not persisted");
    }

    async fn submit(state: &ApiState, temporary: Option<&str>, new_password: &str) -> Response {
        settle(
            bootstrap_submit(
                State(state.clone()),
                Json(BootstrapSubmitRequest {
                    temporary_password: temporary.map(str::to_string),
                    new_password: new_password.to_string(),
                }),
            )
            .await,
        )
    }

    async fn bootstrap_admin(state: &ApiState) {
        let (status, _, value) = body_json(submit(state, None, GOOD).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
    }

    async fn sign_in(state: &ApiState, username: &str, password: &str) -> Response {
        settle(
            login(
                State(state.clone()),
                Json(LoginRequest {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
            )
            .await,
        )
    }

    async fn device(
        state: &ApiState,
        client_id: &str,
        host: Option<&str>,
        scopes: Vec<Scope>,
    ) -> Response {
        let mut headers = HeaderMap::new();
        if let Some(host) = host {
            headers.insert(header::HOST, host.parse().unwrap());
        }
        settle(
            device_code(
                State(state.clone()),
                headers,
                Json(DeviceAuthorizationRequest {
                    client_id: client_id.to_string(),
                    scopes,
                }),
            )
            .await,
        )
    }

    async fn exchange(state: &ApiState, request: TokenRequest) -> Response {
        settle(token(State(state.clone()), Json(request)).await)
    }

    fn is_user_code(code: &str) -> bool {
        let alphabet = b"ABCDEFGHJKMNPQRSTVWXYZ23456789";
        let Some((left, right)) = code.split_once('-') else {
            return false;
        };
        [left, right]
            .into_iter()
            .all(|part| part.len() == 4 && part.bytes().all(|byte| alphabet.contains(&byte)))
    }

    fn session_principal(state: &ApiState, cookie: &str) -> Principal {
        state
            .auth
            .store
            .authenticate_session(cookie)
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn test_bootstrap_short_password_stays_uninitialized_until_a_valid_one() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        let status = bootstrap_status(State(state.clone())).await.unwrap().0;
        assert_eq!(status.state, BootstrapState::Uninitialized);
        assert!(!status.requires_temporary_password);

        for _ in 0..4 {
            let (status, _, value) = body_json(submit(&state, None, SHORT).await).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            assert_eq!(value["error"], "validation_error");
        }

        let (status, _, value) = body_json(submit(&state, None, GOOD).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let body: BootstrapSubmitResponse = parse(value);
        assert_eq!(body.state, BootstrapState::Complete);
        assert_eq!(body.username, ADMIN_SUBJECT);

        let (status, _, value) = body_json(submit(&state, None, GOOD).await).await;
        assert_eq!(status, StatusCode::CONFLICT, "{value}");
        assert_eq!(value["error"], "conflict");
        assert_eq!(
            bootstrap_status(State(state)).await.unwrap().0.state,
            BootstrapState::Complete
        );
    }

    #[tokio::test]
    async fn test_mounted_bootstrap_secret_backs_off_before_an_account_exists() {
        let _lock = bootstrap_lock().await;
        let (dir, state) = fresh();
        let secret_path = dir.path().join("bootstrap-secret");
        std::fs::write(&secret_path, "mounted-temporary-secret\n").unwrap();
        let _env = RestoreEnv::set(&secret_path.to_string_lossy());

        let status = bootstrap_status(State(state.clone())).await.unwrap().0;
        assert!(status.requires_temporary_password);
        assert_eq!(status.state, BootstrapState::Uninitialized);

        for _ in 0..4 {
            let (status, _, value) =
                body_json(submit(&state, Some("wrong-temporary"), GOOD).await).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
            assert_eq!(value["error"], "unauthorized");
        }
        let (status, headers, value) =
            body_json(submit(&state, Some("mounted-temporary-secret"), GOOD).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{value}");
        assert!(retry_after(&headers) >= 1);
        assert_eq!(value["error"], "rate_limited");
        assert_eq!(
            bootstrap_status(State(state)).await.unwrap().0.state,
            BootstrapState::Uninitialized
        );
    }

    #[tokio::test]
    async fn test_mounted_bootstrap_accepts_the_secret_and_rejects_a_short_password() {
        let _lock = bootstrap_lock().await;
        let (dir, state) = fresh();
        let secret_path = dir.path().join("bootstrap-secret");
        std::fs::write(&secret_path, "mounted-temporary-secret").unwrap();
        let _env = RestoreEnv::set(secret_path.to_str().unwrap());

        let (status, _, value) =
            body_json(submit(&state, Some("mounted-temporary-secret"), SHORT).await).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            bootstrap_status(State(state.clone()))
                .await
                .unwrap()
                .0
                .state,
            BootstrapState::Uninitialized
        );

        let (status, _, value) = body_json(submit(&state, Some("wrong"), GOOD).await).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");

        let (status, _, value) =
            body_json(submit(&state, Some("mounted-temporary-secret"), GOOD).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(
            bootstrap_status(State(state)).await.unwrap().0.state,
            BootstrapState::Complete
        );
    }

    #[tokio::test]
    async fn test_awaiting_password_requires_the_temporary_password() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        state.auth.store.create_admin(TEMPORARY, true).unwrap();
        assert_eq!(
            bootstrap_status(State(state.clone()))
                .await
                .unwrap()
                .0
                .state,
            BootstrapState::AwaitingPassword
        );

        let (status, _, value) =
            body_json(submit(&state, Some("wrong-temporary"), GOOD).await).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");

        let (status, _, value) = body_json(submit(&state, Some(TEMPORARY), SHORT).await).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            bootstrap_status(State(state.clone()))
                .await
                .unwrap()
                .0
                .state,
            BootstrapState::AwaitingPassword
        );

        let (status, _, value) =
            body_json(submit(&state, Some(TEMPORARY), REPLACEMENT).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let body: BootstrapSubmitResponse = parse(value);
        assert_eq!(body.state, BootstrapState::Complete);

        let (status, _, _) = body_json(sign_in(&state, ADMIN_SUBJECT, TEMPORARY).await).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let (status, _, _) = body_json(sign_in(&state, ADMIN_SUBJECT, REPLACEMENT).await).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_failure_message_does_not_reveal_which_part_was_wrong() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        bootstrap_admin(&state).await;

        let (wrong_status, _, wrong) =
            body_json(sign_in(&state, ADMIN_SUBJECT, REPLACEMENT).await).await;
        let (unknown_status, _, unknown) = body_json(sign_in(&state, "someone", GOOD).await).await;
        assert_eq!(wrong_status, StatusCode::UNAUTHORIZED);
        assert_eq!(unknown_status, StatusCode::UNAUTHORIZED);
        assert_eq!(wrong, unknown);
        assert_eq!(wrong["message"], "incorrect username or password");
    }

    #[tokio::test]
    async fn test_login_backoff_is_persisted_for_a_reopened_store() {
        let (dir, state) = fresh();
        for _ in 0..4 {
            let (status, _, value) = body_json(sign_in(&state, "", "x").await).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
        }
        let (status, headers, value) = body_json(sign_in(&state, "", "x").await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{value}");
        assert!(retry_after(&headers) >= 1);
        assert_eq!(value["error"], "rate_limited");

        drop(state);
        widen_persisted_backoff(dir.path(), BUCKET_LOGIN);
        let state = state_at(dir.path());
        let (status, headers, _) = body_json(sign_in(&state, ADMIN_SUBJECT, GOOD).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(retry_after(&headers) >= 1);
    }

    #[tokio::test]
    async fn test_forgot_password_response_ignores_the_username() {
        let first = forgot_password(Json(ForgotPasswordRequest {
            username: ADMIN_SUBJECT.to_string(),
        }))
        .await
        .0
        .message;
        for username in ["missing-user", ""] {
            let message = forgot_password(Json(ForgotPasswordRequest {
                username: username.to_string(),
            }))
            .await
            .0
            .message;
            assert_eq!(message, first);
        }
        assert!(first.contains("operator auth reset-admin-password"));
    }

    #[tokio::test]
    async fn test_reset_password_replaces_the_credential_and_ignores_short_passwords() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        bootstrap_admin(&state).await;

        let (status, _, value) = body_json(settle(
            reset_password(
                State(state.clone()),
                Json(ResetPasswordRequest {
                    username: ADMIN_SUBJECT.to_string(),
                    current_password: "not the current password".into(),
                    new_password: REPLACEMENT.into(),
                }),
            )
            .await,
        ))
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
        assert_eq!(value["message"], "incorrect username or password");

        for _ in 0..5 {
            let (status, _, value) = body_json(settle(
                reset_password(
                    State(state.clone()),
                    Json(ResetPasswordRequest {
                        username: ADMIN_SUBJECT.to_string(),
                        current_password: GOOD.into(),
                        new_password: SHORT.into(),
                    }),
                )
                .await,
            ))
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            assert_eq!(value["error"], "validation_error");
        }

        let (status, _, value) = body_json(settle(
            reset_password(
                State(state.clone()),
                Json(ResetPasswordRequest {
                    username: ADMIN_SUBJECT.to_string(),
                    current_password: GOOD.into(),
                    new_password: REPLACEMENT.into(),
                }),
            )
            .await,
        ))
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let body: ResetPasswordResponse = parse(value);
        assert!(body.changed);

        let (status, _, _) = body_json(sign_in(&state, ADMIN_SUBJECT, GOOD).await).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let (status, _, _) = body_json(sign_in(&state, ADMIN_SUBJECT, REPLACEMENT).await).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_reset_password_backoff_is_persisted() {
        let (dir, state) = fresh();
        for _ in 0..4 {
            let (status, _, _) = body_json(settle(
                reset_password(
                    State(state.clone()),
                    Json(ResetPasswordRequest {
                        username: String::new(),
                        current_password: "x".into(),
                        new_password: SHORT.into(),
                    }),
                )
                .await,
            ))
            .await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }
        let (status, headers, value) = body_json(settle(
            reset_password(
                State(state.clone()),
                Json(ResetPasswordRequest {
                    username: String::new(),
                    current_password: "x".into(),
                    new_password: SHORT.into(),
                }),
            )
            .await,
        ))
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{value}");
        assert!(retry_after(&headers) >= 1);

        drop(state);
        widen_persisted_backoff(dir.path(), BUCKET_PASSWORD_RESET);
        let state = state_at(dir.path());
        let (status, _, _) = body_json(settle(
            reset_password(
                State(state),
                Json(ResetPasswordRequest {
                    username: ADMIN_SUBJECT.into(),
                    current_password: GOOD.into(),
                    new_password: REPLACEMENT.into(),
                }),
            )
            .await,
        ))
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_login_cookie_csrf_logout_and_session_revocation() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        bootstrap_admin(&state).await;
        let (status, headers, value) = body_json(sign_in(&state, ADMIN_SUBJECT, GOOD).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let cookie = session_cookie(&headers);
        let login_body: LoginResponse = parse(value);
        assert_ne!(cookie, login_body.csrf_token);
        assert_eq!(login_body.scopes, Scope::ALL.to_vec());

        let current = current_session(Authenticated(session_principal(&state, &cookie)))
            .await
            .0;
        assert_eq!(current.subject, ADMIN_SUBJECT);
        assert_eq!(current.principal_kind, PrincipalKind::Session);
        assert_eq!(current.scopes, Scope::ALL.to_vec());

        let principal = session_principal(&state, &cookie);
        let session_id = principal.session_id.clone().unwrap();
        let rotated = csrf_token(State(state.clone()), Authenticated(principal.clone()))
            .await
            .unwrap()
            .0
            .csrf_token;
        assert_ne!(rotated, login_body.csrf_token);
        assert!(state
            .auth
            .store
            .verify_csrf(&session_id, &login_body.csrf_token)
            .unwrap());
        assert!(state.auth.store.verify_csrf(&session_id, &rotated).unwrap());

        let rotated_again = csrf_token(State(state.clone()), Authenticated(principal.clone()))
            .await
            .unwrap()
            .0
            .csrf_token;
        assert!(!state
            .auth
            .store
            .verify_csrf(&session_id, &login_body.csrf_token)
            .unwrap());
        assert!(state.auth.store.verify_csrf(&session_id, &rotated).unwrap());
        assert!(state
            .auth
            .store
            .verify_csrf(&session_id, &rotated_again)
            .unwrap());

        let bearer = Principal {
            kind: PrincipalKind::AccessToken,
            session_id: None,
            ..Principal::local(ADMIN_SUBJECT)
        };
        assert_eq!(
            csrf_token(State(state.clone()), Authenticated(bearer))
                .await
                .unwrap_err()
                .parts()
                .0,
            StatusCode::BAD_REQUEST
        );

        let listed = list_sessions(State(state.clone()), Authenticated(principal.clone()))
            .await
            .unwrap()
            .0;
        assert!(listed.sessions.iter().any(|session| session.current));
        assert!(listed.devices.is_empty());

        let (extra, _, _) = state.auth.store.create_session().unwrap();
        let extra_id = session_principal(&state, &extra).session_id.unwrap();
        assert!(
            revoke_session(State(state.clone()), Path(extra_id))
                .await
                .unwrap()
                .0
                .ended
        );
        assert!(state
            .auth
            .store
            .authenticate_session(&extra)
            .unwrap()
            .is_none());

        let response = logout(State(state.clone()), Authenticated(principal))
            .await
            .unwrap();
        let (status, headers, value) = body_json(response).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(
            headers.get(header::SET_COOKIE).unwrap().to_str().unwrap(),
            cleared_cookie()
        );
        let body: LogoutResponse = parse(value);
        assert!(body.ended);
        assert!(state
            .auth
            .store
            .authenticate_session(&cookie)
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_device_code_uses_host_until_a_public_url_is_configured() {
        let (_dir, state) = fresh();
        let (status, _, value) = body_json(
            device(
                &state,
                "ide.editor_1:main-box",
                Some("operator.example:7008"),
                vec![],
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let body: DeviceAuthorizationResponse = parse(value);
        assert!(is_user_code(&body.user_code));
        assert_ne!(body.device_code, body.user_code);
        assert_eq!(body.expires_in, DEVICE_CODE_TTL_SECS);
        assert_eq!(body.interval, DEVICE_POLL_INTERVAL_SECS);
        assert_eq!(
            body.verification_uri,
            "http://operator.example:7008/#/device"
        );
        assert_eq!(
            body.verification_uri_complete,
            format!(
                "http://operator.example:7008/#/device?user_code={}",
                body.user_code
            )
        );

        let dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.paths.state = dir.path().to_string_lossy().into_owned();
        config.rest_api.public_url = Some("https://operator.example.com/".into());
        let state = ApiState::new(config, dir.path().join("tickets"));
        let (status, _, value) =
            body_json(device(&state, "vscode", Some("evil.example"), vec![]).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let body: DeviceAuthorizationResponse = parse(value);
        assert_eq!(
            body.verification_uri,
            "https://operator.example.com/#/device"
        );
        assert!(body
            .verification_uri_complete
            .starts_with("https://operator.example.com/#/device?user_code="));
    }

    #[tokio::test]
    async fn test_device_code_rejects_a_malformed_client_id() {
        let (_dir, state) = fresh();
        for client_id in ["", "bad/id", &"a".repeat(129)] {
            let (status, _, value) = body_json(device(&state, client_id, None, vec![]).await).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            let body: OAuthErrorResponse = parse(value);
            assert_eq!(body.error, OAuthErrorCode::InvalidClient);
        }
    }

    #[tokio::test]
    async fn test_device_approval_reports_granted_scopes_and_unknown_codes() {
        let (_dir, state) = fresh();
        let (status, _, value) = body_json(settle_api(
            device_approve(
                State(state.clone()),
                Json(DeviceApprovalRequest {
                    user_code: "0000-0000".into(),
                }),
            )
            .await,
        ))
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{value}");
        assert_eq!(value["error"], "not_found");

        let issued: DeviceAuthorizationResponse = parse(
            body_json(device(&state, "vscode", None, vec![Scope::Read]).await)
                .await
                .2,
        );
        let approved: DeviceApprovalResponse = parse(
            body_json(settle_api(
                device_approve(
                    State(state.clone()),
                    Json(DeviceApprovalRequest {
                        user_code: issued.user_code.to_lowercase(),
                    }),
                )
                .await,
            ))
            .await
            .2,
        );
        assert!(approved.approved);
        assert_eq!(approved.client_id, "vscode");
        assert_eq!(approved.scopes, vec![Scope::Read]);

        let all: DeviceAuthorizationResponse = parse(
            body_json(device(&state, "codex", None, vec![]).await)
                .await
                .2,
        );
        let approved: DeviceApprovalResponse = parse(
            body_json(settle_api(
                device_approve(
                    State(state),
                    Json(DeviceApprovalRequest {
                        user_code: all.user_code,
                    }),
                )
                .await,
            ))
            .await
            .2,
        );
        assert_eq!(approved.scopes, Scope::ALL.to_vec());
    }

    #[tokio::test]
    async fn test_device_poll_exchange_rotates_refresh_tokens_and_lists_the_device() {
        let (dir, state) = fresh();
        let issued: DeviceAuthorizationResponse = parse(
            body_json(device(&state, "vscode", Some("localhost:7008"), vec![]).await)
                .await
                .2,
        );

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code.clone(),
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::AuthorizationPending
        );

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code.clone(),
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::SlowDown
        );

        let approved = device_approve(
            State(state.clone()),
            Json(DeviceApprovalRequest {
                user_code: issued.user_code.clone(),
            }),
        )
        .await
        .unwrap()
        .0;
        assert!(approved.approved);
        rewind_device_poll(dir.path());

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code.clone(),
                    client_id: "other-client".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::ExpiredToken
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code.clone(),
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let issued_token: TokenResponse = parse(value);
        assert_eq!(issued_token.token_type, "Bearer");
        assert_eq!(
            issued_token.expires_in,
            ACCESS_TOKEN_TTL.num_seconds().max(0) as u64
        );
        assert_eq!(issued_token.scopes, Scope::ALL.to_vec());
        let refresh = issued_token.refresh_token.clone().unwrap();
        assert_ne!(refresh, issued.device_code);

        let listed = list_sessions(
            State(state.clone()),
            Authenticated(Principal::local(ADMIN_SUBJECT)),
        )
        .await
        .unwrap()
        .0;
        assert!(listed
            .devices
            .iter()
            .any(|device| device.client_id == "vscode" && device.revoked_at.is_none()));

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::RefreshToken {
                    refresh_token: refresh.clone(),
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let rotated: TokenResponse = parse(value);
        let next = rotated.refresh_token.unwrap();
        assert_ne!(next, refresh);

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::RefreshToken {
                    refresh_token: refresh,
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::InvalidGrant
        );

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::RefreshToken {
                    refresh_token: next,
                    client_id: "bad/id".to_string(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::InvalidClient
        );

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code,
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::ExpiredToken
        );
    }

    #[tokio::test]
    async fn test_password_reset_denies_a_pending_device() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        bootstrap_admin(&state).await;
        let issued: DeviceAuthorizationResponse = parse(
            body_json(device(&state, "vscode", None, vec![]).await)
                .await
                .2,
        );
        let (status, _, _) = body_json(settle(
            reset_password(
                State(state.clone()),
                Json(ResetPasswordRequest {
                    username: ADMIN_SUBJECT.into(),
                    current_password: GOOD.into(),
                    new_password: REPLACEMENT.into(),
                }),
            )
            .await,
        ))
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code,
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::AccessDenied
        );
    }

    #[tokio::test]
    async fn test_device_code_backoff_persists_and_blocks_a_later_valid_client() {
        let (dir, state) = fresh();
        for _ in 0..4 {
            let (status, _, value) = body_json(device(&state, "bad/id", None, vec![]).await).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            assert_eq!(
                parse::<OAuthErrorResponse>(value).error,
                OAuthErrorCode::InvalidClient
            );
        }
        let (status, headers, value) =
            body_json(device(&state, "vscode", None, vec![]).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{value}");
        assert!(retry_after(&headers) >= 1);

        drop(state);
        widen_persisted_backoff(dir.path(), BUCKET_DEVICE_CODE);
        let state = state_at(dir.path());
        let (status, _, _) = body_json(device(&state, "vscode", None, vec![]).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_pending_polls_do_not_fill_the_token_bucket() {
        let (_dir, state) = fresh();
        for index in 0..4 {
            let (status, _, value) =
                body_json(device(&state, &format!("client-{index}"), None, vec![]).await).await;
            assert_eq!(status, StatusCode::OK, "{value}");
            let issued: DeviceAuthorizationResponse = parse(value);
            let (status, _, value) = body_json(
                exchange(
                    &state,
                    TokenRequest::DeviceCode {
                        device_code: issued.device_code.clone(),
                        client_id: format!("client-{index}"),
                    },
                )
                .await,
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            assert_eq!(
                parse::<OAuthErrorResponse>(value).error,
                OAuthErrorCode::AuthorizationPending
            );
        }
        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::AccessKey {
                    access_key: "not-a-key".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::InvalidGrant
        );
    }

    #[tokio::test]
    async fn test_token_invalid_client_backoff_is_persisted() {
        let (dir, state) = fresh();
        let request = || TokenRequest::DeviceCode {
            device_code: "device-code".into(),
            client_id: "bad/id".into(),
        };
        for _ in 0..4 {
            let (status, _, value) = body_json(exchange(&state, request()).await).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            assert_eq!(
                parse::<OAuthErrorResponse>(value).error,
                OAuthErrorCode::InvalidClient
            );
        }
        let (status, headers, value) = body_json(exchange(&state, request()).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{value}");
        assert!(retry_after(&headers) >= 1);

        drop(state);
        widen_persisted_backoff(dir.path(), BUCKET_TOKEN);
        let state = state_at(dir.path());
        let (status, _, _) = body_json(exchange(&state, request()).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_access_key_is_returned_once_and_exchanges_without_a_refresh_token() {
        let (_dir, state) = fresh();
        let created = create_access_key(
            State(state.clone()),
            Json(CreateAccessKeyRequest {
                name: "  ci  ".into(),
                scopes: vec![Scope::Read],
                expires_in_days: 30,
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(created.key.name, "ci");
        assert_eq!(created.secret.len(), 47);
        assert!(created.secret.starts_with("opk_"));

        let listed = list_access_keys(State(state.clone())).await.unwrap().0;
        let json = serde_json::to_string(&listed).unwrap();
        assert!(!json.contains(&created.secret));
        assert_eq!(listed.keys.len(), 1);

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::AccessKey {
                    access_key: "not-a-key".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(
            parse::<OAuthErrorResponse>(value).error,
            OAuthErrorCode::InvalidGrant
        );

        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::AccessKey {
                    access_key: created.secret,
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let token_body: TokenResponse = parse(value);
        assert_eq!(token_body.token_type, "Bearer");
        assert!(token_body.refresh_token.is_none());
        assert_eq!(token_body.scopes, vec![Scope::Read]);

        let revoked = revoke_access_key(State(state.clone()), Path(created.key.id.clone()))
            .await
            .unwrap()
            .0;
        assert_eq!(revoked.id, created.key.id);
        let error = revoke_access_key(State(state), Path(created.key.id))
            .await
            .unwrap_err();
        assert_eq!(error.parts().0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_access_key_creation_rejects_bad_names_scopes_and_expiry() {
        let (_dir, state) = fresh();
        let cases = [
            (String::new(), vec![Scope::Read], 30),
            ("   ".into(), vec![Scope::Read], 30),
            ("n".repeat(129), vec![Scope::Read], 30),
            ("ci".into(), vec![], 30),
            ("ci".into(), vec![Scope::Read, Scope::Read], 30),
            ("ci".into(), vec![Scope::Read], 0),
            ("ci".into(), vec![Scope::Read], 366),
        ];
        for (name, scopes, expires_in_days) in cases {
            let error = create_access_key(
                State(state.clone()),
                Json(CreateAccessKeyRequest {
                    name,
                    scopes,
                    expires_in_days,
                }),
            )
            .await
            .unwrap_err();
            assert_eq!(error.parts().0, StatusCode::BAD_REQUEST);
        }
        assert_eq!(
            revoke_access_key(State(state), Path("missing".into()))
                .await
                .unwrap_err()
                .parts()
                .0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_access_key_exchange_backoff_is_persisted() {
        let (dir, state) = fresh();
        let bad = || TokenRequest::AccessKey {
            access_key: "not-a-key".into(),
        };
        for _ in 0..4 {
            let (status, _, value) = body_json(exchange(&state, bad()).await).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
            assert_eq!(
                parse::<OAuthErrorResponse>(value).error,
                OAuthErrorCode::InvalidGrant
            );
        }
        let (status, headers, _) = body_json(exchange(&state, bad()).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(retry_after(&headers) >= 1);

        drop(state);
        widen_persisted_backoff(dir.path(), BUCKET_TOKEN);
        let state = state_at(dir.path());
        let (status, _, _) = body_json(exchange(&state, bad()).await).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    fn router_request(
        method: &str,
        uri: &str,
        cookie: Option<&str>,
        csrf: Option<&str>,
        origin: &str,
    ) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::HOST, "127.0.0.1:7008")
            .header(header::ORIGIN, origin);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, format!("{SESSION_COOKIE}={cookie}"));
        }
        if let Some(csrf) = csrf {
            builder = builder.header(CSRF_HEADER, csrf);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn test_cookie_mutations_require_csrf_and_same_origin() {
        let _lock = bootstrap_lock().await;
        let (_dir, state) = fresh();
        bootstrap_admin(&state).await;
        let (status, headers, value) = body_json(sign_in(&state, ADMIN_SUBJECT, GOOD).await).await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let cookie = session_cookie(&headers);
        let csrf = parse::<LoginResponse>(value).csrf_token;
        let app = crate::rest::build_profile_router(state.clone());
        let same_origin = "http://127.0.0.1:7008";

        let (status, _, value) = body_json(
            app.clone()
                .oneshot(router_request(
                    "GET",
                    "/api/v1/auth/session",
                    Some(&cookie),
                    None,
                    same_origin,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(
            parse::<CurrentSessionResponse>(value).subject,
            ADMIN_SUBJECT
        );

        let (status, _, value) = body_json(
            app.clone()
                .oneshot(router_request(
                    "GET",
                    "/api/v1/auth/keys",
                    Some(&cookie),
                    None,
                    same_origin,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");

        let (status, _, value) = body_json(
            app.clone()
                .oneshot(router_request(
                    "GET",
                    "/api/v1/auth/keys",
                    None,
                    None,
                    same_origin,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");

        let (extra, _, _) = state.auth.store.create_session().unwrap();
        let extra_id = session_principal(&state, &extra).session_id.unwrap();
        let (status, _, value) = body_json(
            app.clone()
                .oneshot(router_request(
                    "DELETE",
                    &format!("/api/v1/auth/sessions/{extra_id}"),
                    None,
                    None,
                    same_origin,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");

        let (_, other_csrf, _) = state.auth.store.create_session().unwrap();
        for (csrf_header, origin) in [
            (None, same_origin),
            (Some(csrf.as_str()), "https://evil.example.com"),
            (Some(other_csrf.as_str()), same_origin),
        ] {
            let (status, _, value) = body_json(
                app.clone()
                    .oneshot(router_request(
                        "POST",
                        "/api/v1/auth/logout",
                        Some(&cookie),
                        csrf_header,
                        origin,
                    ))
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{value}");
            assert_eq!(value["error"], "csrf_failed");
        }

        let (status, _, _) = body_json(
            app.clone()
                .oneshot(router_request(
                    "DELETE",
                    &format!("/api/v1/auth/sessions/{extra_id}"),
                    Some(&cookie),
                    Some(&csrf),
                    same_origin,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(state
            .auth
            .store
            .authenticate_session(&extra)
            .unwrap()
            .is_none());

        // One superseded token stays valid, so retiring the login token takes two.
        const ROTATIONS_TO_RETIRE_A_TOKEN: usize = 2;
        let mut rotated = csrf.clone();
        for _ in 0..ROTATIONS_TO_RETIRE_A_TOKEN {
            let (status, _, value) = body_json(
                app.clone()
                    .oneshot(router_request(
                        "GET",
                        "/api/v1/auth/csrf",
                        Some(&cookie),
                        None,
                        same_origin,
                    ))
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{value}");
            rotated = parse::<CsrfTokenResponse>(value).csrf_token;
            assert_ne!(rotated, csrf);
        }

        let (status, _, value) = body_json(
            app.clone()
                .oneshot(router_request(
                    "POST",
                    "/api/v1/auth/logout",
                    Some(&cookie),
                    Some(&csrf),
                    same_origin,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{value}");

        let (status, headers, value) = body_json(
            app.oneshot(router_request(
                "POST",
                "/api/v1/auth/logout",
                Some(&cookie),
                Some(&rotated),
                same_origin,
            ))
            .await
            .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(
            headers.get(header::SET_COOKIE).unwrap().to_str().unwrap(),
            cleared_cookie()
        );
    }

    #[tokio::test]
    async fn test_bearer_logout_does_not_require_a_csrf_token() {
        let (_dir, state) = fresh();
        let issued: DeviceAuthorizationResponse = parse(
            body_json(device(&state, "vscode", None, vec![]).await)
                .await
                .2,
        );
        let approved = device_approve(
            State(state.clone()),
            Json(DeviceApprovalRequest {
                user_code: issued.user_code,
            }),
        )
        .await
        .unwrap()
        .0;
        assert!(approved.approved);
        let (status, _, value) = body_json(
            exchange(
                &state,
                TokenRequest::DeviceCode {
                    device_code: issued.device_code,
                    client_id: "vscode".into(),
                },
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let access_token = parse::<TokenResponse>(value).access_token;

        let app = crate::rest::build_profile_router(state);
        let (status, _, value) = body_json(
            app.oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/logout")
                    .header(header::HOST, "127.0.0.1:7008")
                    .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert!(parse::<LogoutResponse>(value).ended);
    }
}
