//! The one write path for creating a ticket.
//!
//! Every surface - the REST API (and so the web UI, the TUI and the VS Code
//! extension), MCP and the CLI - creates tickets through [`create`], so a
//! ticket made anywhere is validated against the same issue type schema,
//! rendered from the same template, numbered by the same allocator and pushed
//! to the same kanban board.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::api::kanban_sync::KanbanBidirectionalSync;
use crate::config::Config;
use crate::issuetypes::{IssueType, IssueTypeRegistry};
use crate::queue::{creator, Ticket, TicketCreator};
use crate::rest::dto::{
    CreateTicketRequest, CreateTicketResponse, FieldError, KanbanPushOutcome, KanbanTarget,
    TicketPreviewResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::templates::schema::{FieldSchema, FieldType};

/// Values operator generates for every ticket; callers may not set them.
const GENERATED_KEYS: &[&str] = &[
    "id",
    "created",
    "created_date",
    "created_datetime",
    "status",
    "step",
    "branch",
];

/// Request keys accepted beyond the issue type's own fields.
const PROJECT_KEY: &str = "project";
const SUMMARY_KEY: &str = "summary";

/// Legacy long-form issue type names some clients still send.
const LEGACY_KEY_ALIASES: &[(&str, &str)] = &[("FEATURE", "FEAT"), ("INVESTIGATION", "INV")];

/// Id shown in previews, where no id has been allocated.
const PREVIEW_ID_SUFFIX: &str = "NEW";

const BOOL_VALUES: &[&str] = &["true", "false"];
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Why a create request was rejected before anything was written.
#[derive(Debug)]
pub enum CreationError {
    UnknownIssueType { key: String, valid: Vec<String> },
    Invalid(Vec<FieldError>),
    Write(String),
}

impl From<CreationError> for ApiError {
    fn from(error: CreationError) -> Self {
        match error {
            CreationError::UnknownIssueType { key, valid } => ApiError::BadRequest(format!(
                "Unknown issue type: {key}. Valid issue types: {}",
                valid.join(", ")
            )),
            CreationError::Invalid(errors) => ApiError::ValidationError(
                errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            ),
            CreationError::Write(message) => ApiError::InternalError(message),
        }
    }
}

/// A validated request, ready to be numbered and written.
#[derive(Debug, Clone)]
pub struct TicketDraft {
    pub issue_type: IssueType,
    /// Handlebars template the ticket renders from.
    pub template: String,
    /// User values plus schema defaults, keyed by field name.
    pub values: HashMap<String, String>,
    pub project: String,
    pub sync_to_kanban: Option<bool>,
}

/// Validate `request` against its issue type without writing anything.
pub fn prepare(
    config: &Config,
    registry: &IssueTypeRegistry,
    request: CreateTicketRequest,
) -> Result<TicketDraft, CreationError> {
    let key = canonical_key(&request.issue_type);
    let Some(issue_type) = registry.get(&key) else {
        let mut valid: Vec<String> = registry
            .active_types()
            .iter()
            .map(|t| t.key.clone())
            .collect();
        valid.sort();
        return Err(CreationError::UnknownIssueType { key, valid });
    };
    let collection = registry.collection_of(&key).unwrap_or_default().to_string();

    let mut values = request.values;
    if let Some(project) = request.project {
        values.insert(PROJECT_KEY.to_string(), project);
    }
    if let Some(summary) = request.summary {
        values.insert(SUMMARY_KEY.to_string(), summary);
    }
    let project = values
        .get(PROJECT_KEY)
        .map(|p| p.trim().to_string())
        .unwrap_or_default();

    let mut errors = validate_values(&issue_type.fields, &values);
    errors.extend(validate_project(config, issue_type, &project));
    if !errors.is_empty() {
        return Err(CreationError::Invalid(errors));
    }

    apply_defaults(&issue_type.fields, &mut values);
    values.insert(PROJECT_KEY.to_string(), project.clone());

    Ok(TicketDraft {
        template: template_markdown(&config.tickets_path(), &collection, issue_type),
        issue_type: issue_type.clone(),
        values,
        project,
        sync_to_kanban: request.sync_to_kanban,
    })
}

/// Render `draft` with a placeholder id, for previews.
pub fn render_preview(config: &Config, draft: &TicketDraft) -> Result<String, CreationError> {
    let creator = TicketCreator::new(config);
    let mut values = generated_values(&creator, draft);
    values.insert(
        "id".to_string(),
        format!("{}-{PREVIEW_ID_SUFFIX}", draft.issue_type.key),
    );
    values.extend(draft.values.clone());
    render(&draft.template, &values)
}

/// Number, render and write `draft` to the queue under the creation lock.
pub fn write(config: &Config, draft: &TicketDraft) -> Result<PathBuf, CreationError> {
    let creator = TicketCreator::new(config);
    let _guard = TicketCreator::creation_lock(&config.tickets_path())
        .map_err(|e| CreationError::Write(e.to_string()))?;
    let mut values = generated_values(&creator, draft);
    values.extend(draft.values.clone());
    let content = render(&draft.template, &values)?;
    let content = match config.templates.active_collection.as_deref() {
        Some(collection) => creator::stamp_frontmatter_field(&content, "collection", collection),
        None => content,
    };
    let content = if draft.project.is_empty() {
        content
    } else {
        creator::stamp_frontmatter_field(&content, PROJECT_KEY, &draft.project)
    };
    creator
        .write_rendered(&draft.issue_type.key, &values, &content)
        .map_err(|e| CreationError::Write(e.to_string()))
}

/// Create a ticket and mirror it to its kanban board.
pub async fn create(
    state: &ApiState,
    request: CreateTicketRequest,
) -> Result<CreateTicketResponse, ApiError> {
    let config = (*state.config()).clone();
    let draft = {
        let registry = state.registry.read().await;
        prepare(&config, &registry, request)?
    };
    create_draft(config, draft, state.kanban_sync.as_deref()).await
}

/// Write a prepared draft and push it to `kanban`, for callers that hold no
/// [`ApiState`] (the CLI when no API server is running).
pub async fn create_draft(
    config: Config,
    draft: TicketDraft,
    kanban: Option<&KanbanBidirectionalSync>,
) -> Result<CreateTicketResponse, ApiError> {
    let sync_to_kanban = draft.sync_to_kanban;
    let path = tokio::task::spawn_blocking(move || write(&config, &draft))
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))??;
    let ticket = Ticket::from_file(&path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let kanban = push_to_kanban(kanban, &ticket, sync_to_kanban).await;
    if let KanbanPushOutcome::Pushed {
        provider,
        external_id,
        external_url,
        ..
    } = &kanban
    {
        record_external(&path, provider, external_id, external_url.as_deref())
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
    }

    Ok(CreateTicketResponse {
        id: ticket.id,
        filename: ticket.filename,
        path: path.to_string_lossy().into_owned(),
        kanban,
    })
}

