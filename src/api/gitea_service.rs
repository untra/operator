use super::{pr_service::PrService, tea_cli::TeaCli};
use crate::config::GiteaConfig;
use crate::types::pr::{
    CreatePrError, CreatePrRequest, PrReviewState, PrState, PullRequestInfo, RepoInfo,
    UnifiedPrComment,
};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use backon::{ExponentialBuilder, Retryable};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

const PAGE_SIZE: usize = 50;
const MAX_PAGES: usize = 1000;

pub struct GiteaService {
    cli: TeaCli,
    wip_prefix: String,
}

#[derive(Deserialize)]
struct Pull {
    number: i64,
    html_url: String,
    state: String,
    #[serde(default)]
    merged: bool,
    #[serde(default)]
    draft: bool,
    merge_commit_sha: Option<String>,
    title: Option<String>,
}

impl From<Pull> for PullRequestInfo {
    fn from(pr: Pull) -> Self {
        Self {
            number: pr.number,
            url: pr.html_url,
            state: if pr.merged {
                PrState::Merged
            } else if pr.state == "closed" {
                PrState::Closed
            } else {
                PrState::Open
            },
            merge_commit_sha: pr.merge_commit_sha,
            title: pr.title,
            is_draft: pr.draft,
        }
    }
}

impl GiteaService {
    pub fn new(config: GiteaConfig) -> Self {
        Self {
            wip_prefix: config.wip_prefix.clone(),
            cli: TeaCli::new(config),
        }
    }

    fn repo_path(&self, repo: &RepoInfo) -> Result<String> {
        if let Some(host) = &repo.host {
            ensure!(
                self.cli
                    .base_url()?
                    .host_str()
                    .is_some_and(|configured| configured.eq_ignore_ascii_case(host)),
                "Repository host does not match configured Gitea host"
            );
        }
        ensure!(
            !repo.owner.contains('/') && !repo.owner.is_empty() && !repo.repo_name.is_empty(),
            "Invalid Gitea repository owner/name"
        );
        let mut url = self.cli.base_url()?;
        url.path_segments_mut()
            .map_err(|()| anyhow::anyhow!("Invalid Gitea base URL"))?
            .extend(["repos", &repo.owner, &repo.repo_name]);
        Ok(url.path().trim_start_matches('/').to_owned())
    }

    async fn get(&self, endpoint: &str) -> Result<Value> {
        (|| self.cli.request("GET", endpoint, None, None))
            .retry(ExponentialBuilder::default().with_max_times(3))
            .when(|e| e.to_string().contains("transient") || e.to_string().contains("timed out"))
            .await
    }

    async fn pages(&self, endpoint: &str) -> Result<Vec<Value>> {
        let mut all = Vec::new();
        for page in 1..=MAX_PAGES {
            let separator = if endpoint.contains('?') { '&' } else { '?' };
            let response = self
                .get(&format!(
                    "{endpoint}{separator}limit={PAGE_SIZE}&page={page}"
                ))
                .await?;
            let entries = response
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Expected Gitea collection"))?;
            if entries.is_empty() {
                return Ok(all);
            }
            all.extend(entries.iter().cloned());
        }
        anyhow::bail!("Gitea pagination limit exceeded")
    }
}

impl GiteaService {
    /// The instance this service talks to. Exposed so callers can prove the
    /// configured self-hosted host actually threaded through.
    pub fn base_url(&self) -> Result<url::Url> {
        self.cli.base_url()
    }
}

