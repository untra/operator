#![allow(dead_code)]
#![allow(unused_imports)]

//! API client modules for external service integrations
//!
//! This module provides:
//! - Provider traits for AI, Repo, and PM services
//! - Capabilities system for capability-based feature enablement
//! - Error handling with auth failure tracking

pub mod error;
pub mod providers;

// Legacy modules (kept for backward compatibility during migration)
pub mod anthropic;
pub mod github;

// Re-export commonly used types from providers
pub use error::ApiError;
pub use providers::ai::{AiProvider, AnthropicProvider, RateLimitInfo};
pub use providers::repo::{GitHubProvider, IssueStatus, PrStatus, RepoProvider};

// Legacy re-exports (for backward compatibility)
pub use anthropic::AnthropicClient;
pub use github::GitHubClient;

use std::collections::HashMap;
use std::time::Instant;

/// Capabilities system for managing available API integrations
///
/// Features are enabled based on which API tokens are configured.
/// Tracks auth failures and provides graceful degradation.
pub struct Capabilities {
    /// Active AI provider (if configured)
    ai_provider: Option<Box<dyn AiProvider>>,

    /// Active Repo provider (if configured)
    repo_provider: Option<Box<dyn RepoProvider>>,

    /// Auth failure tracking per provider (consecutive 401 count)
    auth_failures: HashMap<String, u32>,

    /// Last successful rate limit info
    pub last_rate_limit: Option<RateLimitInfo>,

    /// When rate limits were last checked
    pub last_rate_limit_check: Option<Instant>,

    /// Threshold for marking a provider as needing token refresh
    auth_failure_threshold: u32,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self::new()
    }
}

impl Capabilities {
    /// Default threshold for auth failures before warning user
    const DEFAULT_AUTH_FAILURE_THRESHOLD: u32 = 3;

    /// Create new capabilities (no providers configured)
    pub fn new() -> Self {
        Self {
            ai_provider: None,
            repo_provider: None,
            auth_failures: HashMap::new(),
            last_rate_limit: None,
            last_rate_limit_check: None,
            auth_failure_threshold: Self::DEFAULT_AUTH_FAILURE_THRESHOLD,
        }
    }

    /// Build capabilities from environment variables
    ///
    /// Checks for:
    /// - OPERATOR_ANTHROPIC_API_KEY -> Anthropic AI provider
    /// - OPERATOR_GITHUB_TOKEN -> GitHub repo provider
    pub fn from_env() -> Self {
        let mut caps = Self::new();

        // Try to configure AI provider (Anthropic)
        if let Ok(Some(provider)) = AnthropicProvider::from_env() {
            caps.ai_provider = Some(Box::new(provider));
        }

        // Try to configure Repo provider (GitHub)
        if let Ok(Some(provider)) = GitHubProvider::from_env() {
            caps.repo_provider = Some(Box::new(provider));
        }

        caps
    }

    /// Check if AI provider is available
    pub fn has_ai(&self) -> bool {
        self.ai_provider.is_some()
    }

    /// Check if Repo provider is available
    pub fn has_repo(&self) -> bool {
        self.repo_provider.is_some()
    }

    /// Get the AI provider name (if configured)
    pub fn ai_provider_name(&self) -> Option<&str> {
        self.ai_provider.as_ref().map(|p| p.name())
    }

    /// Get the Repo provider name (if configured)
    pub fn repo_provider_name(&self) -> Option<&str> {
        self.repo_provider.as_ref().map(|p| p.name())
    }

    /// Record an API error, tracking auth failures
    pub fn record_error(&mut self, err: &ApiError) {
        if let ApiError::Unauthorized { provider, .. } = err {
            let count = self.auth_failures.entry(provider.clone()).or_insert(0);
            *count += 1;
        }
    }

    /// Clear auth failures for a provider (after successful call)
    pub fn clear_auth_failures(&mut self, provider: &str) {
        self.auth_failures.remove(provider);
    }

    /// Get providers that need token refresh (persistent 401s)
    pub fn providers_needing_refresh(&self) -> Vec<&str> {
        self.auth_failures
            .iter()
            .filter(|(_, count)| **count >= self.auth_failure_threshold)
            .map(|(provider, _)| provider.as_str())
            .collect()
    }

    /// Check if a specific provider needs token refresh
    pub fn needs_refresh(&self, provider: &str) -> bool {
        self.auth_failures
            .get(provider)
            .map(|count| *count >= self.auth_failure_threshold)
            .unwrap_or(false)
    }

    /// Get the current auth failure count for a provider
    pub fn auth_failure_count(&self, provider: &str) -> u32 {
        *self.auth_failures.get(provider).unwrap_or(&0)
    }

