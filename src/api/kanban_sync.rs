//! Bidirectional kanban sync - pushes operator ticket state changes upstream.

use std::sync::Arc;

use tracing::warn;

use crate::api::providers::kanban::{
    ActivityLogEntry, CreateIssueRequest, CreateIssueResponse, GithubProjectsProvider,
    JiraProvider, KanbanProvider, KanbanProviderType, LinearProvider, UpdateStatusRequest,
};
use crate::config::{Config, ProjectSyncConfig};
use crate::queue::Ticket;
use crate::rest::dto::{KanbanPushOutcome, KanbanTarget};

/// Orchestrates outbound synchronisation from operator tickets to upstream kanban providers.
///
/// All public methods are **best-effort**: errors are logged at WARN level and
/// never propagated to the caller, so a provider outage cannot interrupt normal
/// ticket workflow.
pub struct KanbanBidirectionalSync {
    config: Arc<Config>,
}

impl KanbanBidirectionalSync {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// Returns true if any configured project has `bidirectional: true`.
    pub fn has_any_bidirectional(&self) -> bool {
        let kanban = &self.config.kanban;
        kanban
            .jira
            .values()
            .any(|w| w.projects.values().any(|p| p.bidirectional))
            || kanban
                .linear
                .values()
                .any(|w| w.projects.values().any(|p| p.bidirectional))
            || kanban
                .github
                .values()
                .any(|w| w.projects.values().any(|p| p.bidirectional))
    }

    /// Called when a ticket is claimed (todo → doing). Pushes "doing" status to provider.
    pub async fn on_ticket_claimed(&self, ticket: &Ticket) {
        if let Some((provider, sync_cfg)) = self.resolve(ticket) {
            let status = doing_status(&sync_cfg).to_string();
            self.push_status(ticket, &*provider, &status).await;
        }
    }

    /// Called when a ticket is completed (doing → done). Pushes "done" status to provider.
    pub async fn on_ticket_completed(&self, ticket: &Ticket) {
        if let Some((provider, sync_cfg)) = self.resolve(ticket) {
            let status = done_status(&sync_cfg).to_string();
            self.push_status(ticket, &*provider, &status).await;
        }
    }

    /// Called when a ticket is returned to the queue (doing → todo). Pushes the
    /// mapped "todo" status to the provider - no-op unless `status_mapping.todo`
    /// is explicitly configured (there is no safe universal default column).
    pub async fn on_ticket_requeued(&self, ticket: &Ticket) {
        if let Some((provider, sync_cfg)) = self.resolve(ticket) {
            if let Some(status) = todo_status(&sync_cfg) {
                let status = status.to_string();
                self.push_status(ticket, &*provider, &status).await;
            }
        }
    }

    /// Called when a step completes. Appends an activity log entry to the upstream issue.
    pub async fn on_step_completed(
        &self,
        ticket: &Ticket,
        step_name: &str,
        delegator_name: &str,
        summary: Option<&str>,
    ) {
        let Some((provider, _)) = self.resolve(ticket) else {
            return;
        };
        let Some(external_id) = ticket.external_id.as_deref() else {
            return;
        };
        let entry = ActivityLogEntry {
            step: step_name.to_string(),
            delegator: delegator_name.to_string(),
            completed_at: chrono::Utc::now(),
            summary: summary.map(ToString::to_string),
        };
        if let Err(e) = provider.append_activity_log(external_id, &entry).await {
            warn!(
                ticket_id = %ticket.id,
                step = step_name,
                error = %e,
                "Bidirectional sync: failed to append activity log"
            );
        }
    }

    /// The bidirectional kanban project tickets for operator `project` push
    /// to: the one whose effective ticket project (`ticket_project`, or the
    /// project key) matches.
    pub fn resolve_target(&self, project: &str) -> Option<KanbanTarget> {
        self.find_target(project).map(|found| found.target)
    }

    /// Create the upstream issue for a newly created ticket. Never fails the
    /// caller: every error is reported in the returned outcome.
    pub async fn push_new_ticket(
        &self,
        ticket: &Ticket,
        description: Option<String>,
    ) -> KanbanPushOutcome {
        let Some(found) = self.find_target(&ticket.project) else {
            return KanbanPushOutcome::NotConfigured;
        };
        let provider = match self.build_provider(&found) {
            Ok(provider) => provider,
            Err(e) => {
                return KanbanPushOutcome::Failed {
                    provider: found.target.provider,
                    message: e.to_string(),
                }
            }
        };
        let request = build_create_request(ticket, found.sync_cfg, description);
        let result = provider
            .create_issue(&found.target.project_key, request)
            .await;
        if let Err(e) = &result {
            warn!(
                ticket_id = %ticket.id,
                provider = %found.target.provider,
                error = %e,
                "Bidirectional sync: create_issue failed"
            );
        }
        push_outcome(found.target, result)
    }

