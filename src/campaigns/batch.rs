//! Plan and create a batch of related tickets: ids allocated up front, local refs wired as dependencies.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::plan::{waves, PlanError};
use super::CampaignStore;
use crate::config::Config;
use crate::issuetypes::{IssueType, IssueTypeRegistry};
use crate::queue::creator::render_template;
use crate::queue::deps::{provider_ref, resolve, Resolved};
use crate::queue::{ids, Queue, Ticket, TicketCreator, TicketPriority};
use crate::rest::dto::CreateTicketRequest;
use crate::services::ticket_creation::{self, CreationError};
use crate::templates::TemplateType;
use serde_json::json;
use sha2::{Digest, Sha256};

/// One ticket to create; `ref` names it within the batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BatchItem {
    /// Batch-local name other items use in `depends_on`.
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(skip)]
    pub(crate) record_identity: Option<String>,
    #[serde(skip)]
    pub(crate) source_issue: Option<super::board::SourceIssue>,
    /// Template key (FEAT, FIX, TASK, SPIKE, INV, ...).
    pub template: String,
    #[serde(default)]
    #[ts(optional)]
    pub collection: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    /// Extra Handlebars values; `project`/`summary` above take precedence.
    #[serde(default)]
    pub values: HashMap<String, String>,
    /// Batch refs, existing ticket ids, or `provider:key` refs.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// A batch of tickets, optionally created inside a campaign.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BatchRequest {
    #[serde(default)]
    pub campaign: Option<String>,
    #[serde(default)]
    pub tickets: Vec<BatchItem>,
    #[serde(default)]
    #[ts(optional)]
    pub dataset: Option<super::recipe::WorkflowDataset>,
    #[serde(default)]
    #[ts(optional)]
    pub board: Option<super::board::BoardSelection>,
    #[serde(default)]
    #[ts(optional)]
    pub confirmation: Option<String>,
}

/// One item as it would be written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct PlannedTicket {
    #[serde(rename = "ref")]
    pub reference: String,
    /// Allocated id; empty when the template is unknown.
    pub id: String,
    pub template: String,
    /// Filename without the date prefix: `TYPE-project-slug`.
    pub filename_stem: String,
    /// Dependencies with batch refs resolved to allocated ids.
    pub depends_on: Vec<String>,
    #[ts(optional)]
    pub source_record: Option<String>,
    pub markdown: String,
    pub values: BTreeMap<String, String>,
    pub errors: Vec<String>,
}

/// What a batch would create; nothing is written.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS,
)]
#[ts(export)]
pub struct BatchPreview {
    pub tickets: Vec<PlannedTicket>,
    /// Allocated ids in dependency waves.
    pub waves: Vec<Vec<String>>,
    /// Batch-wide errors (cycles, unknown campaign).
    pub errors: Vec<String>,
    pub valid: bool,
    pub confirmation: String,
    pub previously_created: BTreeMap<String, String>,
}

/// A ticket written by a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreatedTicket {
    #[serde(rename = "ref")]
    pub reference: String,
    pub id: String,
    pub filename: String,
}

/// Result of creating a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BatchCreated {
    pub tickets: Vec<CreatedTicket>,
    pub waves: Vec<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("batch is invalid")]
    Invalid(Box<BatchPreview>),
    #[error("batch write failed and was rolled back: {0}")]
    Write(String),
}

#[cfg(test)]
fn preview(
    config: &Config,
    existing: &[Ticket],
    request: &BatchRequest,
    campaign_exists: bool,
) -> BatchPreview {
    let registry = crate::startup::templates::load_active_registry(
        &config.tickets_path(),
        config.templates.active_collection.as_deref(),
    );
    preview_with_registry(config, &registry, existing, request, campaign_exists)
}

