//! Live model-listing probe for a configured model server.
//!
//! [`probe_models`] hits a server's [`ModelServerKind::models_endpoint`] and
//! returns the models it serves. The same request doubles as a reachability
//! health check — a successful probe means the endpoint is up and (where
//! relevant) the API key is accepted, so there is no separate "test connection".
//!
//! Parsing is split out from the HTTP call ([`parse_models`]) so the per-protocol
//! response shapes can be unit-tested without a live server.

use std::time::Duration;

use serde_json::Value;

use super::ModelServerKind;
use crate::config::ModelServer;

/// A single model offered by a server. Minimal by design — id is the wire name
/// passed to `--model`; `display_name` is shown in UIs when the server provides one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: Option<String>,
}

/// Why a probe failed. Distinguished so UIs can show a useful status.
#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("server '{0}' has no base_url to probe")]
    NoBaseUrl(String),
    #[error("unknown model server kind '{0}'")]
    UnknownKind(String),
    #[error("request failed: {0}")]
    Network(String),
    #[error("server returned {status}: {body}")]
    HttpStatus { status: u16, body: String },
    #[error("could not parse model list: {0}")]
    Parse(String),
}

/// Result of probing a server: reachability plus the model list (when reachable).
#[derive(Debug, Clone)]
pub struct ProbeOutcome {
    pub reachable: bool,
    pub models: Vec<ModelInfo>,
    pub error: Option<String>,
}

impl ProbeOutcome {
    fn ok(models: Vec<ModelInfo>) -> Self {
        Self {
            reachable: true,
            models,
            error: None,
        }
    }

    fn failed(err: &ProbeError) -> Self {
        Self {
            reachable: false,
            models: Vec::new(),
            error: Some(err.to_string()),
        }
    }
}

/// Probe a server for its model list. Never panics; returns a [`ProbeOutcome`]
/// summarizing reachability so callers can render status without handling errors.
pub async fn probe_models(server: &ModelServer) -> ProbeOutcome {
    match probe_models_inner(server).await {
        Ok(models) => ProbeOutcome::ok(models),
        Err(err) => ProbeOutcome::failed(&err),
    }
}

async fn probe_models_inner(server: &ModelServer) -> Result<Vec<ModelInfo>, ProbeError> {
    let kind = ModelServerKind::from_slug(&server.kind)
        .ok_or_else(|| ProbeError::UnknownKind(server.kind.clone()))?;

    let base = server
        .base_url
        .as_deref()
        .filter(|u| !u.is_empty())
        .ok_or_else(|| ProbeError::NoBaseUrl(server.name.clone()))?;

    let url = format!("{}{}", base.trim_end_matches('/'), kind.models_endpoint());

    let api_key = server
        .api_key_env
        .as_deref()
        .and_then(|var| std::env::var(var).ok())
        .filter(|k| !k.is_empty());

    let client = reqwest::Client::builder()
        .user_agent("operator-tui")
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| ProbeError::Network(e.to_string()))?;

    let mut req = client.get(&url);
    // Per-protocol auth header conventions.
    match kind {
        ModelServerKind::AnthropicApi => {
            req = req.header("anthropic-version", "2023-06-01");
            if let Some(key) = &api_key {
                req = req.header("x-api-key", key);
            }
        }
        ModelServerKind::GoogleApi => {
            if let Some(key) = &api_key {
                req = req.query(&[("key", key.as_str())]);
            }
        }
        // OpenAI-protocol + ollama use bearer auth (ollama ignores it).
        _ => {
            if let Some(key) = &api_key {
                req = req.bearer_auth(key);
            }
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|e| ProbeError::Network(e.to_string()))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| ProbeError::Network(e.to_string()))?;

    if !status.is_success() {
        return Err(ProbeError::HttpStatus {
            status: status.as_u16(),
            body: body.chars().take(200).collect(),
        });
    }

    parse_models(kind, &body)
}

