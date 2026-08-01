//! `OpenSpec` (spec-driven development) kanban provider — alpha.
//!
//! Reads local `OpenSpec` change bundles (`openspec/changes/<id>/{proposal,tasks}.md`)
//! and exposes each change as a kanban "project" whose issues are the `## N.`
//! task groups from `tasks.md`. Pull-only: this provider is intentionally not
//! wired into the bidirectional push path, and its mutating trait methods
//! return errors.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::{
    CreateIssueRequest, CreateIssueResponse, ExternalIssue, ExternalIssueType, ExternalUser,
    KanbanProvider, ProjectInfo, UpdateStatusRequest,
};
use crate::api::error::ApiError;
use crate::config::OpenspecConfig;

const PROVIDER_NAME: &str = "openspec";
const CHANGES_DIR: &str = "changes";
const ARCHIVE_DIR: &str = "archive";
const TASKS_FILE: &str = "tasks.md";
const PROPOSAL_FILE: &str = "proposal.md";

pub const OPENSPEC_STATUS_TODO: &str = "todo";
pub const OPENSPEC_STATUS_DONE: &str = "done";

fn openspec_error(status: u16, message: String) -> ApiError {
    ApiError::HttpError {
        provider: PROVIDER_NAME.to_string(),
        status,
        message,
    }
}

// ─── Markdown parsing ────────────────────────────────────────────────────────

/// A single checklist item within a task group
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub checked: bool,
    pub text: String,
}

/// A `## N. Title` group of checklist items from tasks.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGroup {
    pub number: u32,
    pub title: String,
    pub items: Vec<TaskItem>,
}

impl TaskGroup {
    /// A group is done when it has items and every one is checked
    pub fn is_done(&self) -> bool {
        !self.items.is_empty() && self.items.iter().all(|i| i.checked)
    }
}

/// Proposal metadata extracted from proposal.md
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProposalMeta {
    pub title: Option<String>,
    pub why_excerpt: Option<String>,
}

fn parse_checklist_item(line: &str) -> Option<TaskItem> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))?;
    let (checked, text) = if let Some(t) = rest.strip_prefix("[ ]") {
        (false, t)
    } else if let Some(t) = rest
        .strip_prefix("[x]")
        .or_else(|| rest.strip_prefix("[X]"))
    {
        (true, t)
    } else {
        return None;
    };
    Some(TaskItem {
        checked,
        text: text.trim().to_string(),
    })
}

/// Parse a `## ...` heading into (number, title); headings without a leading
/// number get the provided fallback ordinal.
fn parse_group_heading(heading: &str, fallback_number: u32) -> (u32, String) {
    let text = heading.trim();
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    if !digits.is_empty() {
        if let Ok(n) = digits.parse::<u32>() {
            let title = text[digits.len()..]
                .trim_start_matches(['.', ')', ':'])
                .trim()
                .to_string();
            if !title.is_empty() {
                return (n, title);
            }
        }
    }
    (fallback_number, text.to_string())
}

/// Parse an `OpenSpec` tasks.md into its task groups.
///
/// Checklist items before the first `## ` heading are collected into an
/// implicit group 1 titled "Tasks".
pub fn parse_tasks_md(content: &str) -> Vec<TaskGroup> {
    let mut groups: Vec<TaskGroup> = Vec::new();
    let mut current: Option<TaskGroup> = None;
    let mut in_code_fence = false;

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(group) = current.take() {
                groups.push(group);
            }
            let fallback = groups.len() as u32 + 1;
            let (number, title) = parse_group_heading(heading, fallback);
            current = Some(TaskGroup {
                number,
                title,
                items: Vec::new(),
            });
            continue;
        }
        if let Some(item) = parse_checklist_item(line) {
            let group = current.get_or_insert_with(|| TaskGroup {
                number: 1,
                title: "Tasks".to_string(),
                items: Vec::new(),
            });
            group.items.push(item);
        }
    }
    if let Some(group) = current.take() {
        groups.push(group);
    }
    // Drop headings that contained no checklist items (prose sections)
    groups.retain(|g| !g.items.is_empty());
    groups
}

