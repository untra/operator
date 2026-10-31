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
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::api::providers::kanban::{
    get_provider, get_provider_from_config, ExternalIssue, KanbanProvider, OpenspecProvider,
    ScopeContents,
};
use crate::campaigns::{CampaignStore, NewCampaign};
use crate::config::{Config, KanbanStatusMapping, ProjectSyncConfig, SyncScope};
use crate::issuetypes::kanban_type::KanbanIssueTypeRef;
use crate::licensing::PremiumFeature;
use crate::queue::{Ticket, TicketPriority};

/// A collection that can be synced from a kanban provider
#[derive(Debug, Clone)]
pub struct SyncableCollection {
    /// Provider name (e.g., "jira", "linear")
    pub provider: String,
    /// Project/team key in the provider
    pub project_key: String,
    /// Optional `IssueTypeCollection` name in Operator
    pub collection_name: Option<String>,
    /// User ID to sync issues for
    pub sync_user_id: String,
    /// Mapping of operator todo/doing/done to external board columns
    pub status_mapping: KanbanStatusMapping,
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Tickets that were created
    pub created: Vec<String>,
    /// Issues that were skipped (already exist locally, nothing changed)
    pub skipped: Vec<String>,
    /// Existing tickets whose synced relationships were refreshed
    pub updated: Vec<String>,
    /// Campaign the synced tickets were stamped with (scoped syncs)
    pub campaign: Option<String>,
    /// Issues that failed to sync
    pub errors: Vec<String>,
    /// Total number of issues processed
    pub total_processed: usize,
}

impl SyncResult {
    /// Check if the sync was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        format!(
            "Created: {}, Updated: {}, Skipped: {}, Errors: {}",
            self.created.len(),
            self.updated.len(),
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

