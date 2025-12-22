#![allow(dead_code)]

//! GitHub API client for PR and issue status tracking

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";

/// GitHub Pull Request status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestStatus {
    /// PR number
    pub number: u64,
    /// State: "open" or "closed"
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
    /// Review decision from GraphQL (optional, requires separate call)
    #[serde(skip)]
    pub review_decision: Option<String>,
    /// Whether all required checks have passed
    #[serde(skip)]
    pub checks_passed: Option<bool>,
}

/// GitHub Issue status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatus {
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

/// Check run status from GitHub Checks API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRunStatus {
    /// Check name
    pub name: String,
    /// Status: "queued", "in_progress", "completed"
    pub status: String,
    /// Conclusion: "success", "failure", "neutral", "cancelled", "skipped", "timed_out", "action_required"
    pub conclusion: Option<String>,
}

/// Review status from PR reviews endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStatus {
    /// Review state: "APPROVED", "CHANGES_REQUESTED", "COMMENTED", "PENDING"
    pub state: String,
    /// Reviewer login
    pub user_login: String,
}

/// GitHub API client
pub struct GitHubClient {
    token: String,
    client: reqwest::Client,
}

// Response types for API deserialization
#[derive(Debug, Deserialize)]
struct PrResponse {
    number: u64,
    state: String,
    title: String,
    html_url: String,
    draft: Option<bool>,
    merged: Option<bool>,
    mergeable: Option<bool>,
    head: HeadRef,
}

#[derive(Debug, Deserialize)]
struct HeadRef {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct IssueResponse {
    number: u64,
    state: String,
    title: String,
    html_url: String,
    labels: Vec<LabelResponse>,
}

#[derive(Debug, Deserialize)]
struct LabelResponse {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CheckRunsResponse {
    check_runs: Vec<CheckRunResponse>,
}

#[derive(Debug, Deserialize)]
struct CheckRunResponse {
    name: String,
    status: String,
    conclusion: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReviewResponse {
    state: String,
    user: UserResponse,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    login: String,
}

impl GitHubClient {
    /// Create a new GitHub client from the OPERATOR_GITHUB_TOKEN environment variable
    pub fn from_env() -> Result<Option<Self>> {
        match env::var("OPERATOR_GITHUB_TOKEN") {
            Ok(token) if !token.is_empty() => {
                let client = reqwest::Client::builder()
                    .user_agent("operator-tui/0.1.0")
                    .build()
                    .context("Failed to build HTTP client")?;
                Ok(Some(Self { token, client }))
            }
            _ => Ok(None),
        }
    }

    /// Check if the client is configured
    pub fn is_configured() -> bool {
        env::var("OPERATOR_GITHUB_TOKEN")
            .map(|t| !t.is_empty())
            .unwrap_or(false)
    }

    /// Get PR status
    pub async fn get_pr_status(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<PullRequestStatus> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            GITHUB_API_BASE, owner, repo, pr_number
        );

        let response: PrResponse = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .context("Failed to send request to GitHub API")?
            .error_for_status()
            .context("GitHub API returned error status")?
            .json()
            .await
            .context("Failed to parse GitHub PR response")?;

        Ok(PullRequestStatus {
            number: response.number,
            state: response.state,
            title: response.title,
            html_url: response.html_url,
            draft: response.draft.unwrap_or(false),
            merged: response.merged.unwrap_or(false),
            mergeable: response.mergeable,
            head_sha: response.head.sha,
            review_decision: None,
            checks_passed: None,
        })
    }

    /// Get issue status
    pub async fn get_issue_status(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<IssueStatus> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            GITHUB_API_BASE, owner, repo, issue_number
        );

        let response: IssueResponse = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .context("Failed to send request to GitHub API")?
            .error_for_status()
            .context("GitHub API returned error status")?
            .json()
            .await
            .context("Failed to parse GitHub issue response")?;

        Ok(IssueStatus {
            number: response.number,
            state: response.state,
            title: response.title,
            html_url: response.html_url,
            labels: response.labels.into_iter().map(|l| l.name).collect(),
        })
    }

