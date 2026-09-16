//! The authorization layer.
//!
//! One `middleware::from_fn_with_state` layer decides every request. It runs
//! over the *composed* router - the documented API routes, the Swagger UI, and
//! the config-gated MCP transport routes - so no surface can be mounted outside
//! its reach.
//!
//! The decision is: resolve a principal from the request's credentials, look up
//! what the matched route requires, and compare. A route the table does not
//! know is **denied**, because an unclassified route is a bug and failing open
//! would make it an exploitable one.

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::auth::scope::{required_access, Access, Principal};
use crate::auth::store::RateLimitDecision;
use crate::auth::tokens::{AUDIENCE_API, AUDIENCE_CALLBACK};
use crate::rest::dto::auth::{PrincipalKind, Scope};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// Name of the browser session cookie.
///
/// The `__Host-` prefix is enforced *by the browser*: it refuses the cookie
/// unless it is `Secure`, carries no `Domain`, and has `Path=/`. That makes it
/// impossible for a sibling subdomain to set or overwrite the session.
pub const SESSION_COOKIE: &str = "__Host-operator_session";

/// Header carrying the CSRF token on cookie-authenticated mutations.
pub const CSRF_HEADER: &str = "x-operator-csrf";

/// Paths served to an unauthenticated browser so it can render the login, bootstrap, and device-approval screens.
///
/// This is the whole SPA bundle, unavoidably: the dashboard uses fragment
/// routing, so `#/login` and `#/config` are indistinguishable to the server.
/// The bundle carries no workspace data or credentials; everything it displays arrives over authenticated API calls.
fn is_public_asset(path: &str) -> bool {
    !(path.starts_with("/api/") || path.starts_with("/swagger-ui") || path.starts_with("/api-docs"))
}

/// Requirement for a path that matched no route.
///
/// Swagger UI and the raw OpenAPI document are served by `SwaggerUi`, not by a
/// `routes!` entry, so they never produce a `MatchedPath` and cannot be listed
/// in `ROUTE_RULES`. They still need classifying: the spec enumerates every
/// endpoint this server exposes, which is not something to hand out
/// anonymously - but an authenticated admin should be able to open it.
fn unmatched_access(path: &str) -> Option<Access> {
    if path.starts_with("/swagger-ui") || path.starts_with("/api-docs") {
        return Some(Access::Scoped(Scope::Read));
    }
    if is_public_asset(path) {
        // The SPA bundle. See the note on `is_public_asset`.
        return Some(Access::Public);
    }
    // An unknown /api/ path: deny, so a route mounted without a rule fails
    // closed rather than silently open.
    None
}

/// Extract a cookie value from a `Cookie` header.
fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.split_once('='))
        .find(|(k, _)| k.trim() == name)
        .map(|(_, v)| v.trim().to_string())
}

/// Extract a bearer token from an `Authorization` header.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then(|| token.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Resolve the caller's identity from whatever credential they presented.
///
/// Order matters only in that a bearer token is checked before a cookie: an
/// explicit `Authorization` header is a deliberate act, while a cookie is sent
/// ambiently by the browser.
pub async fn resolve_principal(state: &ApiState, headers: &HeaderMap) -> Option<Principal> {
    if let Some(token) = bearer_token(headers) {
        // The local-unlock token is a plain opaque string, not a JWT.
        if let Some(local) = state.auth.local_token.as_deref() {
            if crate::auth::local::matches(local, &token) {
                return Some(Principal::local(crate::auth::store::ADMIN_SUBJECT));
            }
        }

        if let Ok(claims) = state.auth.signing_key.verify(&token, AUDIENCE_API) {
            return Some(Principal {
                subject: claims.sub.clone(),
                scopes: claims.scopes(),
                kind: PrincipalKind::AccessToken,
                expires_at: chrono::DateTime::from_timestamp(claims.exp, 0),
                session_id: None,
                ticket_id: None,
                step: None,
            });
        }

        // An agent step-completion token. It verifies under a different
        // audience, so it can never satisfy an ordinary API route; the handler
        // additionally matches its ticket/step claims against the request path.
        if let Ok(claims) = state.auth.signing_key.verify(&token, AUDIENCE_CALLBACK) {
            if claims.profile_id != Some(state.config().profile.id)
                && !(claims.profile_id.is_none() && state.config().profile.id.is_nil())
            {
                return None;
            }
            return Some(Principal {
                subject: claims.sub.clone(),
                scopes: claims.scopes(),
                kind: PrincipalKind::AgentCallback,
                expires_at: chrono::DateTime::from_timestamp(claims.exp, 0),
                session_id: None,
                ticket_id: claims.ticket_id,
                step: claims.step,
            });
        }
        return None;
    }

    if let Some(cookie) = cookie_value(headers, SESSION_COOKIE) {
        let store = state.auth.store.clone();
        return tokio::task::spawn_blocking(move || store.authenticate_session(&cookie))
            .await
            .ok()?
            .ok()?;
    }

    None
}

