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
use crate::rest::error::ApiError;
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
/// `docker inspect`, and to every child process Operator spawns — including the
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
) -> Result<Json<BootstrapSubmitResponse>, Response> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_BOOTSTRAP).await {
        return Err(limited);
    }

    let s = store(&state);
    let current = blocking(move || s.bootstrap_state())
        .await
        .map_err(IntoResponse::into_response)?;

    let mounted = mounted_bootstrap_password();

    match current {
        BootstrapState::Complete => Err(ApiError::Conflict(
            "the admin account already exists; use login".to_string(),
        )
        .into_response()),

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
                    .into_response());
                }
            }

            let password = req.new_password.clone();
            let s = store(&state);
            let created = blocking(move || s.create_admin(&password, false))
                .await
                .map_err(IntoResponse::into_response)?;

            let s = store(&state);
            if !created {
                let _ = blocking(move || s.audit("bootstrap", Some("lost the race"), false)).await;
                return Err(ApiError::Conflict(
                    "the admin account was created concurrently".to_string(),
                )
                .into_response());
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
            let ok = blocking(move || s.verify_admin_password(&provided))
                .await
                .map_err(IntoResponse::into_response)?;
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
                .into_response());
            }

            let password = req.new_password.clone();
            let s = store(&state);
            blocking(move || {
                s.set_admin_password(&password)?;
                s.clear_rate_limit(BUCKET_BOOTSTRAP)?;
                s.audit("bootstrap", Some("password set"), true)
            })
            .await
            .map_err(IntoResponse::into_response)?;

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
) -> Result<Response, Response> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_LOGIN).await {
        return Err(limited);
    }

    if !valid_username(&req.username)
        || req.password.chars().count() > crate::auth::password::MAX_PASSWORD_LENGTH
    {
        return Err(reject_login(&state, "invalid credentials").await);
    }

    let username = req.username.clone();
    let password = req.password.clone();
    let s = store(&state);
    let ok = blocking(move || s.verify_credentials(&username, &password))
        .await
        .map_err(IntoResponse::into_response)?;

    if !ok {
        return Err(reject_login(&state, "invalid credentials").await);
    }

    let s = store(&state);
    let (token, csrf, expires_at) = blocking(move || {
        let session = s.create_session()?;
        s.clear_rate_limit(BUCKET_LOGIN)?;
        s.audit("login", None, true)?;
        Ok(session)
    })
    .await
    .map_err(IntoResponse::into_response)?;

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
) -> Result<Json<ResetPasswordResponse>, Response> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_PASSWORD_RESET).await {
        return Err(limited);
    }
    if !valid_username(&req.username)
        || req.current_password.chars().count() > crate::auth::password::MAX_PASSWORD_LENGTH
    {
        return Err(reject_password_reset(&state).await);
    }
    crate::auth::password::validate_password(&req.new_password)
        .map_err(|error| ApiError::ValidationError(error.to_string()).into_response())?;

    let s = store(&state);
    let changed =
        blocking(move || s.reset_password(&req.username, &req.current_password, &req.new_password))
            .await
            .map_err(IntoResponse::into_response)?;
    if !changed {
        return Err(reject_password_reset(&state).await);
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
) -> Result<Json<DeviceAuthorizationResponse>, Response> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_DEVICE_CODE).await {
        return Err(limited);
    }
    if !valid_client_id(&req.client_id) {
        return Err(oauth_error(
            StatusCode::BAD_REQUEST,
            OAuthErrorCode::InvalidClient,
            "client_id must contain 1 to 128 letters, numbers, periods, underscores, colons, or hyphens",
        ));
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
        Ok(pair)
    })
    .await
    .map_err(IntoResponse::into_response)?;

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

// =============================================================================
// Token endpoint
// =============================================================================

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
) -> Result<Json<TokenResponse>, Response> {
    if let Some(limited) = enforce_backoff(&state, BUCKET_TOKEN).await {
        return Err(limited);
    }

    let (scopes, refresh_token) = match req {
        TokenRequest::DeviceCode {
            device_code: code,
            client_id,
        } => {
            if !valid_client_id(&client_id) {
                return Err(oauth_error(
                    StatusCode::BAD_REQUEST,
                    OAuthErrorCode::InvalidClient,
                    "client_id is invalid",
                ));
            }
            let s = store(&state);
            let outcome = blocking(move || s.poll_device(&code, &client_id))
                .await
                .map_err(IntoResponse::into_response)?;

            match outcome {
                DevicePollOutcome::Approved { client_id, scopes } => {
                    let s = store(&state);
                    let owned = scopes.clone();
                    let refresh = blocking(move || s.create_refresh_family(&client_id, &owned))
                        .await
                        .map_err(IntoResponse::into_response)?;
                    (scopes, Some(refresh))
                }
                DevicePollOutcome::Pending => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::AuthorizationPending,
                        "the user has not yet approved this device",
                    ))
                }
                DevicePollOutcome::SlowDown => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::SlowDown,
                        "polling faster than the advertised interval",
                    ))
                }
                DevicePollOutcome::Denied => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::AccessDenied,
                        "the user declined this device",
                    ))
                }
                DevicePollOutcome::Expired => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::ExpiredToken,
                        "the device code has expired or was already used",
                    ))
                }
            }
        }

        TokenRequest::RefreshToken {
            refresh_token: token,
            client_id,
        } => {
            if !valid_client_id(&client_id) {
                return Err(oauth_error(
                    StatusCode::BAD_REQUEST,
                    OAuthErrorCode::InvalidClient,
                    "client_id is invalid",
                ));
            }
            let s = store(&state);
            let outcome = blocking(move || s.redeem_refresh_token(&token, &client_id))
                .await
                .map_err(IntoResponse::into_response)?;

            match outcome {
                RefreshOutcome::Rotated {
                    refresh_token,
                    scopes,
                } => (scopes, Some(refresh_token)),
                RefreshOutcome::Reused => {
                    let s = store(&state);
                    let _ = blocking(move || {
                        s.audit("refresh token reuse", Some("family revoked"), false)
                    })
                    .await;
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::InvalidGrant,
                        "this refresh token was already used; the token family has been revoked",
                    ));
                }
                RefreshOutcome::Invalid => {
                    return Err(oauth_error(
                        StatusCode::BAD_REQUEST,
                        OAuthErrorCode::InvalidGrant,
                        "the refresh token is invalid, expired, or revoked",
                    ))
                }
            }
        }

        TokenRequest::AccessKey { access_key: key } => {
            let s = store(&state);
            let scopes = blocking(move || s.redeem_access_key(&key))
                .await
                .map_err(IntoResponse::into_response)?;
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
                ));
            };
            // An access key is re-presented on each exchange, so it produces no
            // refresh token — there is nothing to refresh.
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
        .map_err(|e| ApiError::InternalError(e.to_string()).into_response())?;

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

// =============================================================================
// Access keys
// =============================================================================

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
