//! `ModelServer` → Rig completion model. Every client is built on the
//! egress-checked `reqwest::Client`, so a model call can't reach a destination (or redirect) the policy forbids.

use std::time::Duration;

use rig_core::client::CompletionClient;
use rig_core::completion::{CompletionError, CompletionModel, CompletionResponse, ToolDefinition};
use rig_core::message::ToolChoice;
use rig_core::providers::{anthropic, gemini, ollama, openai, openrouter, xai};

use crate::api::providers::model_server::{resolve_api_key, ModelServerKind};
use crate::auth::egress::{self, EgressPolicy};
use crate::config::ModelServer;
use crate::llm::native::NativeLlmError;

type AnthropicModel = <anthropic::Client as CompletionClient>::CompletionModel;
type OpenAiModel = <openai::CompletionsClient as CompletionClient>::CompletionModel;
type GeminiModel = <gemini::Client as CompletionClient>::CompletionModel;
type XaiModel = <xai::Client as CompletionClient>::CompletionModel;
type OllamaModel = <ollama::Client as CompletionClient>::CompletionModel;
type OpenRouterModel = <openrouter::Client as CompletionClient>::CompletionModel;

/// One completion model per provider protocol. OpenAI-protocol kinds (`openai-api`, `openai-compat`, `lmstudio`)
/// all use Chat Completions, the endpoint every compatible server implements.
#[derive(Clone)]
pub(super) enum JudgeModel {
    Anthropic(AnthropicModel),
    OpenAi(OpenAiModel),
    Gemini(GeminiModel),
    Xai(XaiModel),
    Ollama(OllamaModel),
    OpenRouter(OpenRouterModel),
}

impl JudgeModel {
    /// One tool-forced completion: `tool` is the only tool and must be called.
    pub(super) async fn send_forced_tool(
        &self,
        prompt: String,
        preamble: &str,
        tool: ToolDefinition,
        max_tokens: u64,
    ) -> Result<CompletionResponse, CompletionError> {
        match self {
            Self::Anthropic(m) => send_forced_tool(m, prompt, preamble, tool, max_tokens).await,
            Self::OpenAi(m) => send_forced_tool(m, prompt, preamble, tool, max_tokens).await,
            Self::Gemini(m) => send_forced_tool(m, prompt, preamble, tool, max_tokens).await,
            Self::Xai(m) => send_forced_tool(m, prompt, preamble, tool, max_tokens).await,
            Self::Ollama(m) => send_forced_tool(m, prompt, preamble, tool, max_tokens).await,
            Self::OpenRouter(m) => send_forced_tool(m, prompt, preamble, tool, max_tokens).await,
        }
    }
}

async fn send_forced_tool<M: CompletionModel + Clone>(
    model: &M,
    prompt: String,
    preamble: &str,
    tool: ToolDefinition,
    max_tokens: u64,
) -> Result<CompletionResponse, CompletionError> {
    model
        .completion_request(prompt)
        .preamble(preamble.to_string())
        .tool(tool)
        .tool_choice(ToolChoice::Required)
        .max_tokens(max_tokens)
        .send()
        .await
}

pub(super) fn build(
    server: &ModelServer,
    model: &str,
    policy: &EgressPolicy,
    timeout: Duration,
) -> Result<JudgeModel, NativeLlmError> {
    let kind = ModelServerKind::from_slug(&server.kind).ok_or_else(|| {
        NativeLlmError::Config(format!("unknown model server kind '{}'", server.kind))
    })?;
    let base = server
        .base_url
        .as_deref()
        .filter(|u| !u.is_empty())
        .or_else(|| kind.default_base_url())
        .ok_or_else(|| {
            NativeLlmError::Config(format!("model server '{}' has no base_url", server.name))
        })?
        .trim_end_matches('/')
        .to_string();
    egress::validate(&base, policy).map_err(|e| NativeLlmError::Egress(e.to_string()))?;

    let key_required = server.api_key_env.is_some() || kind.default_api_key_env().is_some();
    let key = match resolve_api_key(server, kind) {
        Some(key) => key,
        None if key_required => {
            let var = server
                .api_key_env
                .as_deref()
                .or_else(|| kind.default_api_key_env())
                .unwrap_or_default();
            return Err(NativeLlmError::Config(format!(
                "API key env var {var} is not set in the operator process"
            )));
        }
        None => String::new(),
    };

    let http = egress::validated_client(policy.clone(), timeout)
        .map_err(|e| NativeLlmError::Config(e.to_string()))?;
    let built = |e: rig_core::http_client::Error| NativeLlmError::Config(e.to_string());

    Ok(match kind {
        ModelServerKind::AnthropicApi => JudgeModel::Anthropic(
            anthropic::Client::builder()
                .api_key(key)
                .base_url(&base)
                .http_client(http)
                .build()
                .map_err(built)?
                .completion_model(model),
        ),
        ModelServerKind::OpenAiApi | ModelServerKind::OpenAiCompat | ModelServerKind::LmStudio => {
            JudgeModel::OpenAi(
                openai::CompletionsClient::builder()
                    .api_key(key)
                    .base_url(openai_v1_base(&base))
                    .http_client(http)
                    .build()
                    .map_err(built)?
                    .completion_model(model),
            )
        }
        ModelServerKind::GoogleApi => JudgeModel::Gemini(
            gemini::Client::builder()
                .api_key(key)
                .base_url(&base)
                .http_client(http)
                .build()
                .map_err(built)?
                .completion_model(model),
        ),
        ModelServerKind::XaiApi => JudgeModel::Xai(
            xai::Client::builder()
                .api_key(key)
                .base_url(&base)
                .http_client(http)
                .build()
                .map_err(built)?
                .completion_model(model),
        ),
        ModelServerKind::Ollama => JudgeModel::Ollama(
            ollama::Client::builder()
                .api_key(key)
                .base_url(&base)
                .http_client(http)
                .build()
                .map_err(built)?
                .completion_model(model),
        ),
        ModelServerKind::OpenRouter => JudgeModel::OpenRouter(
            openrouter::Client::builder()
                .api_key(key)
                .base_url(&base)
                .http_client(http)
                .build()
                .map_err(built)?
                .completion_model(model),
        ),
    })
}

/// Operator stores OpenAI-protocol bases at the host root (the probe appends
/// `/v1/models`); Rig's OpenAI client expects the `/v1` prefix in the base.
fn openai_v1_base(base: &str) -> String {
    if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_base_gains_v1_once() {
        assert_eq!(
            openai_v1_base("https://api.openai.com"),
            "https://api.openai.com/v1"
        );
        assert_eq!(openai_v1_base("http://h:1234/v1"), "http://h:1234/v1");
    }
}
