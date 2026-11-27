use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::batch::BatchItem;
use super::recipe::{Record, WorkflowDataset, WorkflowRecipe};
use crate::api::providers::kanban::{
    ExternalIssue, GithubProjectsProvider, KanbanProvider, LinearProvider,
};
use crate::config::{Config, ProjectSyncConfig};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, JsonSchema, TS,
)]
#[ts(export)]
pub struct BoardScope {
    pub provider: String,
    pub instance: String,
    pub project: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BoardSelection {
    pub scope: BoardScope,
    pub recipe: WorkflowRecipe,
    pub issue_ids: Vec<String>,
    #[serde(default)]
    pub status_writers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIssue {
    pub scope: BoardScope,
    pub id: String,
    pub key: String,
    pub url: String,
    pub description: String,
    pub status_writer: bool,
    #[serde(default)]
    pub include_legacy: bool,
}

impl BoardScope {
    pub fn configured(config: &Config) -> Vec<Self> {
        let mut scopes = Vec::new();
        for (instance, settings) in config.kanban.linear.iter().filter(|(_, s)| s.enabled) {
            scopes.extend(settings.projects.keys().map(|project| Self {
                provider: "linear".into(),
                instance: instance.clone(),
                project: project.clone(),
            }));
        }
        for (instance, settings) in config.kanban.github.iter().filter(|(_, s)| s.enabled) {
            scopes.extend(settings.projects.keys().map(|project| Self {
                provider: "github".into(),
                instance: instance.clone(),
                project: project.clone(),
            }));
        }
        scopes.sort();
        scopes
    }

    pub fn config<'a>(&self, config: &'a Config) -> Option<&'a ProjectSyncConfig> {
        match self.provider.as_str() {
            "linear" => config
                .kanban
                .linear
                .get(&self.instance)
                .filter(|s| s.enabled)?
                .projects
                .get(&self.project),
            "github" => config
                .kanban
                .github
                .get(&self.instance)
                .filter(|s| s.enabled)?
                .projects
                .get(&self.project),
            _ => None,
        }
    }

    pub fn provider(&self, config: &Config) -> Result<Box<dyn KanbanProvider>> {
        self.config(config)
            .context("Board is no longer configured")?;
        match self.provider.as_str() {
            "linear" => Ok(Box::new(LinearProvider::from_config(
                &self.instance,
                &config.kanban.linear[&self.instance],
            )?)),
            "github" => Ok(Box::new(GithubProjectsProvider::from_config(
                &self.instance,
                &config.kanban.github[&self.instance],
            )?)),
            _ => anyhow::bail!("Unknown board provider"),
        }
    }

    pub async fn issues(&self, config: &Config) -> Result<Vec<ExternalIssue>> {
        let settings = self
            .config(config)
            .context("Board is no longer configured")?;
        let provider = self.provider(config)?;
        if let Some(scope) = &settings.scope {
            crate::licensing::require_premium(config, crate::licensing::PremiumFeature::Campaigns)?;
            return Ok(provider
                .list_scope(scope, &settings.pull_statuses())
                .await?
                .issues);
        }
        Ok(provider
            .list_issues(
                &self.project,
                &settings.sync_user_id,
                &settings.pull_statuses(),
            )
            .await?)
    }
}

pub fn candidates<'a>(
    issue: &ExternalIssue,
    mappings: &'a HashMap<String, String>,
) -> BTreeSet<&'a str> {
    issue
        .kanban_issue_types
        .iter()
        .filter_map(|kind| mappings.get(&kind.id).map(String::as_str))
        .collect()
}