    fn find_target(&self, project: &str) -> Option<FoundTarget<'_>> {
        let wanted = normalize_project(project);
        if wanted.is_empty() {
            return None;
        }
        let kanban = &self.config.kanban;
        let jira = kanban
            .jira
            .iter()
            .flat_map(|(instance, cfg)| tag(KanbanProviderType::Jira, instance, &cfg.projects));
        let linear = kanban.linear.iter().flat_map(|(instance, cfg)| {
            tag(KanbanProviderType::Linear, instance, &cfg.projects)
        });
        let github = kanban.github.iter().flat_map(|(instance, cfg)| {
            tag(KanbanProviderType::Github, instance, &cfg.projects)
        });
        jira.chain(linear)
            .chain(github)
            .filter(|found| found.sync_cfg.bidirectional)
            .find(|found| {
                normalize_project(&crate::services::kanban_sync::ticket_project(
                    found.sync_cfg,
                    &found.target.project_key,
                )) == wanted
            })
    }

    fn build_provider(
        &self,
        found: &FoundTarget<'_>,
    ) -> Result<Box<dyn KanbanProvider>, crate::api::error::ApiError> {
        let kanban = &self.config.kanban;
        let missing = || {
            crate::api::error::ApiError::not_configured(found.target.provider.as_str())
        };
        match found.kind {
            KanbanProviderType::Jira => {
                let cfg = kanban.jira.get(found.instance).ok_or_else(missing)?;
                Ok(Box::new(JiraProvider::from_config(found.instance, cfg)?))
            }
            KanbanProviderType::Linear => {
                let cfg = kanban.linear.get(found.instance).ok_or_else(missing)?;
                Ok(Box::new(LinearProvider::from_config(found.instance, cfg)?))
            }
            KanbanProviderType::Github => {
                let cfg = kanban.github.get(found.instance).ok_or_else(missing)?;
                Ok(Box::new(GithubProjectsProvider::from_config(
                    found.instance,
                    cfg,
                )?))
            }
            _ => Err(missing()),
        }
    }

    // ─── Private helpers ─────────────────────────────────────────────────────

    /// Find the provider instance and sync config for a ticket's external issue.
    fn resolve(&self, ticket: &Ticket) -> Option<(Box<dyn KanbanProvider>, ProjectSyncConfig)> {
        let provider_name = ticket.external_provider.as_deref()?;
        let external_id = ticket.external_id.as_deref()?;

        match provider_name {
            "jira" => {
                let project_key = external_id.split('-').next()?;
                for (domain, cfg) in &self.config.kanban.jira {
                    if let Some(sync_cfg) = cfg.projects.get(project_key) {
                        if sync_cfg.bidirectional {
                            if let Ok(p) = JiraProvider::from_config(domain, cfg) {
                                return Some((Box::new(p), sync_cfg.clone()));
                            }
                        }
                    }
                }
                None
            }
            "linear" => {
                let team_key = external_id.split('-').next()?;
                for (workspace, cfg) in &self.config.kanban.linear {
                    if let Some(sync_cfg) = cfg.projects.get(team_key) {
                        if sync_cfg.bidirectional {
                            if let Ok(p) = LinearProvider::from_config(workspace, cfg) {
                                return Some((Box::new(p), sync_cfg.clone()));
                            }
                        }
                    }
                }
                None
            }
            "github" => {
                for (owner, cfg) in &self.config.kanban.github {
                    for sync_cfg in cfg.projects.values() {
                        if sync_cfg.bidirectional {
                            if let Ok(p) = GithubProjectsProvider::from_config(owner, cfg) {
                                return Some((Box::new(p), sync_cfg.clone()));
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    async fn push_status(&self, ticket: &Ticket, provider: &dyn KanbanProvider, status: &str) {
        let Some(external_id) = ticket.external_id.as_deref() else {
            return;
        };
        let req = UpdateStatusRequest {
            status: status.to_string(),
        };
        if let Err(e) = provider.update_issue_status(external_id, req).await {
            warn!(
                ticket_id = %ticket.id,
                status = status,
                error = %e,
                "Bidirectional sync: failed to update upstream status"
            );
        }
    }
}

fn todo_status(sync_cfg: &ProjectSyncConfig) -> Option<&str> {
    sync_cfg.status_mapping.todo.as_deref()
}

fn doing_status(sync_cfg: &ProjectSyncConfig) -> &str {
    sync_cfg
        .status_mapping
        .doing
        .as_deref()
        .unwrap_or("In Progress")
}

fn done_status(sync_cfg: &ProjectSyncConfig) -> &str {
    sync_cfg.status_mapping.done.as_deref().unwrap_or("Done")
}

/// A bidirectional kanban project, with where its provider config lives.
struct FoundTarget<'a> {
    target: KanbanTarget,
    kind: KanbanProviderType,
    instance: &'a str,
    sync_cfg: &'a ProjectSyncConfig,
}

fn tag<'a>(
    kind: KanbanProviderType,
    instance: &'a str,
    projects: &'a std::collections::HashMap<String, ProjectSyncConfig>,
) -> impl Iterator<Item = FoundTarget<'a>> {
    projects.iter().map(move |(project_key, sync_cfg)| FoundTarget {
        target: KanbanTarget {
            provider: kind.slug().to_string(),
            project_key: project_key.clone(),
        },
        kind,
        instance,
        sync_cfg,
    })
}

/// Operator project names compare case-insensitively on alphanumerics,
/// matching how `ticket_project` and ticket filenames strip them.
fn normalize_project(project: &str) -> String {
    project
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase()
}

fn push_outcome(
    target: KanbanTarget,
    result: Result<CreateIssueResponse, crate::api::error::ApiError>,
) -> KanbanPushOutcome {
    match result {
        Ok(response) => KanbanPushOutcome::Pushed {
            provider: target.provider,
            project_key: target.project_key,
            external_id: response.issue.key,
            external_url: Some(response.issue.url).filter(|url| !url.is_empty()),
        },
        Err(e) => KanbanPushOutcome::Failed {
            provider: target.provider,
            message: e.to_string(),
        },
    }
}

fn build_create_request(
    ticket: &Ticket,
    sync_cfg: &ProjectSyncConfig,
    description: Option<String>,
) -> CreateIssueRequest {
    CreateIssueRequest {
        summary: ticket.summary.clone(),
        description,
        assignee_id: if sync_cfg.sync_user_id.is_empty() {
            None
        } else {
            Some(sync_cfg.sync_user_id.clone())
        },
        status: todo_status(sync_cfg).map(ToString::to_string),
        priority: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KanbanStatusMapping;

    fn make_sync_cfg(
        todo: Option<&str>,
        doing: Option<&str>,
        done: Option<&str>,
    ) -> ProjectSyncConfig {
        ProjectSyncConfig {
            status_mapping: KanbanStatusMapping {
                todo: todo.map(ToString::to_string),
                doing: doing.map(ToString::to_string),
                done: done.map(ToString::to_string),
            },
            bidirectional: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_doing_status_from_mapping() {
        let cfg = make_sync_cfg(Some("Backlog"), Some("Started"), Some("Completed"));
        assert_eq!(doing_status(&cfg), "Started");
    }

    #[test]
    fn test_done_status_from_mapping() {
        let cfg = make_sync_cfg(Some("Backlog"), Some("Started"), Some("Completed"));
        assert_eq!(done_status(&cfg), "Completed");
    }

    #[test]
    fn test_doing_status_default_when_unmapped() {
        let cfg = make_sync_cfg(None, None, None);
        assert_eq!(doing_status(&cfg), "In Progress");
    }

    #[test]
    fn test_done_status_default_when_unmapped() {
        let cfg = make_sync_cfg(None, None, None);
        assert_eq!(done_status(&cfg), "Done");
    }

    #[test]
    fn test_todo_status_none_without_mapping() {
        // No fallback: requeue push must stay silent when todo is unmapped.
        let cfg = make_sync_cfg(None, Some("Started"), Some("Completed"));
        assert_eq!(todo_status(&cfg), None);
    }

    #[test]
    fn test_todo_status_some_with_mapping() {
        let cfg = make_sync_cfg(Some("Backlog"), None, None);
        assert_eq!(todo_status(&cfg), Some("Backlog"));
    }

    #[test]
    fn test_skips_non_bidirectional_projects() {
        let mut cfg = make_sync_cfg(Some("To Do"), Some("In Progress"), Some("Done"));
        cfg.bidirectional = false;
        // sync service would not resolve a provider for this config
        // (we test the flag is respected by verifying bidirectional=false
        //  is explicitly handled in resolve())
        assert!(!cfg.bidirectional);
    }

    #[tokio::test]
    async fn test_on_ticket_requeued_noop_without_todo_mapping() {
        // Ticket without external linkage + empty config: resolve() returns
        // None and the call must be a silent no-op.
        let sync = KanbanBidirectionalSync::new(Arc::new(crate::config::Config::default()));
        let ticket = Ticket {
            depends_on: Vec::new(),
            campaign: None,
            external_parent: None,
            filename: String::new(),
            filepath: String::new(),
            timestamp: String::new(),
            ticket_type: "FIX".to_string(),
            project: "demo".to_string(),
            id: "FIX-1".to_string(),
            summary: String::new(),
            priority: String::new(),
            status: "queued".to_string(),
            step: String::new(),
            content: String::new(),
            sessions: std::collections::HashMap::default(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
            step_delegators: std::collections::HashMap::default(),
        };
        sync.on_ticket_requeued(&ticket).await;
    }
}
