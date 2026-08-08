//! GitLab service with retry logic for MR operations.
//!
//! Wraps `GlabCli` with exponential backoff retry for transient failures.
//! Mirrors `GitHubService`'s retry/backoff shape.

use anyhow::Result;
use backon::{ExponentialBuilder, Retryable};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

use crate::api::GlabCli;
use crate::types::pr::{
    CreatePrError, CreatePrRequest, PrReviewState, PrState, PullRequestInfo, RepoInfo,
    UnifiedPrComment,
};

/// GitLab service with retry logic
pub struct GitLabService {
    /// Maximum retry attempts
    max_retries: usize,
    /// Base delay for exponential backoff
    base_delay: Duration,
    /// Maximum delay between retries
    max_delay: Duration,
}

impl Default for GitLabService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLabService {
    /// Create a new GitLab service with default retry settings
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
        }
    }

    /// Create with custom retry settings
    pub fn with_retry_config(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }

    /// Build the retry strategy
    fn retry_strategy(&self) -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_min_delay(self.base_delay)
            .with_max_delay(self.max_delay)
            .with_max_times(self.max_retries)
    }

    /// Check if an error is retryable
    fn should_retry(err: &anyhow::Error) -> bool {
        let err_str = err.to_string().to_lowercase();

        // Network/transient errors are retryable
        if err_str.contains("timeout")
            || err_str.contains("connection")
            || err_str.contains("temporary")
            || err_str.contains("rate limit")
            || err_str.contains("503")
            || err_str.contains("502")
            || err_str.contains("504")
        {
            return true;
        }

        // Auth errors are not retryable
        if err_str.contains("401")
            || err_str.contains("403")
            || err_str.contains("unauthorized")
            || err_str.contains("not logged in")
        {
            return false;
        }

        // Default: don't retry unknown errors
        false
    }

    /// Check if glab CLI is available and authenticated
    pub async fn check_available(&self) -> Result<bool> {
        if !GlabCli::is_installed().await {
            return Ok(false);
        }
        GlabCli::check_auth().await
    }

    /// Get the authenticated user
    #[instrument(skip(self))]
    pub async fn get_authenticated_user(&self) -> Result<String> {
        let op = || async { GlabCli::get_authenticated_user().await };

        op.retry(self.retry_strategy())
            .when(Self::should_retry)
            .notify(|err, dur| {
                warn!("Retrying get_authenticated_user after {:?}: {}", dur, err);
            })
            .await
    }

    /// Create an MR with retry
    #[instrument(skip(self, request))]
    pub async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError> {
        // MR creation errors are not typically retryable (validation errors, already exists, etc.)
        // So we don't use retry here
        GlabCli::create_pr(repo_info, request, cwd).await
    }

    /// Get MR info with retry
    #[instrument(skip(self))]
    pub async fn get_pr(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
        let op = || async { GlabCli::get_pr(repo_info, pr_number).await };

        op.retry(self.retry_strategy())
            .when(Self::should_retry)
            .notify(|err, dur| {
                warn!("Retrying get_pr after {:?}: {}", dur, err);
            })
            .await
    }

    /// List MRs for a branch with retry
    #[instrument(skip(self))]
    pub async fn list_prs_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        let op = || async { GlabCli::list_prs_for_branch(repo_info, branch).await };

        op.retry(self.retry_strategy())
            .when(Self::should_retry)
            .notify(|err, dur| {
                warn!("Retrying list_prs_for_branch after {:?}: {}", dur, err);
            })
            .await
    }

    /// Get all MR comments with retry
    #[instrument(skip(self))]
    pub async fn get_all_pr_comments(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        let op = || async { GlabCli::get_all_pr_comments(repo_info, pr_number).await };

        op.retry(self.retry_strategy())
            .when(Self::should_retry)
            .notify(|err, dur| {
                warn!("Retrying get_all_pr_comments after {:?}: {}", dur, err);
            })
            .await
    }

    /// Get MR review state with retry
    #[instrument(skip(self))]
    pub async fn get_pr_review_state(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PrReviewState> {
        let op = || async { GlabCli::get_pr_review_state(repo_info, pr_number).await };

        op.retry(self.retry_strategy())
            .when(Self::should_retry)
            .notify(|err, dur| {
                warn!("Retrying get_pr_review_state after {:?}: {}", dur, err);
            })
            .await
    }

    /// Check if MR is ready to merge with retry
    #[instrument(skip(self))]
    pub async fn is_pr_ready_to_merge(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<bool> {
        let op = || async { GlabCli::is_pr_ready_to_merge(repo_info, pr_number).await };

        op.retry(self.retry_strategy())
            .when(Self::should_retry)
            .notify(|err, dur| {
                warn!("Retrying is_pr_ready_to_merge after {:?}: {}", dur, err);
            })
            .await
    }

    /// Open MR in browser (no retry needed)
    pub async fn open_pr_in_browser(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<()> {
        GlabCli::open_pr_in_browser(repo_info, pr_number).await
    }

    /// Poll MR status until a condition is met or timeout
    #[instrument(skip(self, condition))]
    pub async fn poll_pr_until(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        poll_interval: Duration,
        timeout: Duration,
        mut condition: impl FnMut(&PullRequestInfo) -> bool,
    ) -> Result<PullRequestInfo> {
        let start = std::time::Instant::now();

        loop {
            let pr = self.get_pr(repo_info, pr_number).await?;

            if condition(&pr) {
                return Ok(pr);
            }

            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for MR #{pr_number} condition"
                ));
            }

            debug!(
                "MR #{} state: {:?}, waiting {:?} before next poll",
                pr_number, pr.state, poll_interval
            );
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Wait for MR to be merged
    #[instrument(skip(self))]
    pub async fn wait_for_merge(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        poll_interval: Duration,
        timeout: Duration,
    ) -> Result<PullRequestInfo> {
        info!("Waiting for MR #{} to be merged", pr_number);

        self.poll_pr_until(repo_info, pr_number, poll_interval, timeout, |pr| {
            pr.state == PrState::Merged
        })
        .await
    }

    /// Get new comments since a given time
    #[instrument(skip(self))]
    pub async fn get_comments_since(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UnifiedPrComment>> {
        let all_comments = self.get_all_pr_comments(repo_info, pr_number).await?;

        Ok(all_comments
            .into_iter()
            .filter(|c| c.created_at() > since)
            .collect())
    }

    /// Find an existing MR for a branch
    #[instrument(skip(self))]
    pub async fn find_pr_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>> {
        let prs = self.list_prs_for_branch(repo_info, branch).await?;

        // Find the first open MR, or the most recent if none are open
        let open_pr = prs.iter().find(|p| p.state == PrState::Open);
        if let Some(pr) = open_pr {
            return Ok(Some(pr.clone()));
        }

        // Return most recent (first in list)
        Ok(prs.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry_timeout() {
        let err = anyhow::anyhow!("Connection timeout");
        assert!(GitLabService::should_retry(&err));
    }

    #[test]
    fn test_should_retry_rate_limit() {
        let err = anyhow::anyhow!("Rate limit exceeded");
        assert!(GitLabService::should_retry(&err));
    }

    #[test]
    fn test_should_not_retry_auth() {
        let err = anyhow::anyhow!("401 Unauthorized");
        assert!(!GitLabService::should_retry(&err));
    }

    #[test]
    fn test_should_not_retry_forbidden() {
        let err = anyhow::anyhow!("403 Forbidden");
        assert!(!GitLabService::should_retry(&err));
    }

    #[test]
    fn test_default_config() {
        let service = GitLabService::new();
        assert_eq!(service.max_retries, 3);
    }
}
