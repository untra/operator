//! PR Workflow Handler - Manages PR creation and lifecycle.
//!
//! Follows vibe-kanban patterns:
//! - Push branch to remote
//! - Create PR via gh CLI
//! - Open PR in browser
//! - Track PR for merge detection
//! - Cleanup on merge

use anyhow::{Context, Result};
use std::path::Path;
use tracing::{info, instrument, warn};

use crate::api::GitHubService;
use crate::git::GitCli;
use crate::services::PrMonitorService;
use crate::types::pr::{CreatePrError, CreatePrRequest, GitHubRepoInfo, PrState, PullRequestInfo};

/// Handles the PR workflow for a step
pub struct PrWorkflow {
    github: GitHubService,
}

impl Default for PrWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

impl PrWorkflow {
    /// Create a new PR workflow handler
    pub fn new() -> Self {
        Self {
            github: GitHubService::new(),
        }
    }

    /// Get repo info from a worktree path
    #[instrument(skip(self))]
    pub async fn get_repo_info(&self, worktree_path: &Path) -> Result<GitHubRepoInfo> {
        let remote_url = GitCli::get_remote_url(worktree_path)
            .await
            .context("Failed to get remote URL")?;

        GitHubRepoInfo::from_remote_url(&remote_url)
            .map_err(|e| anyhow::anyhow!("Failed to parse GitHub URL: {}", e))
    }

    /// Push branch to remote
    #[instrument(skip(self))]
    pub async fn push_branch(
        &self,
        worktree_path: &Path,
        branch: &str,
        set_upstream: bool,
    ) -> Result<()> {
        info!("Pushing branch {} to remote", branch);
        GitCli::push(worktree_path, "origin", branch, set_upstream).await
    }

    /// Create a PR for the current branch
    #[instrument(skip(self))]
    pub async fn create_pr(
        &self,
        worktree_path: &Path,
        title: &str,
        body: Option<String>,
        base_branch: &str,
        draft: bool,
    ) -> Result<PullRequestInfo, CreatePrError> {
        let repo_info =
            self.get_repo_info(worktree_path)
                .await
                .map_err(|e| CreatePrError::GithubApiError {
                    message: e.to_string(),
                })?;

        let current_branch = GitCli::current_branch(worktree_path).await.map_err(|e| {
            CreatePrError::GithubApiError {
                message: format!("Failed to get current branch: {}", e),
            }
        })?;

        // Push branch first
        if let Err(_e) = self.push_branch(worktree_path, &current_branch, true).await {
            return Err(CreatePrError::BranchNotPushed {
                branch: current_branch,
            });
        }

        let request = CreatePrRequest {
            title: title.to_string(),
            body,
            head_branch: current_branch,
            base_branch: base_branch.to_string(),
            draft: Some(draft),
        };

        let pr = self
            .github
            .create_pr(&repo_info, &request, worktree_path)
            .await?;

        info!("Created PR #{}: {}", pr.number, pr.url);

        // Open in browser
        if let Err(e) = self.github.open_pr_in_browser(&repo_info, pr.number).await {
            warn!("Failed to open PR in browser: {}", e);
        }

        Ok(pr)
    }

    /// Find an existing PR for the current branch
    #[instrument(skip(self))]
    pub async fn find_existing_pr(&self, worktree_path: &Path) -> Result<Option<PullRequestInfo>> {
        let repo_info = self.get_repo_info(worktree_path).await?;
        let current_branch = GitCli::current_branch(worktree_path).await?;

        self.github
            .find_pr_for_branch(&repo_info, &current_branch)
            .await
    }

    /// Create or attach to existing PR
    #[instrument(skip(self))]
    pub async fn create_or_attach_pr(
        &self,
        worktree_path: &Path,
        title: &str,
        body: Option<String>,
        base_branch: &str,
        draft: bool,
    ) -> Result<PullRequestInfo> {
        // Check for existing PR first
        if let Ok(Some(existing)) = self.find_existing_pr(worktree_path).await {
            if existing.state == PrState::Open {
                info!("Found existing open PR #{}", existing.number);
                return Ok(existing);
            }
        }

        // Create new PR
        self.create_pr(worktree_path, title, body, base_branch, draft)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create PR: {:?}", e))
    }

    /// Start tracking a PR for merge detection
    #[instrument(skip(self, monitor))]
    pub async fn start_tracking(
        &self,
        monitor: &PrMonitorService,
        worktree_path: &Path,
        pr_number: i64,
        ticket_id: &str,
    ) -> Result<()> {
        let repo_info = self.get_repo_info(worktree_path).await?;
        monitor
            .track_pr(repo_info, pr_number, ticket_id.to_string())
            .await
    }

    /// Stop tracking a PR
    #[instrument(skip(self, monitor))]
    pub async fn stop_tracking(
        &self,
        monitor: &PrMonitorService,
        worktree_path: &Path,
        pr_number: i64,
    ) {
        if let Ok(repo_info) = self.get_repo_info(worktree_path).await {
            monitor.untrack_pr(&repo_info, pr_number).await;
        }
    }

    /// Get PR status
    #[instrument(skip(self))]
    pub async fn get_pr_status(
        &self,
        worktree_path: &Path,
        pr_number: i64,
    ) -> Result<PullRequestInfo> {
        let repo_info = self.get_repo_info(worktree_path).await?;
        self.github.get_pr(&repo_info, pr_number).await
    }

    /// Check if PR is ready to merge
    #[instrument(skip(self))]
    pub async fn is_ready_to_merge(&self, worktree_path: &Path, pr_number: i64) -> Result<bool> {
        let repo_info = self.get_repo_info(worktree_path).await?;
        self.github
            .is_pr_ready_to_merge(&repo_info, pr_number)
            .await
    }

    /// Get new comments since last check
    #[instrument(skip(self))]
    pub async fn get_new_comments(
        &self,
        worktree_path: &Path,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<crate::types::pr::UnifiedPrComment>> {
        let repo_info = self.get_repo_info(worktree_path).await?;
        self.github
            .get_comments_since(&repo_info, pr_number, since)
            .await
    }

    /// Full PR creation flow:
    /// 1. Push branch
    /// 2. Create PR (or attach to existing)
    /// 3. Start tracking
    /// 4. Open in browser
    #[instrument(skip(self, monitor))]
    #[allow(clippy::too_many_arguments)]
    pub async fn run_pr_flow(
        &self,
        worktree_path: &Path,
        title: &str,
        body: Option<String>,
        base_branch: &str,
        ticket_id: &str,
        monitor: &PrMonitorService,
        draft: bool,
    ) -> Result<PullRequestInfo> {
        info!("Running PR flow for ticket {}", ticket_id);

        // Create or attach to PR
        let pr = self
            .create_or_attach_pr(worktree_path, title, body, base_branch, draft)
            .await?;

        // Start tracking for merge detection
        self.start_tracking(monitor, worktree_path, pr.number, ticket_id)
            .await?;

        info!("PR flow complete: {} (PR #{})", pr.url, pr.number);
        Ok(pr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workflow() {
        let _workflow = PrWorkflow::new();
    }
}