pub fn record(issue: &ExternalIssue, mappings: &HashMap<String, String>) -> Record {
    Record::from([
        ("id".into(), issue.id.clone()),
        (
            "mapped_workflows".into(),
            candidates(issue, mappings)
                .into_iter()
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("key".into(), issue.key.clone()),
        ("summary".into(), issue.summary.clone()),
        (
            "description".into(),
            issue.description.clone().unwrap_or_default(),
        ),
        ("url".into(), issue.url.clone()),
        ("status".into(), issue.status.clone()),
        (
            "priority".into(),
            issue.priority.clone().unwrap_or_default(),
        ),
    ])
}

impl BoardSelection {
    pub fn expand(
        &self,
        issues: &[ExternalIssue],
        mappings: &HashMap<String, String>,
    ) -> Result<Vec<BatchItem>> {
        let wanted: BTreeSet<_> = self.issue_ids.iter().collect();
        ensure!(
            !wanted.is_empty() && wanted.len() == self.issue_ids.len(),
            "Select distinct board records"
        );
        let by_id: BTreeMap<_, _> = issues.iter().map(|issue| (&issue.id, issue)).collect();
        let mut recipe = self.recipe.clone();
        recipe.source = serde_json::to_string(&(&self.scope.provider, &self.scope.instance))?;
        recipe.key_columns = vec!["id".into()];
        let selected: Vec<_> = self
            .issue_ids
            .iter()
            .map(|id| {
                by_id
                    .get(id)
                    .copied()
                    .with_context(|| format!("Board record {id} is no longer in this scope"))
            })
            .collect::<Result<_>>()?;
        let mut items = Vec::new();
        for (index, issue) in selected.into_iter().enumerate() {
            let mut resolved = recipe.clone();
            if let Some(policy) = recipe.unresolved_labels {
                let candidates = candidates(issue, mappings);
                if candidates.len() == 1 {
                    resolved.workflow = candidates
                        .into_iter()
                        .next()
                        .expect("one candidate")
                        .to_owned();
                } else {
                    match policy {
                        super::recipe::LabelPolicy::Skip => continue,
                        super::recipe::LabelPolicy::Review => anyhow::bail!(
                            "{} needs review: {} matching workflows",
                            issue.key,
                            candidates.len()
                        ),
                        super::recipe::LabelPolicy::UseSelected => {}
                    }
                }
            }
            let dataset = WorkflowDataset {
                recipe: resolved,
                rows: vec![record(issue, mappings)],
            };
            let mut variants = dataset.expand()?;
            ensure!(
                items.len() + variants.len() <= super::recipe::MAX_EXPANDED_TICKETS,
                "Too many ticket combinations"
            );
            for (variant, item) in variants.iter_mut().enumerate() {
                item.reference = format!("row-{}-{}", index + 1, variant + 1);
                item.source_issue = Some(SourceIssue {
                    scope: self.scope.clone(),
                    id: issue.id.clone(),
                    key: issue.key.clone(),
                    url: issue.url.clone(),
                    description: issue.description.clone().unwrap_or_default(),
                    include_legacy: self.recipe.include_legacy.unwrap_or(false),
                    status_writer: self
                        .status_writers
                        .get(&issue.id)
                        .is_some_and(|reference| reference == &item.reference),
                });
            }
            items.extend(variants);
        }
        for (id, reference) in &self.status_writers {
            ensure!(
                items.iter().any(|item| &item.reference == reference
                    && item
                        .source_issue
                        .as_ref()
                        .is_some_and(|source| source.status_writer && &source.id == id)),
                "Invalid status writer for {id}"
            );
        }
        Ok(items)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BoardMapping {
    pub scope: BoardScope,
    pub recipe: WorkflowRecipe,
}

impl BoardScope {
    fn mapping_path(&self, config: &Config) -> Result<std::path::PathBuf> {
        let digest = crate::collections::fetch::sha256_hex(&serde_json::to_vec(self)?);
        Ok(config
            .state_path()
            .join("workflow-board-mappings")
            .join(format!("{digest}.json")))
    }

    pub fn saved_mapping(&self, config: &Config) -> Result<Option<WorkflowRecipe>> {
        match std::fs::read(self.mapping_path(config)?) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save_mapping(&self, config: &Config, recipe: &WorkflowRecipe) -> Result<()> {
        let path = self.mapping_path(config)?;
        let dir = path.parent().context("Mapping needs a directory")?;
        std::fs::create_dir_all(dir)?;
        let file = tempfile::NamedTempFile::new_in(dir)?;
        serde_json::to_writer_pretty(file.as_file(), recipe)?;
        file.persist(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ambiguous_board_labels_follow_the_selected_policy() {
        let issue: ExternalIssue = serde_json::from_value(json!({
            "id": "source", "key": "A-1", "summary": "Review", "url": "https://example.test",
            "status": "Open", "kanban_issue_types": [{"id": "a", "name": "A"}, {"id": "b", "name": "B"}]
        })).unwrap();
        let mut selection: BoardSelection = serde_json::from_value(json!({
            "scope": {"provider": "linear", "instance": "team", "project": "board"},
            "recipe": {"workflow": "REVIEW", "collection": "research", "source": "", "columns": {"labels": "mapped_workflows"}, "repeat": {}, "shared": {}, "key_columns": []},
            "issue_ids": ["source"], "status_writers": {}
        })).unwrap();
        let labels = HashMap::from([("a".into(), "FIX".into()), ("b".into(), "TASK".into())]);
        let issues = [issue];
        use crate::campaigns::recipe::LabelPolicy;
        selection.recipe.unresolved_labels = Some(LabelPolicy::Review);
        assert!(selection
            .expand(&issues, &labels)
            .unwrap_err()
            .to_string()
            .contains("A-1"));
        selection.recipe.unresolved_labels = Some(LabelPolicy::Skip);
        assert!(selection.expand(&issues, &labels).unwrap().is_empty());
        selection.recipe.unresolved_labels = Some(LabelPolicy::UseSelected);
        let items = selection.expand(&issues, &labels).unwrap();
        assert_eq!(items[0].template, "REVIEW");
        assert_eq!(items[0].values["labels"], "FIX, TASK");
    }

    #[test]
    fn test_board_expansion_keeps_issue_and_status_owner_for_each_variant() {
        for provider in ["github", "linear"] {
            let issue:ExternalIssue=serde_json::from_value(json!({"id":"node-id","key":"ENG-1","summary":"Review","description":"source text","url":"https://example.test/1","status":"open","kanban_issue_types":[]})).unwrap();
            let selection:BoardSelection=serde_json::from_value(json!({
                "scope":{"provider":provider,"instance":"team","project":"board"},
                "recipe":{"workflow":"REVIEW","collection":"research","source":"","columns":{"summary":"summary"},"repeat":{"perspective":["author","reviewer"]},"shared":{},"key_columns":[]},
                "issue_ids":["node-id"],"status_writers":{"node-id":"row-1-2"}
            })).unwrap();
            let tickets = selection.expand(&[issue], &HashMap::new()).unwrap();
            assert_ne!(tickets[0].record_identity, tickets[1].record_identity);
            assert!(tickets
                .iter()
                .all(|t| t.source_issue.as_ref().unwrap().scope.provider == provider));
            assert!(!tickets[0].source_issue.as_ref().unwrap().status_writer);
            assert!(tickets[1].source_issue.as_ref().unwrap().status_writer);
        }
    }
}
