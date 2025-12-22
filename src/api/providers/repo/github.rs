#![allow(dead_code)]

//! GitHub API provider implementation

use async_trait::async_trait;
use serde::Deserialize;
use std::env;

use super::{CheckStatus, IssueStatus, PrStatus, RepoProvider};
use crate::api::error::ApiError;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";
const PROVIDER_NAME: &str = "github";

/// GitHub API provider for PR and issue status tracking
pub struct GitHubProvider {
    token: String,
    client: reqwest::Client,
    base_url: String,
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
}

impl GitHubProvider {
    /// Create a new GitHub provider with the given token
    pub fn new(token: impl Into<String>) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .user_agent("operator-tui/0.1.0")
            .build()
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        Ok(Self {
            token: token.into(),
            client,
            base_url: GITHUB_API_BASE.to_string(),
        })
    }

    /// Create provider from OPERATOR_GITHUB_TOKEN environment variable
    pub fn from_env() -> Result<Option<Self>, ApiError> {
        match env::var("OPERATOR_GITHUB_TOKEN") {
            Ok(token) if !token.is_empty() => Ok(Some(Self::new(token)?)),
            _ => Ok(None),
        }
    }

    /// Create provider with a custom base URL (for testing)
    #[cfg(test)]
    pub fn new_with_base_url(
        token: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let mut provider = Self::new(token)?;
        provider.base_url = base_url.into();
        Ok(provider)
    }

    /// Check if the provider is configured (env var is set)
    pub fn is_env_configured() -> bool {
        env::var("OPERATOR_GITHUB_TOKEN")
            .map(|t| !t.is_empty())
            .unwrap_or(false)
    }

    /// Get reviews for a PR and determine overall review status
    async fn get_review_status(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/reviews",
            self.base_url, owner, repo, pr_number
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        match status.as_u16() {
            200..=299 => {}
            401 => return Err(ApiError::unauthorized(PROVIDER_NAME)),
            403 => return Err(ApiError::forbidden(PROVIDER_NAME)),
            404 => return Ok("none".to_string()),
            status => {
                let body = response.text().await.unwrap_or_default();
                return Err(ApiError::http(PROVIDER_NAME, status, body));
            }
        }

        let reviews: Vec<ReviewResponse> = response
            .json()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        // Determine overall status (most recent relevant review wins)
        let has_changes_requested = reviews.iter().any(|r| r.state == "CHANGES_REQUESTED");
        let has_approval = reviews.iter().any(|r| r.state == "APPROVED");

        if has_changes_requested {
            Ok("changes_requested".to_string())
        } else if has_approval {
            Ok("approved".to_string())
        } else if reviews.is_empty() {
            Ok("none".to_string())
        } else {
            Ok("pending".to_string())
        }
    }

    /// Check if all checks pass for a commit
    async fn checks_pass(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<Option<bool>, ApiError> {
        let checks = self.get_check_runs_internal(owner, repo, sha).await?;

        if checks.is_empty() {
            return Ok(None); // No checks configured
        }

        let all_pass = checks.iter().all(|c| c.is_passed());
        Ok(Some(all_pass))
    }

    async fn get_check_runs_internal(
        &self,
        owner: &str,
        repo: &str,
        ref_sha: &str,
    ) -> Result<Vec<CheckStatus>, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/commits/{}/check-runs",
            self.base_url, owner, repo, ref_sha
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        match status.as_u16() {
            200..=299 => {}
            401 => return Err(ApiError::unauthorized(PROVIDER_NAME)),
            403 => return Err(ApiError::forbidden(PROVIDER_NAME)),
            404 => return Ok(Vec::new()),
            status => {
                let body = response.text().await.unwrap_or_default();
                return Err(ApiError::http(PROVIDER_NAME, status, body));
            }
        }

        let check_runs: CheckRunsResponse = response
            .json()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        Ok(check_runs
            .check_runs
            .into_iter()
            .map(|c| CheckStatus {
                name: c.name,
                status: c.status,
                conclusion: c.conclusion,
            })
            .collect())
    }
}

