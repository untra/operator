//! Provider-agnostic PR service trait.
//!
//! Abstracts PR operations to support multiple Git providers (GitHub, GitLab, etc.)
//! while maintaining a consistent interface for the PR monitor and other consumers.

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use crate::types::pr::{
    CreatePrError, CreatePrRequest, GitProvider, PrReviewState, PullRequestInfo, RepoInfo,
    UnifiedPrComment,
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

/// Implement `PrService` for `GitHubService`
use crate::api::GitHubService;

#[async_trait]
impl PrService for GitHubService {
    fn provider_name(&self) -> &'static str {
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

/// Implement `PrService` for `GitLabService`
use crate::api::GitLabService;

#[async_trait]
impl PrService for GitLabService {
    fn provider_name(&self) -> &'static str {
        "gitlab"
    }

    async fn check_available(&self) -> Result<bool> {
        GitLabService::check_available(self).await
    }

    async fn get_authenticated_user(&self) -> Result<String> {
        GitLabService::get_authenticated_user(self).await
    }

    async fn get_pr(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
        GitLabService::get_pr(self, repo_info, pr_number).await
    }

    async fn is_ready_to_merge(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<bool> {
        GitLabService::is_pr_ready_to_merge(self, repo_info, pr_number).await
    }

    async fn get_review_state(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PrReviewState> {
        GitLabService::get_pr_review_state(self, repo_info, pr_number).await
    }

    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError> {
        GitLabService::create_pr(self, repo_info, request, cwd).await
    }

    async fn list_prs_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        GitLabService::list_prs_for_branch(self, repo_info, branch).await
    }

    async fn get_all_comments(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        GitLabService::get_all_pr_comments(self, repo_info, pr_number).await
    }

    async fn open_in_browser(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<()> {
        GitLabService::open_pr_in_browser(self, repo_info, pr_number).await
    }

    async fn get_comments_since(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UnifiedPrComment>> {
        GitLabService::get_comments_since(self, repo_info, pr_number, since).await
    }

    async fn find_pr_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>> {
        GitLabService::find_pr_for_branch(self, repo_info, branch).await
    }
}

/// Error returned when a provider has no operational `PrService` yet.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{provider} is detect-only; operations not yet supported")]
pub struct UnsupportedProviderError {
    provider: GitProvider,
}

/// Build the `PrService` for a given provider.
///
/// GitHub and GitLab are operational; Bitbucket, Azure DevOps, Forgejo, and
/// Gitea are detect-only today (see `GitProvider::ALL`) and return
/// `UnsupportedProviderError` until their CLI stacks are implemented.
pub fn pr_service_for(
    provider: GitProvider,
) -> Result<Arc<dyn PrService>, UnsupportedProviderError> {
    match provider {
        GitProvider::GitHub => Ok(Arc::new(GitHubService::new())),
        GitProvider::GitLab => Ok(Arc::new(GitLabService::new())),
        GitProvider::Bitbucket
        | GitProvider::AzureDevOps
        | GitProvider::Forgejo
        | GitProvider::Gitea => Err(UnsupportedProviderError { provider }),
    }
}

/// A provider -> `PrService` resolver, matching `pr_service_for`'s signature.
/// Boxed so tests can inject a resolver backed by mocks instead of real CLIs.
type Resolver =
    Box<dyn Fn(GitProvider) -> Result<Arc<dyn PrService>, UnsupportedProviderError> + Send + Sync>;

/// Provider-routing `PrService`: dispatches each call to the operational
/// service for `repo_info.provider`.
///
/// `provider_name()`, `check_available()`, and `get_authenticated_user()`
/// take no `RepoInfo`, so there's no per-call provider to route on. They
/// fall back to GitHub (the pre-router default) — `provider_name()` reports
/// `"auto"` so callers can tell it's the router rather than a concrete
/// provider.
pub struct PrServiceRouter {
    resolve: Resolver,
}

impl Default for PrServiceRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl PrServiceRouter {
    /// Create a new provider-routing PR service, backed by `pr_service_for`
    pub fn new() -> Self {
        Self {
            resolve: Box::new(pr_service_for),
        }
    }

    /// Create a router backed by a custom resolver (tests inject mocks here)
    #[cfg(test)]
    fn with_resolver(
        resolve: impl Fn(GitProvider) -> Result<Arc<dyn PrService>, UnsupportedProviderError>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            resolve: Box::new(resolve),
        }
    }
}

#[async_trait]
impl PrService for PrServiceRouter {
    fn provider_name(&self) -> &'static str {
        "auto"
    }

    async fn check_available(&self) -> Result<bool> {
        GitHubService::new().check_available().await
    }

    async fn get_authenticated_user(&self) -> Result<String> {
        GitHubService::new().get_authenticated_user().await
    }

    async fn get_pr(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
        (self.resolve)(repo_info.provider)?
            .get_pr(repo_info, pr_number)
            .await
    }

    async fn is_ready_to_merge(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<bool> {
        (self.resolve)(repo_info.provider)?
            .is_ready_to_merge(repo_info, pr_number)
            .await
    }

    async fn get_review_state(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PrReviewState> {
        (self.resolve)(repo_info.provider)?
            .get_review_state(repo_info, pr_number)
            .await
    }

    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError> {
        let service =
            (self.resolve)(repo_info.provider).map_err(|e| CreatePrError::ProviderApiError {
                message: e.to_string(),
            })?;
        service.create_pr(repo_info, request, cwd).await
    }

    async fn list_prs_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        (self.resolve)(repo_info.provider)?
            .list_prs_for_branch(repo_info, branch)
            .await
    }

    async fn get_all_comments(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        (self.resolve)(repo_info.provider)?
            .get_all_comments(repo_info, pr_number)
            .await
    }

    async fn open_in_browser(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<()> {
        (self.resolve)(repo_info.provider)?
            .open_in_browser(repo_info, pr_number)
            .await
    }

    async fn get_comments_since(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UnifiedPrComment>> {
        (self.resolve)(repo_info.provider)?
            .get_comments_since(repo_info, pr_number, since)
            .await
    }

    async fn find_pr_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>> {
        (self.resolve)(repo_info.provider)?
            .find_pr_for_branch(repo_info, branch)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::pr::PrState;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_github_service_provider_name() {
        let service = GitHubService::new();
        assert_eq!(service.provider_name(), "github");
    }

    #[test]
    fn test_gitlab_service_provider_name() {
        let service = GitLabService::new();
        assert_eq!(service.provider_name(), "gitlab");
    }

    #[test]
    fn test_pr_service_for_github() {
        let service = pr_service_for(GitProvider::GitHub).unwrap();
        assert_eq!(service.provider_name(), "github");
    }

    #[test]
    fn test_pr_service_for_gitlab() {
        let service = pr_service_for(GitProvider::GitLab).unwrap();
        assert_eq!(service.provider_name(), "gitlab");
    }

    #[test]
    fn test_pr_service_for_unsupported_provider_errors() {
        let result = pr_service_for(GitProvider::Bitbucket);
        let message = match result {
            Ok(_) => panic!("expected UnsupportedProviderError for Bitbucket"),
            Err(e) => e.to_string(),
        };
        assert!(message.contains("bitbucket"));
        assert!(message.contains("detect-only"));
    }

    #[test]
    fn test_pr_service_for_all_unsupported_providers() {
        for provider in [
            GitProvider::Bitbucket,
            GitProvider::AzureDevOps,
            GitProvider::Forgejo,
            GitProvider::Gitea,
        ] {
            assert!(pr_service_for(provider).is_err());
        }
    }

    #[test]
    fn test_router_provider_name_is_auto() {
        let router = PrServiceRouter::new();
        assert_eq!(router.provider_name(), "auto");
    }

    /// Mock `PrService` that records which provider it was dispatched to.
    struct MockPrService {
        provider: &'static str,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl PrService for MockPrService {
        fn provider_name(&self) -> &str {
            self.provider
        }

        async fn check_available(&self) -> Result<bool> {
            Ok(true)
        }

        async fn get_authenticated_user(&self) -> Result<String> {
            Ok("mock-user".to_string())
        }

        async fn get_pr(&self, _repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(PullRequestInfo {
                number: pr_number,
                url: format!("https://example.com/{}/pr/{pr_number}", self.provider),
                state: PrState::Open,
                merge_commit_sha: None,
                title: None,
                is_draft: false,
            })
        }

        async fn is_ready_to_merge(&self, _repo_info: &RepoInfo, _pr_number: i64) -> Result<bool> {
            Ok(false)
        }

        async fn get_review_state(
            &self,
            _repo_info: &RepoInfo,
            _pr_number: i64,
        ) -> Result<PrReviewState> {
            Ok(PrReviewState::Pending)
        }

        async fn create_pr(
            &self,
            _repo_info: &RepoInfo,
            _request: &CreatePrRequest,
            _cwd: &Path,
        ) -> Result<PullRequestInfo, CreatePrError> {
            Err(CreatePrError::ProviderApiError {
                message: "mock".to_string(),
            })
        }

        async fn list_prs_for_branch(
            &self,
            _repo_info: &RepoInfo,
            _branch: &str,
        ) -> Result<Vec<PullRequestInfo>> {
            Ok(vec![])
        }

        async fn get_all_comments(
            &self,
            _repo_info: &RepoInfo,
            _pr_number: i64,
        ) -> Result<Vec<UnifiedPrComment>> {
            Ok(vec![])
        }

        async fn open_in_browser(&self, _repo_info: &RepoInfo, _pr_number: i64) -> Result<()> {
            Ok(())
        }

        async fn get_comments_since(
            &self,
            _repo_info: &RepoInfo,
            _pr_number: i64,
            _since: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<UnifiedPrComment>> {
            Ok(vec![])
        }

        async fn find_pr_for_branch(
            &self,
            _repo_info: &RepoInfo,
            _branch: &str,
        ) -> Result<Option<PullRequestInfo>> {
            Ok(None)
        }
    }

    /// Router backed by a resolver over two mocks, one per provider.
    fn router_with_mocks(calls: Arc<AtomicUsize>) -> PrServiceRouter {
        let github: Arc<dyn PrService> = Arc::new(MockPrService {
            provider: "github",
            calls: calls.clone(),
        });
        let gitlab: Arc<dyn PrService> = Arc::new(MockPrService {
            provider: "gitlab",
            calls,
        });

        PrServiceRouter::with_resolver(move |provider| match provider {
            GitProvider::GitHub => Ok(github.clone()),
            GitProvider::GitLab => Ok(gitlab.clone()),
            other => Err(UnsupportedProviderError { provider: other }),
        })
    }

    #[tokio::test]
    async fn test_router_dispatches_to_github_mock() {
        let calls = Arc::new(AtomicUsize::new(0));
        let router = router_with_mocks(calls.clone());
        let repo = RepoInfo::new(GitProvider::GitHub, "owner", "repo");

        let pr = router.get_pr(&repo, 1).await.unwrap();

        assert_eq!(pr.url, "https://example.com/github/pr/1");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_router_dispatches_to_gitlab_mock() {
        let calls = Arc::new(AtomicUsize::new(0));
        let router = router_with_mocks(calls.clone());
        let repo = RepoInfo::new(GitProvider::GitLab, "owner", "repo");

        let pr = router.get_pr(&repo, 2).await.unwrap();

        assert_eq!(pr.url, "https://example.com/gitlab/pr/2");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_router_errors_for_unresolved_provider() {
        let calls = Arc::new(AtomicUsize::new(0));
        let router = router_with_mocks(calls);
        let repo = RepoInfo::new(GitProvider::Bitbucket, "owner", "repo");

        let result = router.get_pr(&repo, 1).await;

        assert!(result.is_err());
    }
}
