//! Pull Request types for Git provider integration.
//!
//! These types support multiple Git providers (GitHub, GitLab, Bitbucket, Azure DevOps)
//! with provider-specific CLI wrappers for operations.

use chrono::{DateTime, Utc};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use ts_rs::TS;

/// Supported Git hosting providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum GitProvider {
    /// GitHub (github.com)
    #[default]
    GitHub,
    /// GitLab (gitlab.com or self-hosted)
    GitLab,
    /// Bitbucket (bitbucket.org)
    Bitbucket,
    /// Azure DevOps (dev.azure.com)
    AzureDevOps,
}

impl fmt::Display for GitProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitProvider::GitHub => write!(f, "github"),
            GitProvider::GitLab => write!(f, "gitlab"),
            GitProvider::Bitbucket => write!(f, "bitbucket"),
            GitProvider::AzureDevOps => write!(f, "azure"),
        }
    }
}

impl GitProvider {
    /// Detect provider from a remote URL
    pub fn from_remote_url(remote_url: &str) -> Option<Self> {
        let url_lower = remote_url.to_lowercase();
        if url_lower.contains("github.com") {
            Some(GitProvider::GitHub)
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab.") {
            Some(GitProvider::GitLab)
        } else if url_lower.contains("bitbucket.org") {
            Some(GitProvider::Bitbucket)
        } else if url_lower.contains("dev.azure.com") || url_lower.contains("visualstudio.com") {
            Some(GitProvider::AzureDevOps)
        } else {
            None
        }
    }
}

/// Repository info parsed from remote URL (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct RepoInfo {
    /// Git hosting provider
    #[serde(default)]
    pub provider: GitProvider,
    /// Repository owner (user or organization)
    pub owner: String,
    /// Repository name
    pub repo_name: String,
}

impl RepoInfo {
    /// Create a new RepoInfo with explicit provider
    pub fn new(
        provider: GitProvider,
        owner: impl Into<String>,
        repo_name: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            owner: owner.into(),
            repo_name: repo_name.into(),
        }
    }

    /// Parse from remote URL (SSH or HTTPS)
    ///
    /// Supports formats for GitHub:
    /// - `git@github.com:owner/repo.git`
    /// - `https://github.com/owner/repo`
    /// - `https://github.com/owner/repo.git`
    /// - `github.com/owner/repo`
    ///
    /// Similar formats supported for GitLab, Bitbucket, and Azure DevOps.
    pub fn from_remote_url(remote_url: &str) -> Result<Self, RepoInfoError> {
        let provider = GitProvider::from_remote_url(remote_url)
            .ok_or_else(|| RepoInfoError::UnknownProvider(remote_url.to_string()))?;

        let (owner, repo_name) = parse_owner_repo(remote_url, provider)?;

        Ok(Self {
            provider,
            owner,
            repo_name,
        })
    }

    /// Get the full repo identifier (owner/repo)
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo_name)
    }
}

/// Backward compatibility alias
pub type GitHubRepoInfo = RepoInfo;

/// Helper function to parse owner/repo from various URL formats
fn parse_owner_repo(
    remote_url: &str,
    provider: GitProvider,
) -> Result<(String, String), RepoInfoError> {
    let pattern = match provider {
        GitProvider::GitHub => r"github\.com[:/](?P<owner>[^/]+)/(?P<repo>[^/]+?)(?:\.git)?(?:/|$)",
        GitProvider::GitLab => r"gitlab[^/]*[:/](?P<owner>[^/]+)/(?P<repo>[^/]+?)(?:\.git)?(?:/|$)",
        GitProvider::Bitbucket => {
            r"bitbucket\.org[:/](?P<owner>[^/]+)/(?P<repo>[^/]+?)(?:\.git)?(?:/|$)"
        }
        GitProvider::AzureDevOps => {
            r"(?:dev\.azure\.com|visualstudio\.com)[:/](?P<owner>[^/]+)/(?P<repo>[^/]+?)(?:\.git)?(?:/|$)"
        }
    };

    let re = Regex::new(pattern)
        .map_err(|e| RepoInfoError::InvalidUrl(format!("Failed to compile regex: {e}")))?;

    let caps = re.captures(remote_url).ok_or_else(|| {
        RepoInfoError::InvalidUrl(format!("Invalid {} URL format: {remote_url}", provider))
    })?;

    let owner = caps
        .name("owner")
        .ok_or_else(|| {
            RepoInfoError::InvalidUrl(format!("Failed to extract owner from URL: {remote_url}"))
        })?
        .as_str()
        .to_string();

    let repo_name = caps
        .name("repo")
        .ok_or_else(|| {
            RepoInfoError::InvalidUrl(format!(
                "Failed to extract repo name from URL: {remote_url}"
            ))
        })?
        .as_str()
        .to_string();

    Ok((owner, repo_name))
}