/// Whether a request method mutates.
fn is_mutation(method: &axum::http::Method) -> bool {
    !matches!(
        *method,
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    )
}

/// The `host[:port]` part of an origin, so `https://x.example:443` and a `Host`
/// header of `x.example:443` compare equal without guessing a scheme.
fn origin_authority(origin: &str) -> Option<&str> {
    origin
        .split_once("://")
        .map(|(_scheme, rest)| rest)
        .filter(|rest| !rest.is_empty())
}

/// Reject a cross-origin `Origin` on a cookie-authenticated mutation.
///
/// Belt and braces alongside `SameSite=Strict`: the cookie should never be sent
/// cross-site in the first place, but `Origin` costs nothing to check and
/// covers flows where the cookie policy is weaker than expected.
///
/// Three cases count as acceptable, and the first is easy to forget: a browser
/// sends `Origin` on a **same-origin** POST too. Checking only the configured
/// CORS allowlist therefore blocked the dashboard's own mutations, since that
/// list is empty by default - and a curl test never catches it, because curl
/// sends no `Origin` at all.
fn origin_is_acceptable(headers: &HeaderMap, allowed: &[String]) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) else {
        // No Origin header: not a browser cross-site request.
        return true;
    };

    // Same-origin: the Origin's authority matches the Host being addressed.
    if let (Some(origin_authority), Some(host)) = (
        origin_authority(origin),
        headers.get(header::HOST).and_then(|v| v.to_str().ok()),
    ) {
        if origin_authority.eq_ignore_ascii_case(host) {
            return true;
        }
    }

    // Explicitly configured cross-origin caller.
    allowed.iter().any(|a| a == origin)
}

/// The authorization layer.
pub async fn authorize(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let headers = request.headers().clone();

    // The route pattern axum matched (`/api/v1/tickets/{id}`), not the concrete
    // path, so the table is written once per route rather than per request.
    let matched = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|m| m.as_str().to_string());

    // Prefer the route table. Fall back to path-based classification when the
    // table has no entry: `SwaggerUi` mounts its own wildcard route, so it does
    // produce a `MatchedPath` - just not one that can appear in `ROUTE_RULES`.
    // The fallback still denies any unclassified `/api/` path.
    let access = matched
        .as_deref()
        .and_then(|pattern| required_access(method.as_str(), pattern))
        .or_else(|| unmatched_access(&path));

    let Some(access) = access else {
        // Unknown or unclassified: deny. `tests/route_scope_parity.rs` makes
        // this unreachable for mounted routes.
        return Err(
            ApiError::Unauthorized("this endpoint requires authentication".to_string())
                .into_response(),
        );
    };

    let required = match access {
        Access::Public => return Ok(next.run(request).await),
        Access::Scoped(scope) => scope,
    };

    let Some(principal) = resolve_principal(&state, &headers).await else {
        return Err(
            ApiError::Unauthorized("no valid credential was presented".to_string()).into_response(),
        );
    };

    if !principal.has_scope(required) {
        return Err(
            ApiError::Forbidden(format!("this endpoint requires the `{required}` scope"))
                .into_response(),
        );
    }

    // A cookie is sent automatically by the browser, so a cookie-authenticated
    // mutation needs proof the request was intended. A bearer token is never
    // sent ambiently, so it needs no such proof.
    if principal.kind == PrincipalKind::Session && is_mutation(&method) {
        let config = state.config();
        if !origin_is_acceptable(&headers, &config.rest_api.cors_origins) {
            return Err(
                ApiError::CsrfFailed("request Origin is not allowed".to_string()).into_response(),
            );
        }

        let Some(session_id) = principal.session_id.clone() else {
            return Err(
                ApiError::CsrfFailed("session is not identifiable".to_string()).into_response(),
            );
        };
        let Some(csrf) = headers
            .get(CSRF_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
        else {
            return Err(ApiError::CsrfFailed(format!(
                "cookie-authenticated mutations require the `{CSRF_HEADER}` header"
            ))
            .into_response());
        };

        let store = state.auth.store.clone();
        let ok = tokio::task::spawn_blocking(move || store.verify_csrf(&session_id, &csrf))
            .await
            .map_err(|_| {
                ApiError::InternalError("CSRF verification task failed".to_string()).into_response()
            })?
            .unwrap_or(false);
        if !ok {
            return Err(ApiError::CsrfFailed("CSRF token is invalid".to_string()).into_response());
        }
    }

    let mut request = request;
    request.extensions_mut().insert(principal);
    Ok(next.run(request).await)
}