pub fn preview_with_registry(
    config: &Config,
    registry: &IssueTypeRegistry,
    existing: &[Ticket],
    request: &BatchRequest,
    campaign_exists: bool,
) -> BatchPreview {
    let (items, previously_created) = fresh_items(existing, &request.tickets);
    let items = &items;
    let mut taken: Vec<String> = existing.iter().map(|t| t.id.clone()).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut tickets: Vec<PlannedTicket> = items
        .iter()
        .map(|item| {
            let mut errors = Vec::new();
            if item.reference.trim().is_empty() {
                errors.push("ref is required".to_string());
            } else if !seen.insert(item.reference.as_str()) {
                errors.push(format!("duplicate ref '{}'", item.reference));
            }
            let (id, stem) = match workflow(registry, item) {
                Some((_, definition)) => {
                    let id = ids::next_ticket_id(&definition.key, taken.iter().map(String::as_str));
                    taken.push(id.clone());
                    let project = item
                        .project
                        .as_deref()
                        .filter(|p| !p.is_empty())
                        .unwrap_or("global");
                    let slug = item
                        .summary
                        .as_deref()
                        .map(|s| ids::slugify(s, 50))
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| ids::slugify(&id, 50));
                    let stem = format!("{}-{project}-{slug}", definition.key);
                    (id, stem)
                }
                None => {
                    errors.push(format!("unknown template '{}'", item.template));
                    (String::new(), String::new())
                }
            };
            PlannedTicket {
                reference: item.reference.clone(),
                id,
                template: item.template.clone(),
                filename_stem: stem,
                depends_on: Vec::new(),
                source_record: item.source_issue.as_ref().map(|issue| issue.id.clone()),
                markdown: String::new(),
                values: BTreeMap::new(),
                errors,
            }
        })
        .collect();

    let by_ref: BTreeMap<String, String> = tickets
        .iter()
        .filter(|t| !t.id.is_empty())
        .map(|t| (t.reference.clone(), t.id.clone()))
        .collect();
    let batch_ids: BTreeSet<&str> = by_ref.values().map(String::as_str).collect();
    let resolved: Vec<(Vec<String>, Vec<String>)> = items
        .iter()
        .map(|item| {
            let mut deps = Vec::new();
            let mut errors = Vec::new();
            for dep in &item.depends_on {
                if let Some(id) = by_ref.get(dep) {
                    deps.push(id.clone());
                } else if batch_ids.contains(dep.as_str())
                    || provider_ref(dep).is_some()
                    || matches!(resolve(dep, existing), Resolved::Local(_))
                {
                    deps.push(dep.clone());
                } else {
                    errors.push(format!("unknown dependency '{dep}'"));
                }
            }
            (deps, errors)
        })
        .collect();
    for (ticket, (deps, errors)) in tickets.iter_mut().zip(resolved) {
        ticket.depends_on = deps;
        ticket.errors.extend(errors);
    }

    let mut errors = Vec::new();
    if let Some(campaign) = request.campaign.as_deref().filter(|_| !campaign_exists) {
        errors.push(format!("campaign {campaign} not found"));
    }
    let blockers: BTreeMap<String, BTreeSet<String>> = tickets
        .iter()
        .filter(|t| !t.id.is_empty())
        .map(|t| {
            let local = t
                .depends_on
                .iter()
                .filter(|d| batch_ids.contains(d.as_str()))
                .cloned()
                .collect();
            (t.id.clone(), local)
        })
        .collect();
    let rank: BTreeMap<&str, (usize, TicketPriority, usize)> = tickets
        .iter()
        .zip(items.iter().copied())
        .enumerate()
        .map(|(n, (t, item))| {
            let priority = item
                .values
                .get("priority")
                .map_or(TicketPriority::P2Medium, |p| {
                    TicketPriority::from_frontmatter(p)
                });
            let kind = item.template.to_ascii_uppercase();
            (t.id.as_str(), (config.priority_index(&kind), priority, n))
        })
        .collect();
    let waves = match waves(blockers, |id| rank[id]) {
        Ok(waves) => waves,
        Err(PlanError::Cycle(ids)) => {
            errors.push(format!("dependency cycle among {}", ids.join(", ")));
            Vec::new()
        }
    };
    let mut writers: BTreeSet<_> = existing
        .iter()
        .filter_map(|t| {
            let (map, _) = crate::queue::parse_frontmatter_mapping(&t.content)?;
            let source: super::board::SourceIssue =
                serde_json::from_value(map.get("board_source")?.clone()).ok()?;
            source.status_writer.then_some((
                source.scope.provider,
                source.scope.instance,
                source.id,
            ))
        })
        .collect();
    for item in items {
        if let Some(source) = item
            .source_issue
            .as_ref()
            .filter(|source| source.status_writer)
        {
            if existing.iter().any(|ticket| {
                ticket.external_url.as_ref() == Some(&source.url)
                    && ticket.external_provider.as_ref() == Some(&source.scope.provider)
                    && is_original_import(ticket)
            }) {
                errors.push(format!(
                    "The original imported ticket owns status updates for {}",
                    source.key
                ));
            }
            if !writers.insert((
                source.scope.provider.clone(),
                source.scope.instance.clone(),
                source.id.clone(),
            )) {
                errors.push(format!("{} already has a status writer", source.key));
            }
        }
    }
    let mut fingerprint = Sha256::new();
    for (ticket, item) in tickets.iter_mut().zip(items.iter().copied()) {
        if let Some((owner, definition)) = workflow(registry, item) {
            match render_item(
                config,
                item,
                ticket,
                owner,
                definition,
                request.campaign.as_deref(),
            ) {
                Ok((markdown, values, template_digest)) => {
                    ticket.markdown = markdown;
                    ticket.values = values;
                    let mut receipt = json!([
                        owner,
                        definition,
                        template_digest,
                        ticket.id,
                        ticket.reference,
                        ticket.values,
                        ticket.depends_on,
                        request.campaign,
                        item.record_identity,
                        item.source_issue
                    ]);
                    receipt.sort_all_objects();
                    fingerprint.update(receipt.to_string());
                }
                Err(e) => ticket.errors.push(e.to_string()),
            }
        }
    }
    fingerprint
        .update(serde_json::to_vec(&previously_created).expect("serializable record identifiers"));
    let confirmation = hex::encode(fingerprint.finalize());
    let valid = errors.is_empty() && tickets.iter().all(|t| t.errors.is_empty());
    BatchPreview {
        tickets,
        waves,
        errors,
        valid,
        confirmation,
        previously_created,
    }
}

