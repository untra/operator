//! Kanban Sync Service - Syncs issues from external providers to local tickets.
//!
//! Supports syncing work items from Jira, Linear, and other kanban providers
//! into the local tickets queue.
//!
//! NOTE: Most sync methods are currently unused as actual provider API calls
//! are not yet implemented - this is infrastructure for future sync operations.

#![allow(dead_code)] // Infrastructure for future sync - provider APIs not yet implemented

use anyhow::{Context, Result};
use chrono::Local;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::api::providers::kanban::{get_provider, ExternalIssue, KanbanProvider};
use crate::config::{Config, ProjectSyncConfig};

/// A collection that can be synced from a kanban provider
#[derive(Debug, Clone)]
pub struct SyncableCollection {
    /// Provider name (e.g., "jira", "linear")
    pub provider: String,
    /// Project/team key in the provider
    pub project_key: String,
    /// IssueTypeCollection name in Operator
    pub collection_name: String,
    /// User ID to sync issues for
    pub sync_user_id: String,
    /// Statuses to sync (empty = default only)
    pub sync_statuses: Vec<String>,
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Tickets that were created
    pub created: Vec<String>,
    /// Issues that were skipped (already exist locally)
    pub skipped: Vec<String>,
    /// Issues that failed to sync
    pub errors: Vec<String>,
}

impl SyncResult {
    /// Check if the sync was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        format!(
            "Created: {}, Skipped: {}, Errors: {}",
            self.created.len(),
            self.skipped.len(),
            self.errors.len()
        )
    }
}

/// Service for syncing issues from kanban providers
pub struct KanbanSyncService {
    config: Config,
}

impl KanbanSyncService {
    /// Create a new sync service with the given config
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Get all configured syncable collections
    pub fn configured_collections(&self) -> Vec<SyncableCollection> {
        let mut collections = Vec::new();

        // Check all Jira instances (keyed by domain)
        for jira_config in self.config.kanban.jira.values() {
            if jira_config.enabled {
                for (project_key, project_config) in &jira_config.projects {
                    collections.push(SyncableCollection {
                        provider: "jira".to_string(),
                        project_key: project_key.clone(),
                        collection_name: project_config.collection_name.clone(),
                        sync_user_id: project_config.sync_user_id.clone(),
                        sync_statuses: project_config.sync_statuses.clone(),
                    });
                }
            }
        }

        // Check all Linear instances (keyed by workspace)
        for linear_config in self.config.kanban.linear.values() {
            if linear_config.enabled {
                for (project_key, project_config) in &linear_config.projects {
                    collections.push(SyncableCollection {
                        provider: "linear".to_string(),
                        project_key: project_key.clone(),
                        collection_name: project_config.collection_name.clone(),
                        sync_user_id: project_config.sync_user_id.clone(),
                        sync_statuses: project_config.sync_statuses.clone(),
                    });
                }
            }
        }

        collections
    }

