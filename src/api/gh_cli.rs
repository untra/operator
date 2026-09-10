//! GitHub CLI (`gh`) wrapper for PR operations.
//!
//! Uses the `gh` CLI (<https://cli.github.com>) for GitHub operations.
//! This approach, following vibe-kanban patterns, provides:
//! - Built-in authentication management (gh auth login)
//! - Simpler PR creation with automatic remote detection
//! - Access to PR comments and reviews via gh api
//!
//! The gh CLI handles authentication via `gh auth login` and stores
//! credentials securely, avoiding the need for manual token management.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, instrument, warn};

use crate::api::argv::ProviderCommand;
use crate::api::cli_detection::binary_for;
use crate::types::pr::GitProvider;
use crate::types::pr::{
    CreatePrError, CreatePrRequest, GitHubRepoInfo, PrReviewState, PrState, PullRequestInfo,
    UnifiedPrComment,
};

/// GitHub CLI wrapper for PR operations
pub struct GhCli;

impl GhCli {
    /// Execute a gh command and return stdout
    async fn run_gh(args: &[&str], cwd: Option<&Path>) -> Result<String> {
        debug!(?args, "Running gh command");

        let mut cmd = Command::new(binary_for(GitProvider::GitHub));
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let _git_runtime = crate::git::runtime::configure_command(&mut cmd)?;

        let output = cmd.output().await.context("Failed to execute gh command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "gh {} failed: {}",
                args.first().unwrap_or(&""),
                stderr.trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if gh CLI is installed
    pub async fn is_installed() -> bool {
        Command::new(binary_for(GitProvider::GitHub))
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if gh CLI is authenticated
    #[instrument]
    pub async fn check_auth() -> Result<bool> {
        let result = Self::run_gh(&["auth", "status"], None).await;
        Ok(result.is_ok())
    }

    /// Get the authenticated user
    pub async fn get_authenticated_user() -> Result<String> {
        Self::run_gh(&["api", "user", "--jq", ".login"], None).await
    }

    /// Build the `gh pr create` invocation. Pure, so the flags are asserted
    /// without spawning `gh`.
    ///
    /// Note there is no `--json`: `gh pr create` does not accept it and prints
    /// the new PR's URL on stdout instead. The number is read back from that
    /// URL and hydrated via `get_pr`, matching the GitLab path.
    pub fn create_pr_argv(
        repo_info: &GitHubRepoInfo,
        request: &CreatePrRequest,
    ) -> ProviderCommand {
        let mut args = vec![
            "pr".to_string(),
            "create".to_string(),
            "--repo".to_string(),
            repo_info.full_name(),
            "--head".to_string(),
            request.head_branch.clone(),
            "--base".to_string(),
            request.base_branch.clone(),
            "--title".to_string(),
            request.title.clone(),
        ];
        if let Some(body) = &request.body {
            args.push("--body".to_string());
            args.push(body.clone());
        }
        if request.draft.unwrap_or(false) {
            args.push("--draft".to_string());
        }
        ProviderCommand::new(binary_for(GitProvider::GitHub), args)
    }

    /// Create a PR using gh CLI
    #[instrument(skip(request))]
    pub async fn create_pr(
        repo_info: &GitHubRepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError> {
        if !Self::is_installed().await {
            return Err(CreatePrError::ProviderCliNotInstalled);
        }

        if !Self::check_auth().await.unwrap_or(false) {
            return Err(CreatePrError::ProviderCliNotLoggedIn);
        }

        let command = Self::create_pr_argv(repo_info, request);
        let output = Self::run_gh(&command.arg_refs(), Some(cwd))
            .await
            .map_err(|e| classify_create_error(&e.to_string(), request))?;

        // `gh pr create` prints the PR URL; everything else comes from `get_pr`.
        let number =
            pr_number_from_url(&output).ok_or_else(|| CreatePrError::ProviderApiError {
                message: format!("Could not read a PR number from gh output: {output}"),
            })?;

        Self::get_pr(repo_info, number)
            .await
            .map_err(|e| CreatePrError::ProviderApiError {
                message: e.to_string(),
            })
    }

    /// Get PR info using gh CLI
    #[instrument]
    pub async fn get_pr(repo_info: &GitHubRepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
        let pr_num_str = pr_number.to_string();
        let output = Self::run_gh(
            &[
                "pr",
                "view",
                &pr_num_str,
                "--repo",
                &repo_info.full_name(),
                "--json",
                "number,url,state,isDraft,title,mergeCommit",
            ],
            None,
        )
        .await?;

        let response: GhPrViewResponse =
            serde_json::from_str(&output).context("Failed to parse PR view response")?;

        Ok(PullRequestInfo {
            number: response.number,
            url: response.url,
            state: match response.state.to_uppercase().as_str() {
                "OPEN" => PrState::Open,
                "MERGED" => PrState::Merged,
                _ => PrState::Closed,
            },
            merge_commit_sha: response.merge_commit.map(|c| c.oid),
            title: Some(response.title),
            is_draft: response.is_draft,
        })
    }

    /// List PRs for a branch
    #[instrument]
    pub async fn list_prs_for_branch(
        repo_info: &GitHubRepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        let output = Self::run_gh(
            &[
                "pr",
                "list",
                "--repo",
                &repo_info.full_name(),
                "--head",
                branch,
                "--json",
                "number,url,state,isDraft,title",
                "--state",
                "all",
            ],
            None,
        )
        .await?;

        let responses: Vec<GhPrListResponse> =
            serde_json::from_str(&output).context("Failed to parse PR list response")?;

        Ok(responses
            .into_iter()
            .map(|r| PullRequestInfo {
                number: r.number,
                url: r.url,
                state: match r.state.to_uppercase().as_str() {
                    "OPEN" => PrState::Open,
                    "MERGED" => PrState::Merged,
                    _ => PrState::Closed,
                },
                merge_commit_sha: None,
                title: Some(r.title),
                is_draft: r.is_draft,
            })
            .collect())
    }

    /// Get PR comments (general issue comments)
    #[instrument]
    pub async fn get_pr_comments(
        repo_info: &GitHubRepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        let endpoint = format!(
            "repos/{}/{}/issues/{}/comments",
            repo_info.owner, repo_info.repo_name, pr_number
        );

        let output = Self::run_gh(&["api", &endpoint], None).await?;

        let comments: Vec<GhIssueComment> =
            serde_json::from_str(&output).context("Failed to parse issue comments")?;

        Ok(comments
            .into_iter()
            .map(|c| UnifiedPrComment::General {
                id: c.id.to_string(),
                author: c.user.login,
                author_association: c.author_association,
                body: c.body,
                created_at: c.created_at,
                url: c.html_url,
            })
            .collect())
    }

    /// Get PR review comments (inline code comments)
    #[instrument]
    pub async fn get_pr_review_comments(
        repo_info: &GitHubRepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        let endpoint = format!(
            "repos/{}/{}/pulls/{}/comments",
            repo_info.owner, repo_info.repo_name, pr_number
        );

        let output = Self::run_gh(&["api", &endpoint], None).await?;

        let comments: Vec<GhReviewComment> =
            serde_json::from_str(&output).context("Failed to parse review comments")?;

        Ok(comments
            .into_iter()
            .map(|c| UnifiedPrComment::Review {
                id: c.id,
                author: c.user.login,
                author_association: c.author_association,
                body: c.body,
                created_at: c.created_at,
                url: c.html_url,
                path: c.path,
                line: c.line,
                diff_hunk: c.diff_hunk,
            })
            .collect())
    }

    /// Get all PR comments (general + review), merged and sorted by time
    #[instrument]
    pub async fn get_all_pr_comments(
        repo_info: &GitHubRepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        // Fetch both types in parallel
        let (general, review) = tokio::try_join!(
            Self::get_pr_comments(repo_info, pr_number),
            Self::get_pr_review_comments(repo_info, pr_number)
        )?;

        let mut all_comments = general;
        all_comments.extend(review);

        // Sort by creation time
        all_comments.sort_by_key(super::super::types::pr::UnifiedPrComment::created_at);

        Ok(all_comments)
    }

    /// Get the latest review state for a PR
    #[instrument]
    pub async fn get_pr_review_state(
        repo_info: &GitHubRepoInfo,
        pr_number: i64,
    ) -> Result<PrReviewState> {
        let endpoint = format!(
            "repos/{}/{}/pulls/{}/reviews",
            repo_info.owner, repo_info.repo_name, pr_number
        );

        let output = Self::run_gh(&["api", &endpoint], None).await?;

        let reviews: Vec<GhReview> =
            serde_json::from_str(&output).context("Failed to parse reviews")?;

        // Find the most recent non-COMMENTED, non-PENDING review
        let latest_decision = reviews
            .iter()
            .rev() // Most recent first
            .find(|r| r.state != "COMMENTED" && r.state != "PENDING")
            .map(|r| r.state.as_str());

        Ok(match latest_decision {
            Some("APPROVED") => PrReviewState::Approved,
            Some("CHANGES_REQUESTED") => PrReviewState::ChangesRequested,
            Some("DISMISSED") => PrReviewState::Dismissed,
            _ => PrReviewState::Pending,
        })
    }

    /// Open a PR in the browser
    pub async fn open_pr_in_browser(repo_info: &GitHubRepoInfo, pr_number: i64) -> Result<()> {
        let pr_num_str = pr_number.to_string();
        Self::run_gh(
            &[
                "pr",
                "view",
                &pr_num_str,
                "--repo",
                &repo_info.full_name(),
                "--web",
            ],
            None,
        )
        .await?;
        Ok(())
    }

    /// Check if a PR is ready to merge (approved, no changes requested, checks pass)
    #[instrument]
    pub async fn is_pr_ready_to_merge(repo_info: &GitHubRepoInfo, pr_number: i64) -> Result<bool> {
        let pr = Self::get_pr(repo_info, pr_number).await?;

        // Must be open and not a draft
        if pr.state != PrState::Open || pr.is_draft {
            return Ok(false);
        }

        // Check review state
        let review_state = Self::get_pr_review_state(repo_info, pr_number).await?;

        if review_state == PrReviewState::ChangesRequested {
            return Ok(false);
        }

        if review_state != PrReviewState::Approved {
            return Ok(false);
        }

        // TODO: Check status checks when needed
        // For now, just require approval

        Ok(true)
    }
}

// Response types for gh CLI JSON output

#[derive(Debug, Deserialize)]
struct GhPrCreateResponse {
    number: i64,
    url: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    title: String,
}

#[derive(Debug, Deserialize)]
struct GhPrViewResponse {
    number: i64,
    url: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    title: String,
    #[serde(rename = "mergeCommit")]
    merge_commit: Option<MergeCommit>,
}

#[derive(Debug, Deserialize)]
struct MergeCommit {
    oid: String,
}

#[derive(Debug, Deserialize)]
struct GhPrListResponse {
    number: i64,
    url: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    title: String,
}

#[derive(Debug, Deserialize)]
struct GhIssueComment {
    id: i64,
    body: String,
    html_url: String,
    user: GhUser,
    author_association: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct GhReviewComment {
    id: i64,
    body: String,
    html_url: String,
    user: GhUser,
    author_association: String,
    created_at: DateTime<Utc>,
    path: String,
    line: Option<i64>,
    diff_hunk: String,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GhReview {
    id: i64,
    state: String,
    user: GhUser,
    submitted_at: Option<DateTime<Utc>>,
}

/// Extract PR number and URL from "already exists" error message
fn extract_existing_pr_info(error: &str) -> Option<(i64, String)> {
    // Try to extract PR URL like https://github.com/owner/repo/pull/123
    let url_regex = regex::Regex::new(r"https://github\.com/[^/]+/[^/]+/pull/(\d+)").ok()?;
    if let Some(caps) = url_regex.captures(error) {
        let url = caps.get(0)?.as_str().to_string();
        let number: i64 = caps.get(1)?.as_str().parse().ok()?;
        return Some((number, url));
    }

    // Try to extract just the number
    let num_regex = regex::Regex::new(r"#(\d+)").ok()?;
    if let Some(caps) = num_regex.captures(error) {
        let number: i64 = caps.get(1)?.as_str().parse().ok()?;
        return Some((number, format!("(PR #{number})")));
    }

    None
}

/// Map a failed `gh pr create` to a structured error the UI can act on.
fn classify_create_error(err: &str, request: &CreatePrRequest) -> CreatePrError {
    if err.contains("already exists") {
        if let Some((pr_number, url)) = extract_existing_pr_info(err) {
            return CreatePrError::PrAlreadyExists { pr_number, url };
        }
    }
    if err.contains("not pushed") || err.contains("has no commits") {
        return CreatePrError::BranchNotPushed {
            branch: request.head_branch.clone(),
        };
    }
    if err.contains("not found") && err.contains(&request.base_branch) {
        return CreatePrError::TargetBranchNotFound {
            branch: request.base_branch.clone(),
        };
    }
    CreatePrError::ProviderApiError {
        message: err.to_string(),
    }
}

/// Read the PR number out of a `.../pull/<n>` URL anywhere in `output`.
fn pr_number_from_url(output: &str) -> Option<i64> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"/pull/(\d+)").expect("static regex"))
        .captures(output)?
        .get(1)?
        .as_str()
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> CreatePrRequest {
        CreatePrRequest {
            title: "Add widget".into(),
            body: Some("Body text".into()),
            head_branch: "feat/widget".into(),
            base_branch: "main".into(),
            draft: Some(true),
        }
    }

    #[test]
    fn create_pr_argv_is_the_documented_gh_contract() {
        let repo = GitHubRepoInfo::new(GitProvider::GitHub, "owner", "repo");
        let cmd = GhCli::create_pr_argv(&repo, &request());
        assert_eq!(cmd.program, "gh");
        assert_eq!(
            cmd.args,
            [
                "pr",
                "create",
                "--repo",
                "owner/repo",
                "--head",
                "feat/widget",
                "--base",
                "main",
                "--title",
                "Add widget",
                "--body",
                "Body text",
                "--draft",
            ]
        );
    }

    /// `gh pr create` has no `--json`; it prints the new PR's URL on stdout.
    /// Asking for JSON made every GitHub PR creation fail on flag parse.
    #[test]
    fn create_pr_argv_does_not_ask_gh_for_json() {
        let repo = GitHubRepoInfo::new(GitProvider::GitHub, "owner", "repo");
        let cmd = GhCli::create_pr_argv(&repo, &request());
        assert!(
            !cmd.long_flags().contains(&"--json"),
            "gh pr create does not accept --json"
        );
    }

    #[test]
    fn create_pr_argv_omits_optional_flags_when_unset() {
        let repo = GitHubRepoInfo::new(GitProvider::GitHub, "owner", "repo");
        let bare = CreatePrRequest {
            body: None,
            draft: None,
            ..request()
        };
        let flags = GhCli::create_pr_argv(&repo, &bare);
        assert!(!flags.long_flags().contains(&"--body"));
        assert!(!flags.long_flags().contains(&"--draft"));
    }

    #[test]
    fn pr_number_is_read_from_the_created_url() {
        assert_eq!(
            pr_number_from_url("https://github.com/owner/repo/pull/42"),
            Some(42)
        );
        assert_eq!(
            pr_number_from_url("noise\nhttps://github.com/o/r/pull/7\n"),
            Some(7)
        );
        assert_eq!(pr_number_from_url("no url here"), None);
    }

    #[tokio::test]
    async fn test_is_installed() {
        // This test just verifies the function doesn't panic
        let _ = GhCli::is_installed().await;
    }

    #[test]
    fn test_extract_existing_pr_info() {
        let error = "a pull request for branch 'feat-123' into 'main' already exists: https://github.com/owner/repo/pull/42";
        let result = extract_existing_pr_info(error);
        assert!(result.is_some());
        let (num, url) = result.unwrap();
        assert_eq!(num, 42);
        assert!(url.contains("42"));
    }

    #[test]
    fn test_extract_existing_pr_info_with_hash() {
        let error = "PR already exists #123";
        let result = extract_existing_pr_info(error);
        assert!(result.is_some());
        let (num, _) = result.unwrap();
        assert_eq!(num, 123);
    }

    #[test]
    fn test_extract_existing_pr_info_no_match() {
        let error = "some other error";
        let result = extract_existing_pr_info(error);
        assert!(result.is_none());
    }
}