    /// Get all configured syncable collections.
    ///
    /// Only *external* providers appear here. The built-in Operator board is the
    /// sync destination - listing it would make every sync re-import operator's
    /// own tickets. Pinned by `test_configured_collections_never_include_the_builtin_board`.
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
                        status_mapping: project_config.status_mapping.clone(),
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
                        status_mapping: project_config.status_mapping.clone(),
                    });
                }
            }
        }

        // Check all GitHub Projects instances (keyed by owner login)
        for github_config in self.config.kanban.github.values() {
            if github_config.enabled {
                for (project_key, project_config) in &github_config.projects {
                    collections.push(SyncableCollection {
                        provider: "github".to_string(),
                        project_key: project_key.clone(),
                        collection_name: project_config.collection_name.clone(),
                        sync_user_id: project_config.sync_user_id.clone(),
                        status_mapping: project_config.status_mapping.clone(),
                    });
                }
            }
        }

        for cfg in self.config.kanban.gitea.values().filter(|cfg| cfg.enabled) {
            for (key, project) in &cfg.projects {
                collections.push(SyncableCollection {
                    provider: "gitea".into(),
                    project_key: key.clone(),
                    collection_name: project.collection_name.clone(),
                    sync_user_id: project.sync_user_id.clone(),
                    status_mapping: project.status_mapping.clone(),
                });
            }
        }

        // OpenSpec instances: each active change directory is a syncable collection
        for (instance, openspec_config) in &self.config.kanban.openspec {
            if !openspec_config.enabled {
                continue;
            }
            let provider = OpenspecProvider::from_config(instance, openspec_config);
            match provider.list_change_ids() {
                Ok(change_ids) => {
                    for change_id in change_ids {
                        collections.push(SyncableCollection {
                            provider: "openspec".to_string(),
                            project_key: change_id,
                            collection_name: None,
                            sync_user_id: String::new(),
                            status_mapping: openspec_status_mapping(),
                        });
                    }
                }
                Err(e) => {
                    warn!("Cannot enumerate openspec changes for '{instance}': {e}");
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

        let project_config = self
            .get_project_config(provider_name, project_key)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Project '{project_key}' not configured for provider '{provider_name}'"
                )
            })?;
        if let Some(scope) = project_config.scope.clone() {
            return self.sync_scope(provider_name, project_key, &scope).await;
        }
        let provider = self.provider(provider_name, project_key)?;

        // Fetch issues from the provider
        let issues = provider
            .list_issues(
                project_key,
                &project_config.sync_user_id,
                &project_config.pull_statuses(),
            )
            .await
            .context("Failed to fetch issues from provider")?;

        info!(
            "Fetched {} issues from {}/{}",
            issues.len(),
            provider_name,
            project_key
        );

        Ok(self.apply_issues(provider_name, project_key, &project_config, issues))
    }

    /// Env-configured provider first, then config-backed (openspec is config-only).
    fn provider(&self, provider_name: &str, project_key: &str) -> Result<Box<dyn KanbanProvider>> {
        match get_provider(provider_name) {
            Some(p) => Ok(p),
            None => get_provider_from_config(&self.config.kanban, provider_name, project_key)
                .map_err(|e| anyhow::anyhow!("Provider '{provider_name}' not configured: {e}")),
        }
    }

    /// Sync one upstream container into a campaign (Premium).
    pub async fn sync_scope(
        &self,
        provider_name: &str,
        project_key: &str,
        scope: &SyncScope,
    ) -> Result<SyncResult> {
        crate::licensing::require_premium(&self.config, PremiumFeature::Campaigns)?;
        let project_config = self
            .get_project_config(provider_name, project_key)
            .unwrap_or_default();
        let contents = self
            .provider(provider_name, project_key)?
            .list_scope(scope, &project_config.pull_statuses())
            .await
            .context("Failed to fetch scope from provider")?;
        info!(
            "Fetched {} issues in {provider_name}:{}",
            contents.issues.len(),
            contents.key
        );
        self.apply_scope(provider_name, project_key, &project_config, contents)
    }

    /// Find or create the campaign for a fetched scope and stamp its issues into it.
    pub fn apply_scope(
        &self,
        provider_name: &str,
        project_key: &str,
        project_config: &ProjectSyncConfig,
        contents: ScopeContents,
    ) -> Result<SyncResult> {
        crate::licensing::require_premium(&self.config, PremiumFeature::Campaigns)?;
        self.stamp_scope(provider_name, project_key, project_config, contents)
    }

    fn stamp_scope(
        &self,
        provider_name: &str,
        project_key: &str,
        project_config: &ProjectSyncConfig,
        contents: ScopeContents,
    ) -> Result<SyncResult> {
        let store = CampaignStore::new(&self.config);
        let external_ref = format!("{provider_name}:{}", contents.key);
        let campaign = match store.find_by_external_ref(&external_ref) {
            Some(campaign) => campaign,
            None => store.create(NewCampaign {
                title: contents.title,
                goal: contents.description,
                external_ref: Some(external_ref),
                ..Default::default()
            })?,
        };
        let mut result = self.apply_issues_in(
            provider_name,
            project_key,
            project_config,
            contents.issues,
            Some(&campaign.id),
        );
        result.campaign = Some(campaign.id);
        Ok(result)
    }

    /// Write fetched issues as tickets: create new ones, refresh relationships on existing ones.
    pub fn apply_issues(
        &self,
        provider_name: &str,
        project_key: &str,
        project_config: &ProjectSyncConfig,
        issues: Vec<ExternalIssue>,
    ) -> SyncResult {
        self.apply_issues_in(provider_name, project_key, project_config, issues, None)
    }

    fn apply_issues_in(
        &self,
        provider_name: &str,
        project_key: &str,
        project_config: &ProjectSyncConfig,
        issues: Vec<ExternalIssue>,
        campaign: Option<&str>,
    ) -> SyncResult {
        let mut result = SyncResult::default();
        let mut existing = self.existing_tickets_by_external_id(provider_name);

        for issue in issues {
            if let Some(ticket) = existing.get_mut(&issue.key) {
                match refresh_relationships(ticket, provider_name, &issue, campaign) {
                    Ok(true) => result.updated.push(issue.key.clone()),
                    Ok(false) => {
                        debug!("Skipping unchanged issue: {}", issue.key);
                        result.skipped.push(issue.key.clone());
                    }
                    Err(e) => result.errors.push(format!("{}: {e}", issue.key)),
                }
                continue;
            }

            let type_mappings = if project_config.type_mappings.is_empty() {
                None
            } else {
                Some(&project_config.type_mappings)
            };
            match self.create_ticket_from_issue(
                &issue,
                provider_name,
                ticket_project(project_config, project_key).as_str(),
                type_mappings,
                project_config.collection_name.as_deref(),
                campaign,
            ) {
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

        result.total_processed = result.created.len()
            + result.updated.len()
            + result.skipped.len()
            + result.errors.len();
        result
    }

    /// Sync all configured collections
    ///
    /// Iterates through all configured kanban providers/projects and syncs them.
    pub async fn sync_all(&self) -> Result<SyncResult> {
        info!("Starting sync of all configured collections");

        let mut combined = SyncResult::default();
        let collections = self.configured_collections();

        if collections.is_empty() {
            info!("No kanban collections configured");
            return Ok(combined);
        }

        for collection in collections {
            info!(
                "Syncing collection: {}/{}",
                collection.provider, collection.project_key
            );

            match self
                .sync_collection(&collection.provider, &collection.project_key)
                .await
            {
                Ok(result) => {
                    combined.created.extend(result.created);
                    combined.skipped.extend(result.skipped);
                    combined.errors.extend(result.errors);
                }
                Err(e) => {
                    warn!(
                        "Failed to sync {}/{}: {}",
                        collection.provider, collection.project_key, e
                    );
                    combined.errors.push(format!(
                        "{}/{}: {}",
                        collection.provider, collection.project_key, e
                    ));
                }
            }
        }

        combined.total_processed =
            combined.created.len() + combined.skipped.len() + combined.errors.len();

        info!("Sync complete: {}", combined.summary());
        Ok(combined)
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
            "gitea" => {
                let mut configs = self
                    .config
                    .kanban
                    .gitea
                    .values()
                    .filter(|cfg| cfg.enabled)
                    .filter_map(|cfg| cfg.projects.get(project_key));
                let config = configs.next()?.clone();
                if configs.next().is_some() {
                    return None;
                }
                Some(config)
            }
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
            "github" => {
                for github_config in self.config.kanban.github.values() {
                    if let Some(config) = github_config.projects.get(project_key) {
                        return Some(config.clone());
                    }
                }
                None
            }
            // OpenSpec has no per-change config; synthesize one from the
            // enabled instance (pull unchecked groups only).
            "openspec" => {
                let cfg = self.config.kanban.openspec.values().find(|c| c.enabled)?;
                Some(ProjectSyncConfig {
                    status_mapping: openspec_status_mapping(),
                    ticket_project: cfg.project.clone(),
                    ..Default::default()
                })
            }
            _ => None,
        }
    }

    /// This provider's synced tickets in queue, in-progress and completed, keyed by external id.
    fn existing_tickets_by_external_id(&self, provider: &str) -> HashMap<String, Ticket> {
        let tickets = Path::new(&self.config.paths.tickets);
        ["queue", "in-progress", "completed"]
            .iter()
            .filter_map(|dir| fs::read_dir(tickets.join(dir)).ok())
            .flatten()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.extension().is_some_and(|e| e == "md"))
            .filter_map(|path| Ticket::from_file(&path).ok())
            .filter(|t| t.external_provider.as_deref().is_none_or(|p| p == provider))
            .filter_map(|t| Some((t.external_id.clone()?, t)))
            .collect()
    }

    /// Create a ticket file from an external issue
    fn create_ticket_from_issue(
        &self,
        issue: &ExternalIssue,
        provider: &str,
        project_key: &str,
        type_mappings: Option<&std::collections::HashMap<String, String>>,
        collection: Option<&str>,
        campaign: Option<&str>,
    ) -> Result<String> {
        let queue_path = Path::new(&self.config.paths.tickets).join("queue");
        fs::create_dir_all(&queue_path)?;

        // Generate filename: YYYYMMDD-HHMM-TYPE-PROJECT-summary.md
        let now = Local::now();
        let timestamp = now.format("%Y%m%d-%H%M").to_string();
        // Resolve operator type from kanban issue type refs via type_mappings,
        // falling back to TASK with needs_issuetype_mapping flag
        let (ticket_type, needs_mapping) =
            resolve_ticket_type(&issue.kanban_issue_types, type_mappings);
        let slug = crate::queue::ids::slugify(&issue.summary, 50);
        let file_path = crate::queue::ids::unique_path(
            &queue_path,
            &format!("{timestamp}-{ticket_type}-{project_key}-{slug}"),
        );
        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut frontmatter =
            ticket_frontmatter(ticket_type, issue, provider, needs_mapping, collection);
        let (depends_on, external_parent) = relationship_refs(provider, issue);
        frontmatter = frontmatter.trim_end_matches("---").to_string()
            + &relationship_yaml(&depends_on, external_parent.as_deref(), campaign)
            + "---";

        // Build content
        let description = issue
            .description
            .as_deref()
            .unwrap_or("No description provided.");
        let content = format!(
            r"{}

# {}: {}

{}

## Source

- **Provider**: {}
- **Issue**: [{}]({})
",
            frontmatter, ticket_type, issue.summary, description, provider, issue.key, issue.url,
        );

        fs::write(&file_path, content)?;

        Ok(filename)
    }
}

