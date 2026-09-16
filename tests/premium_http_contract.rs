//! The HTTP contract for entitlement: 401, 403 and 402 are three different
//! answers, and callers act on the difference.
//!
//! Driven in-process against `build_router`, so these assert the real router,
//! middleware and scope table rather than a hand-rolled stand-in.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use operator::auth::tokens::api_claims;
use operator::config::Config;
use operator::rest::state::ApiState;
use operator::rest::{build_router, dto::auth::Scope};

const SSH_TARGET: &str =
    r#"{"name":"build-host","kind":"ssh","ssh_alias":"build","workdir":"/srv/work"}"#;

struct Server {
    state: ApiState,
    _directory: tempfile::TempDir,
}

impl Server {
    fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory
            .path()
            .join("state")
            .to_string_lossy()
            .into_owned();
        config.paths.tickets = directory
            .path()
            .join("tickets")
            .to_string_lossy()
            .into_owned();
        config.paths.projects = directory.path().to_string_lossy().into_owned();
        config.profile.id = uuid::Uuid::new_v4();
        config.profile_registry = Some(directory.path().join("profiles.sqlite"));
        let tickets = config.tickets_path();
        Self {
            state: ApiState::new(config, tickets),
            _directory: directory,
        }
    }

    /// The loopback local-unlock credential, which carries every scope.
    fn admin_token(&self) -> String {
        self.state
            .auth
            .local_token
            .clone()
            .expect("a loopback bind issues a local token")
    }

    /// An access token carrying exactly `scopes`.
    fn token_with(&self, scopes: &[Scope]) -> String {
        let claims = api_claims(
            "admin",
            scopes,
            chrono::Utc::now(),
            uuid::Uuid::new_v4().to_string(),
        );
        self.state.auth.signing_key.sign(&claims).unwrap()
    }

    async fn send(&self, request: Request<Body>) -> axum::response::Response {
        build_router(self.state.clone())
            .oneshot(request)
            .await
            .unwrap()
    }

    async fn get(&self, path: &str, token: Option<&str>) -> axum::response::Response {
        let mut builder = Request::builder().method("GET").uri(path);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        self.send(builder.body(Body::empty()).unwrap()).await
    }

    async fn post(&self, path: &str, token: Option<&str>, body: &str) -> axum::response::Response {
        let mut builder = Request::builder()
            .method("POST")
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        self.send(builder.body(Body::from(body.to_string())).unwrap())
            .await
    }
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

#[tokio::test]
async fn an_unauthenticated_request_is_401_with_a_challenge() {
    let server = Server::start();

    let response = server.get("/api/v1/license", None).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(
        response.headers().contains_key(header::WWW_AUTHENTICATE),
        "a 401 tells the caller how to authenticate"
    );
}

#[tokio::test]
async fn an_authenticated_caller_without_the_scope_is_403_not_401() {
    let server = Server::start();
    // Read-only: enough to be recognised, not enough to register a target.
    let token = server.token_with(&[Scope::Read]);

    let response = server
        .post("/api/v1/targets", Some(&token), SSH_TARGET)
        .await;

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "a recognised credential missing a scope is forbidden, not unauthenticated"
    );
    assert!(
        !response.headers().contains_key(header::WWW_AUTHENTICATE),
        "a 403 must not invite the caller to retry with a credential they already have"
    );
}

#[tokio::test]
async fn an_authorized_caller_without_a_licence_is_402_with_the_feature_named() {
    let server = Server::start();
    let token = server.admin_token();

    let response = server
        .post("/api/v1/targets", Some(&token), SSH_TARGET)
        .await;

    assert_eq!(
        response.status(),
        StatusCode::PAYMENT_REQUIRED,
        "authorized but unentitled is 402, distinct from 401 and 403"
    );
    let body = body_json(response).await;
    assert_eq!(body["error"], "not_entitled");
    assert_eq!(body["feature"], "remote_targets");
    assert_eq!(body["required_tier"], "premium");
}

#[tokio::test]
async fn reading_the_licence_and_targets_needs_no_entitlement() {
    let server = Server::start();
    let token = server.admin_token();

    let license = server.get("/api/v1/license", Some(&token)).await;
    assert_eq!(license.status(), StatusCode::OK);
    let body = body_json(license).await;
    assert_eq!(body["status"], "missing");
    assert_eq!(body["premium"], false);
    assert!(
        body.get("license_key").is_none(),
        "a read must never carry the raw key"
    );

    let targets = server.get("/api/v1/targets", Some(&token)).await;
    assert_eq!(
        targets.status(),
        StatusCode::OK,
        "configured targets stay readable without Premium"
    );
}

#[tokio::test]
async fn a_licence_token_cannot_authenticate_the_api() {
    let server = Server::start();
    // A licence is a signed token too. Even one signed by this server's own key,
    // naming the admin subject and every scope, must not authenticate: the
    // audience is the boundary.
    let mut claims = api_claims(
        "admin",
        &[Scope::Read, Scope::Write, Scope::Execute],
        chrono::Utc::now(),
        uuid::Uuid::new_v4().to_string(),
    );
    claims.aud = operator::licensing::LICENSE_AUDIENCE.to_string();
    let forged = server.state.auth.signing_key.sign(&claims).unwrap();

    let response = server.get("/api/v1/license", Some(&forged)).await;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "licence audience and API audience are disjoint"
    );
}

#[tokio::test]
async fn an_invalid_licence_is_rejected_without_disturbing_the_installed_one() {
    let server = Server::start();
    let token = server.admin_token();

    let response = server
        .send(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/license")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(r#"{"license_key":"not-a-licence"}"#))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let after = body_json(server.get("/api/v1/license", Some(&token)).await).await;
    assert_eq!(after["status"], "missing", "nothing was written");
}

/// The configuration-qualified routes are mounted by a dispatcher that sits
/// outside the authorize layer and re-dispatches into a router that has it.
/// That indirection is exactly the kind of thing that silently stops enforcing,
/// and `tests/route_scope_parity.rs` cannot see the wildcard route.
#[tokio::test]
async fn a_configuration_qualified_route_still_requires_authentication() {
    let server = Server::start();
    let id = server.state.config().profile.id;

    let response = server
        .get(&format!("/api/v1/profiles/{id}/license"), None)
        .await;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "tenant dispatch must not bypass the authorize layer"
    );
}

#[tokio::test]
async fn a_configuration_qualified_route_enforces_entitlement() {
    let server = Server::start();
    let id = server.state.config().profile.id;
    let token = server.admin_token();

    let response = server
        .post(
            &format!("/api/v1/profiles/{id}/targets"),
            Some(&token),
            SSH_TARGET,
        )
        .await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    let body = body_json(response).await;
    assert_eq!(body["error"], "not_entitled");
}

/// A request for a configuration this server does not host must not fall
/// through to the default one.
#[tokio::test]
async fn an_unknown_configuration_is_not_served_by_the_default() {
    let server = Server::start();
    let token = server.admin_token();
    let stranger = uuid::Uuid::new_v4();

    let response = server
        .get(
            &format!("/api/v1/profiles/{stranger}/license"),
            Some(&token),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