    /// Sync issues from a specific collection
    pub async fn sync_collection(
        &self,
        provider_name: &str,
        project_key: &str,
    ) -> Result<SyncResult> {
        info!("Syncing collection: {}/{}", provider_name, project_key);

        let mut result = SyncResult::default();

        // Get the provider
        let provider = get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not configured", provider_name))?;

        // Get the project config
        let project_config = self
            .get_project_config(provider_name, project_key)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Project '{}' not configured for provider '{}'",
                    project_key,
                    provider_name
                )
            })?;

        // Fetch issues from the provider
        let issues = provider
            .list_issues(
                project_key,
                &project_config.sync_user_id,
                &project_config.sync_statuses,
            )
            .await
            .context("Failed to fetch issues from provider")?;

        info!(
            "Fetched {} issues from {}/{}",
            issues.len(),
            provider_name,
            project_key
        );

        // Get existing external IDs in the queue
        let existing_ids = self.get_existing_external_ids()?;

        // Create tickets for new issues
        for issue in issues {
            if existing_ids.contains(&issue.key) {
                debug!("Skipping existing issue: {}", issue.key);
                result.skipped.push(issue.key.clone());
                continue;
            }

            match self.create_ticket_from_issue(&issue, provider_name, project_key) {
                Ok(filename) => {
                    info!("Created ticket: {}", filename);
                    result.created.push(issue.key.clone());
                }
                Err(e) => {
                    warn!("Failed to create ticket for {}: {}", issue.key, e);
                    result.errors.push(format!("{}: {}", issue.key, e));
                }
            }
        }

        Ok(result)
    }

    /// Get project config for a provider/project combination
    ///
    /// Searches across all instances of the provider to find the project.
    fn get_project_config(
        &self,
        provider_name: &str,
        project_key: &str,
    ) -> Option<ProjectSyncConfig> {
        match provider_name {
            "jira" => {
                for jira_config in self.config.kanban.jira.values() {
                    if let Some(config) = jira_config.projects.get(project_key) {
                        return Some(config.clone());
                    }
                }
                None
            }
            "linear" => {
                for linear_config in self.config.kanban.linear.values() {
                    if let Some(config) = linear_config.projects.get(project_key) {
                        return Some(config.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Get set of external IDs that already exist in the queue
    fn get_existing_external_ids(&self) -> Result<HashSet<String>> {
        let queue_path = Path::new(&self.config.paths.tickets).join("queue");
        let mut ids = HashSet::new();

        if !queue_path.exists() {
            return Ok(ids);
        }

        for entry in fs::read_dir(&queue_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                // Try to read external_id from frontmatter
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(external_id) = extract_external_id(&content) {
                        ids.insert(external_id);
                    }
                }
            }
        }

        Ok(ids)
    }

    /// Create a ticket file from an external issue
    fn create_ticket_from_issue(
        &self,
        issue: &ExternalIssue,
        provider: &str,
        project_key: &str,
    ) -> Result<String> {
        let queue_path = Path::new(&self.config.paths.tickets).join("queue");
        fs::create_dir_all(&queue_path)?;

        // Generate filename: YYYYMMDD-HHMM-TYPE-PROJECT-summary.md
        let now = Local::now();
        let timestamp = now.format("%Y%m%d-%H%M").to_string();
        let ticket_type = map_issue_type_to_operator(&issue.issue_type);
        let slug = slugify(&issue.summary, 50);
        let filename = format!("{}-{}-{}-{}.md", timestamp, ticket_type, project_key, slug);

        // Build frontmatter
        let frontmatter = format!(
            r#"---
id: {}-{}
status: queued
priority: {}
step: plan
external_id: {}
external_url: {}
external_provider: {}
---"#,
            ticket_type,
            issue.key.replace('-', ""),
            map_priority(&issue.priority),
            issue.key,
            issue.url,
            provider,
        );

        // Build content
        let description = issue
            .description
            .as_deref()
            .unwrap_or("No description provided.");
        let content = format!(
            r#"{}

# {}: {}

{}

## Source

- **Provider**: {}
- **Issue**: [{}]({})
"#,
            frontmatter, ticket_type, issue.summary, description, provider, issue.key, issue.url,
        );

        let file_path = queue_path.join(&filename);
        fs::write(&file_path, content)?;

        Ok(filename)
    }
}

/// Extract external_id from ticket content frontmatter
fn extract_external_id(content: &str) -> Option<String> {
    // Simple extraction - look for "external_id: <value>" in frontmatter
    if !content.starts_with("---") {
        return None;
    }

    for line in content.lines().skip(1) {
        if line == "---" {
            break;
        }
        if let Some(id) = line.strip_prefix("external_id:") {
            return Some(id.trim().to_string());
        }
    }

    None
}

/// Map external issue type to Operator type
fn map_issue_type_to_operator(issue_type: &str) -> &'static str {
    match issue_type.to_lowercase().as_str() {
        "bug" | "fix" | "defect" => "FIX",
        "feature" | "story" | "user story" | "enhancement" => "FEAT",
        "spike" | "research" | "investigation" => "SPIKE",
        "task" | "sub-task" | "subtask" => "TASK",
        _ => "TASK", // Default to TASK for unknown types
    }
}

/// Map external priority to Operator priority
fn map_priority(priority: &Option<String>) -> &'static str {
    match priority.as_deref().map(|s| s.to_lowercase()).as_deref() {
        Some("highest") | Some("critical") | Some("urgent") | Some("p0") => "P0-critical",
        Some("high") | Some("p1") => "P1-high",
        Some("medium") | Some("normal") | Some("p2") => "P2-medium",
        Some("low") | Some("p3") => "P3-low",
        Some("lowest") | Some("trivial") | Some("p4") => "P4-trivial",
        _ => "P2-medium", // Default to medium
    }
}

/// Convert a string to a URL-safe slug
fn slugify(s: &str, max_len: usize) -> String {
    let slug: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.len() <= max_len {
        slug
    } else {
        // Truncate at word boundary if possible
        let truncated = &slug[..max_len];
        if let Some(last_dash) = truncated.rfind('-') {
            truncated[..last_dash].to_string()
        } else {
            truncated.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_issue_type_to_operator() {
        assert_eq!(map_issue_type_to_operator("Bug"), "FIX");
        assert_eq!(map_issue_type_to_operator("bug"), "FIX");
        assert_eq!(map_issue_type_to_operator("Feature"), "FEAT");
        assert_eq!(map_issue_type_to_operator("Story"), "FEAT");
        assert_eq!(map_issue_type_to_operator("Spike"), "SPIKE");
        assert_eq!(map_issue_type_to_operator("Task"), "TASK");
        assert_eq!(map_issue_type_to_operator("Unknown"), "TASK");
    }

    #[test]
    fn test_map_priority() {
        assert_eq!(map_priority(&Some("Highest".to_string())), "P0-critical");
        assert_eq!(map_priority(&Some("high".to_string())), "P1-high");
        assert_eq!(map_priority(&Some("medium".to_string())), "P2-medium");
        assert_eq!(map_priority(&Some("low".to_string())), "P3-low");
        assert_eq!(map_priority(&Some("lowest".to_string())), "P4-trivial");
        assert_eq!(map_priority(&None), "P2-medium");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World", 50), "hello-world");
        assert_eq!(slugify("Fix: Login Bug!", 50), "fix-login-bug");
        assert_eq!(slugify("  Multiple   Spaces  ", 50), "multiple-spaces");
    }

    #[test]
    fn test_slugify_truncation() {
        let long_string = "this is a very long string that should be truncated at a word boundary";
        let slug = slugify(long_string, 30);
        assert!(slug.len() <= 30);
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn test_extract_external_id() {
        let content = r#"---
id: FEAT-123
status: queued
external_id: PROJ-456
external_url: https://example.com
---

# Content here
"#;
        assert_eq!(extract_external_id(content), Some("PROJ-456".to_string()));
    }

    #[test]
    fn test_extract_external_id_missing() {
        let content = r#"---
id: FEAT-123
status: queued
---

# No external_id
"#;
        assert_eq!(extract_external_id(content), None);
    }

    #[test]
    fn test_extract_external_id_no_frontmatter() {
        let content = "# Just a title\n\nNo frontmatter here.";
        assert_eq!(extract_external_id(content), None);
    }

    #[test]
    fn test_sync_result_summary() {
        let mut result = SyncResult::default();
        result.created.push("PROJ-1".to_string());
        result.created.push("PROJ-2".to_string());
        result.skipped.push("PROJ-3".to_string());

        assert!(result.is_success());
        assert_eq!(result.summary(), "Created: 2, Skipped: 1, Errors: 0");
    }

    #[test]
    fn test_sync_result_with_errors() {
        let mut result = SyncResult::default();
        result.errors.push("PROJ-1: Failed".to_string());

        assert!(!result.is_success());
    }
}
