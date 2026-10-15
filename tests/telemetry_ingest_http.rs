//! The OTLP ingest HTTP contract, driven in-process against `build_router`.

use std::io::Write;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, KeyValue};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use operator_telemetry::attr;
use prost::Message;
use tower::ServiceExt;

use operator::auth::tokens::{api_claims, telemetry_claims};
use operator::config::Config;
use operator::rest::dto::{auth::Scope, TelemetrySessionWindow};
use operator::rest::{build_router, state::ApiState};

const PROTOBUF: &str = "application/x-protobuf";
const LOGS: &str = "/otlp/v1/logs";
const SESSION: &str = "sess-1";

struct Server {
    state: ApiState,
    _directory: tempfile::TempDir,
}

impl Server {
    fn start(configure: impl FnOnce(&mut Config)) -> Self {
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
        configure(&mut config);
        let tickets = config.tickets_path();
        Self {
            state: ApiState::new(config, tickets),
            _directory: directory,
        }
    }

    fn enabled() -> Self {
        Self::start(|c| c.telemetry.enabled = true)
    }

    fn ingest_token(&self, profile: uuid::Uuid) -> String {
        let mut claims = telemetry_claims(
            "admin",
            "FEAT-1",
            "build",
            SESSION,
            chrono::Duration::hours(1),
            chrono::Utc::now(),
            uuid::Uuid::new_v4().to_string(),
        );
        claims.profile_id = Some(profile);
        self.state.auth.signing_key.sign(&claims).unwrap()
    }

    fn own_ingest_token(&self) -> String {
        self.ingest_token(self.state.config().profile.id)
    }

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

    async fn post(
        &self,
        path: &str,
        token: Option<&str>,
        content_type: &str,
        encoding: Option<&str>,
        body: Vec<u8>,
    ) -> axum::response::Response {
        let mut builder = Request::builder()
            .method("POST")
            .uri(path)
            .header(header::CONTENT_TYPE, content_type);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(encoding) = encoding {
            builder = builder.header(header::CONTENT_ENCODING, encoding);
        }
        self.send(builder.body(Body::from(body)).unwrap()).await
    }

    async fn get(&self, path: &str, token: &str) -> axum::response::Response {
        self.send(
            Request::builder()
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    /// Ingest is applied to the window by a worker, so poll briefly for it.
    async fn window(&self, session: &str) -> TelemetrySessionWindow {
        let token = self.token_with(&[Scope::Admin]);
        let path = format!("/api/v1/telemetry/sessions/{session}");
        for _ in 0..200 {
            let response = self.get(&path, &token).await;
            if response.status() == StatusCode::OK {
                let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                return serde_json::from_slice(&bytes).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("session {session} never appeared in the window");
    }
}

fn kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..KeyValue::default()
    }
}

fn logs_body() -> Vec<u8> {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![LogRecord {
                    event_name: "claude_code.api_request".to_string(),
                    attributes: vec![
                        kv(attr::OPERATOR_SESSION_ID, "spoofed"),
                        kv(attr::OPERATOR_TICKET_ID, "OTHER-9"),
                        kv(attr::OPERATOR_AUTH_TOKEN, "sk-ant-leak"),
                    ],
                    ..LogRecord::default()
                }],
                ..ScopeLogs::default()
            }],
            ..ResourceLogs::default()
        }],
    }
    .encode_to_vec()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(bytes).unwrap();
    enc.finish().unwrap()
}

#[tokio::test]
async fn ingest_is_not_found_while_telemetry_is_disabled() {
    let server = Server::start(|_| {});
    let token = server.own_ingest_token();
    let response = server
        .post(LOGS, Some(&token), PROTOBUF, None, logs_body())
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ingested_logs_reach_the_window_with_trusted_correlation_and_no_secrets() {
    let server = Server::enabled();
    let token = server.own_ingest_token();
    let response = server
        .post(
            LOGS,
            Some(&token),
            PROTOBUF,
            Some("gzip"),
            gzip(&logs_body()),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], PROTOBUF);

    let window = server.window(SESSION).await;
    let attributes = &window.records[0].attributes;
    assert_eq!(attributes[attr::OPERATOR_SESSION_ID], SESSION);
    assert_eq!(attributes[attr::OPERATOR_TICKET_ID], "FEAT-1");
    assert!(!attributes.contains_key(attr::OPERATOR_AUTH_TOKEN));
}