/// Build the YAML frontmatter for a ticket created from an external issue.
///
/// `collection` is the project sync's issuetype collection
/// (`ProjectSyncConfig.collection_name`): when set, the ticket's type
/// resolves within that collection.
fn ticket_frontmatter(
    ticket_type: &str,
    issue: &ExternalIssue,
    provider: &str,
    needs_mapping: bool,
    collection: Option<&str>,
) -> String {
    let needs_mapping_line = if needs_mapping {
        "\nneeds_issuetype_mapping: true"
    } else {
        ""
    };
    let collection_line = collection
        .map(|c| format!("\ncollection: {c}"))
        .unwrap_or_default();
    format!(
        r"---
id: {}-{}
status: queued
priority: {}
step: plan{}
external_id: {}
external_url: {}
external_provider: {}{}
---",
        ticket_type,
        sanitize_external_key(&issue.key),
        map_priority(&issue.priority),
        collection_line,
        issue.key,
        issue.url,
        provider,
        needs_mapping_line,
    )
}

/// Provider-qualified relationship refs for an issue: `(depends_on, external_parent)`.
///
/// Blockers and sub-tasks both gate the issue, so both become dependencies.
pub fn relationship_refs(provider: &str, issue: &ExternalIssue) -> (Vec<String>, Option<String>) {
    let depends_on: BTreeSet<String> = issue
        .blocked_by
        .iter()
        .chain(&issue.subtasks)
        .filter(|key| **key != issue.key)
        .map(|key| format!("{provider}:{key}"))
        .collect();
    let parent = issue
        .parent
        .as_ref()
        .map(|p| format!("{provider}:{}", p.key));
    (depends_on.into_iter().collect(), parent)
}

