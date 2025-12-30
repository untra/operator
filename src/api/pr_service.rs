//! Provider-agnostic PR service trait.
//!
//! Abstracts PR operations to support multiple Git providers (GitHub, GitLab, etc.)
//! while maintaining a consistent interface for the PR monitor and other consumers.

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

use crate::types::pr::{
    CreatePrError, CreatePrRequest, PrReviewState, PullRequestInfo, RepoInfo, UnifiedPrComment,
};

/// Provider-agnostic trait for PR/MR operations.
///
/// This trait abstracts the core operations needed for PR monitoring and management,
/// allowing different Git providers (GitHub, GitLab, Bitbucket, etc.) to be used
/// interchangeably.
///
/// Convenience methods like `poll_until` and `wait_for_merge` are not included
/// in this trait as they can be implemented using `get_pr` in a provider-agnostic way.
#[async_trait]
pub trait PrService: Send + Sync {
    /// Get the provider name (e.g., "github", "gitlab")
    fn provider_name(&self) -> &str;

    /// Check if the service is available and authenticated
    async fn check_available(&self) -> Result<bool>;

    /// Get the authenticated user
    async fn get_authenticated_user(&self) -> Result<String>;

    /// Get PR/MR information
    async fn get_pr(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo>;

    /// Check if PR/MR is ready to merge (approved + checks pass)
    async fn is_ready_to_merge(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<bool>;

    /// Get the review state of a PR/MR
    async fn get_review_state(&self, repo_info: &RepoInfo, pr_number: i64)
        -> Result<PrReviewState>;

    /// Create a new PR/MR
    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError>;

    /// List PRs/MRs for a branch
    async fn list_prs_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>>;

    /// Get all comments on a PR/MR
    async fn get_all_comments(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>>;

    /// Open PR/MR in browser
    async fn open_in_browser(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<()>;

    /// Get comments since a given time
    async fn get_comments_since(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UnifiedPrComment>>;

    /// Find an existing PR for a branch
    async fn find_pr_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>>;
}

/// Implement PrService for GitHubService
use crate::api::GitHubService;

#[async_trait]
impl PrService for GitHubService {
    fn provider_name(&self) -> &str {
        "github"
    }

    async fn check_available(&self) -> Result<bool> {
        GitHubService::check_available(self).await
    }

    async fn get_authenticated_user(&self) -> Result<String> {
        GitHubService::get_authenticated_user(self).await
    }

    async fn get_pr(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
        GitHubService::get_pr(self, repo_info, pr_number).await
    }

    async fn is_ready_to_merge(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<bool> {
        GitHubService::is_pr_ready_to_merge(self, repo_info, pr_number).await
    }

    async fn get_review_state(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PrReviewState> {
        GitHubService::get_pr_review_state(self, repo_info, pr_number).await
    }

    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError> {
        GitHubService::create_pr(self, repo_info, request, cwd).await
    }

    async fn list_prs_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        GitHubService::list_prs_for_branch(self, repo_info, branch).await
    }

    async fn get_all_comments(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        GitHubService::get_all_pr_comments(self, repo_info, pr_number).await
    }

    async fn open_in_browser(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<()> {
        GitHubService::open_pr_in_browser(self, repo_info, pr_number).await
    }

    async fn get_comments_since(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UnifiedPrComment>> {
        GitHubService::get_comments_since(self, repo_info, pr_number, since).await
    }

    async fn find_pr_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>> {
        GitHubService::find_pr_for_branch(self, repo_info, branch).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_service_provider_name() {
        let service = GitHubService::new();
        assert_eq!(service.provider_name(), "github");
    }
}
