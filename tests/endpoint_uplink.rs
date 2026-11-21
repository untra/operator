//! The endpoint uplink end to end over a real socket: pair through the device
//! flow, open the WebSocket, and exercise supersede, revoke, and versioning.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use operator::auth::tokens::{api_claims, ENDPOINT_CLIENT_ID};
use operator::config::Config;
use operator::endpoint::protocol::{
    Capabilities, DownlinkFrame, Heartbeat, Hello, UplinkFrame, PROTOCOL_VERSION,
};
use operator::endpoint::registry::{INCOMPATIBLE_VERSION, REVOKED, SUPERSEDED};
use operator::rest::dto::auth::Scope;
use operator::rest::dto::{EndpointsResponse, TokenResponse};
use operator::rest::{build_router, state::ApiState};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const HOSTNAME: &str = "GAMING-PC";
const WAIT: Duration = Duration::from_secs(5);

struct Server {
    state: ApiState,
    base: String,
    http: reqwest::Client,
    _directory: tempfile::TempDir,
}

impl Server {
    async fn start() -> Self {
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
        let tickets = config.tickets_path();
        let state = ApiState::try_new(config, tickets).unwrap();

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = build_router(state.clone()).unwrap();
        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });
        Self {
            state,
            base: format!("http://{addr}"),
            http: operator::http_client::client_builder().build().unwrap(),
            _directory: directory,
        }
    }

    fn admin_token(&self) -> String {
        let claims = api_claims(
            "admin",
            &Scope::ALL,
            chrono::Utc::now(),
            uuid::Uuid::new_v4().to_string(),
        );
        self.state.auth.signing_key.sign(&claims).unwrap()
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> reqwest::Response {
        self.http
            .post(format!("{}{path}", self.base))
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    /// Device flow: request a code, approve it as the admin, redeem it.
    async fn pair(&self) -> TokenResponse {
        let issued: serde_json::Value = self
            .post(
                "/api/v1/auth/device/code",
                serde_json::json!({ "client_id": ENDPOINT_CLIENT_ID }),
            )
            .await
            .json()
            .await
            .unwrap();
        let user_code = issued["user_code"].as_str().unwrap();
        self.state
            .auth
            .store
            .approve_device(user_code)
            .unwrap()
            .expect("pending authorization");
        let response = self
            .post(
                "/api/v1/auth/token",
                serde_json::json!({
                    "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                    "device_code": issued["device_code"],
                    "client_id": ENDPOINT_CLIENT_ID,
                }),
            )
            .await;
        assert_eq!(response.status(), 200);
        response.json().await.unwrap()
    }

    async fn connect(&self, access_token: &str) -> Result<Socket, WsError> {
        let url = format!(
            "{}/api/v1/endpoints/uplink",
            self.base.replacen("http", "ws", 1)
        );
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert(
            "authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );
        tokio_tungstenite::connect_async(request)
            .await
            .map(|(s, _)| s)
    }

    async fn endpoints(&self) -> EndpointsResponse {
        self.http
            .get(format!("{}/api/v1/endpoints", self.base))
            .bearer_auth(self.admin_token())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

fn capabilities(hash: &str) -> Capabilities {
    Capabilities {
        hash: hash.into(),
        hostname: HOSTNAME.into(),
        os: "windows".into(),
        arch: "x86_64".into(),
        gpus: vec![],
        llm_tools: vec![],
        model_servers: vec![],
        projects: vec![],
        launch_enabled: false,
    }
}

async fn send(socket: &mut Socket, frame: &UplinkFrame) {
    let text = serde_json::to_string(frame).unwrap();
    socket.send(Message::Text(text.into())).await.unwrap();
}

async fn hello(socket: &mut Socket, version: u32) {
    send(
        socket,
        &UplinkFrame::Hello(Hello {
            protocol_version: version,
            client_version: "test".into(),
            capabilities: capabilities("h1"),
        }),
    )
    .await;
}

async fn next_message(socket: &mut Socket) -> Message {
    tokio::time::timeout(WAIT, socket.next())
        .await
        .expect("timed out waiting for the server")
        .expect("socket ended")
        .expect("socket error")
}

async fn expect_welcome(socket: &mut Socket) -> String {
    match next_message(socket).await {
        Message::Text(text) => match serde_json::from_str(text.as_str()).unwrap() {
            DownlinkFrame::Welcome(welcome) => welcome.endpoint_id,
            other => panic!("expected welcome, got {other:?}"),
        },
        other => panic!("expected text, got {other:?}"),
    }
}

async fn expect_close(socket: &mut Socket, code: u16) {
    match next_message(socket).await {
        Message::Close(Some(frame)) => assert_eq!(frame.code, CloseCode::from(code)),
        other => panic!("expected close {code}, got {other:?}"),
    }
}

#[tokio::test]
async fn an_endpoint_pairs_connects_heartbeats_and_is_listed() {
    let server = Server::start().await;
    let token = server.pair().await;
    assert!(token.scopes.is_empty());

    let mut socket = server.connect(&token.access_token).await.unwrap();
    hello(&mut socket, PROTOCOL_VERSION).await;
    let endpoint_id = expect_welcome(&mut socket).await;
    send(
        &mut socket,
        &UplinkFrame::Heartbeat(Heartbeat {
            seq: 1,
            capabilities_hash: "h1".into(),
        }),
    )
    .await;

    let listed = server.endpoints().await;
    assert_eq!(listed.total, 1);
    let entry = &listed.endpoints[0];
    assert_eq!(entry.id, endpoint_id);
    assert_eq!(entry.name, HOSTNAME, "the hostname names a new endpoint");
    assert_eq!(
        serde_json::to_value(entry.liveness).unwrap(),
        serde_json::json!("live")
    );
}

#[tokio::test]
async fn a_second_uplink_supersedes_the_first_and_revoke_closes_it() {
    let server = Server::start().await;
    let token = server.pair().await;

    let mut first = server.connect(&token.access_token).await.unwrap();
    hello(&mut first, PROTOCOL_VERSION).await;
    let endpoint_id = expect_welcome(&mut first).await;

    let mut second = server.connect(&token.access_token).await.unwrap();
    hello(&mut second, PROTOCOL_VERSION).await;
    expect_welcome(&mut second).await;
    expect_close(&mut first, SUPERSEDED.code).await;

    let response = server
        .http
        .delete(format!("{}/api/v1/endpoints/{endpoint_id}", server.base))
        .bearer_auth(server.admin_token())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
    expect_close(&mut second, REVOKED.code).await;

    match server.connect(&token.access_token).await {
        Err(WsError::Http(response)) => assert_eq!(response.status(), 401),
        other => panic!("a revoked endpoint must not reconnect: {other:?}"),
    }
    let refresh = server
        .post(
            "/api/v1/auth/token",
            serde_json::json!({
                "grant_type": "refresh_token",
                "refresh_token": token.refresh_token.unwrap(),
                "client_id": ENDPOINT_CLIENT_ID,
            }),
        )
        .await;
    assert_eq!(refresh.status(), 400);
    assert_eq!(server.endpoints().await.total, 0);
}

#[tokio::test]
async fn an_incompatible_client_is_closed() {
    let server = Server::start().await;
    let token = server.pair().await;
    let mut socket = server.connect(&token.access_token).await.unwrap();
    hello(&mut socket, PROTOCOL_VERSION + 1).await;
    expect_close(&mut socket, INCOMPATIBLE_VERSION.code).await;
}

#[tokio::test]
async fn only_endpoints_may_open_the_uplink() {
    let server = Server::start().await;
    match server.connect(&server.admin_token()).await {
        Err(WsError::Http(response)) => assert_eq!(response.status(), 403),
        other => panic!("an admin token must not open the uplink: {other:?}"),
    }
}