/// Extract the H1 title and the first `## Why` / `## Intent` section body.
pub fn parse_proposal(content: &str) -> ProposalMeta {
    let mut meta = ProposalMeta::default();
    let mut why_lines: Vec<String> = Vec::new();
    let mut in_why = false;

    for line in content.lines() {
        if let Some(h1) = line.strip_prefix("# ") {
            if meta.title.is_none() {
                meta.title = Some(h1.trim().to_string());
            }
            continue;
        }
        if let Some(h2) = line.strip_prefix("## ") {
            let name = h2.trim().to_lowercase();
            in_why = matches!(name.as_str(), "why" | "intent");
            continue;
        }
        if in_why {
            why_lines.push(line.to_string());
        }
    }
    let excerpt = why_lines.join("\n").trim().to_string();
    if !excerpt.is_empty() {
        meta.why_excerpt = Some(excerpt);
    }
    meta
}

// ─── Provider ────────────────────────────────────────────────────────────────

/// Kanban provider over a local `OpenSpec` root directory
pub struct OpenspecProvider {
    /// Instance key from config (`[kanban.openspec.<key>]`)
    instance_key: String,
    /// Directory containing `changes/` (typically `<repo>/openspec`)
    root_path: PathBuf,
}

impl OpenspecProvider {
    pub fn new(instance_key: impl Into<String>, root_path: impl Into<PathBuf>) -> Self {
        Self {
            instance_key: instance_key.into(),
            root_path: root_path.into(),
        }
    }

    pub fn from_config(instance_key: &str, config: &OpenspecConfig) -> Self {
        Self::new(instance_key, PathBuf::from(&config.root_path))
    }

    pub fn instance_key(&self) -> &str {
        &self.instance_key
    }

    fn changes_dir(&self) -> PathBuf {
        self.root_path.join(CHANGES_DIR)
    }

    fn change_dir(&self, change_id: &str) -> Result<PathBuf, ApiError> {
        // Change ids are directory names; refuse anything path-like
        if change_id.contains(['/', '\\']) || change_id == ".." {
            return Err(openspec_error(
                400,
                format!("invalid openspec change id '{change_id}'"),
            ));
        }
        let dir = self.changes_dir().join(change_id);
        if !dir.is_dir() {
            return Err(openspec_error(
                404,
                format!(
                    "openspec change '{change_id}' not found under {}",
                    self.changes_dir().display()
                ),
            ));
        }
        Ok(dir)
    }

    /// List active (non-archived) change ids, sorted
    pub fn list_change_ids(&self) -> Result<Vec<String>, ApiError> {
        let changes = self.changes_dir();
        let entries = std::fs::read_dir(&changes).map_err(|e| {
            openspec_error(
                404,
                format!(
                    "cannot read openspec changes dir {}: {e}",
                    changes.display()
                ),
            )
        })?;
        let mut ids: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name != ARCHIVE_DIR && !name.starts_with('.'))
            .collect();
        ids.sort();
        Ok(ids)
    }

    fn proposal_meta(&self, change_dir: &Path) -> ProposalMeta {
        std::fs::read_to_string(change_dir.join(PROPOSAL_FILE))
            .map(|content| parse_proposal(&content))
            .unwrap_or_default()
    }

    fn issue_for_group(
        &self,
        change_id: &str,
        change_dir: &Path,
        proposal: &ProposalMeta,
        group: &TaskGroup,
    ) -> ExternalIssue {
        let key = format!("{change_id}#{}", group.number);
        let status = if group.is_done() {
            OPENSPEC_STATUS_DONE
        } else {
            OPENSPEC_STATUS_TODO
        };

        let mut description = String::new();
        if let Some(title) = &proposal.title {
            description.push_str(&format!("OpenSpec change **{change_id}** — {title}\n\n"));
        } else {
            description.push_str(&format!("OpenSpec change **{change_id}**\n\n"));
        }
        if let Some(why) = &proposal.why_excerpt {
            description.push_str(&format!("## Why\n\n{why}\n\n"));
        }
        description.push_str(&format!("## Tasks ({})\n\n", group.title));
        for item in &group.items {
            let mark = if item.checked { "x" } else { " " };
            description.push_str(&format!("- [{mark}] {}\n", item.text));
        }
        description.push_str(&format!(
            "\n## Spec Context\n\nRead the full change (proposal, design, spec deltas) at `{}` before starting.\n",
            change_dir.display()
        ));

        ExternalIssue {
            id: key.clone(),
            key,
            summary: group.title.clone(),
            description: Some(description),
            kanban_issue_types: Vec::new(),
            status: status.to_string(),
            assignee: None,
            url: format!("file://{}", change_dir.join(TASKS_FILE).display()),
            priority: None,
        }
    }
}