/// Dry-run `create`: validation errors, rendered markdown and kanban target.
pub async fn preview(
    state: &ApiState,
    request: CreateTicketRequest,
) -> Result<TicketPreviewResponse, ApiError> {
    let config = (*state.config()).clone();
    let project = request
        .project
        .clone()
        .or_else(|| request.values.get(PROJECT_KEY).cloned())
        .unwrap_or_default();
    let kanban_target = kanban_target(state, &project);
    let prepared = {
        let registry = state.registry.read().await;
        prepare(&config, &registry, request)
    };
    let draft = match prepared {
        Ok(draft) => draft,
        Err(CreationError::Invalid(errors)) => {
            return Ok(TicketPreviewResponse {
                markdown: String::new(),
                kanban_target,
                errors,
            })
        }
        Err(other) => return Err(other.into()),
    };
    let markdown = tokio::task::spawn_blocking(move || render_preview(&config, &draft))
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))??;
    Ok(TicketPreviewResponse {
        markdown,
        kanban_target,
        errors: Vec::new(),
    })
}

fn kanban_target(state: &ApiState, project: &str) -> Option<KanbanTarget> {
    state.kanban_sync.as_ref()?.resolve_target(project)
}

async fn push_to_kanban(
    sync: Option<&KanbanBidirectionalSync>,
    ticket: &Ticket,
    sync_to_kanban: Option<bool>,
) -> KanbanPushOutcome {
    let Some(sync) = sync else {
        return KanbanPushOutcome::NotConfigured;
    };
    if sync.resolve_target(&ticket.project).is_none() {
        return KanbanPushOutcome::NotConfigured;
    }
    if sync_to_kanban == Some(false) {
        return KanbanPushOutcome::Skipped {
            reason: "sync_to_kanban was false".to_string(),
        };
    }
    sync.push_new_ticket(ticket, ticket_description(&ticket.content))
        .await
}