#[async_trait]
impl RepoProvider for GitHubProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }

    async fn get_pr_status(&self, repo: &str, pr_number: u64) -> Result<PrStatus, ApiError> {
        let (owner, repo_name) = Self::parse_repo(repo).ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                400,
                "Invalid repo format, expected 'owner/repo'",
            )
        })?;

        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.base_url, owner, repo_name, pr_number
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        match status.as_u16() {
            200..=299 => {}
            401 => return Err(ApiError::unauthorized(PROVIDER_NAME)),
            403 => return Err(ApiError::forbidden(PROVIDER_NAME)),
            429 => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok());
                return Err(ApiError::rate_limited(PROVIDER_NAME, retry_after));
            }
            status => {
                let body = response.text().await.unwrap_or_default();
                return Err(ApiError::http(PROVIDER_NAME, status, body));
            }
        }

        let pr: PrResponse = response
            .json()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        // Get review status
        let review_status = self.get_review_status(owner, repo_name, pr_number).await?;

        // Get checks status
        let checks_passed = self.checks_pass(owner, repo_name, &pr.head.sha).await?;

        Ok(PrStatus {
            provider: PROVIDER_NAME.to_string(),
            number: pr.number,
            state: pr.state,
            title: pr.title,
            html_url: pr.html_url,
            draft: pr.draft.unwrap_or(false),
            merged: pr.merged.unwrap_or(false),
            mergeable: pr.mergeable,
            head_sha: pr.head.sha,
            review_status,
            checks_passed,
        })
    }

    async fn get_issue_status(
        &self,
        repo: &str,
        issue_number: u64,
    ) -> Result<IssueStatus, ApiError> {
        let (owner, repo_name) = Self::parse_repo(repo).ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                400,
                "Invalid repo format, expected 'owner/repo'",
            )
        })?;

        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.base_url, owner, repo_name, issue_number
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        match status.as_u16() {
            200..=299 => {}
            401 => return Err(ApiError::unauthorized(PROVIDER_NAME)),
            403 => return Err(ApiError::forbidden(PROVIDER_NAME)),
            status => {
                let body = response.text().await.unwrap_or_default();
                return Err(ApiError::http(PROVIDER_NAME, status, body));
            }
        }

        let issue: IssueResponse = response
            .json()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        Ok(IssueStatus {
            provider: PROVIDER_NAME.to_string(),
            number: issue.number,
            state: issue.state,
            title: issue.title,
            html_url: issue.html_url,
            labels: issue.labels.into_iter().map(|l| l.name).collect(),
        })
    }

    async fn get_check_runs(
        &self,
        repo: &str,
        ref_sha: &str,
    ) -> Result<Vec<CheckStatus>, ApiError> {
        let (owner, repo_name) = Self::parse_repo(repo).ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                400,
                "Invalid repo format, expected 'owner/repo'",
            )
        })?;

        self.get_check_runs_internal(owner, repo_name, ref_sha)
            .await
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        let url = format!("{}/rate_limit", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        match status.as_u16() {
            200..=299 => Ok(true),
            401 => Err(ApiError::unauthorized(PROVIDER_NAME)),
            403 => Err(ApiError::forbidden(PROVIDER_NAME)),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(ApiError::http(PROVIDER_NAME, status, body))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo() {
        assert_eq!(
            GitHubProvider::parse_repo("owner/repo"),
            Some(("owner", "repo"))
        );
        assert_eq!(
            GitHubProvider::parse_repo("gbqr-us/gamesvc"),
            Some(("gbqr-us", "gamesvc"))
        );
        assert_eq!(GitHubProvider::parse_repo("invalid"), None);
        assert_eq!(GitHubProvider::parse_repo("a/b/c"), None);
    }

    #[test]
    fn test_provider_name() {
        let provider = GitHubProvider::new("test-token").unwrap();
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn test_is_configured() {
        let provider = GitHubProvider::new("test-token").unwrap();
        assert!(provider.is_configured());

        let provider = GitHubProvider::new("").unwrap();
        assert!(!provider.is_configured());
    }
}