/// Replace this provider's synced refs, keeping local ids and other providers' refs.
pub fn merge_depends_on(existing: &[String], provider: &str, synced: &[String]) -> Vec<String> {
    let prefix = format!("{provider}:");
    let kept = existing.iter().filter(|r| !r.starts_with(&prefix));
    let mut merged: Vec<String> = Vec::new();
    for r in kept.chain(synced) {
        if !merged.contains(r) {
            merged.push(r.clone());
        }
    }
    merged
}

/// Refresh synced relationships on an existing ticket; returns whether anything changed.
fn refresh_relationships(
    ticket: &mut Ticket,
    provider: &str,
    issue: &ExternalIssue,
    campaign: Option<&str>,
) -> Result<bool> {
    let (synced, parent) = relationship_refs(provider, issue);
    let merged = merge_depends_on(&ticket.depends_on, provider, &synced);
    let mut changed = false;
    if merged != ticket.depends_on {
        ticket.set_depends_on(&merged)?;
        changed = true;
    }
    if let Some(parent) = parent.filter(|p| ticket.external_parent.as_ref() != Some(p)) {
        ticket.update_field("external_parent", &parent)?;
        changed = true;
    }
    if let Some(campaign) = campaign.filter(|_| ticket.campaign.is_none()) {
        ticket.set_campaign(Some(campaign))?;
        changed = true;
    }
    Ok(changed)
}

/// YAML lines for relationship fields; values are JSON-quoted, which is valid YAML.
fn relationship_yaml(
    depends_on: &[String],
    external_parent: Option<&str>,
    campaign: Option<&str>,
) -> String {
    let quote = |v: &str| serde_json::to_string(v).unwrap_or_default();
    let mut yaml = String::new();
    if let Some(campaign) = campaign {
        yaml.push_str(&format!("campaign: {}\n", quote(campaign)));
    }
    if let Some(parent) = external_parent {
        yaml.push_str(&format!("external_parent: {}\n", quote(parent)));
    }
    if !depends_on.is_empty() {
        yaml.push_str("depends_on:\n");
        for r in depends_on {
            yaml.push_str(&format!("  - {}\n", quote(r)));
        }
    }
    yaml
}

/// Extract `external_id` from ticket content frontmatter
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

/// Resolve operator issuetype from kanban issue type refs.
///
/// Uses `type_mappings` (keyed by provider type ID) to resolve.
/// For Linear: sorts labels by name, picks first mapped label.
/// Returns `(operator_key, needs_mapping)` -- if unmapped, returns `("TASK", true)`.
fn resolve_ticket_type(
    kanban_refs: &[KanbanIssueTypeRef],
    type_mappings: Option<&std::collections::HashMap<String, String>>,
) -> (&'static str, bool) {
    if let Some(mappings) = type_mappings {
        // Sort refs by name for deterministic resolution (important for Linear labels)
        let mut sorted_refs: Vec<_> = kanban_refs.iter().collect();
        sorted_refs.sort_by(|a, b| a.name.cmp(&b.name));

        for r in &sorted_refs {
            if let Some(operator_key) = mappings.get(&r.id) {
                return (leak_string(operator_key), false);
            }
        }
    }

    // No mapping found -- fallback to TASK with mapping nudge
    ("TASK", true)
}