/// Write the upstream issue's identity into the ticket's frontmatter.
pub fn record_external(
    path: &Path,
    provider: &str,
    external_id: &str,
    external_url: Option<&str>,
) -> anyhow::Result<()> {
    let mut content = fs::read_to_string(path)?;
    let mut fields = vec![("external_provider", provider), ("external_id", external_id)];
    if let Some(url) = external_url {
        fields.push(("external_url", url));
    }
    for (field, value) in fields {
        content = creator::stamp_frontmatter_field(&content, field, value);
    }
    fs::write(path, content)?;
    Ok(())
}

/// The ticket body after its frontmatter and title, as an upstream description.
fn ticket_description(content: &str) -> Option<String> {
    let body = content
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---").map(|end| &rest[end + 4..]))
        .unwrap_or(content);
    let description = body
        .trim_start()
        .split_once('\n')
        .map_or("", |(title, rest)| if title.starts_with("# ") { rest } else { body })
        .trim();
    (!description.is_empty()).then(|| description.to_string())
}

fn canonical_key(raw: &str) -> String {
    let key = raw.trim().to_ascii_uppercase();
    LEGACY_KEY_ALIASES
        .iter()
        .find(|(alias, _)| *alias == key)
        .map_or(key, |(_, canonical)| (*canonical).to_string())
}

fn generated_values(creator: &TicketCreator, draft: &TicketDraft) -> HashMap<String, String> {
    let first_step = draft
        .issue_type
        .steps
        .first()
        .map(|step| step.name.as_str())
        .unwrap_or_default();
    creator.generate_default_values_for(&draft.issue_type.key, first_step, &draft.project)
}

fn render(template: &str, values: &HashMap<String, String>) -> Result<String, CreationError> {
    creator::render_template(template, values).map_err(|e| CreationError::Write(e.to_string()))
}

fn validate_values(fields: &[FieldSchema], values: &HashMap<String, String>) -> Vec<FieldError> {
    let mut errors: Vec<FieldError> = values
        .keys()
        .filter(|key| key.as_str() != PROJECT_KEY)
        .filter_map(|key| {
            let message = if GENERATED_KEYS.contains(&key.as_str()) {
                "is generated by operator and cannot be set"
            } else {
                match fields.iter().find(|f| &f.name == key) {
                    None => "is not a field of this issue type",
                    Some(field) if !field.user_editable => "is not user-editable",
                    Some(_) => return None,
                }
            };
            Some(field_error(key, message))
        })
        .collect();

    for field in fields.iter().filter(|f| f.user_editable) {
        let value = values.get(&field.name).map_or("", |v| v.trim());
        if value.is_empty() {
            if field.required && field.default.as_deref().is_none_or(str::is_empty) {
                errors.push(field_error(&field.name, "is required"));
            }
            continue;
        }
        if let Some(message) = type_error(field, value) {
            errors.push(field_error(&field.name, &message));
        }
    }
    errors.sort_by(|a, b| a.field.cmp(&b.field));
    errors
}

