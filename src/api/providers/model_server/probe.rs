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

    // Base URL: the instance's own, else the kind's probe-only default (lets the
    // first-party providers / OpenRouter / ollama be probed without a declared
    // base_url). Still NoBaseUrl for bring-your-own-endpoint kinds.
    let base = server
        .base_url
        .as_deref()
        .filter(|u| !u.is_empty())
        .or_else(|| kind.default_base_url())
        .ok_or_else(|| ProbeError::NoBaseUrl(server.name.clone()))?;

    let url = format!("{}{}", base.trim_end_matches('/'), kind.models_endpoint());

    // API key: read the instance's named env var, else the kind's default probe
    // env var (ANTHROPIC_API_KEY / OPENAI_API_KEY / GEMINI_API_KEY / …).
    let api_key = server
        .api_key_env
        .as_deref()
        .or_else(|| kind.default_api_key_env())
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
        ModelServerKind::OpenRouter => parse_openrouter(&value),
        // OpenAI-protocol + anthropic both use a top-level `data` array of
        // objects keyed by `id`. The kind is threaded through so OpenAI's
        // non-text models can be filtered out (see `is_text_model`).
        _ => parse_openai_like(kind, &value),
    };

    Ok(models)
}

/// Whether a single raw model entry is an LLM text/chat model suitable for an
/// agent model dropdown.
///
/// Listing endpoints return more than chat models — `OpenAI` and Google mix in
/// embeddings, audio (TTS/Whisper), image, and moderation models that must never
/// surface in a model picker. Each provider exposes a different capability signal
/// (or none), so the rule is per-kind:
///
/// - Google: authoritative — keep iff `supportedGenerationMethods` advertises
///   `generateContent`. Absent field ⇒ keep (tolerant of API drift / fixtures).
/// - `OpenRouter`: keep iff the architecture's output modalities include `text`.
///   Absent ⇒ keep.
/// - `OpenAI`: no capability field, so classify by id family (deny embeddings /
///   audio / image / moderation; allow the gpt / o-series / chatgpt families;
///   deny anything unrecognized so unknown non-text families stay out).
/// - Anthropic / ollama / openai-compat / lmstudio: pass-through — Anthropic
///   lists only chat models, and BYO/local hosts serve whatever the user runs.
fn is_text_model(kind: ModelServerKind, raw: &Value, id: &str) -> bool {
    match kind {
        ModelServerKind::GoogleApi => raw
            .get("supportedGenerationMethods")
            .and_then(Value::as_array)
            .map(|methods| {
                methods
                    .iter()
                    .any(|m| m.as_str() == Some("generateContent"))
            })
            // Absent capability field ⇒ keep (don't over-filter on drift).
            .unwrap_or(true),
        ModelServerKind::OpenRouter => openrouter_outputs_text(raw),
        ModelServerKind::OpenAiApi => openai_id_is_text(id),
        ModelServerKind::AnthropicApi
        | ModelServerKind::Ollama
        | ModelServerKind::OpenAiCompat
        | ModelServerKind::LmStudio => true,
    }
}

/// `OpenRouter` advertises modalities under `architecture`. Prefer the explicit
/// `output_modalities` array; fall back to the `modality` string
/// (`"text->text"`, `"text+image->text"`); keep when neither is present.
fn openrouter_outputs_text(raw: &Value) -> bool {
    let arch = raw.get("architecture");
    if let Some(mods) = arch
        .and_then(|a| a.get("output_modalities"))
        .and_then(Value::as_array)
    {
        return mods.iter().any(|m| m.as_str() == Some("text"));
    }
    if let Some(modality) = arch.and_then(|a| a.get("modality")).and_then(Value::as_str) {
        // The output side is the segment after "->" (e.g. "text+image->text").
        let output = modality.split("->").nth(1).unwrap_or(modality);
        return output.contains("text");
    }
    true
}