#[async_trait]
impl KanbanProvider for OpenspecProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn is_configured(&self) -> bool {
        self.changes_dir().is_dir()
    }

    async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ApiError> {
        let ids = self.list_change_ids()?;
        Ok(ids
            .into_iter()
            .map(|id| {
                let name = self
                    .proposal_meta(&self.changes_dir().join(&id))
                    .title
                    .unwrap_or_else(|| id.clone());
                ProjectInfo {
                    id: id.clone(),
                    key: id,
                    name,
                }
            })
            .collect())
    }

    async fn get_issue_types(
        &self,
        _project_key: &str,
    ) -> Result<Vec<ExternalIssueType>, ApiError> {
        Ok(Vec::new())
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        Ok(self.changes_dir().is_dir())
    }

    async fn list_users(&self, _project_key: &str) -> Result<Vec<ExternalUser>, ApiError> {
        Ok(Vec::new())
    }

    async fn list_statuses(&self, _project_key: &str) -> Result<Vec<String>, ApiError> {
        Ok(vec![
            OPENSPEC_STATUS_TODO.to_string(),
            OPENSPEC_STATUS_DONE.to_string(),
        ])
    }

    async fn list_issues(
        &self,
        project_key: &str,
        _user_id: &str,
        statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError> {
        let change_dir = self.change_dir(project_key)?;
        let tasks_path = change_dir.join(TASKS_FILE);
        let content = std::fs::read_to_string(&tasks_path).map_err(|e| {
            openspec_error(404, format!("cannot read {}: {e}", tasks_path.display()))
        })?;
        let proposal = self.proposal_meta(&change_dir);
        let issues = parse_tasks_md(&content)
            .iter()
            .map(|group| self.issue_for_group(project_key, &change_dir, &proposal, group))
            .filter(|issue| statuses.is_empty() || statuses.contains(&issue.status))
            .collect();
        Ok(issues)
    }

    async fn create_issue(
        &self,
        _project_key: &str,
        _request: CreateIssueRequest,
    ) -> Result<CreateIssueResponse, ApiError> {
        Err(openspec_error(
            400,
            "openspec provider is pull-only; edit tasks.md directly".to_string(),
        ))
    }

    async fn update_issue_status(
        &self,
        _issue_key: &str,
        _request: UpdateStatusRequest,
    ) -> Result<ExternalIssue, ApiError> {
        Err(openspec_error(
            400,
            "openspec provider is pull-only; check off tasks.md directly".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TASKS: &str = "\
# Tasks

## 1. Theme Infrastructure
- [ ] 1.1 Create ThemeContext with light/dark state
- [x] 1.2 Add CSS custom properties for colors

## 2. UI Components
- [ ] 2.1 Create ThemeToggle component

## Notes
Some prose without checkboxes.
";

    #[test]
    fn test_parse_tasks_md_groups_and_items() {
        let groups = parse_tasks_md(SAMPLE_TASKS);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].number, 1);
        assert_eq!(groups[0].title, "Theme Infrastructure");
        assert_eq!(groups[0].items.len(), 2);
        assert!(!groups[0].items[0].checked);
        assert!(groups[0].items[1].checked);
        assert_eq!(
            groups[0].items[0].text,
            "1.1 Create ThemeContext with light/dark state"
        );
        assert_eq!(groups[1].number, 2);
        assert_eq!(groups[1].title, "UI Components");
    }

    #[test]
    fn test_parse_tasks_md_unnumbered_headings_get_ordinals() {
        let groups = parse_tasks_md("## Setup\n- [ ] do a thing\n## Teardown\n- [x] undo it\n");
        assert_eq!(groups.len(), 2);
        assert_eq!((groups[0].number, groups[0].title.as_str()), (1, "Setup"));
        assert_eq!(
            (groups[1].number, groups[1].title.as_str()),
            (2, "Teardown")
        );
    }

    #[test]
    fn test_parse_tasks_md_items_before_heading_form_implicit_group() {
        let groups = parse_tasks_md("- [ ] loose item\n- [X] another\n");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].number, 1);
        assert_eq!(groups[0].title, "Tasks");
        assert_eq!(groups[0].items.len(), 2);
        assert!(groups[0].items[1].checked);
    }

    #[test]
    fn test_parse_tasks_md_empty_and_prose_only() {
        assert!(parse_tasks_md("").is_empty());
        assert!(parse_tasks_md("# Tasks\n\njust prose\n\n## Heading\nmore prose\n").is_empty());
    }

    #[test]
    fn test_parse_tasks_md_ignores_code_fences() {
        let content =
            "## 1. Real\n- [ ] real item\n```\n- [ ] fake item in code\n## 9. fake\n```\n";
        let groups = parse_tasks_md(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].items.len(), 1);
    }

    #[test]
    fn test_task_group_is_done() {
        let groups =
            parse_tasks_md("## 1. A\n- [x] one\n- [x] two\n## 2. B\n- [x] one\n- [ ] two\n");
        assert!(groups[0].is_done());
        assert!(!groups[1].is_done());
    }

    #[test]
    fn test_parse_proposal_title_and_why() {
        let meta = parse_proposal(
            "# Proposal: Add dark mode\n\n## Why\n\nUsers asked.\nA lot.\n\n## What Changes\n\nstuff\n",
        );
        assert_eq!(meta.title.as_deref(), Some("Proposal: Add dark mode"));
        assert_eq!(meta.why_excerpt.as_deref(), Some("Users asked.\nA lot."));
    }

    #[test]
    fn test_parse_proposal_intent_section_and_missing() {
        let meta = parse_proposal("# T\n## Intent\nbecause\n");
        assert_eq!(meta.why_excerpt.as_deref(), Some("because"));
        assert_eq!(parse_proposal("no headings"), ProposalMeta::default());
    }

    // ── Provider tests over a fixture tree ──────────────────────────────────

    fn fixture_root() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let change = dir.path().join("changes/add-dark-mode");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(
            change.join("proposal.md"),
            "# Proposal: Add dark mode\n\n## Why\n\nUsers asked.\n",
        )
        .unwrap();
        std::fs::write(change.join("tasks.md"), SAMPLE_TASKS).unwrap();
        // Archived + hidden entries must be skipped
        std::fs::create_dir_all(dir.path().join("changes/archive/2026-01-01-old")).unwrap();
        std::fs::create_dir_all(dir.path().join("changes/.hidden")).unwrap();
        dir
    }

    #[tokio::test]
    async fn test_openspec_list_projects_skips_archive() {
        let root = fixture_root();
        let provider = OpenspecProvider::new("demo", root.path());
        let projects = provider.list_projects().await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].key, "add-dark-mode");
        assert_eq!(projects[0].name, "Proposal: Add dark mode");
    }

    #[tokio::test]
    async fn test_openspec_list_issues_maps_groups() {
        let root = fixture_root();
        let provider = OpenspecProvider::new("demo", root.path());
        let issues = provider
            .list_issues("add-dark-mode", "", &[])
            .await
            .unwrap();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].key, "add-dark-mode#1");
        assert_eq!(issues[0].summary, "Theme Infrastructure");
        assert_eq!(issues[0].status, OPENSPEC_STATUS_TODO);
        let desc = issues[0].description.as_deref().unwrap();
        assert!(desc.contains("Users asked."));
        assert!(desc.contains("- [ ] 1.1 Create ThemeContext"));
        assert!(desc.contains("## Spec Context"));
        assert!(issues[0].url.starts_with("file://"));
    }

    #[tokio::test]
    async fn test_openspec_list_issues_status_filter() {
        let root = fixture_root();
        let change = root.path().join("changes/add-dark-mode");
        std::fs::write(
            change.join("tasks.md"),
            "## 1. A\n- [x] done item\n## 2. B\n- [ ] open\n",
        )
        .unwrap();
        let provider = OpenspecProvider::new("demo", root.path());
        let todo = provider
            .list_issues("add-dark-mode", "", &[OPENSPEC_STATUS_TODO.to_string()])
            .await
            .unwrap();
        assert_eq!(todo.len(), 1);
        assert_eq!(todo[0].key, "add-dark-mode#2");
    }

    #[tokio::test]
    async fn test_openspec_unknown_change_and_path_traversal_rejected() {
        let root = fixture_root();
        let provider = OpenspecProvider::new("demo", root.path());
        assert!(provider.list_issues("nope", "", &[]).await.is_err());
        assert!(provider.list_issues("../etc", "", &[]).await.is_err());
    }

    #[tokio::test]
    async fn test_openspec_pull_only_stubs() {
        let root = fixture_root();
        let provider = OpenspecProvider::new("demo", root.path());
        let create = provider
            .create_issue(
                "add-dark-mode",
                CreateIssueRequest {
                    summary: "x".into(),
                    description: None,
                    assignee_id: None,
                    status: None,
                    priority: None,
                },
            )
            .await;
        assert!(create.is_err());
        let update = provider
            .update_issue_status(
                "add-dark-mode#1",
                UpdateStatusRequest {
                    status: "done".into(),
                },
            )
            .await;
        assert!(update.is_err());
    }
}