/// Error parsing repository info
#[derive(Debug, Error)]
pub enum RepoInfoError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Unknown Git provider for URL: {0}")]
    UnknownProvider(String),
}

/// Backward compatibility alias
pub type GitHubRepoError = RepoInfoError;

/// PR creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrRequest {
    /// PR title
    pub title: String,
    /// PR body/description (optional)
    pub body: Option<String>,
    /// Branch containing the changes
    pub head_branch: String,
    /// Target branch to merge into
    pub base_branch: String,
    /// Create as draft PR
    pub draft: Option<bool>,
}

/// PR info returned from GitHub
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct PullRequestInfo {
    /// PR number
    pub number: i64,
    /// PR URL on GitHub
    pub url: String,
    /// Current PR state
    pub state: PrState,
    /// Merge commit SHA (if merged)
    #[serde(default)]
    pub merge_commit_sha: Option<String>,
    /// PR title
    #[serde(default)]
    pub title: Option<String>,
    /// Whether PR is a draft
    #[serde(default)]
    pub is_draft: bool,
}

/// PR state (open, merged, closed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
#[ts(export)]
pub enum PrState {
    /// PR is open and accepting reviews
    Open,
    /// PR has been merged
    Merged,
    /// PR was closed without merging
    Closed,
}

/// Review status for a PR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum PrReviewState {
    /// Pending review
    Pending,
    /// PR approved
    Approved,
    /// Changes requested
    ChangesRequested,
    /// Review commented (no decision)
    Commented,
    /// Review dismissed
    Dismissed,
}

/// Unified PR comment (general or inline review)
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "comment_type", rename_all = "snake_case")]
#[ts(export)]
pub enum UnifiedPrComment {
    /// General PR comment (conversation)
    General {
        id: String,
        author: String,
        author_association: String,
        body: String,
        #[ts(type = "string")]
        created_at: DateTime<Utc>,
        url: String,
    },
    /// Inline review comment (on code)
    Review {
        id: i64,
        author: String,
        author_association: String,
        body: String,
        #[ts(type = "string")]
        created_at: DateTime<Utc>,
        url: String,
        /// File path the comment is on
        path: String,
        /// Line number (if applicable)
        line: Option<i64>,
        /// Diff hunk context
        diff_hunk: String,
    },
}

impl UnifiedPrComment {
    /// Get the creation time of the comment
    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            UnifiedPrComment::General { created_at, .. } => *created_at,
            UnifiedPrComment::Review { created_at, .. } => *created_at,
        }
    }

    /// Get the comment author
    pub fn author(&self) -> &str {
        match self {
            UnifiedPrComment::General { author, .. } => author,
            UnifiedPrComment::Review { author, .. } => author,
        }
    }

    /// Get the comment body
    pub fn body(&self) -> &str {
        match self {
            UnifiedPrComment::General { body, .. } => body,
            UnifiedPrComment::Review { body, .. } => body,
        }
    }
}