fn is_original_import(ticket: &Ticket) -> bool {
    ticket.external_id.is_some()
        && crate::queue::parse_frontmatter_mapping(&ticket.content)
            .is_some_and(|(metadata, _)| !metadata.contains_key("board_source"))
}

fn fresh_items<'a>(
    existing: &[Ticket],
    items: &'a [BatchItem],
) -> (Vec<&'a BatchItem>, BTreeMap<String, String>) {
    let mut seen = BTreeMap::new();
    if items.iter().any(|item| item.record_identity.is_some()) {
        for ticket in existing {
            if let Some((map, _)) = crate::queue::parse_frontmatter_mapping(&ticket.content) {
                if let Some(key) = map
                    .get("record_identity")
                    .and_then(serde_json::Value::as_str)
                {
                    seen.insert(key.to_owned(), &ticket.id);
                }
            }
        }
    }
    let legacy: BTreeMap<_, _> = existing
        .iter()
        .filter(|ticket| is_original_import(ticket))
        .filter_map(|ticket| {
            Some((
                (
                    ticket.external_provider.as_deref()?,
                    ticket.external_id.as_deref()?,
                    ticket.external_url.as_deref()?,
                ),
                &ticket.id,
            ))
        })
        .collect();
    let mut previous = BTreeMap::new();
    let fresh = items
        .iter()
        .filter(|item| {
            if let Some(id) = item.record_identity.as_ref().and_then(|key| seen.get(key)) {
                previous.insert(item.reference.clone(), (*id).clone());
                return false;
            }
            if let Some(source) = &item.source_issue {
                if let Some(id) = legacy.get(&(
                    source.scope.provider.as_str(),
                    source.key.as_str(),
                    source.url.as_str(),
                )) {
                    previous.insert(item.reference.clone(), (*id).clone());
                    return source.include_legacy;
                }
            }
            true
        })
        .collect();
    (fresh, previous)
}

impl BatchRequest {
    pub fn expanded(mut self) -> anyhow::Result<Self> {
        anyhow::ensure!(self.board.is_none(), "Fetch board records before expanding");
        if let Some(dataset) = self.dataset.take() {
            anyhow::ensure!(
                self.tickets.is_empty(),
                "Choose either a dataset or individual tickets"
            );
            self.tickets = dataset.expand()?;
        }
        Ok(self)
    }
}

fn workflow<'a>(
    registry: &'a IssueTypeRegistry,
    item: &BatchItem,
) -> Option<(&'a str, &'a IssueType)> {
    let key = item.template.to_ascii_uppercase();
    let definition = match &item.collection {
        Some(owner) => registry.get_in(owner, &key)?,
        None => registry.get(&key)?,
    };
    let owner = if let Some(name) = &item.collection {
        registry
            .all_collections()
            .find(|entry| entry.name == *name)?
            .name
            .as_str()
    } else {
        registry.collection_of(&key).unwrap_or_default()
    };
    Some((owner, definition))
}