/// Apply persisted backoff to a credential-issuing endpoint.
///
/// Returns the `Retry-After` response when the caller must wait.
pub async fn enforce_backoff(state: &ApiState, bucket: &str) -> Option<Response> {
    let store = state.auth.store.clone();
    let owned = bucket.to_string();
    let decision = tokio::task::spawn_blocking(move || store.check_rate_limit(&owned))
        .await
        .ok()?
        .ok()?;

    match decision {
        RateLimitDecision::Allow => None,
        RateLimitDecision::Backoff { retry_after_secs } => Some(
            (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, retry_after_secs.to_string())],
                axum::Json(serde_json::json!({
                    "error": "rate_limited",
                    "message": format!("too many attempts; retry in {retry_after_secs}s"),
                })),
            )
                .into_response(),
        ),
    }
}

/// Extractor for the principal the [`authorize`] layer attached.
///
/// Infallible by construction: a handler only runs after `authorize` inserted a
/// principal, or the route was public. A public route that asks for one gets
/// the anonymous fallback rather than a 500.
pub struct Authenticated(pub Principal);

impl<S> axum::extract::FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Authenticated(
            parts
                .extensions
                .get::<Principal>()
                .cloned()
                .unwrap_or_else(|| Principal {
                    subject: String::new(),
                    scopes: Vec::new(),
                    kind: PrincipalKind::AccessToken,
                    expires_at: None,
                    session_id: None,
                    ticket_id: None,
                    step: None,
                }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        h
    }

    #[test]
    fn test_bearer_token_parsing() {
        assert_eq!(
            bearer_token(&headers(&[("authorization", "Bearer abc123")])).as_deref(),
            Some("abc123")
        );
        // Scheme is case-insensitive per RFC 7235.
        assert_eq!(
            bearer_token(&headers(&[("authorization", "bearer abc123")])).as_deref(),
            Some("abc123")
        );
        assert!(bearer_token(&headers(&[("authorization", "Basic abc123")])).is_none());
        assert!(bearer_token(&headers(&[("authorization", "Bearer")])).is_none());
        assert!(bearer_token(&headers(&[("authorization", "Bearer ")])).is_none());
        assert!(bearer_token(&HeaderMap::new()).is_none());
    }

    #[test]
    fn test_cookie_extraction_finds_the_named_cookie_among_others() {
        let h = headers(&[(
            "cookie",
            "theme=dark; __Host-operator_session=sess-value; other=x",
        )]);
        assert_eq!(
            cookie_value(&h, SESSION_COOKIE).as_deref(),
            Some("sess-value")
        );
        assert!(cookie_value(&h, "nonexistent").is_none());
    }

    #[test]
    fn test_cookie_name_keeps_the_host_prefix() {
        // The browser enforces Secure + no Domain + Path=/ on this prefix;
        // renaming it silently drops those guarantees.
        assert_eq!(SESSION_COOKIE, "__Host-operator_session");
    }

    #[test]
    fn test_api_surfaces_are_never_treated_as_public_assets() {
        for path in [
            "/api/v1/health",
            "/api/v1/configuration",
            "/swagger-ui",
            "/swagger-ui/index.html",
            "/api-docs/openapi.json",
        ] {
            assert!(
                !is_public_asset(path),
                "{path} must not be reachable as a static asset"
            );
        }
    }

    #[test]
    fn test_spa_assets_are_public_so_the_login_screen_can_render() {
        for path in ["/", "/index.html", "/assets/index-abc123.js"] {
            assert!(is_public_asset(path));
            assert_eq!(unmatched_access(path), Some(Access::Public));
        }
    }

    #[test]
    fn test_swagger_needs_a_credential_but_is_reachable_with_one() {
        // It is served by SwaggerUi rather than a `routes!` entry, so it never
        // produces a MatchedPath and cannot live in ROUTE_RULES. Denying it
        // outright would make the API docs unusable for the admin.
        for path in [
            "/swagger-ui/",
            "/swagger-ui/index.html",
            "/api-docs/openapi.json",
        ] {
            assert_eq!(
                unmatched_access(path),
                Some(Access::Scoped(Scope::Read)),
                "{path} should require a credential but remain reachable"
            );
        }
    }

    #[test]
    fn test_an_unknown_api_path_fails_closed() {
        // A route mounted without a ROUTE_RULES entry must be denied, not
        // silently served.
        assert_eq!(unmatched_access("/api/v1/not-a-route"), None);
    }

    #[test]
    fn test_mutation_classification() {
        use axum::http::Method;
        assert!(!is_mutation(&Method::GET));
        assert!(!is_mutation(&Method::HEAD));
        assert!(!is_mutation(&Method::OPTIONS));
        assert!(is_mutation(&Method::POST));
        assert!(is_mutation(&Method::PUT));
        assert!(is_mutation(&Method::DELETE));
        assert!(is_mutation(&Method::PATCH));
    }

    #[test]
    fn test_absent_origin_is_accepted_but_a_foreign_one_is_not() {
        let allowed = vec!["https://operator.example.com".to_string()];
        // A non-browser client sends no Origin at all.
        assert!(origin_is_acceptable(&HeaderMap::new(), &allowed));
        assert!(origin_is_acceptable(
            &headers(&[("origin", "https://operator.example.com")]),
            &allowed
        ));
        assert!(!origin_is_acceptable(
            &headers(&[("origin", "https://evil.example.com")]),
            &allowed
        ));
    }

    #[test]
    fn test_same_origin_mutation_is_accepted_with_no_configured_origins() {
        // Regression: the dashboard's own POSTs were rejected because browsers
        // send `Origin` on same-origin mutations too and the default
        // `cors_origins` list is empty. curl never reproduced it - curl sends
        // no Origin header, so the check passed there.
        for (origin, host) in [
            ("http://127.0.0.1:7008", "127.0.0.1:7008"),
            ("http://localhost:7008", "localhost:7008"),
            ("https://operator.example.com", "operator.example.com"),
        ] {
            assert!(
                origin_is_acceptable(&headers(&[("origin", origin), ("host", host)]), &[]),
                "same-origin mutation from {origin} must be allowed"
            );
        }
    }

    #[test]
    fn test_a_different_host_is_still_rejected_without_configuration() {
        assert!(!origin_is_acceptable(
            &headers(&[
                ("origin", "https://evil.example.com"),
                ("host", "operator.example.com")
            ]),
            &[]
        ));
        // A port mismatch is a different origin.
        assert!(!origin_is_acceptable(
            &headers(&[
                ("origin", "http://127.0.0.1:9999"),
                ("host", "127.0.0.1:7008")
            ]),
            &[]
        ));
    }

    #[test]
    fn test_origin_authority_extraction() {
        assert_eq!(
            origin_authority("https://operator.example.com"),
            Some("operator.example.com")
        );
        assert_eq!(
            origin_authority("http://127.0.0.1:7008"),
            Some("127.0.0.1:7008")
        );
        // `null` is what a sandboxed iframe or a file:// page sends; it has no
        // authority and must not match anything.
        assert_eq!(origin_authority("null"), None);
    }
}