/// Error when creating a PR
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum CreatePrError {
    /// GitHub CLI is not installed
    GithubCliNotInstalled,
    /// GitHub CLI is not authenticated
    GithubCliNotLoggedIn,
    /// Git CLI is not installed
    GitCliNotInstalled,
    /// Git remote is not configured
    GitRemoteNotConfigured,
    /// Target branch does not exist on remote
    TargetBranchNotFound { branch: String },
    /// Branch has not been pushed to remote
    BranchNotPushed { branch: String },
    /// PR already exists for this branch
    PrAlreadyExists { pr_number: i64, url: String },
    /// GitHub API error
    GithubApiError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Provider detection tests
    #[test]
    fn test_detect_github_provider() {
        assert_eq!(
            GitProvider::from_remote_url("git@github.com:owner/repo.git"),
            Some(GitProvider::GitHub)
        );
        assert_eq!(
            GitProvider::from_remote_url("https://github.com/owner/repo"),
            Some(GitProvider::GitHub)
        );
    }

    #[test]
    fn test_detect_gitlab_provider() {
        assert_eq!(
            GitProvider::from_remote_url("git@gitlab.com:owner/repo.git"),
            Some(GitProvider::GitLab)
        );
        assert_eq!(
            GitProvider::from_remote_url("https://gitlab.example.com/owner/repo"),
            Some(GitProvider::GitLab)
        );
    }

    #[test]
    fn test_detect_bitbucket_provider() {
        assert_eq!(
            GitProvider::from_remote_url("git@bitbucket.org:owner/repo.git"),
            Some(GitProvider::Bitbucket)
        );
    }

    #[test]
    fn test_detect_azure_provider() {
        assert_eq!(
            GitProvider::from_remote_url("https://dev.azure.com/org/project"),
            Some(GitProvider::AzureDevOps)
        );
        assert_eq!(
            GitProvider::from_remote_url("https://org.visualstudio.com/project"),
            Some(GitProvider::AzureDevOps)
        );
    }

    #[test]
    fn test_detect_unknown_provider() {
        assert_eq!(
            GitProvider::from_remote_url("https://example.com/repo"),
            None
        );
    }

    // RepoInfo parsing tests (GitHub)
    #[test]
    fn test_parse_github_ssh_url() {
        let info = RepoInfo::from_remote_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    #[test]
    fn test_parse_github_https_url() {
        let info = RepoInfo::from_remote_url("https://github.com/owner/repo").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    #[test]
    fn test_parse_github_https_url_with_git() {
        let info = RepoInfo::from_remote_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitHub);
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    // RepoInfo parsing tests (GitLab)
    #[test]
    fn test_parse_gitlab_ssh_url() {
        let info = RepoInfo::from_remote_url("git@gitlab.com:owner/repo.git").unwrap();
        assert_eq!(info.provider, GitProvider::GitLab);
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    #[test]
    fn test_parse_gitlab_self_hosted() {
        let info = RepoInfo::from_remote_url("https://gitlab.example.com/owner/repo").unwrap();
        assert_eq!(info.provider, GitProvider::GitLab);
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    // Error handling tests
    #[test]
    fn test_invalid_url() {
        let result = RepoInfo::from_remote_url("not-a-valid-url");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RepoInfoError::UnknownProvider(_)
        ));
    }

    #[test]
    fn test_full_name() {
        let info = RepoInfo::new(GitProvider::GitHub, "anthropic", "claude-code");
        assert_eq!(info.full_name(), "anthropic/claude-code");
    }

    // Backward compatibility tests
    #[test]
    fn test_github_repo_info_alias() {
        // GitHubRepoInfo should work as an alias for RepoInfo
        let info: GitHubRepoInfo =
            RepoInfo::from_remote_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo_name, "repo");
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(GitProvider::GitHub.to_string(), "github");
        assert_eq!(GitProvider::GitLab.to_string(), "gitlab");
        assert_eq!(GitProvider::Bitbucket.to_string(), "bitbucket");
        assert_eq!(GitProvider::AzureDevOps.to_string(), "azure");
    }

    // PR comment tests
    #[test]
    fn test_pr_comment_created_at() {
        let comment = UnifiedPrComment::General {
            id: "1".to_string(),
            author: "user".to_string(),
            author_association: "CONTRIBUTOR".to_string(),
            body: "LGTM".to_string(),
            created_at: Utc::now(),
            url: "https://github.com/...".to_string(),
        };
        assert!(comment.created_at() <= Utc::now());
    }

    // TypeScript binding tests
    #[test]
    fn test_export_bindings_pullrequestinfo() {
        let _ = PullRequestInfo::export_to_string();
    }

    #[test]
    fn test_export_bindings_repoinfo() {
        let _ = RepoInfo::export_to_string();
    }

    #[test]
    fn test_export_bindings_gitprovider() {
        let _ = GitProvider::export_to_string();
    }
}
