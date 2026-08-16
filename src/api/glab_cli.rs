//! GitLab CLI (`glab`) wrapper for MR operations.
//!
//! Uses the `glab` CLI (<https://gitlab.com/gitlab-org/cli>) for GitLab operations,
//! mirroring `GhCli`'s shape so both providers plug into the same `PrService` trait.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, instrument};

use crate::api::argv::ProviderCommand;
use crate::api::cli_detection::binary_for;
use crate::types::pr::GitProvider;
use crate::types::pr::{
    CreatePrError, CreatePrRequest, PrReviewState, PrState, PullRequestInfo, RepoInfo,
    UnifiedPrComment,
};

/// GitLab CLI wrapper for MR operations
pub struct GlabCli;

impl GlabCli {
    /// Execute a glab command and return stdout
    async fn run_glab(args: &[&str], cwd: Option<&Path>) -> Result<String> {
        debug!(?args, "Running glab command");

        let mut cmd = Command::new(binary_for(GitProvider::GitLab));
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let _git_runtime = crate::git::runtime::configure_command(&mut cmd)?;

        let output = cmd
            .output()
            .await
            .context("Failed to execute glab command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "glab {} failed: {}",
                args.first().unwrap_or(&""),
                stderr.trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if glab CLI is installed
    pub async fn is_installed() -> bool {
        Command::new(binary_for(GitProvider::GitLab))
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if glab CLI is authenticated
    #[instrument]
    pub async fn check_auth() -> Result<bool> {
        let result = Self::run_glab(&["auth", "status"], None).await;
        Ok(result.is_ok())
    }

    /// Get the authenticated user
    pub async fn get_authenticated_user() -> Result<String> {
        let output = Self::run_glab(&["api", "user"], None).await?;
        let user: GlabUser =
            serde_json::from_str(&output).context("Failed to parse authenticated user")?;
        Ok(user.username)
    }

    /// Build the `glab mr create` invocation. Pure, so the flags are asserted
    /// without spawning `glab`.
    ///
    /// GitLab spells the same request differently from GitHub:
    /// `--source-branch`/`--target-branch`/`--description`, and `--yes` to
    /// skip the interactive prompt.
    pub fn create_pr_argv(repo_info: &RepoInfo, request: &CreatePrRequest) -> ProviderCommand {
        let mut args = vec![
            "mr".to_string(),
            "create".to_string(),
            "--repo".to_string(),
            repo_info.full_name(),
            "--source-branch".to_string(),
            request.head_branch.clone(),
            "--target-branch".to_string(),
            request.base_branch.clone(),
            "--title".to_string(),
            request.title.clone(),
        ];
        if let Some(body) = &request.body {
            args.push("--description".to_string());
            args.push(body.clone());
        }
        if request.draft.unwrap_or(false) {
            args.push("--draft".to_string());
        }
        args.push("--yes".to_string());
        ProviderCommand::new(binary_for(GitProvider::GitLab), args)
    }

    /// Create an MR using glab CLI
    #[instrument(skip(request))]
    pub async fn create_pr(
        repo_info: &RepoInfo,
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
        let output = Self::run_glab(&command.arg_refs(), Some(cwd))
            .await
            .map_err(|e| classify_create_error(&e.to_string(), request))?;

        let mr_number =
            extract_mr_number(&output).ok_or_else(|| CreatePrError::ProviderApiError {
                message: format!("Failed to parse MR number from glab output: {output}"),
            })?;

        Self::get_pr(repo_info, mr_number)
            .await
            .map_err(|e| CreatePrError::ProviderApiError {
                message: e.to_string(),
            })
    }

    /// Get MR info using glab CLI
    #[instrument]
    pub async fn get_pr(repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo> {
        let pr_num_str = pr_number.to_string();
        let output = Self::run_glab(
            &[
                "mr",
                "view",
                &pr_num_str,
                "--repo",
                &repo_info.full_name(),
                "--output",
                "json",
            ],
            None,
        )
        .await?;

        let response: GlabMr =
            serde_json::from_str(&output).context("Failed to parse MR view response")?;
        Ok(response.into())
    }

    /// List MRs for a branch
    #[instrument]
    pub async fn list_prs_for_branch(
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>> {
        let output = Self::run_glab(
            &[
                "mr",
                "list",
                "--repo",
                &repo_info.full_name(),
                "--source-branch",
                branch,
                "--all",
                "--output",
                "json",
            ],
            None,
        )
        .await?;

        let responses: Vec<GlabMr> =
            serde_json::from_str(&output).context("Failed to parse MR list response")?;

        Ok(responses.into_iter().map(Into::into).collect())
    }

    /// Get all MR comments (general + review), notes API covers both
    #[instrument]
    pub async fn get_all_pr_comments(
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>> {
        let project_path = encode_project_path(repo_info);
        let endpoint = format!("projects/{project_path}/merge_requests/{pr_number}/notes?sort=asc");

        let output = Self::run_glab(&["api", &endpoint], None).await?;
        let notes: Vec<GlabNote> =
            serde_json::from_str(&output).context("Failed to parse notes")?;

        Ok(notes
            .into_iter()
            .filter(|n| !n.system)
            .map(note_to_comment)
            .collect())
    }

    /// Get the review state of an MR
    #[instrument]
    pub async fn get_pr_review_state(
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<PrReviewState> {
        let project_path = encode_project_path(repo_info);

        let approvals_endpoint =
            format!("projects/{project_path}/merge_requests/{pr_number}/approvals");
        let output = Self::run_glab(&["api", &approvals_endpoint], None).await?;
        let approvals: GlabApprovals =
            serde_json::from_str(&output).context("Failed to parse approvals")?;

        if approvals.approved {
            return Ok(PrReviewState::Approved);
        }

        // GitLab has no "dismissed" state; fall back to reviewer change-requests,
        // and treat any missing/unparseable data as Pending (conservative default).
        let reviewers_endpoint =
            format!("projects/{project_path}/merge_requests/{pr_number}/reviewers");
        let reviewers_output = Self::run_glab(&["api", &reviewers_endpoint], None)
            .await
            .unwrap_or_default();
        let reviewers: Vec<GlabReviewer> =
            serde_json::from_str(&reviewers_output).unwrap_or_default();

        Ok(reviewer_review_state(&reviewers))
    }

    /// Open an MR in the browser
    pub async fn open_pr_in_browser(repo_info: &RepoInfo, pr_number: i64) -> Result<()> {
        let pr_num_str = pr_number.to_string();
        Self::run_glab(
            &[
                "mr",
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

    /// Check if an MR is ready to merge (approved, no changes requested, checks pass)
    #[instrument]
    pub async fn is_pr_ready_to_merge(repo_info: &RepoInfo, pr_number: i64) -> Result<bool> {
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

// Response types for glab CLI JSON output

#[derive(Debug, Deserialize)]
struct GlabUser {
    username: String,
}

#[derive(Debug, Deserialize)]
struct GlabMr {
    iid: i64,
    web_url: String,
    state: String,
    title: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    merge_commit_sha: Option<String>,
}

impl From<GlabMr> for PullRequestInfo {
    fn from(mr: GlabMr) -> Self {
        PullRequestInfo {
            number: mr.iid,
            url: mr.web_url,
            state: map_state(&mr.state),
            merge_commit_sha: mr.merge_commit_sha,
            title: Some(mr.title),
            is_draft: mr.draft,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct GlabApprovals {
    #[serde(default)]
    approved: bool,
}

#[derive(Debug, Default, Deserialize)]
struct GlabReviewer {
    #[serde(default)]
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GlabNoteAuthor {
    username: String,
}

#[derive(Debug, Deserialize)]
struct GlabNotePosition {
    new_path: String,
    #[serde(default)]
    new_line: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct GlabNote {
    id: i64,
    body: String,
    author: GlabNoteAuthor,
    created_at: DateTime<Utc>,
    #[serde(default)]
    system: bool,
    #[serde(default)]
    position: Option<GlabNotePosition>,
}

/// Map a GitLab MR `state` string to the provider-agnostic `PrState`.
fn map_state(state: &str) -> PrState {
    match state {
        "opened" => PrState::Open,
        "merged" => PrState::Merged,
        "closed" | "locked" => PrState::Closed,
        _ => PrState::Closed,
    }
}

/// Reduce a reviewers list to a review state: any `requested_changes` wins,
/// otherwise conservative Pending (GitLab has no Dismissed equivalent).
fn reviewer_review_state(reviewers: &[GlabReviewer]) -> PrReviewState {
    if reviewers
        .iter()
        .any(|r| r.state.as_deref() == Some("requested_changes"))
    {
        PrReviewState::ChangesRequested
    } else {
        PrReviewState::Pending
    }
}

/// Convert a GitLab note into a `UnifiedPrComment`. Notes with a `position`
/// are inline review comments; others are general conversation comments.
/// GitLab's notes API has no `author_association` or comment `url`, so both
/// are left empty (mirrors `gh_cli`'s handling of provider-missing fields).
fn note_to_comment(note: GlabNote) -> UnifiedPrComment {
    match note.position {
        Some(pos) => UnifiedPrComment::Review {
            id: note.id,
            author: note.author.username,
            author_association: String::new(),
            body: note.body,
            created_at: note.created_at,
            url: String::new(),
            path: pos.new_path,
            line: pos.new_line,
            diff_hunk: String::new(),
        },
        None => UnifiedPrComment::General {
            id: note.id.to_string(),
            author: note.author.username,
            author_association: String::new(),
            body: note.body,
            created_at: note.created_at,
            url: String::new(),
        },
    }
}

/// Percent-encode a repo's `owner/repo` path for GitLab's REST API, which
/// requires every `/` (including subgroup separators) encoded as `%2F`.
fn encode_project_path(repo_info: &RepoInfo) -> String {
    repo_info.full_name().replace('/', "%2F")
}

/// Extract the MR number from `glab mr create`'s stdout, which prints the MR
/// URL (`.../-/merge_requests/N`) rather than JSON.
fn extract_mr_number(output: &str) -> Option<i64> {
    if let Ok(re) = regex::Regex::new(r"/-/merge_requests/(\d+)") {
        if let Some(caps) = re.captures(output) {
            return caps.get(1)?.as_str().parse().ok();
        }
    }

    // Fall back to a trailing "!N" shorthand
    if let Ok(re) = regex::Regex::new(r"!(\d+)\s*$") {
        if let Some(caps) = re.captures(output.trim()) {
            return caps.get(1)?.as_str().parse().ok();
        }
    }

    None
}

/// Map a failed `glab mr create` to a structured error, matching the GitHub
/// path. Previously every GitLab failure collapsed into `ProviderApiError`.
fn classify_create_error(err: &str, request: &CreatePrRequest) -> CreatePrError {
    let lower = err.to_lowercase();
    if lower.contains("already exists") || lower.contains("open merge request") {
        if let Some(pr_number) = extract_mr_number(err) {
            return CreatePrError::PrAlreadyExists {
                pr_number,
                url: err.to_string(),
            };
        }
    }
    if lower.contains("not pushed")
        || lower.contains("has no commits")
        || lower.contains("no commits between")
    {
        return CreatePrError::BranchNotPushed {
            branch: request.head_branch.clone(),
        };
    }
    if lower.contains("not found") && err.contains(&request.base_branch) {
        return CreatePrError::TargetBranchNotFound {
            branch: request.base_branch.clone(),
        };
    }
    CreatePrError::ProviderApiError {
        message: err.to_string(),
    }
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
    fn create_pr_argv_is_the_documented_glab_contract() {
        let repo = RepoInfo::new(GitProvider::GitLab, "group/sub", "repo");
        let cmd = GlabCli::create_pr_argv(&repo, &request());
        assert_eq!(cmd.program, "glab");
        assert_eq!(
            cmd.args,
            [
                "mr",
                "create",
                "--repo",
                "group/sub/repo",
                "--source-branch",
                "feat/widget",
                "--target-branch",
                "main",
                "--title",
                "Add widget",
                "--description",
                "Body text",
                "--draft",
                "--yes",
            ]
        );
    }

    /// GitLab does not take GitHub's flag names; mixing them up is silent
    /// until a live MR is attempted.
    #[test]
    fn create_pr_argv_does_not_borrow_github_flag_names() {
        let repo = RepoInfo::new(GitProvider::GitLab, "owner", "repo");
        let flags = GlabCli::create_pr_argv(&repo, &request());
        for github_only in ["--head", "--base", "--body", "--json"] {
            assert!(
                !flags.long_flags().contains(&github_only),
                "{github_only} is a gh flag, not a glab flag"
            );
        }
    }

    #[test]
    fn create_failures_map_to_structured_errors() {
        let req = request();
        assert!(matches!(
            classify_create_error("no commits between main and feat/widget", &req),
            CreatePrError::BranchNotPushed { .. }
        ));
        assert!(matches!(
            classify_create_error("an open merge request already exists: !42", &req),
            CreatePrError::PrAlreadyExists { pr_number: 42, .. }
        ));
        assert!(matches!(
            classify_create_error("something else broke", &req),
            CreatePrError::ProviderApiError { .. }
        ));
    }

    #[tokio::test]
    async fn test_is_installed() {
        // This test just verifies the function doesn't panic
        let _ = GlabCli::is_installed().await;
    }

    // --- MR number extraction from create output ---

    #[test]
    fn test_extract_mr_number_from_url() {
        let output = "https://gitlab.com/group/project/-/merge_requests/42";
        assert_eq!(extract_mr_number(output), Some(42));
    }

    #[test]
    fn test_extract_mr_number_from_bang_shorthand() {
        let output = "Merge request created !17";
        assert_eq!(extract_mr_number(output), Some(17));
    }

    #[test]
    fn test_extract_mr_number_no_match() {
        assert_eq!(extract_mr_number("no number here"), None);
    }

    // --- glab mr view JSON state mapping ---

    #[test]
    fn test_map_state_opened() {
        assert_eq!(map_state("opened"), PrState::Open);
    }

    #[test]
    fn test_map_state_merged() {
        assert_eq!(map_state("merged"), PrState::Merged);
    }

    #[test]
    fn test_map_state_closed() {
        assert_eq!(map_state("closed"), PrState::Closed);
    }

    #[test]
    fn test_map_state_locked() {
        assert_eq!(map_state("locked"), PrState::Closed);
    }

    #[test]
    fn test_glab_mr_json_deserializes_and_maps_draft() {
        let json = r#"{
            "iid": 5,
            "web_url": "https://gitlab.com/group/project/-/merge_requests/5",
            "state": "opened",
            "title": "Add feature",
            "draft": true,
            "merge_commit_sha": null
        }"#;
        let mr: GlabMr = serde_json::from_str(json).unwrap();
        let info: PullRequestInfo = mr.into();
        assert_eq!(info.number, 5);
        assert_eq!(info.state, PrState::Open);
        assert!(info.is_draft);
        assert_eq!(info.merge_commit_sha, None);
    }

    #[test]
    fn test_glab_mr_json_merged_with_commit_sha() {
        let json = r#"{
            "iid": 9,
            "web_url": "https://gitlab.com/group/project/-/merge_requests/9",
            "state": "merged",
            "title": "Fix bug",
            "draft": false,
            "merge_commit_sha": "abc123"
        }"#;
        let mr: GlabMr = serde_json::from_str(json).unwrap();
        let info: PullRequestInfo = mr.into();
        assert_eq!(info.state, PrState::Merged);
        assert_eq!(info.merge_commit_sha, Some("abc123".to_string()));
    }

    // --- approvals + reviewers -> review-state normalization ---

    #[test]
    fn test_approvals_approved() {
        let json = r#"{"approved": true}"#;
        let approvals: GlabApprovals = serde_json::from_str(json).unwrap();
        assert!(approvals.approved);
    }

    #[test]
    fn test_reviewer_review_state_requested_changes() {
        let reviewers = vec![
            GlabReviewer {
                state: Some("reviewed".to_string()),
            },
            GlabReviewer {
                state: Some("requested_changes".to_string()),
            },
        ];
        assert_eq!(
            reviewer_review_state(&reviewers),
            PrReviewState::ChangesRequested
        );
    }

    #[test]
    fn test_reviewer_review_state_pending_when_none_requested_changes() {
        let reviewers = vec![GlabReviewer {
            state: Some("reviewed".to_string()),
        }];
        assert_eq!(reviewer_review_state(&reviewers), PrReviewState::Pending);
    }

    #[test]
    fn test_reviewer_review_state_pending_on_missing_fields() {
        let reviewers: Vec<GlabReviewer> = vec![GlabReviewer { state: None }];
        assert_eq!(reviewer_review_state(&reviewers), PrReviewState::Pending);
    }

    #[test]
    fn test_reviewer_review_state_pending_on_empty_list() {
        assert_eq!(reviewer_review_state(&[]), PrReviewState::Pending);
    }

    // --- notes JSON -> comment conversion ---

    #[test]
    fn test_note_to_comment_general() {
        let json = r#"{
            "id": 1,
            "body": "LGTM",
            "author": {"username": "alice"},
            "created_at": "2024-01-01T00:00:00Z",
            "system": false
        }"#;
        let note: GlabNote = serde_json::from_str(json).unwrap();
        let comment = note_to_comment(note);
        match comment {
            UnifiedPrComment::General {
                id, author, body, ..
            } => {
                assert_eq!(id, "1");
                assert_eq!(author, "alice");
                assert_eq!(body, "LGTM");
            }
            UnifiedPrComment::Review { .. } => panic!("expected General comment"),
        }
    }

    #[test]
    fn test_note_to_comment_review_with_position() {
        let json = r#"{
            "id": 2,
            "body": "fix this",
            "author": {"username": "bob"},
            "created_at": "2024-01-02T00:00:00Z",
            "system": false,
            "position": {"new_path": "src/main.rs", "new_line": 10}
        }"#;
        let note: GlabNote = serde_json::from_str(json).unwrap();
        let comment = note_to_comment(note);
        match comment {
            UnifiedPrComment::Review {
                id,
                path,
                line,
                diff_hunk,
                ..
            } => {
                assert_eq!(id, 2);
                assert_eq!(path, "src/main.rs");
                assert_eq!(line, Some(10));
                assert_eq!(diff_hunk, "");
            }
            UnifiedPrComment::General { .. } => panic!("expected Review comment"),
        }
    }

    #[test]
    fn test_notes_filter_system_comments() {
        let json = r#"[
            {"id": 1, "body": "hello", "author": {"username": "alice"}, "created_at": "2024-01-01T00:00:00Z", "system": false},
            {"id": 2, "body": "changed target branch", "author": {"username": "bob"}, "created_at": "2024-01-01T00:00:01Z", "system": true}
        ]"#;
        let notes: Vec<GlabNote> = serde_json::from_str(json).unwrap();
        let comments: Vec<_> = notes
            .into_iter()
            .filter(|n| !n.system)
            .map(note_to_comment)
            .collect();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].author(), "alice");
    }

    // --- project path %2F encoding, including subgroups ---

    #[test]
    fn test_encode_project_path_simple() {
        let repo = RepoInfo::new(crate::types::pr::GitProvider::GitLab, "owner", "repo");
        assert_eq!(encode_project_path(&repo), "owner%2Frepo");
    }

    #[test]
    fn test_encode_project_path_subgroup() {
        let repo = RepoInfo::new(crate::types::pr::GitProvider::GitLab, "group/sub", "repo");
        assert_eq!(encode_project_path(&repo), "group%2Fsub%2Frepo");
    }
}
