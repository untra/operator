#![allow(dead_code)]

//! Repository Provider trait and implementations
//!
//! Supports GitHub, GitLab, and Azure Repos for PR/issue status tracking.

mod github;

pub use github::GitHubProvider;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;

/// Pull request status from a repo provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrStatus {
    /// Provider name
    pub provider: String,
    /// PR number
    pub number: u64,
    /// State: "open", "closed", "merged"
    pub state: String,
    /// Title of the PR
    pub title: String,
    /// HTML URL for the PR
    pub html_url: String,
    /// Whether the PR is a draft
    pub draft: bool,
    /// Whether the PR has been merged
    pub merged: bool,
    /// Whether the PR is mergeable (None if not yet computed)
    pub mergeable: Option<bool>,
    /// Head commit SHA
    pub head_sha: String,
    /// Review status: "approved", "changes_requested", "pending", "none"
    pub review_status: String,
    /// Whether all required checks have passed
    pub checks_passed: Option<bool>,
}

impl PrStatus {
    /// Check if PR is ready to merge (approved, checks pass, not draft)
    pub fn is_ready_to_merge(&self) -> bool {
        self.state == "open"
            && !self.draft
            && self.review_status == "approved"
            && self.checks_passed.unwrap_or(true)
    }

    /// Get a summary status string
    pub fn summary_status(&self) -> &str {
        if self.merged {
            "merged"
        } else if self.state == "closed" {
            "closed"
        } else if self.draft {
            "draft"
        } else if self.review_status == "changes_requested" {
            "changes_requested"
        } else if self.review_status == "approved" {
            "approved"
        } else {
            "pending"
        }
    }
}

/// Issue status from a repo provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatus {
    /// Provider name
    pub provider: String,
    /// Issue number
    pub number: u64,
    /// State: "open" or "closed"
    pub state: String,
    /// Title of the issue
    pub title: String,
    /// HTML URL for the issue
    pub html_url: String,
    /// Labels on the issue
    pub labels: Vec<String>,
}

/// Check run status from repo provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStatus {
    /// Check name
    pub name: String,
    /// Status: "queued", "in_progress", "completed"
    pub status: String,
    /// Conclusion: "success", "failure", "neutral", "cancelled", "skipped", "timed_out", "action_required"
    pub conclusion: Option<String>,
}

impl CheckStatus {
    /// Check if this check has passed
    pub fn is_passed(&self) -> bool {
        self.status == "completed"
            && self
                .conclusion
                .as_ref()
                .map(|c| c == "success" || c == "skipped" || c == "neutral")
                .unwrap_or(false)
    }
}

/// Trait for repository service providers (GitHub, GitLab, Azure Repos)
#[async_trait]
pub trait RepoProvider: Send + Sync {
    /// Get the provider name (e.g., "github", "gitlab")
    fn name(&self) -> &str;

    /// Check if the provider is configured (has API token)
    fn is_configured(&self) -> bool;

    /// Get PR status
    async fn get_pr_status(&self, repo: &str, pr_number: u64) -> Result<PrStatus, ApiError>;

    /// Get issue status
    async fn get_issue_status(
        &self,
        repo: &str,
        issue_number: u64,
    ) -> Result<IssueStatus, ApiError>;

    /// Get check runs for a commit
    async fn get_check_runs(&self, repo: &str, ref_sha: &str)
        -> Result<Vec<CheckStatus>, ApiError>;

    /// Test connectivity to the API
    async fn test_connection(&self) -> Result<bool, ApiError>;

    /// Parse repo string into (owner, repo) tuple
    /// Note: This is a helper method that doesn't require Self, but needs Sized bound for dyn compatibility
    fn parse_repo(repo_str: &str) -> Option<(&str, &str)>
    where
        Self: Sized,
    {
        let parts: Vec<&str> = repo_str.split('/').collect();
        if parts.len() == 2 {
            Some((parts[0], parts[1]))
        } else {
            None
        }
    }
}

/// Parse repo string into (owner, repo) tuple - standalone function for use with dyn RepoProvider
pub fn parse_repo_string(repo_str: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = repo_str.split('/').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_status_is_ready_to_merge() {
        let pr = PrStatus {
            provider: "github".to_string(),
            number: 123,
            state: "open".to_string(),
            title: "Test PR".to_string(),
            html_url: "https://github.com/test/repo/pull/123".to_string(),
            draft: false,
            merged: false,
            mergeable: Some(true),
            head_sha: "abc123".to_string(),
            review_status: "approved".to_string(),
            checks_passed: Some(true),
        };
        assert!(pr.is_ready_to_merge());

        // Draft PR not ready
        let mut draft_pr = pr.clone();
        draft_pr.draft = true;
        assert!(!draft_pr.is_ready_to_merge());

        // Changes requested not ready
        let mut changes_pr = pr.clone();
        changes_pr.review_status = "changes_requested".to_string();
        assert!(!changes_pr.is_ready_to_merge());
    }

    #[test]
    fn test_pr_status_summary() {
        let mut pr = PrStatus {
            provider: "github".to_string(),
            number: 123,
            state: "open".to_string(),
            title: "Test".to_string(),
            html_url: "".to_string(),
            draft: false,
            merged: false,
            mergeable: None,
            head_sha: "".to_string(),
            review_status: "pending".to_string(),
            checks_passed: None,
        };

        assert_eq!(pr.summary_status(), "pending");

        pr.review_status = "approved".to_string();
        assert_eq!(pr.summary_status(), "approved");

        pr.merged = true;
        assert_eq!(pr.summary_status(), "merged");
    }

    #[test]
    fn test_check_status_is_passed() {
        let check = CheckStatus {
            name: "CI".to_string(),
            status: "completed".to_string(),
            conclusion: Some("success".to_string()),
        };
        assert!(check.is_passed());

        let check = CheckStatus {
            name: "CI".to_string(),
            status: "completed".to_string(),
            conclusion: Some("failure".to_string()),
        };
        assert!(!check.is_passed());

        let check = CheckStatus {
            name: "CI".to_string(),
            status: "in_progress".to_string(),
            conclusion: None,
        };
        assert!(!check.is_passed());
    }
}
