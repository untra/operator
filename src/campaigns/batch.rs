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
use crate::queue::deps::{provider_ref, resolve, Resolved};
use crate::queue::{ids, Queue, Ticket, TicketCreator, TicketPriority};
use crate::templates::TemplateType;

/// One ticket to create; `ref` names it within the batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BatchItem {
    /// Batch-local name other items use in `depends_on`.
    #[serde(rename = "ref")]
    pub reference: String,
    /// Template key (FEAT, FIX, TASK, SPIKE, INV, ...).
    pub template: String,
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
    pub tickets: Vec<BatchItem>,
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
    Invalid(BatchPreview),
    #[error("batch write failed and was rolled back: {0}")]
    Write(String),
}

/// Plan a batch against the tickets that exist; pure.
pub fn preview(
    config: &Config,
    existing: &[Ticket],
    request: &BatchRequest,
    campaign_exists: bool,
) -> BatchPreview {
    let items = &request.tickets;
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
            let (id, stem) = match TemplateType::from_key(&item.template) {
                Some(kind) => {
                    let id = ids::next_ticket_id(kind.as_str(), taken.iter().map(String::as_str));
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
                    let stem = format!("{}-{project}-{slug}", kind.as_str());
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
        .zip(items)
        .enumerate()
        .map(|(n, (t, item))| {
            let priority = item
                .values
                .get("priority")
                .map_or(TicketPriority::P2Medium, |p| {
                    TicketPriority::from_frontmatter(p)
                });
            let kind = TemplateType::from_key(&item.template).map_or("", |k| k.as_str());
            (t.id.as_str(), (config.priority_index(kind), priority, n))
        })
        .collect();
    let waves = match waves(blockers, |id| rank[id]) {
        Ok(waves) => waves,
        Err(PlanError::Cycle(ids)) => {
            errors.push(format!("dependency cycle among {}", ids.join(", ")));
            Vec::new()
        }
    };
    let valid = errors.is_empty() && tickets.iter().all(|t| t.errors.is_empty());
    BatchPreview {
        tickets,
        waves,
        errors,
        valid,
    }
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

fn all_tickets(config: &Config) -> anyhow::Result<Vec<Ticket>> {
    let queue = Queue::new(config)?;
    let mut all = queue.list_queue()?;
    all.extend(queue.list_in_progress()?);
    all.extend(queue.list_completed()?);
    Ok(all)
}

/// Write every planned ticket or none.
pub fn create(config: &Config, request: &BatchRequest) -> Result<BatchCreated, BatchError> {
    let _guard = TicketCreator::creation_lock(&config.tickets_path())
        .map_err(|e| BatchError::Write(e.to_string()))?;
    let existing = all_tickets(config).map_err(|e| BatchError::Write(e.to_string()))?;
    let campaign_exists = request
        .campaign
        .as_deref()
        .is_some_and(|id| CampaignStore::new(config).get(id).is_some());
    let planned = preview(config, &existing, request, campaign_exists);
    if !planned.valid {
        return Err(BatchError::Invalid(planned));
    }
    let creator = TicketCreator::new(config);
    let paths = write_all(planned.tickets.len(), |i| {
        let item = &request.tickets[i];
        let plan = &planned.tickets[i];
        let kind = TemplateType::from_key(&item.template)
            .ok_or_else(|| anyhow::anyhow!("unknown template '{}'", item.template))?;
        let project = item.project.clone().unwrap_or_default();
        let mut values = creator.generate_default_values(kind, &project);
        values.extend(item.values.clone());
        if let Some(summary) = &item.summary {
            values.insert("summary".into(), summary.clone());
        }
        values.insert("id".into(), plan.id.clone());
        values.insert(
            "branch".into(),
            format!("{}/{}", kind.as_str().to_lowercase(), plan.id),
        );
        let path = creator.create_ticket_headless(kind, &values)?;
        let mut ticket = Ticket::from_file(&path)?;
        ticket.set_campaign(request.campaign.as_deref())?;
        ticket.set_depends_on(&plan.depends_on)?;
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
