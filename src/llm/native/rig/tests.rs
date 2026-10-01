//! One test per `ModelServerKind` against a local axum mock: asserts the wire
//! request (path, auth, forced `submit` tool) and parses a canned provider
//! response from `testdata/native_llm/`. No network, no live keys.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Router;

use super::*;
use crate::api::providers::model_server::ModelServerKind;
use crate::llm::native::{JudgeCandidate, JudgeRequest, JudgeVerdict, NativeLlmError};

#[derive(Debug, Clone)]
struct Recorded {
    path_and_query: String,
    headers: HeaderMap,
    body: serde_json::Value,
}

#[derive(Clone)]
struct Mock {
    reply: &'static str,
    seen: Arc<Mutex<Vec<Recorded>>>,
}

async fn record(
    State(mock): State<Mock>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    mock.seen.lock().unwrap().push(Recorded {
        path_and_query: uri
            .path_and_query()
            .map(ToString::to_string)
            .unwrap_or_default(),
        headers,
        body: serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
    });
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        mock.reply,
    )
}

async fn serve(reply: &'static str) -> (String, Arc<Mutex<Vec<Recorded>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new().fallback(record).with_state(Mock {
        reply,
        seen: Arc::clone(&seen),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), seen)
}

fn loopback_policy() -> EgressPolicy {
    EgressPolicy {
        allow_loopback: true,
        allow_private: false,
    }
}

fn server(kind: &str, base_url: &str, key_env: Option<&str>) -> ModelServer {
    ModelServer {
        name: format!("{kind}-mock"),
        kind: kind.to_string(),
        base_url: Some(base_url.to_string()),
        api_key_env: key_env.map(str::to_string),
        extra_env: HashMap::new(),
        display_name: None,
    }
}

/// Each test uses its own env var name, so parallel tests never race on it.
fn set_key(var: &str) -> &'static str {
    const KEY: &str = "test-key-123";
    std::env::set_var(var, KEY);
    KEY
}

fn request() -> JudgeRequest {
    JudgeRequest {
        instruction: "Pick the better plan.".to_string(),
        candidates: vec![
            JudgeCandidate {
                label: "a".to_string(),
                text: "plan A".to_string(),
            },
            JudgeCandidate {
                label: "b".to_string(),
                text: "plan B".to_string(),
            },
        ],
    }
}

fn expected() -> JudgeVerdict {
    JudgeVerdict {
        winner_index: 1,
        rationale: "more thorough".to_string(),
    }
}

async fn judge_once(
    kind: &str,
    reply: &'static str,
    key_env: Option<&str>,
) -> (Result<JudgeVerdict, NativeLlmError>, Recorded) {
    let (base, seen) = serve(reply).await;
    let judge = RigJudge::new(
        &server(kind, &base, key_env),
        "test-model",
        &loopback_policy(),
        Duration::from_secs(5),
    )
    .unwrap();
    let result = judge.judge(&request()).await;
    let recorded = seen
        .lock()
        .unwrap()
        .pop()
        .expect("no request reached the mock");
    (result, recorded)
}

fn header<'a>(r: &'a Recorded, name: &str) -> Option<&'a str> {
    r.headers.get(name).and_then(|v| v.to_str().ok())
}

fn assert_submit_tool_offered(r: &Recorded) {
    let body = r.body.to_string();
    assert!(body.contains("\"submit\""), "submit tool missing: {body}");
    assert!(body.contains("winner_index"), "schema missing: {body}");
    assert!(body.contains("plan B"), "candidates missing: {body}");
}