/// Leak a string to get a `&'static str`.
/// Used for dynamic operator keys from `type_mappings`.
fn leak_string(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Map external priority to Operator priority.
///
/// A provider's "lowest" tier folds into `P3-low`: operator's set is closed at
/// four, and a fifth value would not validate against the metadata schema.
fn map_priority(priority: &Option<String>) -> TicketPriority {
    match priority.as_deref().map(str::to_lowercase).as_deref() {
        Some("highest" | "critical" | "urgent" | "p0") => TicketPriority::P0Critical,
        Some("high" | "p1") => TicketPriority::P1High,
        Some("low" | "p3" | "lowest" | "trivial" | "p4") => TicketPriority::P3Low,
        _ => TicketPriority::P2Medium,
    }
}

/// Default status mapping for openspec: pull unchecked ("todo") groups only
fn openspec_status_mapping() -> crate::config::KanbanStatusMapping {
    crate::config::KanbanStatusMapping {
        todo: Some(crate::api::providers::kanban::OPENSPEC_STATUS_TODO.to_string()),
        doing: None,
        done: None,
    }
}

/// Effective operator project for tickets from this source. Filename parsing
/// splits on hyphens, so the project component must not contain any.
pub(crate) fn ticket_project(config: &ProjectSyncConfig, project_key: &str) -> String {
    let raw = config.ticket_project.as_deref().unwrap_or(project_key);
    let cleaned: String = raw.chars().filter(char::is_ascii_alphanumeric).collect();
    if cleaned.is_empty() {
        "external".to_string()
    } else {
        cleaned
    }
}

/// External issue keys become the numeric-ish part of the ticket id; keep it
/// to alphanumerics so ids stay one hyphen-delimited token.
fn sanitize_external_key(key: &str) -> String {
    key.chars().filter(char::is_ascii_alphanumeric).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue() -> ExternalIssue {
        ExternalIssue {
            blocked_by: Vec::new(),
            parent: None,
            subtasks: Vec::new(),
            id: "10042".to_string(),
            key: "PROJ-42".to_string(),
            summary: "Fix the widget".to_string(),
            description: None,
            kanban_issue_types: vec![],
            status: "To Do".to_string(),
            assignee: None,
            url: "https://example.atlassian.net/browse/PROJ-42".to_string(),
            priority: Some("Medium".to_string()),
        }
    }

    /// The built-in board is the sync destination. If it ever became a
    /// syncable collection, every sync would re-import operator's own tickets.
    #[test]
    fn test_configured_collections_never_include_the_builtin_board() {
        let mut config = Config::default();
        config.kanban = crate::config::KanbanConfig::default();
        let collections = KanbanSyncService::new(&config).configured_collections();
        assert!(!collections.iter().any(|c| c.provider == "operator"));
    }

    #[test]
    fn test_frontmatter_stamps_sync_collection() {
        let fm = ticket_frontmatter("FIX", &sample_issue(), "jira", false, Some("devops_kanban"));
        assert!(fm.contains("collection: devops_kanban"));
        assert!(fm.contains("external_provider: jira"));
    }

    #[test]
    fn test_frontmatter_omits_collection_when_unset() {
        let fm = ticket_frontmatter("FIX", &sample_issue(), "jira", true, None);
        assert!(!fm.contains("collection:"));
        assert!(fm.contains("needs_issuetype_mapping: true"));
    }

    #[test]
    fn test_resolve_ticket_type_with_mapping() {
        let mut mappings = std::collections::HashMap::new();
        mappings.insert("10001".to_string(), "FIX".to_string());
        mappings.insert("10002".to_string(), "FEAT".to_string());

        let refs = vec![KanbanIssueTypeRef {
            id: "10001".to_string(),
            name: "Bug".to_string(),
        }];
        let (key, needs) = resolve_ticket_type(&refs, Some(&mappings));
        assert_eq!(key, "FIX");
        assert!(!needs);
    }

    #[test]
    fn test_resolve_ticket_type_no_mapping() {
        let refs = vec![KanbanIssueTypeRef {
            id: "10001".to_string(),
            name: "Bug".to_string(),
        }];
        let (key, needs) = resolve_ticket_type(&refs, None);
        assert_eq!(key, "TASK");
        assert!(needs);
    }

    #[test]
    fn test_resolve_ticket_type_unmapped_id() {
        let mut mappings = std::collections::HashMap::new();
        mappings.insert("10002".to_string(), "FEAT".to_string());

        let refs = vec![KanbanIssueTypeRef {
            id: "10001".to_string(),
            name: "Bug".to_string(),
        }];
        let (key, needs) = resolve_ticket_type(&refs, Some(&mappings));
        assert_eq!(key, "TASK");
        assert!(needs);
    }

    #[test]
    fn test_resolve_ticket_type_linear_labels_sorted() {
        let mut mappings = std::collections::HashMap::new();
        mappings.insert("label-bug".to_string(), "FIX".to_string());
        mappings.insert("label-feat".to_string(), "FEAT".to_string());

        // Multiple labels -- should sort by name and pick first mapped
        let refs = vec![
            KanbanIssueTypeRef {
                id: "label-feat".to_string(),
                name: "Feature".to_string(),
            },
            KanbanIssueTypeRef {
                id: "label-bug".to_string(),
                name: "Bug".to_string(),
            },
        ];
        // Sorted by name: Bug < Feature, so Bug matches first -> FIX
        let (key, needs) = resolve_ticket_type(&refs, Some(&mappings));
        assert_eq!(key, "FIX");
        assert!(!needs);
    }

    #[test]
    fn test_resolve_ticket_type_empty_refs() {
        let (key, needs) = resolve_ticket_type(&[], None);
        assert_eq!(key, "TASK");
        assert!(needs);
    }

    #[test]
    fn test_map_priority() {
        assert_eq!(
            map_priority(&Some("Highest".to_string())),
            TicketPriority::P0Critical
        );
        assert_eq!(
            map_priority(&Some("high".to_string())),
            TicketPriority::P1High
        );
        assert_eq!(
            map_priority(&Some("medium".to_string())),
            TicketPriority::P2Medium
        );
        assert_eq!(
            map_priority(&Some("low".to_string())),
            TicketPriority::P3Low
        );
        // A provider's lowest tier has no operator equivalent; it folds into low.
        assert_eq!(
            map_priority(&Some("lowest".to_string())),
            TicketPriority::P3Low
        );
        assert_eq!(map_priority(&None), TicketPriority::P2Medium);
    }

    #[test]
    fn test_extract_external_id() {
        let content = r"---
id: FEAT-123
status: queued
external_id: PROJ-456
external_url: https://example.com
---

# Content here
";
        assert_eq!(extract_external_id(content), Some("PROJ-456".to_string()));
    }

    #[test]
    fn test_extract_external_id_missing() {
        let content = r"---
id: FEAT-123
status: queued
---

# No external_id
";
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
        assert_eq!(
            result.summary(),
            "Created: 2, Updated: 0, Skipped: 1, Errors: 0"
        );
    }

    #[test]
    fn test_sync_result_with_errors() {
        let mut result = SyncResult::default();
        result.errors.push("PROJ-1: Failed".to_string());

        assert!(!result.is_success());
    }

    // ── OpenSpec end-to-end sync ────────────────────────────────────────────

    fn openspec_test_config(root: &std::path::Path) -> Config {
        let mut config = Config::default();
        config.paths.tickets = root.join(".tickets").to_string_lossy().into_owned();
        config.kanban.openspec.insert(
            "demo".to_string(),
            crate::config::OpenspecConfig {
                enabled: true,
                root_path: root.join("openspec").to_string_lossy().into_owned(),
                project: Some("myproj".to_string()),
            },
        );
        config
    }

    fn write_openspec_fixture(root: &std::path::Path) {
        let change = root.join("openspec/changes/add-dark-mode");
        fs::create_dir_all(&change).unwrap();
        fs::write(
            change.join("proposal.md"),
            "# Proposal: Add dark mode\n\n## Why\n\nUsers asked.\n",
        )
        .unwrap();
        fs::write(
            change.join("tasks.md"),
            "# Tasks\n\n## 1. Theme Infrastructure\n- [ ] 1.1 Create ThemeContext\n\n## 2. UI Components\n- [ ] 2.1 Create toggle\n\n## 3. Done Already\n- [x] 3.1 finished\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("openspec/changes/archive/2026-01-01-old")).unwrap();
    }

    #[tokio::test]
    async fn test_openspec_sync_creates_tickets_and_reimport_skips() {
        let temp = tempfile::tempdir().unwrap();
        write_openspec_fixture(temp.path());
        let config = openspec_test_config(temp.path());
        let service = KanbanSyncService::new(&config);

        // Configured collections enumerate the active change (not the archive)
        let collections = service.configured_collections();
        let openspec: Vec<_> = collections
            .iter()
            .filter(|c| c.provider == "openspec")
            .collect();
        assert_eq!(openspec.len(), 1);
        assert_eq!(openspec[0].project_key, "add-dark-mode");

        let result = service
            .sync_collection("openspec", "add-dark-mode")
            .await
            .unwrap();
        // Group 3 is fully checked ("done") and filtered out by the pull statuses
        assert_eq!(result.created.len(), 2, "errors: {:?}", result.errors);
        assert!(result.created.contains(&"add-dark-mode#1".to_string()));

        // Ticket files land in the queue with openspec provenance and the
        // configured operator project (not the hyphenated change id)
        let queue = temp.path().join(".tickets/queue");
        let files: Vec<String> = fs::read_dir(&queue)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.contains("-myproj-")), "{files:?}");

        let content =
            fs::read_to_string(queue.join(files.iter().find(|f| f.contains("theme")).unwrap()))
                .unwrap();
        assert!(content.contains("external_provider: openspec"));
        assert!(content.contains("external_id: add-dark-mode#1"));
        assert!(content.contains("- [ ] 1.1 Create ThemeContext"));
        assert!(content.contains("Users asked."));
        let ui = Ticket::from_file(&queue.join(files.iter().find(|f| f.contains("ui")).unwrap()))
            .unwrap();
        assert_eq!(ui.depends_on, vec!["openspec:add-dark-mode#1"]);
        assert_eq!(
            ui.external_parent.as_deref(),
            Some("openspec:add-dark-mode")
        );

        // Re-import is idempotent: everything skips, nothing new is created
        let rerun = service
            .sync_collection("openspec", "add-dark-mode")
            .await
            .unwrap();
        assert!(rerun.created.is_empty());
        assert_eq!(rerun.skipped.len(), 2);
        assert_eq!(fs::read_dir(&queue).unwrap().count(), 2);
    }

    #[tokio::test]
    async fn test_openspec_sync_unknown_change_errors() {
        let temp = tempfile::tempdir().unwrap();
        write_openspec_fixture(temp.path());
        let config = openspec_test_config(temp.path());
        let service = KanbanSyncService::new(&config);

        assert!(service
            .sync_collection("openspec", "no-such-change")
            .await
            .is_err());
    }

    // ── Relationship sync ───────────────────────────────────────────────────

    fn sync_service(root: &std::path::Path) -> KanbanSyncService {
        let mut config = Config::default();
        config.paths.tickets = root.join(".tickets").to_string_lossy().into_owned();
        KanbanSyncService::new(&config)
    }

    fn synced_ticket(root: &std::path::Path, key: &str) -> Ticket {
        ["queue", "in-progress", "completed"]
            .iter()
            .filter_map(|d| fs::read_dir(root.join(".tickets").join(d)).ok())
            .flatten()
            .filter_map(|e| Ticket::from_file(&e.unwrap().path()).ok())
            .find(|t| t.external_id.as_deref() == Some(key))
            .unwrap_or_else(|| panic!("no ticket for {key}"))
    }

    fn ticket_count(root: &std::path::Path) -> usize {
        ["queue", "in-progress", "completed"]
            .iter()
            .filter_map(|d| fs::read_dir(root.join(".tickets").join(d)).ok())
            .map(Iterator::count)
            .sum()
    }

    #[test]
    fn test_relationship_refs_unions_blockers_and_subtasks() {
        let issue = ExternalIssue {
            blocked_by: vec!["PROJ-12".into(), "PROJ-42".into()],
            subtasks: vec!["PROJ-43".into(), "PROJ-12".into()],
            parent: Some(crate::api::providers::kanban::ExternalParent {
                key: "PROJ-1".into(),
                kind: crate::api::providers::kanban::ParentKind::Epic,
            }),
            ..sample_issue()
        };
        let (deps, parent) = relationship_refs("jira", &issue);
        assert_eq!(deps, vec!["jira:PROJ-12", "jira:PROJ-43"]);
        assert_eq!(parent.as_deref(), Some("jira:PROJ-1"));
    }

    #[test]
    fn test_merge_depends_on_keeps_local_and_foreign_refs() {
        let existing = vec![
            "FEAT-0003".to_string(),
            "jira:PROJ-9".to_string(),
            "linear:ENG-1".to_string(),
        ];
        let merged = merge_depends_on(&existing, "jira", &["jira:PROJ-12".to_string()]);
        assert_eq!(merged, vec!["FEAT-0003", "linear:ENG-1", "jira:PROJ-12"]);
    }

    #[test]
    fn test_jira_fixture_sync_writes_relationships_and_keeps_dangling_blockers() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let result = service.apply_issues(
            "jira",
            "PROJ",
            &ProjectSyncConfig::default(),
            crate::api::providers::kanban::jira_fixture_issues(),
        );
        assert_eq!(result.created.len(), 2, "{:?}", result.errors);
        let story = synced_ticket(temp.path(), "PROJ-42");
        assert_eq!(
            story.depends_on,
            vec!["jira:PROJ-12", "jira:PROJ-43", "jira:PROJ-44"],
            "PROJ-12 is not synced but stays as a blocker"
        );
        assert_eq!(story.external_parent.as_deref(), Some("jira:PROJ-1"));
        let sub = synced_ticket(temp.path(), "PROJ-60");
        assert!(sub.depends_on.is_empty());
        assert_eq!(sub.external_parent.as_deref(), Some("jira:PROJ-42"));
    }

    #[test]
    fn test_linear_fixture_sync_maps_project_parent() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        service.apply_issues(
            "linear",
            "ENG",
            &ProjectSyncConfig::default(),
            crate::api::providers::kanban::linear_fixture_issues(),
        );
        let issue = synced_ticket(temp.path(), "ENG-7");
        assert_eq!(
            issue.depends_on,
            vec!["linear:ENG-3", "linear:ENG-8", "linear:ENG-9"]
        );
        assert_eq!(
            issue.external_parent.as_deref(),
            Some("linear:proj-billing")
        );
    }

    #[test]
    fn test_github_keys_survive_yaml_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let issue = ExternalIssue {
            key: "octo/hello#12".into(),
            blocked_by: vec!["octo/hello#9".into()],
            subtasks: vec!["octo/infra#2".into()],
            parent: Some(crate::api::providers::kanban::ExternalParent {
                key: "octo/hello#3".into(),
                kind: crate::api::providers::kanban::ParentKind::Issue,
            }),
            ..sample_issue()
        };
        service.apply_issues(
            "github",
            "PVT_x",
            &ProjectSyncConfig::default(),
            vec![issue],
        );
        let ticket = synced_ticket(temp.path(), "octo/hello#12");
        assert_eq!(
            ticket.depends_on,
            vec!["github:octo/hello#9", "github:octo/infra#2"]
        );
        assert_eq!(
            ticket.external_parent.as_deref(),
            Some("github:octo/hello#3")
        );
    }

    #[test]
    fn test_resync_refreshes_provider_edges_across_folders_without_duplicating() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let config = ProjectSyncConfig::default();
        let issue = ExternalIssue {
            blocked_by: vec!["PROJ-12".into()],
            ..sample_issue()
        };
        service.apply_issues("jira", "PROJ", &config, vec![issue]);

        let mut ticket = synced_ticket(temp.path(), "PROJ-42");
        ticket
            .set_depends_on(&["jira:PROJ-12".to_string(), "FEAT-0007".to_string()])
            .unwrap();
        let in_progress = temp.path().join(".tickets/in-progress");
        fs::create_dir_all(&in_progress).unwrap();
        fs::rename(&ticket.filepath, in_progress.join(&ticket.filename)).unwrap();

        let rerun = service.apply_issues("jira", "PROJ", &config, vec![sample_issue()]);
        assert_eq!(rerun.updated, vec!["PROJ-42"]);
        assert!(rerun.created.is_empty());
        assert_eq!(ticket_count(temp.path()), 1);
        let refreshed = synced_ticket(temp.path(), "PROJ-42");
        assert_eq!(refreshed.depends_on, vec!["FEAT-0007"]);
        assert_eq!(refreshed.status, "queued");

        let again = service.apply_issues("jira", "PROJ", &config, vec![sample_issue()]);
        assert_eq!(again.skipped, vec!["PROJ-42"]);
    }

    #[test]
    fn test_same_key_from_another_provider_is_a_separate_ticket() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let config = ProjectSyncConfig::default();
        service.apply_issues("jira", "PROJ", &config, vec![sample_issue()]);
        let result = service.apply_issues("linear", "PROJ", &config, vec![sample_issue()]);
        assert_eq!(result.created, vec!["PROJ-42"]);
        assert_eq!(ticket_count(temp.path()), 2);
    }

    // ── Scoped sync into a campaign ─────────────────────────────────────────

    fn all_tickets(root: &std::path::Path) -> Vec<Ticket> {
        ["queue", "in-progress", "completed"]
            .iter()
            .filter_map(|d| fs::read_dir(root.join(".tickets").join(d)).ok())
            .flatten()
            .map(|e| Ticket::from_file(&e.unwrap().path()).unwrap())
            .collect()
    }

    #[test]
    fn test_epic_scope_becomes_a_campaign_planned_into_waves() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let result = service
            .stamp_scope(
                "jira",
                "PROJ-1",
                &ProjectSyncConfig::default(),
                crate::api::providers::kanban::jira_epic_fixture(),
            )
            .unwrap();
        assert_eq!(result.campaign.as_deref(), Some("CAMP-0001"));
        assert_eq!(result.created.len(), 6, "{:?}", result.errors);

        let campaign = CampaignStore::new(&service.config)
            .get("CAMP-0001")
            .unwrap();
        assert_eq!(campaign.title, "Pricing overhaul");
        assert_eq!(campaign.external_ref.as_deref(), Some("jira:PROJ-1"));
        assert!(campaign.body.contains("Ship v2 pricing."));

        let all = all_tickets(temp.path());
        let members = crate::campaigns::members("CAMP-0001", &all);
        assert_eq!(members.len(), 6);
        assert!(members
            .iter()
            .all(|t| t.external_parent.as_deref() == Some("jira:PROJ-1")));

        let plan = crate::campaigns::plan::plan(&service.config, &members, &all).unwrap();
        assert_eq!(
            plan.waves,
            vec![
                vec!["TASK-PROJ11", "TASK-PROJ12", "TASK-PROJ15"],
                vec!["TASK-PROJ13", "TASK-PROJ16"],
                vec!["TASK-PROJ14"],
            ]
        );
        assert_eq!(
            crate::campaigns::plan::ready(&plan, &all),
            vec!["TASK-PROJ11", "TASK-PROJ12", "TASK-PROJ15"]
        );
    }

    #[test]
    fn test_resync_reuses_campaign_and_keeps_reassigned_tickets() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let config = ProjectSyncConfig::default();
        let epic = crate::api::providers::kanban::jira_epic_fixture;
        service
            .stamp_scope("jira", "PROJ-1", &config, epic())
            .unwrap();

        let mut moved = synced_ticket(temp.path(), "PROJ-16");
        moved.set_campaign(Some("CAMP-0099")).unwrap();

        let rerun = service
            .stamp_scope("jira", "PROJ-1", &config, epic())
            .unwrap();
        assert_eq!(rerun.campaign.as_deref(), Some("CAMP-0001"));
        assert!(rerun.created.is_empty());
        assert_eq!(CampaignStore::new(&service.config).list().len(), 1);
        assert_eq!(
            synced_ticket(temp.path(), "PROJ-16").campaign.as_deref(),
            Some("CAMP-0099")
        );
    }

    #[test]
    fn test_scoped_sync_without_premium_writes_nothing() {
        let temp = tempfile::tempdir().unwrap();
        let service = sync_service(temp.path());
        let err = service
            .apply_scope(
                "jira",
                "PROJ-1",
                &ProjectSyncConfig::default(),
                crate::api::providers::kanban::jira_epic_fixture(),
            )
            .unwrap_err();
        assert!(err
            .downcast_ref::<crate::licensing::NotEntitled>()
            .is_some_and(|e| e.feature == PremiumFeature::Campaigns));
        assert!(!temp.path().join(".tickets/campaigns").exists());
        assert_eq!(ticket_count(temp.path()), 0);
    }
}
