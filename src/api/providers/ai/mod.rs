//! AI Provider trait and implementations
//!
//! Supports Anthropic, OpenAI, and Gemini providers for rate limit monitoring.

mod anthropic;

pub use anthropic::AnthropicProvider;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;

/// Rate limit information from an AI provider
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Provider name (e.g., "anthropic", "openai")
    pub provider: String,

    /// Maximum requests allowed per minute
    pub requests_limit: Option<u64>,
    /// Remaining requests before rate limit
    pub requests_remaining: Option<u64>,
    /// Time when request limit resets (RFC 3339)
    pub requests_reset: Option<String>,

    /// Maximum tokens allowed per minute (combined input + output)
    pub tokens_limit: Option<u64>,
    /// Remaining tokens (may be rounded)
    pub tokens_remaining: Option<u64>,
    /// Time when token limit resets (RFC 3339)
    pub tokens_reset: Option<String>,

    /// Maximum input tokens allowed per minute
    pub input_tokens_limit: Option<u64>,
    /// Remaining input tokens
    pub input_tokens_remaining: Option<u64>,
    /// Time when input token limit resets (RFC 3339)
    pub input_tokens_reset: Option<String>,

    /// Maximum output tokens allowed per minute
    pub output_tokens_limit: Option<u64>,
    /// Remaining output tokens
    pub output_tokens_remaining: Option<u64>,
    /// Time when output token limit resets (RFC 3339)
    pub output_tokens_reset: Option<String>,

    /// Whether currently rate limited
    pub is_rate_limited: bool,
    /// Retry after seconds (only present when rate limited)
    pub retry_after_secs: Option<u64>,

    /// Whether the connection was successful
    pub connected: bool,
}

impl RateLimitInfo {
    /// Create a new RateLimitInfo for a provider
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

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

    /// Get the most relevant remaining percentage (prefers input tokens)
    pub fn best_remaining_pct(&self) -> Option<f32> {
        self.input_tokens_remaining_pct()
            .or_else(|| self.tokens_remaining_pct())
    }

    /// Get a summary status string for display
    /// Returns something like "87% tokens", "45% input", or "Rate limited"
    pub fn summary(&self) -> String {
        if self.is_rate_limited {
            if let Some(retry) = self.retry_after_secs {
                return format!("Rate limited ({}s)", retry);
            }
            return "Rate limited".to_string();
        }

        // Prefer input tokens as they're typically more limiting
        if let Some(pct) = self.input_tokens_remaining_pct() {
            return format!("{:.0}% input", pct * 100.0);
        }

        if let Some(pct) = self.tokens_remaining_pct() {
            return format!("{:.0}% tokens", pct * 100.0);
        }

        "Unknown".to_string()
    }

    /// Check if rate limit is below a warning threshold (e.g., 0.2 for 20%)
    pub fn is_below_threshold(&self, threshold: f32) -> bool {
        self.best_remaining_pct()
            .map(|pct| pct < threshold)
            .unwrap_or(false)
    }

    /// Format as a progress bar string (e.g., "████████░░")
    pub fn progress_bar(&self, width: usize) -> String {
        let pct = self.best_remaining_pct().unwrap_or(1.0);
        let filled = (pct * width as f32).round() as usize;
        let empty = width.saturating_sub(filled);

        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }
}

/// Trait for AI service providers (Anthropic, OpenAI, Gemini, etc.)
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Get the provider name (e.g., "anthropic", "openai")
    fn name(&self) -> &str;

    /// Check if the provider is configured (has API token)
    fn is_configured(&self) -> bool;

    /// Check current rate limits by making a minimal API call
    async fn check_rate_limits(&self) -> Result<RateLimitInfo, ApiError>;

    /// Test connectivity to the API
    async fn test_connection(&self) -> Result<bool, ApiError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_info_summary() {
        let mut info = RateLimitInfo::new("anthropic");
        info.input_tokens_limit = Some(100000);
        info.input_tokens_remaining = Some(87000);
        assert_eq!(info.summary(), "87% input");

        let mut info = RateLimitInfo::new("openai");
        info.tokens_limit = Some(100000);
        info.tokens_remaining = Some(45000);
        assert_eq!(info.summary(), "45% tokens");

        let mut info = RateLimitInfo::new("anthropic");
        info.is_rate_limited = true;
        info.retry_after_secs = Some(30);
        assert_eq!(info.summary(), "Rate limited (30s)");
    }

    #[test]
    fn test_is_below_threshold() {
        let mut info = RateLimitInfo::new("anthropic");
        info.input_tokens_limit = Some(100000);
        info.input_tokens_remaining = Some(15000); // 15%

        assert!(info.is_below_threshold(0.2)); // Below 20%
        assert!(!info.is_below_threshold(0.1)); // Not below 10%
    }

    #[test]
    fn test_progress_bar() {
        let mut info = RateLimitInfo::new("test");
        info.tokens_limit = Some(100);
        info.tokens_remaining = Some(75);

        let bar = info.progress_bar(10);
        assert_eq!(bar, "████████░░");
    }

    #[test]
    fn test_best_remaining_pct() {
        let mut info = RateLimitInfo::new("test");

        // No data - None
        assert!(info.best_remaining_pct().is_none());

        // Only tokens
        info.tokens_limit = Some(100);
        info.tokens_remaining = Some(50);
        assert_eq!(info.best_remaining_pct(), Some(0.5));

        // Input tokens takes precedence
        info.input_tokens_limit = Some(100);
        info.input_tokens_remaining = Some(30);
        assert_eq!(info.best_remaining_pct(), Some(0.3));
    }
}
