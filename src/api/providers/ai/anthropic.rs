//! Anthropic API provider implementation

use async_trait::async_trait;
use serde::Serialize;
use std::env;

use super::{AiProvider, RateLimitInfo};
use crate::api::error::ApiError;

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const PROVIDER_NAME: &str = "anthropic";

/// Anthropic API provider for rate limit monitoring
pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

/// Minimal request body for messages API
#[derive(Serialize)]
struct MinimalMessageRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given API key
    pub fn new(api_key: impl Into<String>) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .user_agent("operator-tui/0.1.0")
            .build()
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        Ok(Self {
            api_key: api_key.into(),
            client,
            base_url: ANTHROPIC_API_BASE.to_string(),
        })
    }

    /// Create provider from OPERATOR_ANTHROPIC_API_KEY environment variable
    pub fn from_env() -> Result<Option<Self>, ApiError> {
        match env::var("OPERATOR_ANTHROPIC_API_KEY") {
            Ok(key) if !key.is_empty() => Ok(Some(Self::new(key)?)),
            _ => Ok(None),
        }
    }

    /// Create provider with a custom base URL (for testing)
    #[cfg(test)]
    pub fn new_with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let mut provider = Self::new(api_key)?;
        provider.base_url = base_url.into();
        Ok(provider)
    }

    /// Check if the provider is configured (env var is set)
    pub fn is_env_configured() -> bool {
        env::var("OPERATOR_ANTHROPIC_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    }

    /// Parse rate limit headers from an API response
    fn parse_rate_limit_headers(headers: &reqwest::header::HeaderMap) -> RateLimitInfo {
        let get_u64 = |name: &str| -> Option<u64> {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
        };

        let get_string = |name: &str| -> Option<String> {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        };

        RateLimitInfo {
            provider: PROVIDER_NAME.to_string(),

            requests_limit: get_u64("anthropic-ratelimit-requests-limit"),
            requests_remaining: get_u64("anthropic-ratelimit-requests-remaining"),
            requests_reset: get_string("anthropic-ratelimit-requests-reset"),

            tokens_limit: get_u64("anthropic-ratelimit-tokens-limit"),
            tokens_remaining: get_u64("anthropic-ratelimit-tokens-remaining"),
            tokens_reset: get_string("anthropic-ratelimit-tokens-reset"),

            input_tokens_limit: get_u64("anthropic-ratelimit-input-tokens-limit"),
            input_tokens_remaining: get_u64("anthropic-ratelimit-input-tokens-remaining"),
            input_tokens_reset: get_string("anthropic-ratelimit-input-tokens-reset"),

            output_tokens_limit: get_u64("anthropic-ratelimit-output-tokens-limit"),
            output_tokens_remaining: get_u64("anthropic-ratelimit-output-tokens-remaining"),
            output_tokens_reset: get_string("anthropic-ratelimit-output-tokens-reset"),

            is_rate_limited: false,
            retry_after_secs: None,
            connected: false,
        }
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn check_rate_limits(&self) -> Result<RateLimitInfo, ApiError> {
        let url = format!("{}/v1/messages", self.base_url);

        let request_body = MinimalMessageRequest {
            model: "claude-haiku-4-20250514".to_string(),
            max_tokens: 1,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        let headers = response.headers().clone();
        let mut rate_limit = Self::parse_rate_limit_headers(&headers);

        // Handle different status codes
        match status.as_u16() {
            200..=299 => {
                rate_limit.connected = true;
            }
            401 => {
                return Err(ApiError::unauthorized(PROVIDER_NAME));
            }
            403 => {
                return Err(ApiError::forbidden(PROVIDER_NAME));
            }
            429 => {
                rate_limit.is_rate_limited = true;
                rate_limit.connected = true;

                if let Some(retry_after) = headers.get("retry-after") {
                    if let Ok(seconds) = retry_after.to_str().unwrap_or("0").parse::<u64>() {
                        rate_limit.retry_after_secs = Some(seconds);
                    }
                }
            }
            status => {
                let body = response.text().await.unwrap_or_default();
                return Err(ApiError::http(PROVIDER_NAME, status, body));
            }
        }

        Ok(rate_limit)
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        match self.check_rate_limits().await {
            Ok(info) => Ok(info.connected),
            Err(ApiError::RateLimited { .. }) => Ok(true), // Rate limited means we connected
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_configured() {
        // This test depends on env state, so we just verify it doesn't panic
        let _ = AnthropicProvider::is_env_configured();
    }

    #[test]
    fn test_provider_name() {
        let provider = AnthropicProvider::new("test-key").unwrap();
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_is_configured() {
        let provider = AnthropicProvider::new("test-key").unwrap();
        assert!(provider.is_configured());

        let provider = AnthropicProvider::new("").unwrap();
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_parse_rate_limit_headers() {
        use reqwest::header::{HeaderMap, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(
            "anthropic-ratelimit-tokens-limit",
            HeaderValue::from_static("100000"),
        );
        headers.insert(
            "anthropic-ratelimit-tokens-remaining",
            HeaderValue::from_static("75000"),
        );
        headers.insert(
            "anthropic-ratelimit-input-tokens-limit",
            HeaderValue::from_static("50000"),
        );
        headers.insert(
            "anthropic-ratelimit-input-tokens-remaining",
            HeaderValue::from_static("40000"),
        );

        let info = AnthropicProvider::parse_rate_limit_headers(&headers);

        assert_eq!(info.provider, "anthropic");
        assert_eq!(info.tokens_limit, Some(100000));
        assert_eq!(info.tokens_remaining, Some(75000));
        assert_eq!(info.input_tokens_limit, Some(50000));
        assert_eq!(info.input_tokens_remaining, Some(40000));
    }
}
