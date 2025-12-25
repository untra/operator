//! # DEPRECATED: Legacy Anthropic API Client
//!
//! **Status**: Superseded by `api/providers/ai/anthropic.rs`
//!
//! This file is the original Anthropic API client, kept for reference during
//! migration to the new provider-based architecture. The new implementation
//! follows the `AiProvider` trait pattern.
//!
//! **Migration Path**:
//! - New code should use `crate::api::providers::ai::AnthropicProvider`
//! - This file can be removed once migration is verified complete
//!
//! **Original Purpose**: Rate limit monitoring for Anthropic API

#![allow(dead_code)] // DEPRECATED: Use api/providers/ai/anthropic.rs instead

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Rate limit information from Anthropic API response headers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Maximum requests allowed per minute
    pub requests_limit: Option<u64>,
    /// Remaining requests before rate limit
    pub requests_remaining: Option<u64>,
    /// Time when request limit resets (RFC 3339)
    pub requests_reset: Option<String>,

    /// Maximum tokens allowed per minute (combined)
    pub tokens_limit: Option<u64>,
    /// Remaining tokens (rounded to nearest thousand)
    pub tokens_remaining: Option<u64>,
    /// Time when token limit resets (RFC 3339)
    pub tokens_reset: Option<String>,

    /// Maximum input tokens allowed per minute
    pub input_tokens_limit: Option<u64>,
    /// Remaining input tokens (rounded to nearest thousand)
    pub input_tokens_remaining: Option<u64>,
    /// Time when input token limit resets (RFC 3339)
    pub input_tokens_reset: Option<String>,

    /// Maximum output tokens allowed per minute
    pub output_tokens_limit: Option<u64>,
    /// Remaining output tokens (rounded to nearest thousand)
    pub output_tokens_remaining: Option<u64>,
    /// Time when output token limit resets (RFC 3339)
    pub output_tokens_reset: Option<String>,

    /// Retry after seconds (only present when rate limited)
    pub retry_after_seconds: Option<u64>,

    /// Whether the API connection was successful
    pub connected: bool,
}

impl RateLimitInfo {
    /// Calculate the percentage of tokens remaining (0.0 to 1.0)
    pub fn tokens_remaining_pct(&self) -> Option<f32> {
        match (self.tokens_remaining, self.tokens_limit) {
            (Some(remaining), Some(limit)) if limit > 0 => Some(remaining as f32 / limit as f32),
            _ => None,
        }
    }

    /// Calculate the percentage of input tokens remaining (0.0 to 1.0)
    pub fn input_tokens_remaining_pct(&self) -> Option<f32> {
        match (self.input_tokens_remaining, self.input_tokens_limit) {
            (Some(remaining), Some(limit)) if limit > 0 => Some(remaining as f32 / limit as f32),
            _ => None,
        }
    }

    /// Calculate the percentage of output tokens remaining (0.0 to 1.0)
    pub fn output_tokens_remaining_pct(&self) -> Option<f32> {
        match (self.output_tokens_remaining, self.output_tokens_limit) {
            (Some(remaining), Some(limit)) if limit > 0 => Some(remaining as f32 / limit as f32),
            _ => None,
        }
    }

    /// Get a summary status string for display
    /// Returns something like "87% tokens", "45% input", or "Rate limited"
    pub fn summary(&self) -> String {
        if let Some(retry) = self.retry_after_seconds {
            return format!("Rate limited ({}s)", retry);
        }

        // Prefer input tokens as they're more limiting
        if let Some(pct) = self.input_tokens_remaining_pct() {
            return format!("{:.0}% input", pct * 100.0);
        }

        if let Some(pct) = self.tokens_remaining_pct() {
            return format!("{:.0}% tokens", pct * 100.0);
        }

        "Unknown".to_string()
    }

    /// Check if rate limit is below a warning threshold
    pub fn is_below_threshold(&self, threshold: f32) -> bool {
        self.input_tokens_remaining_pct()
            .or(self.tokens_remaining_pct())
            .map(|pct| pct < threshold)
            .unwrap_or(false)
    }
}

/// Anthropic API client for checking rate limits
pub struct AnthropicClient {
    api_key: String,
    client: reqwest::Client,
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

impl AnthropicClient {
    /// Create a new Anthropic client from the OPERATOR_ANTHROPIC_API_KEY environment variable
    pub fn from_env() -> Result<Option<Self>> {
        match env::var("OPERATOR_ANTHROPIC_API_KEY") {
            Ok(key) if !key.is_empty() => {
                let client = reqwest::Client::builder()
                    .user_agent("operator-tui/0.1.0")
                    .build()
                    .context("Failed to build HTTP client")?;
                Ok(Some(Self {
                    api_key: key,
                    client,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Check if the client is configured
    pub fn is_configured() -> bool {
        env::var("OPERATOR_ANTHROPIC_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    }

    /// Check rate limits by making a minimal API call and reading response headers
    /// This uses the smallest possible request to minimize token usage
    pub async fn check_rate_limits(&self) -> Result<RateLimitInfo> {
        let url = format!("{}/v1/messages", ANTHROPIC_API_BASE);

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
            .context("Failed to send request to Anthropic API")?;

        let headers = response.headers().clone();
        let mut rate_limit = Self::parse_rate_limit_headers(&headers);

        // Check if we got rate limited (429)
        if response.status() == 429 {
            if let Some(retry_after) = headers.get("retry-after") {
                if let Ok(seconds) = retry_after.to_str().unwrap_or("0").parse::<u64>() {
                    rate_limit.retry_after_seconds = Some(seconds);
                }
            }
        }

        rate_limit.connected = response.status().is_success() || response.status() == 429;

        Ok(rate_limit)
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

            retry_after_seconds: None,
            connected: false,
        }
    }

    /// Test connectivity by checking if we can reach the API
    pub async fn test_connection(&self) -> Result<bool> {
        match self.check_rate_limits().await {
            Ok(info) => Ok(info.connected),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_info_summary() {
        let info = RateLimitInfo {
            input_tokens_limit: Some(100000),
            input_tokens_remaining: Some(87000),
            ..Default::default()
        };
        assert_eq!(info.summary(), "87% input");

        let info = RateLimitInfo {
            tokens_limit: Some(100000),
            tokens_remaining: Some(45000),
            ..Default::default()
        };
        assert_eq!(info.summary(), "45% tokens");

        let info = RateLimitInfo {
            retry_after_seconds: Some(30),
            ..Default::default()
        };
        assert_eq!(info.summary(), "Rate limited (30s)");
    }

    #[test]
    fn test_is_below_threshold() {
        let info = RateLimitInfo {
            input_tokens_limit: Some(100000),
            input_tokens_remaining: Some(15000), // 15%
            ..Default::default()
        };
        assert!(info.is_below_threshold(0.2)); // Below 20%
        assert!(!info.is_below_threshold(0.1)); // Not below 10%
    }

    #[test]
    fn test_tokens_remaining_pct() {
        let info = RateLimitInfo {
            tokens_limit: Some(100000),
            tokens_remaining: Some(50000),
            ..Default::default()
        };
        assert_eq!(info.tokens_remaining_pct(), Some(0.5));

        let info = RateLimitInfo::default();
        assert_eq!(info.tokens_remaining_pct(), None);
    }
}