/// `OpenAI`'s `/v1/models` carries no capability field, so classify by id family.
/// Deny embeddings / audio / image / moderation; allow the chat families; deny
/// anything unrecognized so unknown non-text families stay out of pickers.
fn openai_id_is_text(id: &str) -> bool {
    let id = id.to_ascii_lowercase();
    const DENY: [&str; 5] = ["embedding", "whisper", "tts", "dall-e", "moderation"];
    if DENY.iter().any(|d| id.contains(d)) {
        return false;
    }
    const ALLOW_PREFIX: [&str; 5] = ["gpt", "o1", "o3", "o4", "chatgpt"];
    ALLOW_PREFIX.iter().any(|p| id.starts_with(p))
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
///
/// Shared by `OpenAI` / Anthropic / openai-compat / lmstudio, so it takes the
/// `kind` to apply the per-protocol text-model filter (only `OpenAI` filters; the
/// others pass through — see [`is_text_model`]).
fn parse_openai_like(kind: ModelServerKind, value: &Value) -> Vec<ModelInfo> {
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id").and_then(Value::as_str)?;
                    if !is_text_model(kind, m, id) {
                        return None;
                    }
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

/// `OpenRouter` `/models`:
/// `{ "data": [ { "id": "anthropic/claude-3.5-sonnet", "name": "Anthropic: Claude 3.5 Sonnet", … } ] }`.
/// Same `data`-array shape as `OpenAI`, but the human label lives in `name`
/// (`OpenAI` uses `display_name`), so it needs its own reader to keep the label.
fn parse_openrouter(value: &Value) -> Vec<ModelInfo> {
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id").and_then(Value::as_str)?;
                    if !is_text_model(ModelServerKind::OpenRouter, m, id) {
                        return None;
                    }
                    Some(ModelInfo {
                        id: id.to_string(),
                        display_name: m
                            .get("name")
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
                    if !is_text_model(ModelServerKind::GoogleApi, m, id) {
                        return None;
                    }
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
    fn test_parse_openrouter_models() {
        let body = r#"{"data":[
            {"id":"anthropic/claude-3.5-sonnet","name":"Anthropic: Claude 3.5 Sonnet"},
            {"id":"meta-llama/llama-3-70b-instruct","name":"Meta: Llama 3 70B Instruct"}]}"#;
        let models = parse_models(ModelServerKind::OpenRouter, body).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "anthropic/claude-3.5-sonnet");
        assert_eq!(
            models[0].display_name.as_deref(),
            Some("Anthropic: Claude 3.5 Sonnet")
        );
        assert_eq!(models[1].id, "meta-llama/llama-3-70b-instruct");
    }

    #[test]
    fn test_parse_gemini_strips_models_prefix() {
        let body = r#"{"models":[{"name":"models/gemini-1.5-pro","displayName":"Gemini 1.5 Pro",
            "supportedGenerationMethods":["generateContent"]}]}"#;
        let models = parse_models(ModelServerKind::GoogleApi, body).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gemini-1.5-pro");
        assert_eq!(models[0].display_name.as_deref(), Some("Gemini 1.5 Pro"));
    }

    #[test]
    fn test_parse_openai_filters_non_text_models() {
        // A realistic /v1/models mix: chat models plus embeddings/audio/image/
        // moderation that must not reach a model picker.
        let body = r#"{"object":"list","data":[
            {"id":"gpt-4o","object":"model"},
            {"id":"o3-mini","object":"model"},
            {"id":"chatgpt-4o-latest","object":"model"},
            {"id":"text-embedding-3-small","object":"model"},
            {"id":"whisper-1","object":"model"},
            {"id":"tts-1","object":"model"},
            {"id":"dall-e-3","object":"model"},
            {"id":"omni-moderation-latest","object":"model"}]}"#;
        let models = parse_models(ModelServerKind::OpenAiApi, body).unwrap();
        let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["gpt-4o", "o3-mini", "chatgpt-4o-latest"]);
        assert!(models.iter().all(|m| !m.id.contains("text-embedding")
            && !m.id.starts_with("whisper")
            && !m.id.starts_with("tts")
            && !m.id.starts_with("dall-e")
            && !m.id.contains("moderation")));
    }

    #[test]
    fn test_parse_gemini_filters_by_generation_method() {
        // generateContent ⇒ kept; embedding/TTS-only ⇒ dropped.
        let body = r#"{"models":[
            {"name":"models/gemini-1.5-pro","supportedGenerationMethods":["generateContent","countTokens"]},
            {"name":"models/text-embedding-004","supportedGenerationMethods":["embedContent"]},
            {"name":"models/gemini-2.5-flash-tts","supportedGenerationMethods":["countTokens"]}]}"#;
        let models = parse_models(ModelServerKind::GoogleApi, body).unwrap();
        let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["gemini-1.5-pro"]);
        assert!(models.iter().all(|m| !m.id.contains("embedding")));
    }

    #[test]
    fn test_parse_openrouter_filters_non_text_output() {
        // Keep a text->text model; drop an image-output model.
        let body = r#"{"data":[
            {"id":"anthropic/claude-3.5-sonnet","name":"Claude 3.5 Sonnet",
                "architecture":{"output_modalities":["text"]}},
            {"id":"black-forest-labs/flux-1.1-pro","name":"FLUX 1.1 Pro",
                "architecture":{"output_modalities":["image"],"modality":"text->image"}}]}"#;
        let models = parse_models(ModelServerKind::OpenRouter, body).unwrap();
        let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["anthropic/claude-3.5-sonnet"]);
    }

    #[test]
    fn test_parse_openrouter_keeps_models_without_modality_info() {
        // Existing shape with no `architecture` ⇒ keep (don't over-filter).
        let body = r#"{"data":[
            {"id":"anthropic/claude-3.5-sonnet","name":"Anthropic: Claude 3.5 Sonnet"},
            {"id":"meta-llama/llama-3-70b-instruct","name":"Meta: Llama 3 70B Instruct"}]}"#;
        let models = parse_models(ModelServerKind::OpenRouter, body).unwrap();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_parse_compat_and_lmstudio_passthrough() {
        // BYO/local hosts are not filtered — an "embedding"-looking id is kept,
        // guarding against the OpenAI deny-list bleeding into compat kinds.
        let body = r#"{"data":[{"id":"nomic-embed-text","object":"model"},
            {"id":"qwen2.5-coder","object":"model"}]}"#;
        for kind in [ModelServerKind::OpenAiCompat, ModelServerKind::LmStudio] {
            let models = parse_models(kind, body).unwrap();
            assert_eq!(
                models.len(),
                2,
                "kind {kind:?} should pass through unfiltered"
            );
        }
    }

    #[test]
    fn test_parse_anthropic_passthrough_unfiltered() {
        // Anthropic lists only chat models; every entry is kept as-is.
        let body = r#"{"data":[
            {"type":"model","id":"claude-opus-4","display_name":"Claude Opus 4"},
            {"type":"model","id":"claude-3-5-haiku-20241022","display_name":"Claude Haiku 3.5"}]}"#;
        let models = parse_models(ModelServerKind::AnthropicApi, body).unwrap();
        assert_eq!(models.len(), 2);
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
        // openai-compat is bring-your-own-endpoint: no default_base_url, so an
        // instance without a base_url stays NoBaseUrl (no network call).
        let server = ModelServer {
            name: "vllm-undeclared".into(),
            kind: "openai-compat".into(),
            base_url: None,
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: None,
        };
        let outcome = probe_models(&server).await;
        assert!(!outcome.reachable);
        assert!(outcome.error.is_some());
    }

    #[test]
    fn test_probe_url_uses_kind_default_when_instance_has_none() {
        // Pure URL-composition check (no network): the first-party providers
        // compose their probe URL from the kind's default base + endpoint.
        for (kind, want) in [
            (
                ModelServerKind::AnthropicApi,
                "https://api.anthropic.com/v1/models",
            ),
            (
                ModelServerKind::OpenAiApi,
                "https://api.openai.com/v1/models",
            ),
            (
                ModelServerKind::GoogleApi,
                "https://generativelanguage.googleapis.com/v1beta/models",
            ),
        ] {
            let base = kind.default_base_url().unwrap().trim_end_matches('/');
            assert_eq!(format!("{base}{}", kind.models_endpoint()), want);
        }
    }
}