    /// Sync rate limits from AI provider
    pub async fn sync_rate_limits(&mut self) -> Result<RateLimitInfo, ApiError> {
        // Get provider name first to avoid borrow issues
        let provider_name = self
            .ai_provider
            .as_ref()
            .map(|p| p.name().to_string())
            .ok_or_else(|| ApiError::not_configured("ai"))?;

        let provider = self.ai_provider.as_ref().unwrap();
        let result = provider.check_rate_limits().await;

        match result {
            Ok(info) => {
                self.clear_auth_failures(&provider_name);
                self.last_rate_limit = Some(info.clone());
                self.last_rate_limit_check = Some(Instant::now());
                Ok(info)
            }
            Err(e) => {
                self.record_error(&e);
                Err(e)
            }
        }
    }

    /// Get PR status from repo provider
    pub async fn get_pr_status(
        &mut self,
        repo: &str,
        pr_number: u64,
    ) -> Result<PrStatus, ApiError> {
        // Get provider name first to avoid borrow issues
        let provider_name = self
            .repo_provider
            .as_ref()
            .map(|p| p.name().to_string())
            .ok_or_else(|| ApiError::not_configured("repo"))?;

        let provider = self.repo_provider.as_ref().unwrap();
        let result = provider.get_pr_status(repo, pr_number).await;

        match result {
            Ok(status) => {
                self.clear_auth_failures(&provider_name);
                Ok(status)
            }
            Err(e) => {
                self.record_error(&e);
                Err(e)
            }
        }
    }

    /// Test AI provider connection
    pub async fn test_ai_connection(&mut self) -> Result<bool, ApiError> {
        // Get provider name first to avoid borrow issues
        let provider_name = self
            .ai_provider
            .as_ref()
            .map(|p| p.name().to_string())
            .ok_or_else(|| ApiError::not_configured("ai"))?;

        let provider = self.ai_provider.as_ref().unwrap();
        let result = provider.test_connection().await;

        match result {
            Ok(connected) => {
                if connected {
                    self.clear_auth_failures(&provider_name);
                }
                Ok(connected)
            }
            Err(e) => {
                self.record_error(&e);
                Err(e)
            }
        }
    }

    /// Test repo provider connection
    pub async fn test_repo_connection(&mut self) -> Result<bool, ApiError> {
        // Get provider name first to avoid borrow issues
        let provider_name = self
            .repo_provider
            .as_ref()
            .map(|p| p.name().to_string())
            .ok_or_else(|| ApiError::not_configured("repo"))?;

        let provider = self.repo_provider.as_ref().unwrap();
        let result = provider.test_connection().await;

        match result {
            Ok(connected) => {
                if connected {
                    self.clear_auth_failures(&provider_name);
                }
                Ok(connected)
            }
            Err(e) => {
                self.record_error(&e);
                Err(e)
            }
        }
    }

    /// Get a summary of configured capabilities
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ai) = &self.ai_provider {
            parts.push(format!("AI: {}", ai.name()));
        }

        if let Some(repo) = &self.repo_provider {
            parts.push(format!("Repo: {}", repo.name()));
        }

        if parts.is_empty() {
            "No providers configured".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Get age of last rate limit check in seconds
    pub fn rate_limit_age_secs(&self) -> Option<u64> {
        self.last_rate_limit_check.map(|t| t.elapsed().as_secs())
    }

    /// Check if rate limits are stale (older than given seconds)
    pub fn rate_limits_stale(&self, max_age_secs: u64) -> bool {
        self.rate_limit_age_secs()
            .map(|age| age > max_age_secs)
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_default() {
        let caps = Capabilities::new();
        assert!(!caps.has_ai());
        assert!(!caps.has_repo());
        assert!(caps.providers_needing_refresh().is_empty());
    }

    #[test]
    fn test_auth_failure_tracking() {
        let mut caps = Capabilities::new();

        // Record failures below threshold
        for _ in 0..2 {
            caps.record_error(&ApiError::unauthorized("anthropic"));
        }
        assert!(caps.providers_needing_refresh().is_empty());
        assert_eq!(caps.auth_failure_count("anthropic"), 2);

        // Hit threshold
        caps.record_error(&ApiError::unauthorized("anthropic"));
        assert_eq!(caps.providers_needing_refresh(), vec!["anthropic"]);
        assert!(caps.needs_refresh("anthropic"));
    }

    #[test]
    fn test_clear_auth_failures() {
        let mut caps = Capabilities::new();

        for _ in 0..5 {
            caps.record_error(&ApiError::unauthorized("github"));
        }
        assert!(caps.needs_refresh("github"));

        caps.clear_auth_failures("github");
        assert!(!caps.needs_refresh("github"));
        assert_eq!(caps.auth_failure_count("github"), 0);
    }

    #[test]
    fn test_summary() {
        let caps = Capabilities::new();
        assert_eq!(caps.summary(), "No providers configured");
    }

    #[test]
    fn test_rate_limits_stale() {
        let caps = Capabilities::new();
        // No check yet, so stale
        assert!(caps.rate_limits_stale(60));
    }
}