#[async_trait]
impl PrService for GiteaService {
    fn provider_name(&self) -> &'static str {
        "gitea"
    }
    async fn check_available(&self) -> Result<bool> {
        if !self.cli.available().await {
            return Ok(false);
        }
        Ok(self.get_authenticated_user().await.is_ok())
    }
    async fn get_authenticated_user(&self) -> Result<String> {
        let user = self.get("user").await?;
        Ok(user["login"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Gitea user has no login"))?
            .to_owned())
    }
    async fn get_pr(&self, repo: &RepoInfo, number: i64) -> Result<PullRequestInfo> {
        Ok(serde_json::from_value::<Pull>(
            self.get(&format!("{}/pulls/{number}", self.repo_path(repo)?))
                .await?,
        )?
        .into())
    }
    async fn is_ready_to_merge(&self, repo: &RepoInfo, number: i64) -> Result<bool> {
        let base = self.repo_path(repo)?;
        let pr = self.get(&format!("{base}/pulls/{number}")).await?;
        if pr["draft"].as_bool().unwrap_or(true) || pr["state"] != "open" || pr["mergeable"] != true
        {
            return Ok(false);
        }
        if self.get_review_state(repo, number).await? != PrReviewState::Approved {
            return Ok(false);
        }
        let sha = pr["head"]["sha"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing PR head SHA"))?;
        ensure!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "Invalid PR head SHA"
        );
        Ok(self.get(&format!("{base}/commits/{sha}/status")).await?["state"] == "success")
    }
    async fn get_review_state(&self, repo: &RepoInfo, number: i64) -> Result<PrReviewState> {
        let reviews = self
            .pages(&format!("{}/pulls/{number}/reviews", self.repo_path(repo)?))
            .await?;
        Ok(review_state(&reviews))
    }
    async fn create_pr(
        &self,
        repo: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError> {
        let result: Result<PullRequestInfo> = async {
            if let Some(existing) = self.find_pr_for_branch(repo, &request.head_branch).await? { return Ok(existing); }
            let title = if request.draft.unwrap_or(false) && !request.title.starts_with(&self.wip_prefix) { format!("{}{}", self.wip_prefix, request.title) } else { request.title.clone() };
            let body = json!({"title": title, "body": request.body, "head": request.head_branch, "base": request.base_branch});
            let endpoint = format!("{}/pulls", self.repo_path(repo)?);
            match self.cli.request("POST", &endpoint, Some(&body), Some(cwd)).await {
                Ok(value) => Ok(serde_json::from_value::<Pull>(value)?.into()),
                Err(error) => {
                    if let Ok(Some(existing)) = self.find_pr_for_branch(repo, &request.head_branch).await { return Ok(existing); }
                    Err(error)
                }
            }
        }.await;
        result.map_err(|e| CreatePrError::ProviderApiError {
            message: e.to_string(),
        })
    }
    async fn list_prs_for_branch(
        &self,
        repo: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        let prs = self
            .pages(&format!("{}/pulls?state=all", self.repo_path(repo)?))
            .await?;
        prs.into_iter()
            .filter(|p| {
                p["head"]["ref"] == branch && p["head"]["repo"]["full_name"] == repo.full_name()
            })
            .map(|p| Ok(serde_json::from_value::<Pull>(p)?.into()))
            .collect()
    }
    async fn get_all_comments(
        &self,
        repo: &RepoInfo,
        number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        let base = self.repo_path(repo)?;
        let mut comments = self
            .pages(&format!("{base}/issues/{number}/comments"))
            .await?
            .iter()
            .map(|c| comment(c, false))
            .collect::<Result<Vec<_>>>()?;
        for review in self
            .pages(&format!("{base}/pulls/{number}/reviews"))
            .await?
        {
            let id = review["id"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("Missing review ID"))?;
            for inline in self
                .pages(&format!("{base}/pulls/{number}/reviews/{id}/comments"))
                .await?
            {
                comments.push(comment(&inline, true)?);
            }
        }
        comments.sort_by_key(UnifiedPrComment::created_at);
        Ok(comments)
    }
    async fn open_in_browser(&self, repo: &RepoInfo, number: i64) -> Result<()> {
        let mut url = self.cli.base_url()?;
        url.path_segments_mut()
            .map_err(|()| anyhow::anyhow!("Invalid Gitea URL"))?
            .extend([&repo.owner, &repo.repo_name, "pulls", &number.to_string()]);
        #[cfg(target_os = "macos")]
        let program = "open";
        #[cfg(not(target_os = "macos"))]
        let program = "xdg-open";
        ensure!(
            tokio::process::Command::new(program)
                .arg(url.as_str())
                .status()
                .await?
                .success(),
            "Could not open PR in browser"
        );
        Ok(())
    }
    async fn get_comments_since(
        &self,
        repo: &RepoInfo,
        number: i64,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnifiedPrComment>> {
        Ok(self
            .get_all_comments(repo, number)
            .await?
            .into_iter()
            .filter(|c| c.created_at() > since)
            .collect())
    }
    async fn find_pr_for_branch(
        &self,
        repo: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>> {
        Ok(self
            .list_prs_for_branch(repo, branch)
            .await?
            .into_iter()
            .find(|p| p.state == PrState::Open))
    }
}

fn review_state(reviews: &[Value]) -> PrReviewState {
    let mut latest = std::collections::BTreeMap::new();
    for review in reviews {
        let id = review["id"].as_i64().unwrap_or_default();
        let user = review["user"]["login"].as_str().unwrap_or_default();
        let state = review["state"].as_str().unwrap_or_default();
        if !matches!(
            state,
            "APPROVED" | "REQUEST_CHANGES" | "COMMENT" | "DISMISSED"
        ) {
            continue;
        }
        let entry = latest.entry(user).or_insert((id, state));
        if id >= entry.0 {
            *entry = (id, state);
        }
    }
    if latest.values().any(|(_, s)| *s == "REQUEST_CHANGES") {
        PrReviewState::ChangesRequested
    } else if latest.values().any(|(_, s)| *s == "APPROVED") {
        PrReviewState::Approved
    } else if latest.values().any(|(_, s)| *s == "COMMENT") {
        PrReviewState::Commented
    } else {
        PrReviewState::Pending
    }
}

fn comment(value: &Value, inline: bool) -> Result<UnifiedPrComment> {
    let id = value["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing comment ID"))?;
    let author = value["user"]["login"]
        .as_str()
        .unwrap_or("ghost")
        .to_owned();
    let body = value["body"].as_str().unwrap_or_default().to_owned();
    let created_at = serde_json::from_value(value["created_at"].clone())?;
    let url = value["html_url"].as_str().unwrap_or_default().to_owned();
    if inline {
        Ok(UnifiedPrComment::Review {
            id,
            author,
            author_association: "NONE".into(),
            body,
            created_at,
            url,
            path: value["path"].as_str().unwrap_or_default().to_owned(),
            line: value["position"].as_i64(),
            diff_hunk: value["diff_hunk"].as_str().unwrap_or_default().to_owned(),
        })
    } else {
        Ok(UnifiedPrComment::General {
            id: id.to_string(),
            author,
            author_association: "NONE".into(),
            body,
            created_at,
            url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merged_and_draft_are_preserved() {
        let pr: Pull = serde_json::from_value(json!({"number": 2, "html_url": "https://gitea.example/a/b/pulls/2", "state": "closed", "merged": true, "draft": true})).unwrap();
        let info: PullRequestInfo = pr.into();
        assert_eq!(info.state, PrState::Merged);
        assert!(info.is_draft);
    }
    #[test]
    fn comments_have_explicit_unknown_association() {
        let c = comment(
            &json!({"id": 1,"user":{"login":"bot"},"created_at":"2026-01-01T00:00:00Z"}),
            false,
        )
        .unwrap();
        assert!(
            matches!(c, UnifiedPrComment::General { author_association, .. } if author_association == "NONE")
        );
    }
    #[test]
    fn latest_review_replaces_previous_decision() {
        let reviews = json!([{"id": 2,"user":{"login":"a"},"state":"APPROVED"},{"id":1,"user":{"login":"a"},"state":"REQUEST_CHANGES"}]);
        assert_eq!(
            review_state(reviews.as_array().unwrap()),
            PrReviewState::Approved
        );
    }
    #[test]
    fn cli_contract() {
        const CHILD: &str = "OPERATOR_GITEA_CONTRACT_CHILD";
        if std::env::var_os(CHILD).is_some() {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let service = GiteaService::new(GiteaConfig::default());
                let repo = RepoInfo::new(crate::types::pr::GitProvider::Gitea, "team", "repo");
                assert!(service.check_available().await.unwrap());
                assert_eq!(service.get_authenticated_user().await.unwrap(), "agent");
                assert_eq!(service.get_pr(&repo, 1).await.unwrap().number, 1);
                assert!(service.is_ready_to_merge(&repo, 1).await.unwrap());
                assert_eq!(
                    service.get_review_state(&repo, 1).await.unwrap(),
                    PrReviewState::Approved
                );
                assert!(service
                    .list_prs_for_branch(&repo, "new")
                    .await
                    .unwrap()
                    .is_empty());
                let request = CreatePrRequest {
                    title: "Change".into(),
                    body: Some("Description".into()),
                    head_branch: "new".into(),
                    base_branch: "main".into(),
                    draft: Some(true),
                };
                assert_eq!(
                    service
                        .create_pr(&repo, &request, Path::new("/tmp"))
                        .await
                        .unwrap()
                        .number,
                    1
                );
                assert_eq!(service.get_all_comments(&repo, 1).await.unwrap().len(), 1);
                assert_eq!(
                    service
                        .get_comments_since(&repo, 1, "2025-01-01T00:00:00Z".parse().unwrap())
                        .await
                        .unwrap()
                        .len(),
                    1
                );
                assert!(service
                    .find_pr_for_branch(&repo, "new")
                    .await
                    .unwrap()
                    .is_none());
            });
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let script = r#"#!/bin/sh
for arg do endpoint=$arg; done
[ "$endpoint" = --help ] && exit 0
[ -f "$XDG_CONFIG_HOME/tea/config.yml" ] || exit 40
case "$endpoint" in
user) printf '%s' '{"login":"agent"}';;
*page=2) printf '[]';;
*/reviews[?]*) printf '%s' '[{"id":1,"state":"APPROVED","user":{"login":"reviewer"}}]';;
*/issues/*/comments[?]*) printf '%s' '[{"id":1,"body":"review","user":{"login":"reviewer"},"created_at":"2026-01-01T00:00:00Z"}]';;
*/reviews/*/comments[?]*) printf '[]';;
*/status) printf '%s' '{"state":"success"}';;
*pulls[?]*) printf '[]';;
*/pulls) body=$(cat); case "$body" in *'WIP: Change'*) ;; *) exit 41;; esac
printf '%s' '{"number":1,"html_url":"https://gitea.com/team/repo/pulls/1","state":"open","draft":true}';;
*/pulls/1) printf '%s' '{"number":1,"html_url":"https://gitea.com/team/repo/pulls/1","state":"open","draft":false,"mergeable":true,"head":{"sha":"abc123"}}';;
*) exit 42;;
esac
"#;
        crate::git::runtime::private_file(&root.path().join("tea"), script.as_bytes(), true)
            .unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "api::gitea_service::tests::cli_contract",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("GITEA_TOKEN", "contract-secret")
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    root.path().display(),
                    std::env::var("PATH").unwrap_or_default()
                ),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("contract-secret"));
    }
}