/// Parse a model-list response body according to the server's protocol.
pub fn parse_models(kind: ModelServerKind, body: &str) -> Result<Vec<ModelInfo>, ProbeError> {
    let value: Value = serde_json::from_str(body).map_err(|e| ProbeError::Parse(e.to_string()))?;

    let models = match kind {
        ModelServerKind::Ollama => parse_ollama(&value),
        ModelServerKind::GoogleApi => parse_gemini(&value),
        // OpenAI-protocol + anthropic both use a top-level `data` array of
        // objects keyed by `id`.
        _ => parse_openai_like(&value),
    };

    Ok(models)
}

/// ollama `/api/tags`: `{ "models": [ { "name": "llama3:latest", ... } ] }`.
fn parse_ollama(value: &Value) -> Vec<ModelInfo> {
    value
        .get("models")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(Value::as_str))
                .map(|name| ModelInfo {
                    id: name.to_string(),
                    display_name: None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `OpenAI` `/v1/models` and Anthropic `/v1/models`:
/// `{ "data": [ { "id": "...", "display_name"?: "..." } ] }`.
fn parse_openai_like(value: &Value) -> Vec<ModelInfo> {
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id").and_then(Value::as_str)?;
                    Some(ModelInfo {
                        id: id.to_string(),
                        display_name: m
                            .get("display_name")
                            .and_then(Value::as_str)
                            .map(std::string::ToString::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Gemini `/v1beta/models`:
/// `{ "models": [ { "name": "models/gemini-1.5-pro", "displayName": "..." } ] }`.
fn parse_gemini(value: &Value) -> Vec<ModelInfo> {
    value
        .get("models")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let name = m.get("name").and_then(Value::as_str)?;
                    // Strip the "models/" prefix so the id matches `--model`.
                    let id = name.strip_prefix("models/").unwrap_or(name);
                    Some(ModelInfo {
                        id: id.to_string(),
                        display_name: m
                            .get("displayName")
                            .and_then(Value::as_str)
                            .map(std::string::ToString::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ollama_tags() {
        let body = r#"{"models":[{"name":"llama3:latest","model":"llama3:latest"},
            {"name":"qwen2.5-coder:7b"}]}"#;
        let models = parse_models(ModelServerKind::Ollama, body).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "llama3:latest");
        assert_eq!(models[1].id, "qwen2.5-coder:7b");
    }

    #[test]
    fn test_parse_openai_models() {
        let body = r#"{"object":"list","data":[{"id":"gpt-4o","object":"model"},
            {"id":"gpt-4o-mini","object":"model"}]}"#;
        let models = parse_models(ModelServerKind::OpenAiCompat, body).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "gpt-4o");
    }

    #[test]
    fn test_parse_anthropic_models_with_display_name() {
        let body =
            r#"{"data":[{"type":"model","id":"claude-opus-4","display_name":"Claude Opus 4"}]}"#;
        let models = parse_models(ModelServerKind::AnthropicApi, body).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "claude-opus-4");
        assert_eq!(models[0].display_name.as_deref(), Some("Claude Opus 4"));
    }

    #[test]
    fn test_parse_gemini_strips_models_prefix() {
        let body =
            r#"{"models":[{"name":"models/gemini-1.5-pro","displayName":"Gemini 1.5 Pro"}]}"#;
        let models = parse_models(ModelServerKind::GoogleApi, body).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gemini-1.5-pro");
        assert_eq!(models[0].display_name.as_deref(), Some("Gemini 1.5 Pro"));
    }

    #[test]
    fn test_parse_invalid_json_errors() {
        let err = parse_models(ModelServerKind::Ollama, "not json").unwrap_err();
        assert!(matches!(err, ProbeError::Parse(_)));
    }

    #[test]
    fn test_parse_empty_list() {
        let models = parse_models(ModelServerKind::OpenAiCompat, r#"{"data":[]}"#).unwrap();
        assert!(models.is_empty());
    }

    #[tokio::test]
    async fn test_probe_no_base_url_is_unreachable() {
        let server = ModelServer {
            name: "anthropic-api".into(),
            kind: "anthropic-api".into(),
            base_url: None,
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: None,
        };
        let outcome = probe_models(&server).await;
        assert!(!outcome.reachable);
        assert!(outcome.error.is_some());
    }
}