#[tokio::test]
async fn anthropic_api() {
    let key = set_key("OPERATOR_TEST_JUDGE_KEY_ANTHROPIC");
    let (result, r) = judge_once(
        "anthropic-api",
        include_str!("../../../../testdata/native_llm/anthropic.json"),
        Some("OPERATOR_TEST_JUDGE_KEY_ANTHROPIC"),
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/v1/messages");
    assert_eq!(header(&r, "x-api-key"), Some(key));
    assert_eq!(r.body["tool_choice"]["type"], "any");
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn openai_api_uses_chat_completions() {
    let key = set_key("OPERATOR_TEST_JUDGE_KEY_OPENAI");
    let (result, r) = judge_once(
        "openai-api",
        include_str!("../../../../testdata/native_llm/openai.json"),
        Some("OPERATOR_TEST_JUDGE_KEY_OPENAI"),
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/v1/chat/completions");
    assert_eq!(
        header(&r, "authorization"),
        Some(format!("Bearer {key}").as_str())
    );
    assert_eq!(r.body["tool_choice"], "required");
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn openai_compat_uses_chat_completions() {
    let key = set_key("OPERATOR_TEST_JUDGE_KEY_COMPAT");
    let (result, r) = judge_once(
        "openai-compat",
        include_str!("../../../../testdata/native_llm/openai.json"),
        Some("OPERATOR_TEST_JUDGE_KEY_COMPAT"),
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/v1/chat/completions");
    assert_eq!(
        header(&r, "authorization"),
        Some(format!("Bearer {key}").as_str())
    );
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn lmstudio_needs_no_key() {
    let (result, r) = judge_once(
        "lmstudio",
        include_str!("../../../../testdata/native_llm/openai.json"),
        None,
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/v1/chat/completions");
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn google_api() {
    let key = set_key("OPERATOR_TEST_JUDGE_KEY_GOOGLE");
    let (result, r) = judge_once(
        "google-api",
        include_str!("../../../../testdata/native_llm/gemini.json"),
        Some("OPERATOR_TEST_JUDGE_KEY_GOOGLE"),
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert!(
        r.path_and_query
            .starts_with("/v1beta/models/test-model:generateContent"),
        "{}",
        r.path_and_query
    );
    let key_sent = r.path_and_query.contains(&format!("key={key}"))
        || header(&r, "x-goog-api-key") == Some(key);
    assert!(key_sent, "gemini key not sent");
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn xai_api() {
    let key = set_key("OPERATOR_TEST_JUDGE_KEY_XAI");
    let (result, r) = judge_once(
        "xai-api",
        include_str!("../../../../testdata/native_llm/xai.json"),
        Some("OPERATOR_TEST_JUDGE_KEY_XAI"),
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/v1/responses");
    assert_eq!(
        header(&r, "authorization"),
        Some(format!("Bearer {key}").as_str())
    );
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn ollama() {
    let (result, r) = judge_once(
        "ollama",
        include_str!("../../../../testdata/native_llm/ollama.json"),
        None,
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/api/chat");
    assert_eq!(header(&r, "authorization"), None);
    assert_submit_tool_offered(&r);
}

#[tokio::test]
async fn openrouter() {
    let key = set_key("OPERATOR_TEST_JUDGE_KEY_OPENROUTER");
    let (result, r) = judge_once(
        "openrouter",
        include_str!("../../../../testdata/native_llm/openrouter.json"),
        Some("OPERATOR_TEST_JUDGE_KEY_OPENROUTER"),
    )
    .await;
    assert_eq!(result, Ok(expected()));
    assert_eq!(r.path_and_query, "/chat/completions");
    assert_eq!(
        header(&r, "authorization"),
        Some(format!("Bearer {key}").as_str())
    );
    assert_submit_tool_offered(&r);
}

#[test]
fn every_kind_is_covered_by_a_test_above() {
    // Adding a kind makes `client::build`'s exhaustive match fail to compile;
    // this keeps the per-kind wire tests in step with it.
    assert_eq!(ModelServerKind::ALL.len(), 8);
}

#[tokio::test]
async fn hardened_policy_rejects_loopback_before_any_request() {
    let (base, seen) = serve("{}").await;
    let result = RigJudge::new(
        &server("ollama", &base, None),
        "m",
        &EgressPolicy::hardened(),
        Duration::from_secs(5),
    );
    assert!(matches!(result, Err(NativeLlmError::Egress(_))));
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn missing_required_key_is_a_config_error() {
    let result = RigJudge::new(
        &server(
            "anthropic-api",
            "https://api.anthropic.com",
            Some("OPERATOR_TEST_JUDGE_KEY_NEVER_SET"),
        ),
        "m",
        &EgressPolicy::hardened(),
        Duration::from_secs(5),
    );
    assert!(
        matches!(result, Err(NativeLlmError::Config(msg)) if msg.contains("OPERATOR_TEST_JUDGE_KEY_NEVER_SET"))
    );
}

#[tokio::test]
async fn reply_without_tool_call_is_no_tool_call() {
    const TEXT_ONLY: &str = r#"{"id":"msg_01","type":"message","role":"assistant","model":"m","content":[{"type":"text","text":"B is better"}],"stop_reason":"end_turn","stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":1}}"#;
    set_key("OPERATOR_TEST_JUDGE_KEY_TEXT_ONLY");
    let (result, _) = judge_once(
        "anthropic-api",
        TEXT_ONLY,
        Some("OPERATOR_TEST_JUDGE_KEY_TEXT_ONLY"),
    )
    .await;
    assert_eq!(result, Err(NativeLlmError::NoToolCall));
}

#[test]
fn prompt_numbers_candidates_from_zero() {
    let prompt = judge_prompt(&request());
    assert!(prompt.starts_with("Pick the better plan."));
    assert!(prompt.contains("<candidate index=\"0\" label=\"a\">\nplan A"));
    assert!(prompt.contains("<candidate index=\"1\" label=\"b\">\nplan B"));
}