fn type_error(field: &FieldSchema, value: &str) -> Option<String> {
    match field.field_type {
        FieldType::Enum if !field.options.iter().any(|o| o == value) => {
            Some(format!("must be one of: {}", field.options.join(", ")))
        }
        FieldType::Bool if !BOOL_VALUES.contains(&value) => {
            Some("must be true or false".to_string())
        }
        FieldType::Integer if value.parse::<i64>().is_err() => {
            Some("must be a whole number".to_string())
        }
        FieldType::Date if chrono::NaiveDate::parse_from_str(value, DATE_FORMAT).is_err() => {
            Some("must be a date (YYYY-MM-DD)".to_string())
        }
        FieldType::String | FieldType::Text
            if field
                .max_length
                .is_some_and(|max| value.chars().count() > max) =>
        {
            Some(format!(
                "must be at most {} characters",
                field.max_length.unwrap_or_default()
            ))
        }
        _ => None,
    }
}

fn validate_project(config: &Config, issue_type: &IssueType, project: &str) -> Vec<FieldError> {
    if project.is_empty() {
        return if issue_type.project_required {
            vec![field_error(PROJECT_KEY, "is required for this issue type")]
        } else {
            Vec::new()
        };
    }
    // The same set `GET /api/v1/projects` offers as existing.
    let known = config.projects.iter().any(|p| p == project)
        && config.projects_path().join(project).is_dir();
    if known {
        Vec::new()
    } else {
        vec![field_error(PROJECT_KEY, "is not a known project")]
    }
}

fn apply_defaults(fields: &[FieldSchema], values: &mut HashMap<String, String>) {
    for field in fields.iter().filter(|f| f.user_editable) {
        let missing = values.get(&field.name).is_none_or(|v| v.trim().is_empty());
        if let (true, Some(default)) = (missing, field.default.as_ref()) {
            if !default.is_empty() {
                values.insert(field.name.clone(), default.clone());
            }
        }
    }
}

fn field_error(field: &str, message: &str) -> FieldError {
    FieldError {
        field: field.to_string(),
        message: message.to_string(),
    }
}

/// The issue type's markdown template - from its collection on disk, then its
/// embedded collection, then generated from its fields - with a section
/// appended for every user field the template never references, so no value
/// a user enters is silently dropped.
fn template_markdown(tickets_path: &Path, collection: &str, issue_type: &IssueType) -> String {
    let key = issue_type.key.as_str();
    let on_disk = fs::read_to_string(
        tickets_path
            .join("templates")
            .join(collection)
            .join(format!("{key}.md")),
    )
    .ok();
    let embedded = || {
        crate::collections::EMBEDDED_COLLECTIONS
            .iter()
            .filter(|c| c.name == collection)
            .flat_map(|c| c.issuetypes)
            .find(|it| it.key == key)
            .map(|it| it.template_md.to_string())
    };
    let base = on_disk
        .or_else(embedded)
        .unwrap_or_else(|| generic_template(issue_type));
    append_unreferenced_fields(base, &issue_type.fields)
}

fn generic_template(issue_type: &IssueType) -> String {
    format!(
        "---\nid: {{{{ id }}}}\n{{{{#if step }}}}step: {{{{ step }}}}\n{{{{/if}}}}status: {{{{ status }}}}\ncreated: {{{{ created_datetime }}}}\n{{{{#if priority }}}}priority: {{{{ priority }}}}\n{{{{/if}}}}---\n\n# {}: {{{{ summary }}}}\n",
        issue_type.name
    )
}

fn append_unreferenced_fields(mut template: String, fields: &[FieldSchema]) -> String {
    for field in fields
        .iter()
        .filter(|f| f.user_editable && f.name != SUMMARY_KEY && f.name != PROJECT_KEY)
    {
        if references(&template, &field.name) {
            continue;
        }
        let name = &field.name;
        template.push_str(&format!(
            "\n{{{{#if {name} }}}}\n## {}\n{{{{ {name} }}}}\n{{{{/if}}}}\n",
            title_case(name)
        ));
    }
    template
}