#[tokio::test]
async fn otlp_json_is_answered_in_json() {
    let server = Server::enabled();
    let token = server.own_ingest_token();
    let body = br#"{"resourceSpans":[{"scopeSpans":[{"spans":[{"name":"opr8r.step","traceId":"0102030405060708090a0b0c0d0e0f10","spanId":"0102030405060708"}]}]}]}"#;
    let response = server
        .post(
            "/otlp/v1/traces",
            Some(&token),
            "application/json",
            None,
            body.to_vec(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let window = server.window(SESSION).await;
    assert_eq!(window.records[0].name, "opr8r.step");
    assert_eq!(
        window.records[0].trace_id.as_deref(),
        Some("0102030405060708090a0b0c0d0e0f10")
    );
}

#[tokio::test]
async fn ingest_requires_a_bearer_credential_that_may_ingest() {
    let server = Server::enabled();
    let no_token = server.post(LOGS, None, PROTOBUF, None, logs_body()).await;
    assert_eq!(no_token.status(), StatusCode::UNAUTHORIZED);

    let read_only = server.token_with(&[Scope::Read]);
    let response = server
        .post(LOGS, Some(&read_only), PROTOBUF, None, logs_body())
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let admin = server.token_with(&[Scope::Admin]);
    let response = server
        .post(LOGS, Some(&admin), PROTOBUF, None, logs_body())
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn ingest_token_from_another_profile_is_rejected() {
    let server = Server::enabled();
    let token = server.ingest_token(uuid::Uuid::new_v4());
    let response = server
        .post(LOGS, Some(&token), PROTOBUF, None, logs_body())
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ingest_token_cannot_read_the_api_or_the_window() {
    let server = Server::enabled();
    let token = server.own_ingest_token();
    for path in [
        "/api/v1/health",
        "/api/v1/configuration",
        "/api/v1/telemetry/sessions/sess-1",
    ] {
        let response = server.get(path, &token).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{path}");
    }
}

#[tokio::test]
async fn ingest_rejects_unsupported_media_and_malformed_bodies() {
    let server = Server::enabled();
    let token = server.own_ingest_token();
    let response = server
        .post(LOGS, Some(&token), "text/plain", None, logs_body())
        .await;
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let response = server
        .post(LOGS, Some(&token), PROTOBUF, None, vec![0xff, 0xff, 0xff])
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ingest_enforces_the_body_limit_compressed_and_expanded() {
    let server = Server::start(|c| {
        c.telemetry.enabled = true;
        c.telemetry.ingest.max_body_bytes = 64;
    });
    let token = server.own_ingest_token();
    let response = server
        .post(LOGS, Some(&token), PROTOBUF, None, vec![0; 65])
        .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let bomb = gzip(&[0; 4096]);
    assert!(bomb.len() <= 64);
    let response = server
        .post(LOGS, Some(&token), PROTOBUF, Some("gzip"), bomb)
        .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn unknown_otlp_paths_are_never_served_the_spa() {
    let server = Server::enabled();
    let token = server.own_ingest_token();
    let response = server
        .post(
            "/otlp/v1/profiles",
            Some(&token),
            PROTOBUF,
            None,
            Vec::new(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_fixed_policy_matches_across_signals_and_wire_encodings() {
    use opentelemetry_proto::tonic::collector::{
        metrics::v1::ExportMetricsServiceRequest, trace::v1::ExportTraceServiceRequest,
    };

    for enabled in [false, true] {
        for json in [false, true] {
            for signal in ["traces", "logs", "metrics"] {
                let server = Server::start(|config| {
                    config.telemetry.enabled = true;
                    config.telemetry.enable_redaction = enabled;
                });
                let attrs = serde_json::json!([
                    {"key": "user.name", "value": {"stringValue": "alice"}},
                    {"key": "unknown.content", "value": {"stringValue": "private prompt"}},
                    {"key": "operator.auth.token", "value": {"stringValue": "secret"}}
                ]);
                let payload = match signal {
                    "traces" => serde_json::json!({"resourceSpans": [{"scopeSpans": [{"spans": [{
                        "name": "unknown name", "attributes": attrs
                    }]}]}]}),
                    "logs" => serde_json::json!({"resourceLogs": [{"scopeLogs": [{"logRecords": [{
                        "eventName": "unknown name", "attributes": attrs,
                        "body": {"stringValue": "private prompt"}
                    }]}]}]}),
                    _ => serde_json::json!({"resourceMetrics": [{"scopeMetrics": [{"metrics": [{
                        "name": "operator.agents.active", "metadata": attrs, "gauge": {"dataPoints": [{
                            "attributes": attrs, "asInt": "1"
                        }]}
                    }]}]}]}),
                };
                let body = if json {
                    serde_json::to_vec(&payload).unwrap()
                } else {
                    match signal {
                        "traces" => serde_json::from_value::<ExportTraceServiceRequest>(payload)
                            .unwrap()
                            .encode_to_vec(),
                        "logs" => serde_json::from_value::<ExportLogsServiceRequest>(payload)
                            .unwrap()
                            .encode_to_vec(),
                        _ => serde_json::from_value::<ExportMetricsServiceRequest>(payload)
                            .unwrap()
                            .encode_to_vec(),
                    }
                };
                let token = server.own_ingest_token();
                let response = server
                    .post(
                        &format!("/otlp/v1/{signal}"),
                        Some(&token),
                        if json { "application/json" } else { PROTOBUF },
                        None,
                        body,
                    )
                    .await;
                assert_eq!(response.status(), StatusCode::OK);
                let window = server.window(SESSION).await;
                let record = &window.records[0];
                let personal = record.attributes["user.name"].as_str().unwrap();
                if enabled {
                    assert!(personal.starts_with("hmac:"));
                    assert!(!record.attributes.contains_key("unknown.content"));
                    assert_eq!(
                        record.name,
                        match signal {
                            "traces" => "operator.redacted.span",
                            "logs" => "operator.redacted.event",
                            _ => "operator.agents.active",
                        }
                    );
                } else {
                    assert_eq!(personal, "alice");
                    assert_eq!(record.attributes["unknown.content"], "private prompt");
                }
                assert!(!record.attributes.contains_key("operator.auth.token"));
                assert_eq!(record.attributes[attr::OPERATOR_SESSION_ID], SESSION);
                if signal == "logs" {
                    assert_eq!(record.body.is_none(), enabled);
                }
            }
        }
    }
}