fn render_item(
    config: &Config,
    item: &BatchItem,
    planned: &PlannedTicket,
    owner: &str,
    definition: &IssueType,
    campaign: Option<&str>,
) -> anyhow::Result<(String, BTreeMap<String, String>, String)> {
    let mut inputs = item.values.clone();
    if let Some(project) = &item.project {
        inputs.insert("project".into(), project.clone());
    }
    if let Some(summary) = &item.summary {
        inputs.insert("summary".into(), summary.clone());
    }
    let legacy = item.collection.is_none() && TemplateType::from_key(&item.template).is_some();
    let template = if legacy {
        ticket_creation::template_markdown(&config.tickets_path(), owner, definition)
    } else {
        let draft = ticket_creation::prepare_definition(
            config,
            definition,
            owner,
            CreateTicketRequest {
                issue_type: definition.key.clone(),
                project: item.project.clone(),
                summary: item.summary.clone(),
                values: inputs,
                sync_to_kanban: Some(false),
            },
        )
        .map_err(|err| match err {
            CreationError::Invalid(errors) => anyhow::anyhow!(
                "{}",
                errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            other => anyhow::anyhow!("{other:?}"),
        })?;
        inputs = draft.values;
        draft.template
    };
    let project = inputs.get("project").map_or("", String::as_str);
    let step = definition.steps.first().map_or("", |s| s.name.as_str());
    let mut values =
        TicketCreator::values_for_id(&definition.key, step, project, planned.id.clone());
    values.extend(inputs.clone());
    values.insert("id".into(), planned.id.clone());
    values.insert(
        "branch".into(),
        format!("{}/{}", definition.key.to_lowercase(), planned.id),
    );
    let mut markdown = render_template(&template, &values)?;
    if let Some(source) = &item.source_issue {
        use std::fmt::Write;
        write!(
            markdown,
            "\n\n## Source\n\n{}\n\n{}\n",
            source.description, source.url
        )?;
    }
    let (mut metadata, body) = crate::queue::parse_frontmatter_mapping(&markdown)
        .ok_or_else(|| anyhow::anyhow!("invalid rendered ticket frontmatter"))?;
    if let Some(source) = &item.source_issue {
        metadata.insert("board_source".into(), json!(source));
        metadata.insert("external_id".into(), json!(source.key));
        metadata.insert("external_url".into(), json!(source.url));
        metadata.insert("external_provider".into(), json!(source.scope.provider));
    }
    if let Some(identity) = &item.record_identity {
        metadata.insert("record_identity".into(), json!(identity));
    }
    metadata.insert("collection".into(), json!(owner));
    metadata.insert(
        "workflow_inputs".into(),
        json!(inputs.iter().collect::<BTreeMap<_, _>>()),
    );
    if !project.is_empty() {
        metadata.insert("project".into(), json!(project));
    }
    if let Some(id) = campaign {
        metadata.insert("campaign".into(), json!(id));
    }
    if !planned.depends_on.is_empty() {
        metadata.insert("depends_on".into(), json!(planned.depends_on));
    }
    Ok((
        format!("---\n{}---{body}", serde_saphyr::to_string(&metadata)?),
        inputs.into_iter().collect(),
        crate::collections::fetch::sha256_hex(template.as_bytes()),
    ))
}

/// Run `write` for each index; on the first failure remove every file already written.
fn write_all(
    count: usize,
    mut write: impl FnMut(usize) -> anyhow::Result<PathBuf>,
) -> Result<Vec<PathBuf>, BatchError> {
    let mut written = Vec::new();
    for i in 0..count {
        match write(i) {
            Ok(path) => written.push(path),
            Err(e) => {
                for path in &written {
                    let _ = std::fs::remove_file(path);
                }
                return Err(BatchError::Write(e.to_string()));
            }
        }
    }
    Ok(written)
}

pub(crate) fn all_tickets(config: &Config) -> anyhow::Result<Vec<Ticket>> {
    let queue = Queue::new(config)?;
    let mut all = queue.list_queue()?;
    all.extend(queue.list_in_progress()?);
    all.extend(queue.list_completed()?);
    Ok(all)
}

/// Write every planned ticket or none.
pub fn create(config: &Config, request: &BatchRequest) -> Result<BatchCreated, BatchError> {
    let registry = crate::startup::templates::load_active_registry(
        &config.tickets_path(),
        config.templates.active_collection.as_deref(),
    );
    create_with_registry(config, &registry, request)
}

pub fn create_with_registry(
    config: &Config,
    registry: &IssueTypeRegistry,
    request: &BatchRequest,
) -> Result<BatchCreated, BatchError> {
    let _guard = TicketCreator::creation_lock(&config.tickets_path())
        .map_err(|e| BatchError::Write(e.to_string()))?;
    create_locked(config, registry, request)
}

pub(super) fn create_locked(
    config: &Config,
    registry: &IssueTypeRegistry,
    request: &BatchRequest,
) -> Result<BatchCreated, BatchError> {
    let existing = all_tickets(config).map_err(|e| BatchError::Write(e.to_string()))?;
    let campaign_exists = request
        .campaign
        .as_deref()
        .is_some_and(|id| CampaignStore::new(config).get(id).is_some());
    let mut planned = preview_with_registry(config, registry, &existing, request, campaign_exists);
    if request
        .confirmation
        .as_ref()
        .is_some_and(|value| value != &planned.confirmation)
    {
        planned
            .errors
            .push("Preview changed. Review the refreshed batch.".into());
        planned.valid = false;
    }
    if !planned.valid {
        return Err(BatchError::Invalid(Box::new(planned)));
    }
    let creator = TicketCreator::new(config);
    let (fresh, _) = fresh_items(&existing, &request.tickets);
    let paths = write_all(planned.tickets.len(), |i| {
        let item = fresh[i];
        let plan = &planned.tickets[i];
        let mut values: HashMap<_, _> = plan.values.clone().into_iter().collect();
        values.insert("id".into(), plan.id.clone());
        let path =
            creator.write_rendered(&item.template.to_ascii_uppercase(), &values, &plan.markdown)?;
        Ok(path)
    })?;
    let tickets = planned
        .tickets
        .iter()
        .zip(paths)
        .map(|(plan, path)| CreatedTicket {
            reference: plan.reference.clone(),
            id: plan.id.clone(),
            filename: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        })
        .collect();
    Ok(BatchCreated {
        tickets,
        waves: planned.waves,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn item(r: &str, template: &str, deps: &[&str]) -> BatchItem {
        BatchItem {
            reference: r.into(),
            template: template.into(),
            project: Some("api".into()),
            summary: Some(format!("Do {r}")),
            depends_on: deps.iter().map(ToString::to_string).collect(),
            ..BatchItem::default()
        }
    }

    fn request(items: Vec<BatchItem>) -> BatchRequest {
        BatchRequest {
            campaign: None,
            dataset: None,
            board: None,
            confirmation: None,
            tickets: items,
        }
    }

    struct Workspace {
        dir: tempfile::TempDir,
        config: Config,
    }

    impl Workspace {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let mut config = Config::default();
            config.paths.tickets = dir.path().join("tickets").to_string_lossy().into_owned();
            config.paths.state = dir.path().join("state").to_string_lossy().into_owned();
            for sub in ["queue", "in-progress", "completed"] {
                fs::create_dir_all(dir.path().join("tickets").join(sub)).unwrap();
            }
            Self { dir, config }
        }

        fn seed(&self, folder: &str, name: &str, id: &str) {
            fs::write(
                self.dir.path().join("tickets").join(folder).join(name),
                format!("---\nid: {id}\nstatus: queued\n---\n\n# {id}\n"),
            )
            .unwrap();
        }

        fn tickets(&self) -> Vec<Ticket> {
            let queue = Queue::new(&self.config).unwrap();
            let mut all = queue.list_queue().unwrap();
            all.extend(queue.list_in_progress().unwrap());
            all.extend(queue.list_completed().unwrap());
            all
        }

        fn queued(&self) -> usize {
            fs::read_dir(self.dir.path().join("tickets/queue"))
                .unwrap()
                .count()
        }
    }

    fn diamond() -> Vec<BatchItem> {
        vec![
            item("root", "FEAT", &[]),
            item("left", "FEAT", &["root"]),
            item("right", "FIX", &["root"]),
            item("join", "TASK", &["left", "right"]),
        ]
    }

    #[test]
    fn test_batch_explicit_unknown_collection_rejects_builtin_key() {
        let ws = Workspace::new();
        let request = serde_json::from_value(serde_json::json!({
            "tickets": [{"ref": "one", "template": "TASK", "collection": "missing"}]
        }))
        .unwrap();
        assert!(!preview(&ws.config, &[], &request, false).valid);
        assert!(create(&ws.config, &request).is_err());
        assert_eq!(ws.queued(), 0);
    }

    #[test]
    fn test_dataset_selects_collection_preserves_rows_and_recognizes_retries() {
        let ws = Workspace::new();
        for (collection, step) in [("court", "read"), ("other", "edit")] {
            let request:crate::rest::dto::CreateIssueTypeRequest = serde_json::from_value(json!({
                "key":"EVAL","name":"Evaluate","description":"Evaluate an opinion","glyph":">","project_required":false,
                "fields":[{"name":"opinion","description":"Source","required":true},{"name":"perspective","description":"Perspective"}],
                "steps":[{"name":step,"prompt":"Review {{opinion}} as {{perspective}}","outputs":[]}]
            })).unwrap();
            let path = ws.config.tickets_path().join("templates").join(collection);
            fs::create_dir_all(&path).unwrap();
            fs::write(
                path.join("EVAL.json"),
                request.into_issue_type().to_json().unwrap(),
            )
            .unwrap();
        }
        let recipe = json!({"collection":"court","workflow":"EVAL","source":"opinions","key_columns":["id"],"columns":{"opinion":"text"},"repeat":{"perspective":["a","b","c"]},"shared":{}});
        let mut rows: Vec<_> = (1..=12)
            .map(|id| json!({"id":id.to_string(),"text":format!("opinion {id}")}))
            .collect();
        let batch = |rows: &[serde_json::Value]| {
            serde_json::from_value::<BatchRequest>(json!({"dataset":{"recipe":recipe,"rows":rows}}))
                .unwrap()
                .expanded()
                .unwrap()
        };
        let mut request = batch(&rows);
        let planned = preview(&ws.config, &[], &request, false);
        assert!(planned.valid, "{planned:?}");
        assert_eq!(planned.tickets.len(), 36);
        assert_eq!(ws.queued(), 0);
        request.confirmation = Some(planned.confirmation);
        assert_eq!(create(&ws.config, &request).unwrap().tickets.len(), 36);
        let existing = ws.tickets();
        assert!(existing
            .iter()
            .all(|ticket| ticket.step == "read" && ticket.collection.as_deref() == Some("court")));
        assert!(existing
            .iter()
            .all(|ticket| ticket.template_schema().unwrap().steps[0].name == "read"));
        rows.reverse();
        let retry = preview(&ws.config, &existing, &batch(&rows), false);
        assert!(retry.tickets.is_empty());
        assert_eq!(retry.previously_created.len(), 36);
        rows[0]["text"] = json!("updated opinion");
        assert_eq!(
            preview(&ws.config, &existing, &batch(&rows), false)
                .tickets
                .len(),
            3
        );
    }

    #[test]
    fn test_create_rejects_stale_confirmation_without_enqueuing() {
        let ws = Workspace::new();
        let mut batch = request(vec![item("review", "TASK", &[])]);
        batch.confirmation = Some(preview(&ws.config, &[], &batch, false).confirmation);
        ws.seed("completed", "20260101-1200-TASK-api-old.md", "TASK-0001");
        let Err(BatchError::Invalid(changed)) = create(&ws.config, &batch) else {
            panic!("Expected a refreshed preview");
        };
        assert!(changed
            .errors
            .iter()
            .any(|error| error.contains("Preview changed")));
        assert_eq!(ws.queued(), 0);
    }

    #[test]
    fn test_preview_allocates_ids_past_existing_per_type() {
        let ws = Workspace::new();
        ws.seed("completed", "20260101-1200-FEAT-api-old.md", "FEAT-0007");
        let p = preview(&ws.config, &ws.tickets(), &request(diamond()), false);
        let ids: Vec<&str> = p.tickets.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["FEAT-0008", "FEAT-0009", "FIX-0001", "TASK-0001"]);
        assert!(p.valid, "{p:?}");
        assert_eq!(ws.queued(), 0, "preview writes nothing");
    }

    #[test]
    fn test_preview_resolves_refs_and_orders_waves() {
        let ws = Workspace::new();
        let p = preview(&ws.config, &ws.tickets(), &request(diamond()), false);
        assert_eq!(p.tickets[3].depends_on, vec!["FEAT-0002", "FIX-0001"]);
        assert_eq!(
            p.waves,
            vec![
                vec!["FEAT-0001".to_string()],
                vec!["FIX-0001".to_string(), "FEAT-0002".to_string()],
                vec!["TASK-0001".to_string()],
            ]
        );
        assert_eq!(p.tickets[0].filename_stem, "FEAT-api-do-root");
    }

    #[test]
    fn test_preview_passes_existing_ids_and_provider_refs() {
        let ws = Workspace::new();
        ws.seed("queue", "20260101-1200-TASK-api-gate.md", "TASK-0004");
        let items = vec![item("a", "FEAT", &["TASK-0004", "jira:PROJ-9"])];
        let p = preview(&ws.config, &ws.tickets(), &request(items), false);
        assert!(p.valid, "{p:?}");
        assert_eq!(p.tickets[0].depends_on, vec!["TASK-0004", "jira:PROJ-9"]);
        assert_eq!(p.waves, vec![vec!["FEAT-0001".to_string()]]);
    }

    #[test]
    fn test_preview_reports_item_errors() {
        let ws = Workspace::new();
        let items = vec![
            item("a", "NOPE", &[]),
            item("b", "FEAT", &["FEAT-0999"]),
            item("b", "FIX", &[]),
            item("", "TASK", &[]),
        ];
        let p = preview(&ws.config, &ws.tickets(), &request(items), false);
        assert!(!p.valid);
        assert!(p.tickets[0].errors[0].contains("unknown template"));
        assert!(p.tickets[0].id.is_empty());
        assert!(p.tickets[1].errors[0].contains("unknown dependency 'FEAT-0999'"));
        assert!(p.tickets[2].errors[0].contains("duplicate ref 'b'"));
        assert!(p.tickets[3].errors[0].contains("ref is required"));
    }

    #[test]
    fn test_preview_reports_cycle_and_unknown_campaign() {
        let ws = Workspace::new();
        let items = vec![item("a", "FEAT", &["b"]), item("b", "FEAT", &["a"])];
        let mut req = request(items);
        req.campaign = Some("CAMP-0042".into());
        let p = preview(&ws.config, &ws.tickets(), &req, false);
        assert!(!p.valid);
        assert!(
            p.errors.iter().any(|e| e.contains("dependency cycle")),
            "{p:?}"
        );
        assert!(p.errors.iter().any(|e| e.contains("CAMP-0042")), "{p:?}");
    }

    #[test]
    fn test_create_writes_tickets_with_campaign_and_dependencies() {
        let ws = Workspace::new();
        CampaignStore::new(&ws.config)
            .create(super::super::NewCampaign {
                title: "Epic".into(),
                ..Default::default()
            })
            .unwrap();
        let mut req = request(diamond());
        req.campaign = Some("CAMP-0001".into());
        let created = create(&ws.config, &req).unwrap();
        assert_eq!(created.tickets.len(), 4);
        let all = ws.tickets();
        let join = all.iter().find(|t| t.id == "TASK-0001").unwrap();
        assert_eq!(join.depends_on, vec!["FEAT-0002", "FIX-0001"]);
        assert_eq!(join.campaign.as_deref(), Some("CAMP-0001"));
        assert_eq!(join.project, "api");
        let members = super::super::members("CAMP-0001", &all);
        let plan = super::super::plan::plan(&ws.config, &members, &all).unwrap();
        assert_eq!(plan.waves, created.waves);
    }

    #[test]
    fn test_create_refuses_invalid_batch_without_writing() {
        let ws = Workspace::new();
        let mut items = diamond();
        items.push(item("bad", "NOPE", &[]));
        match create(&ws.config, &request(items)) {
            Err(BatchError::Invalid(p)) => assert!(!p.valid),
            other => panic!("expected invalid, got {other:?}"),
        }
        assert_eq!(ws.queued(), 0);
    }

    #[test]
    fn test_write_all_rolls_back_on_failure() {
        let ws = Workspace::new();
        let queue = ws.dir.path().join("tickets/queue");
        let mut n = 0;
        let result = write_all(3, |i| {
            n += 1;
            if i == 2 {
                anyhow::bail!("disk full");
            }
            let path = queue.join(format!("t{i}.md"));
            fs::write(&path, "x")?;
            Ok(path)
        });
        assert!(matches!(result, Err(BatchError::Write(m)) if m.contains("disk full")));
        assert_eq!(n, 3);
        assert_eq!(ws.queued(), 0, "written files are removed");
    }
}