    /// Get check runs for a commit
    pub async fn get_check_runs(
        &self,
        owner: &str,
        repo: &str,
        ref_sha: &str,
    ) -> Result<Vec<CheckRunStatus>> {
        let url = format!(
            "{}/repos/{}/{}/commits/{}/check-runs",
            GITHUB_API_BASE, owner, repo, ref_sha
        );

        let response: CheckRunsResponse = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .context("Failed to send request to GitHub API")?
            .error_for_status()
            .context("GitHub API returned error status")?
            .json()
            .await
            .context("Failed to parse GitHub check runs response")?;

        Ok(response
            .check_runs
            .into_iter()
            .map(|c| CheckRunStatus {
                name: c.name,
                status: c.status,
                conclusion: c.conclusion,
            })
            .collect())
    }

    /// Get reviews for a PR
    pub async fn get_pr_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<ReviewStatus>> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/reviews",
            GITHUB_API_BASE, owner, repo, pr_number
        );

        let response: Vec<ReviewResponse> = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .context("Failed to send request to GitHub API")?
            .error_for_status()
            .context("GitHub API returned error status")?
            .json()
            .await
            .context("Failed to parse GitHub reviews response")?;

        Ok(response
            .into_iter()
            .map(|r| ReviewStatus {
                state: r.state,
                user_login: r.user.login,
            })
            .collect())
    }

    /// Check if PR is approved and ready to merge
    /// Returns true if:
    /// - PR is open (not closed/merged)
    /// - Not a draft
    /// - Has at least one APPROVED review
    /// - All checks are successful (if any)
    pub async fn is_pr_ready_to_merge(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<bool> {
        // Get PR status
        let pr = self.get_pr_status(owner, repo, pr_number).await?;

        // Must be open and not a draft
        if pr.state != "open" || pr.draft {
            return Ok(false);
        }

        // Check for approved reviews
        let reviews = self.get_pr_reviews(owner, repo, pr_number).await?;
        let has_approval = reviews.iter().any(|r| r.state == "APPROVED");
        let has_changes_requested = reviews.iter().any(|r| r.state == "CHANGES_REQUESTED");

        if !has_approval || has_changes_requested {
            return Ok(false);
        }

        // Check if all checks pass
        let checks = self.get_check_runs(owner, repo, &pr.head_sha).await?;
        let all_checks_pass = checks.iter().all(|c| {
            c.status == "completed"
                && c.conclusion
                    .as_ref()
                    .map(|con| con == "success" || con == "skipped" || con == "neutral")
                    .unwrap_or(false)
        });

        // If there are checks, they must all pass
        if !checks.is_empty() && !all_checks_pass {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get a summary status for a PR
    /// Returns: "approved", "changes_requested", "pending", "draft", "merged", "closed"
    pub async fn get_pr_summary_status(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String> {
        let pr = self.get_pr_status(owner, repo, pr_number).await?;

        if pr.merged {
            return Ok("merged".to_string());
        }

        if pr.state == "closed" {
            return Ok("closed".to_string());
        }

        if pr.draft {
            return Ok("draft".to_string());
        }

        let reviews = self.get_pr_reviews(owner, repo, pr_number).await?;
        let has_changes_requested = reviews.iter().any(|r| r.state == "CHANGES_REQUESTED");
        let has_approval = reviews.iter().any(|r| r.state == "APPROVED");

        if has_changes_requested {
            return Ok("changes_requested".to_string());
        }

        if has_approval {
            return Ok("approved".to_string());
        }

        Ok("pending".to_string())
    }

    /// Parse owner and repo from a "owner/repo" string
    pub fn parse_repo(github_repo: &str) -> Option<(&str, &str)> {
        let parts: Vec<&str> = github_repo.split('/').collect();
        if parts.len() == 2 {
            Some((parts[0], parts[1]))
        } else {
            None
        }
    }

    /// Test connectivity by fetching rate limit info
    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/rate_limit", GITHUB_API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo() {
        assert_eq!(
            GitHubClient::parse_repo("gbqr-us/gbqr-us"),
            Some(("gbqr-us", "gbqr-us"))
        );
        assert_eq!(
            GitHubClient::parse_repo("owner/repo"),
            Some(("owner", "repo"))
        );
        assert_eq!(GitHubClient::parse_repo("invalid"), None);
        assert_eq!(GitHubClient::parse_repo("a/b/c"), None);
    }
}