fn references(template: &str, name: &str) -> bool {
    Regex::new(&format!(r"\{{\{{[^}}]*\b{}\b[^}}]*\}}\}}", regex::escape(name)))
        .is_ok_and(|re| re.is_match(template))
}

fn title_case(name: &str) -> String {
    name.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT: &str = "gamesvc";

    fn config_in(dir: &Path) -> Config {
        let mut config = Config::default();
        config.paths.tickets = dir.join("tickets").to_string_lossy().into_owned();
        config.paths.projects = dir.join("projects").to_string_lossy().into_owned();
        add_project(&mut config, PROJECT);
        config
    }

    fn add_project(config: &mut Config, name: &str) {
        fs::create_dir_all(config.projects_path().join(name)).unwrap();
        config.projects.push(name.to_string());
    }

    fn registry() -> IssueTypeRegistry {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();
        registry
    }

    fn request(issue_type: &str, values: &[(&str, &str)]) -> CreateTicketRequest {
        CreateTicketRequest {
            issue_type: issue_type.to_string(),
            project: Some(PROJECT.to_string()),
            summary: Some("Add pagination".to_string()),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            sync_to_kanban: None,
        }
    }

    fn invalid(result: Result<TicketDraft, CreationError>) -> Vec<FieldError> {
        match result {
            Err(CreationError::Invalid(errors)) => errors,
            other => panic!("expected validation errors, got {other:?}"),
        }
    }

    fn fields_of(errors: &[FieldError]) -> Vec<&str> {
        errors.iter().map(|e| e.field.as_str()).collect()
    }

    #[test]
    fn test_unknown_issue_type_lists_valid_keys() {
        let dir = tempfile::tempdir().unwrap();
        let result = prepare(&config_in(dir.path()), &registry(), request("NOPE", &[]));
        match result {
            Err(CreationError::UnknownIssueType { key, valid }) => {
                assert_eq!(key, "NOPE");
                assert!(valid.contains(&"FEAT".to_string()), "{valid:?}");
            }
            other => panic!("expected unknown issue type, got {other:?}"),
        }
    }

    #[test]
    fn test_legacy_long_names_and_lowercase_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        for raw in ["feature", "feat", "FEAT"] {
            let draft = prepare(&config, &registry(), request(raw, &[])).unwrap();
            assert_eq!(draft.issue_type.key, "FEAT");
        }
    }

    #[test]
    fn test_custom_collection_type_is_creatable() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        let mut registry = registry();
        let mut custom = registry.get("TASK").unwrap().clone();
        custom.key = "CHORE".to_string();
        custom.name = "Chore".to_string();
        registry.register_in("custom", custom).unwrap();

        let draft = prepare(
            &config,
            &registry,
            request("CHORE", &[("description", "tidy the build")]),
        )
        .unwrap();
        let path = write(&config, &draft).unwrap();

        let ticket = Ticket::from_file(&path).unwrap();
        assert_eq!(ticket.id, "CHORE-0001");
        assert_eq!(ticket.ticket_type, "CHORE");
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Chore: Add pagination"), "{content}");
        assert!(content.contains("tidy the build"), "{content}");
    }

    #[test]
    fn test_field_validation_reports_every_problem() {
        let dir = tempfile::tempdir().unwrap();
        let long = "x".repeat(121);
        let mut req = request(
            "FEAT",
            &[("priority", "P9-whenever"), ("bogus", "1"), ("id", "FEAT-9")],
        );
        req.summary = Some(long);
        let errors = invalid(prepare(&config_in(dir.path()), &registry(), req));
        assert_eq!(fields_of(&errors), ["bogus", "id", "priority", "summary"]);
    }

    #[test]
    fn test_required_field_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut req = request("FEAT", &[]);
        req.summary = None;
        let errors = invalid(prepare(&config_in(dir.path()), &registry(), req));
        assert_eq!(fields_of(&errors), ["summary"]);
    }

    #[test]
    fn test_typed_fields_are_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let errors = invalid(prepare(
            &config_in(dir.path()),
            &registry(),
            request("TASK", &[("description", "d"), ("points", "three")]),
        ));
        assert_eq!(fields_of(&errors), ["points"]);
    }

    #[test]
    fn test_schema_defaults_are_applied() {
        let dir = tempfile::tempdir().unwrap();
        let draft = prepare(&config_in(dir.path()), &registry(), request("FEAT", &[])).unwrap();
        assert_eq!(
            draft.values.get("priority").map(String::as_str),
            Some("P2-medium")
        );
    }

    #[test]
    fn test_project_rules() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());

        let mut missing = request("FEAT", &[]);
        missing.project = None;
        assert_eq!(
            fields_of(&invalid(prepare(&config, &registry(), missing))),
            ["project"]
        );

        let mut unknown = request("FEAT", &[]);
        unknown.project = Some("nowhere".to_string());
        assert_eq!(
            fields_of(&invalid(prepare(&config, &registry(), unknown))),
            ["project"]
        );

        let mut optional = request("TASK", &[("description", "d")]);
        optional.project = None;
        assert!(prepare(&config, &registry(), optional).is_ok());
    }

    #[test]
    fn test_written_ticket_carries_generated_fields_and_project() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        let draft = prepare(
            &config,
            &registry(),
            request("FEAT", &[("user_story", "As a player I want pages")]),
        )
        .unwrap();
        let path = write(&config, &draft).unwrap();

        let ticket = Ticket::from_file(&path).unwrap();
        assert_eq!(ticket.id, "FEAT-0001");
        assert_eq!(ticket.step, "plan");
        assert_eq!(ticket.project, PROJECT);
        assert_eq!(ticket.priority, "P2-medium");
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("## User Story\nAs a player I want pages"),
            "a field the template never references must still land: {content}"
        );
    }

    #[test]
    fn test_hyphenated_project_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = config_in(dir.path());
        add_project(&mut config, "gbqr-us");
        let mut req = request("FEAT", &[]);
        req.project = Some("gbqr-us".to_string());
        let path = write(&config, &prepare(&config, &registry(), req).unwrap()).unwrap();
        assert_eq!(Ticket::from_file(&path).unwrap().project, "gbqr-us");
    }

    #[test]
    fn test_concurrent_writes_get_distinct_ids() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        let draft = prepare(&config, &registry(), request("FEAT", &[])).unwrap();
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let (config, draft) = (config.clone(), draft.clone());
                std::thread::spawn(move || write(&config, &draft).unwrap())
            })
            .collect();
        let mut ids: Vec<String> = handles
            .into_iter()
            .map(|h| Ticket::from_file(&h.join().unwrap()).unwrap().id)
            .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn test_preview_renders_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        let draft = prepare(&config, &registry(), request("FEAT", &[])).unwrap();
        let markdown = render_preview(&config, &draft).unwrap();
        assert!(markdown.contains("id: FEAT-NEW"), "{markdown}");
        assert!(!config.tickets_path().join("queue").exists());
    }

    #[test]
    fn test_record_external_stamps_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("20260101-1200-FEAT-gamesvc-x.md");
        fs::write(&path, "---\nid: FEAT-0001\n---\n\n# Feature: x\n").unwrap();
        record_external(&path, "linear", "ENG-12", Some("https://linear.app/i/ENG-12")).unwrap();
        let ticket = Ticket::from_file(&path).unwrap();
        assert_eq!(ticket.external_provider.as_deref(), Some("linear"));
        assert_eq!(ticket.external_id.as_deref(), Some("ENG-12"));
        assert_eq!(
            ticket.external_url.as_deref(),
            Some("https://linear.app/i/ENG-12")
        );
    }

    #[test]
    fn test_ticket_description_drops_frontmatter_and_title() {
        let content = "---\nid: FEAT-1\n---\n\n# Feature: x\n\n## User Story\nwhy\n";
        assert_eq!(
            ticket_description(content).as_deref(),
            Some("## User Story\nwhy")
        );
        assert_eq!(ticket_description("---\nid: A\n---\n\n# Feature: x\n"), None);
    }
}
