//! Ticket CRUD endpoints for the REST API.
//!
//! Provides endpoints for fetching ticket details and updating ticket status.
//! These endpoints power the embedded web UI's kanban board and detail drawer.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::agents::tmux::OPERATOR_SOCKET;
use crate::config::Config;
use crate::queue::{deps, Queue, Ticket, TicketColumn, TicketStatus};
use crate::rest::dto::{
    CreateAlertRequest, CreateAlertResponse, CreateTicketRequest, CreateTicketResponse,
    TicketAgent, TicketDependency, TicketDetailResponse, TicketPreviewResponse,
    UpdateTicketStatusRequest, UpdateTicketStatusResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::services::{ticket_creation, ticket_transitions};

/// Find a ticket across all directories (queue, in-progress, completed)
pub(crate) fn find_ticket_anywhere(queue: &Queue, ticket_id: &str) -> Result<Ticket, ApiError> {
    // find_ticket searches queue + in-progress
    if let Some(ticket) = queue
        .find_ticket(ticket_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
    {
        return Ok(ticket);
    }

    // Also search completed
    let completed = queue
        .list_completed()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    completed
        .into_iter()
        .find(|t| t.id == ticket_id || t.filename.contains(ticket_id))
        .ok_or_else(|| ApiError::NotFound(format!("Ticket '{ticket_id}' not found")))
}

/// Get full ticket details by ID
///
/// Returns complete ticket data including content, metadata, step history,
/// and session information. Searches queue, in-progress, and completed directories.
#[utoipa::path(
    operation_id = "tickets_get_one",
    get,
    path = "/api/v1/tickets/{id}",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID (e.g., FEAT-7598)")
    ),
    responses(
        (status = 200, description = "Ticket details", body = TicketDetailResponse),
        (status = 404, description = "Ticket not found")
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<TicketDetailResponse>, ApiError> {
    let config = state.config();
    let queue = Queue::new(&config).map_err(|e| ApiError::InternalError(e.to_string()))?;
    let ticket = find_ticket_anywhere(&queue, &ticket_id)?;
    let all = all_tickets(&queue)?;
    let operator_state = crate::state::State::load(&config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;
    let agent = operator_state.agent_by_ticket(&ticket.id);
    Ok(Json(ticket_detail(&config, &ticket, &all, agent)))
}

fn all_tickets(queue: &Queue) -> Result<Vec<Ticket>, ApiError> {
    let internal = |e: anyhow::Error| ApiError::InternalError(e.to_string());
    let mut all = queue.list_queue().map_err(internal)?;
    all.extend(queue.list_in_progress().map_err(internal)?);
    all.extend(queue.list_completed().map_err(internal)?);
    Ok(all)
}

fn dependency(reference: &str, all: &[Ticket]) -> TicketDependency {
    let local = match deps::resolve(reference, all) {
        deps::Resolved::Local(id) => all.iter().find(|t| t.id == id),
        deps::Resolved::Unsynced(_) => None,
    };
    let column = local.map(Ticket::column);
    TicketDependency {
        reference: reference.to_string(),
        ticket_id: local.map(|t| t.id.clone()),
        satisfied: column.as_deref() == Some("completed"),
        column,
    }
}

fn dependents(ticket: &Ticket, all: &[Ticket]) -> Vec<String> {
    all.iter()
        .filter(|t| {
            t.depends_on
                .iter()
                .any(|r| deps::resolve(r, all) == deps::Resolved::Local(ticket.id.clone()))
        })
        .map(|t| t.id.clone())
        .collect()
}

/// Shell command that attaches to the agent's session; tmux only.
fn attach_command(config: &Config, agent: &crate::state::AgentState) -> Option<String> {
    let session = agent.session_name.as_deref()?;
    let wrapper = agent.session_wrapper.as_deref().unwrap_or("tmux");
    (wrapper == "tmux").then(|| {
        let socket = config.tmux.config_generated && config.tmux_config_path().exists();
        let flag = if socket {
            format!("-L {OPERATOR_SOCKET} ")
        } else {
            String::new()
        };
        format!("tmux {flag}attach -t {session}")
    })
}

/// Full detail of one ticket, its dependencies resolved against `all`.
fn ticket_detail(
    config: &Config,
    ticket: &Ticket,
    all: &[Ticket],
    agent: Option<&crate::state::AgentState>,
) -> TicketDetailResponse {
    let step_display_name = ticket.current_step_display_name();
    TicketDetailResponse {
        id: ticket.id.clone(),
        summary: ticket.summary.clone(),
        ticket_type: ticket.ticket_type.clone(),
        project: ticket.project.clone(),
        status: ticket.status.clone(),
        step: ticket.step.clone(),
        step_display_name: (!step_display_name.is_empty()).then_some(step_display_name),
        priority: ticket.priority.clone(),
        timestamp: ticket.timestamp.clone(),
        content: ticket.content.clone(),
        filename: ticket.filename.clone(),
        filepath: ticket.filepath.clone(),
        sessions: ticket.sessions.clone(),
        step_delegators: ticket.step_delegators.clone(),
        worktree_path: ticket.worktree_path.clone(),
        branch: ticket.branch.clone(),
        external_id: ticket.external_id.clone(),
        external_url: ticket.external_url.clone(),
        external_provider: ticket.external_provider.clone(),
        body: ticket.body().to_string(),
        column: ticket.column(),
        campaign: ticket.campaign.clone(),
        depends_on: ticket.depends_on.clone(),
        dependencies: ticket
            .depends_on
            .iter()
            .map(|r| dependency(r, all))
            .collect(),
        dependents: dependents(ticket, all),
        external_parent: ticket.external_parent.clone(),
        collection: ticket.collection.clone(),
        llm_task: ticket.llm_task.clone(),
        agent: agent.map(|a| TicketAgent {
            attach_command: attach_command(config, a),
            ..TicketAgent::from(a)
        }),
    }
}

/// Update a ticket's status
///
/// Moves a ticket between queue directories based on the target status.
/// Accepts queued, running, awaiting and completed; `done` is an accepted
/// alias for `completed`, which is what gets written to frontmatter.
#[utoipa::path(
    operation_id = "tickets_update_status",
    put,
    path = "/api/v1/tickets/{id}/status",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID to update")
    ),
    request_body = UpdateTicketStatusRequest,
    responses(
        (status = 200, description = "Ticket status updated", body = UpdateTicketStatusResponse),
        (status = 400, description = "Invalid status value"),
        (status = 404, description = "Ticket not found")
    )
)]
pub async fn update_status(
    State(state): State<ApiState>,
    Path(ticket_id): Path<String>,
    Json(request): Json<UpdateTicketStatusRequest>,
) -> Result<Json<UpdateTicketStatusResponse>, ApiError> {
    // Parsed here rather than on the request DTO so an unknown value stays a
    // 400 from us instead of a 422 from the JSON extractor.
    let target = TicketStatus::parse(&request.status).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: queued, running, awaiting, completed",
            request.status
        ))
    })?;

    let queue = Queue::new(&state.config()).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let ticket = find_ticket_anywhere(&queue, &ticket_id)?;

    let previous_status = TicketStatus::from_frontmatter(&ticket.status);

    let column = match target {
        TicketStatus::Queued => TicketColumn::Queue,
        TicketStatus::Running | TicketStatus::Awaiting => TicketColumn::InProgress,
        TicketStatus::Completed => TicketColumn::Completed,
    };

    // The shared write path: it also mirrors the move to the board this ticket
    // was synced from. A direct rename here would strand it upstream.
    ticket_transitions::move_ticket(&state, &ticket, column)
        .await
        .map_err(ApiError::InternalError)?;

    let dst = state
        .config()
        .tickets_path()
        .join(column.dir_name())
        .join(&ticket.filename);

    // Compared against the raw field so a legacy `done` is normalised too.
    if ticket.status != target.as_str() {
        let mut moved_ticket = Ticket::from_file(&dst)
            .map_err(|e| ApiError::InternalError(format!("Failed to reload ticket: {e}")))?;
        moved_ticket
            .update_field("status", target.as_str())
            .map_err(|e| ApiError::InternalError(format!("Failed to update status field: {e}")))?;
    }

    Ok(Json(UpdateTicketStatusResponse {
        id: ticket.id,
        previous_status,
        status: target,
        message: format!("Ticket moved to '{target}'"),
    }))
}

/// Create a new ticket and write it to the queue.
///
/// Validates against the issue type's field schema, numbers and renders the
/// ticket through [`ticket_creation`] - the same path MCP and the CLI use - and
/// pushes it to the project's bidirectional kanban board unless
/// `sync_to_kanban` is `false`.
#[utoipa::path(
    operation_id = "tickets_create",
    post,
    path = "/api/v1/tickets",
    tag = "Tickets",
    request_body = CreateTicketRequest,
    responses(
        (status = 200, description = "Ticket created", body = CreateTicketResponse),
        (status = 400, description = "Unknown issue type or invalid field values", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(request): Json<CreateTicketRequest>,
) -> Result<Json<CreateTicketResponse>, ApiError> {
    ticket_creation::create(&state, request).await.map(Json)
}

/// Dry-run a create: field errors, rendered markdown and kanban target.
#[utoipa::path(
    operation_id = "tickets_preview",
    post,
    path = "/api/v1/tickets/preview",
    tag = "Tickets",
    request_body = CreateTicketRequest,
    responses(
        (status = 200, description = "Preview of the ticket that would be created", body = TicketPreviewResponse),
        (status = 400, description = "Unknown issue type", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn preview(
    State(state): State<ApiState>,
    Json(request): Json<CreateTicketRequest>,
) -> Result<Json<TicketPreviewResponse>, ApiError> {
    ticket_creation::preview(&state, request).await.map(Json)
/// Shared ticket-creation core for the REST surface: overlays caller `values`
/// on top of the template's generated defaults (so the ticket gets a real id,
/// branch, etc.), writes it to the queue via the embedded-template
/// [`TicketCreator`], and reloads it. Used by both `create` and `create_alert`.
pub(crate) async fn create_ticket_from_values(
    state: &ApiState,
    template_type: TemplateType,
    values: HashMap<String, String>,
) -> Result<(Ticket, std::path::PathBuf), ApiError> {
    let config = (*state.config()).clone();
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf, String> {
        let _guard = TicketCreator::creation_lock();
        let creator = TicketCreator::new(&config);
        let project = values.get("project").cloned().unwrap_or_default();
        // Start from generated defaults (id, created, branch, step, status),
        // then overlay the caller's values so explicit fields win.
        let mut merged = creator.generate_default_values(template_type, &project);
        merged.extend(values);
        creator
            .create_ticket_headless(template_type, &merged)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))?
    .map_err(ApiError::InternalError)?;

    let ticket = Ticket::from_file(&path).map_err(|e| ApiError::InternalError(e.to_string()))?;
    Ok((ticket, path))
}

/// Issue type alerts become.
const ALERT_ISSUE_TYPE: &str = "INV";
/// `source` option for tickets raised by an external alert.
const ALERT_SOURCE: &str = "alert";

/// Raise an external alert as an investigation (INV) ticket.
///
/// The alert's free-form `source`/`severity` are recorded in the body; the
/// frontmatter gets the schema's `alert` source and the severity option whose
/// `S0`/`S1`/`S2` prefix matches, if any.
#[utoipa::path(
    operation_id = "alerts_create",
    post,
    path = "/api/v1/alerts",
    tag = "Tickets",
    request_body = CreateAlertRequest,
    responses(
        (status = 200, description = "Investigation created", body = CreateAlertResponse),
        (status = 500, description = "Failed to create investigation", body = crate::rest::error::ErrorResponse)
    )
)]
pub async fn create_alert(
    State(state): State<ApiState>,
    Json(request): Json<CreateAlertRequest>,
) -> Result<Json<CreateAlertResponse>, ApiError> {
    let severity_options = {
        let registry = state.registry.read().await;
        registry
            .get(ALERT_ISSUE_TYPE)
            .and_then(|t| t.fields.iter().find(|f| f.name == "severity"))
            .map(|f| f.options.clone())
            .unwrap_or_default()
    };
    let created = ticket_creation::create(&state, alert_request(request, &severity_options)).await?;
    Ok(Json(CreateAlertResponse {
        id: created.id,
        filename: created.filename,
    }))
}

fn alert_request(alert: CreateAlertRequest, severity_options: &[String]) -> CreateTicketRequest {
    let CreateAlertRequest {
        source,
        message,
        severity,
        project,
    } = alert;
    let mut values = HashMap::from([
        ("source".to_string(), ALERT_SOURCE.to_string()),
        (
            "observed_behavior".to_string(),
            format!("Raised by {source} (severity {severity}): {message}"),
        ),
    ]);
    let wanted = severity.to_ascii_uppercase();
    if let Some(option) = severity_options.iter().find(|option| {
        option
            .split('-')
            .next()
            .is_some_and(|prefix| wanted.starts_with(prefix))
    }) {
        values.insert("severity".to_string(), option.clone());
    }
    CreateTicketRequest {
        issue_type: ALERT_ISSUE_TYPE.to_string(),
        project,
        summary: Some(message),
        values,
        sync_to_kanban: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        make_state_in(&tempfile::TempDir::new().unwrap().keep())
    }

    fn make_state_in(dir: &std::path::Path) -> ApiState {
        let projects = dir.join("projects");
        std::fs::create_dir_all(projects.join("gamesvc")).unwrap();
        let mut config = Config::default();
        config.paths.tickets = dir.to_string_lossy().into_owned();
        config.paths.state = dir.join("state").to_string_lossy().into_owned();
        config.paths.projects = projects.to_string_lossy().into_owned();
        config.projects = vec!["gamesvc".to_string()];
        ApiState::new(config, dir.to_path_buf())
    }

    #[test]
    fn test_valid_statuses() {
        for raw in ["queued", "running", "awaiting", "completed", "done"] {
            assert!(TicketStatus::parse(raw).is_some(), "{raw} should parse");
        }
        assert!(TicketStatus::parse("invalid").is_none());
    }

    /// The API still accepts `done`, but the metadata schema only knows
    /// `completed` - that is what has to land in the file.
    #[tokio::test]
    async fn test_update_status_done_writes_completed_frontmatter() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let created = create(
            State(state.clone()),
            Json(CreateTicketRequest {
                issue_type: "feat".to_string(),
                project: Some("gamesvc".to_string()),
                summary: Some("Ship it".to_string()),
                values: std::collections::HashMap::new(),
                sync_to_kanban: None,
            }),
        )
        .await
        .expect("create should succeed")
        .0;

        let moved = update_status(
            State(state.clone()),
            Path(created.id.clone()),
            Json(UpdateTicketStatusRequest {
                status: "done".to_string(),
            }),
        )
        .await
        .expect("done should be accepted");
        assert_eq!(moved.0.status, TicketStatus::Completed);

        let landed = temp.path().join("completed").join(&created.filename);
        let body = std::fs::read_to_string(&landed).unwrap();
        assert!(body.contains("status: completed"), "got:\n{body}");
        assert!(!body.contains("status: done"));
    }

    #[tokio::test]
    async fn test_update_status_rejects_unknown_status() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let result = update_status(
            State(state),
            Path("FEAT-001".to_string()),
            Json(UpdateTicketStatusRequest {
                status: "shipped".to_string(),
            }),
        )
        .await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_get_ticket_not_found() {
        let state = make_state();
        let result = get_one(State(state), Path("NONEXISTENT-999".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_status_invalid() {
        let state = make_state();
        let request = UpdateTicketStatusRequest {
            status: "invalid".to_string(),
        };
        let result = update_status(State(state), Path("FEAT-001".to_string()), Json(request)).await;
        assert!(result.is_err());
    }

    /// Every column move must relocate the file, including the two the three
    /// named transitions do not cover: skipping in-progress, and reopening a
    /// completed ticket.
    #[tokio::test]
    async fn test_update_status_moves_the_ticket_between_every_column() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let created = create(
            State(state.clone()),
            Json(CreateTicketRequest {
                issue_type: "feat".to_string(),
                project: Some("gamesvc".to_string()),
                summary: Some("Add pagination".to_string()),
                values: std::collections::HashMap::new(),
                sync_to_kanban: None,
            }),
        )
        .await
        .expect("create should succeed")
        .0;

        // `done` is the request word; `completed` is the stored one.
        for (status, expected, dir) in [
            ("done", TicketStatus::Completed, "completed"),
            ("queued", TicketStatus::Queued, "queue"),
            ("running", TicketStatus::Running, "in-progress"),
            ("done", TicketStatus::Completed, "completed"),
        ] {
            let moved = update_status(
                State(state.clone()),
                Path(created.id.clone()),
                Json(UpdateTicketStatusRequest {
                    status: status.to_string(),
                }),
            )
            .await
            .unwrap_or_else(|e| panic!("move to {status} failed: {e:?}"));
            assert_eq!(moved.0.status, expected);

            let landed = temp.path().join(dir).join(&created.filename);
            assert!(landed.exists(), "{status} should land the ticket in {dir}/");
            let body = std::fs::read_to_string(&landed).unwrap();
            assert!(
                body.contains(&format!("status: {expected}")),
                "status field not rewritten for {status}"
            );
        }
    }

    /// Moving a ticket to the column it already occupies must not lose the file.
    #[tokio::test]
    async fn test_update_status_to_the_same_column_is_a_no_op() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let created = create(
            State(state.clone()),
            Json(CreateTicketRequest {
                issue_type: "feat".to_string(),
                project: Some("gamesvc".to_string()),
                summary: Some("Stay put".to_string()),
                values: std::collections::HashMap::new(),
                sync_to_kanban: None,
            }),
        )
        .await
        .expect("create should succeed")
        .0;

        let moved = update_status(
            State(state.clone()),
            Path(created.id.clone()),
            Json(UpdateTicketStatusRequest {
                status: "queued".to_string(),
            }),
        )
        .await
        .expect("no-op move should succeed");
        assert_eq!(moved.0.status, TicketStatus::Queued);

        assert!(temp.path().join("queue").join(&created.filename).exists());
    }

    #[tokio::test]
    async fn test_create_unknown_template_is_bad_request() {
        let state = make_state();
        let request = CreateTicketRequest {
            issue_type: "not-a-template".to_string(),
            project: None,
            summary: None,
            values: std::collections::HashMap::new(),
            sync_to_kanban: None,
        };
        let result = create(State(state), Json(request)).await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_create_ticket_writes_to_queue() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let request = CreateTicketRequest {
            issue_type: "feat".to_string(),
            project: Some("gamesvc".to_string()),
            summary: Some("Add pagination".to_string()),
            values: std::collections::HashMap::new(),
            sync_to_kanban: None,
        };
        let resp = create(State(state), Json(request))
            .await
            .expect("create should succeed")
            .0;
        assert!(!resp.id.is_empty(), "created ticket has an id");
        assert!(resp.filename.contains(".md"), "filename: {}", resp.filename);
        assert!(
            std::path::Path::new(&resp.path).exists(),
            "ticket file written to {}",
            resp.path
        );
    }

    #[tokio::test]
    async fn test_create_alert_creates_investigation() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let request = CreateAlertRequest {
            source: "pagerduty".to_string(),
            message: "500 errors in backend".to_string(),
            severity: "S1".to_string(),
            project: Some("gamesvc".to_string()),
        };
        let resp = create_alert(State(state), Json(request))
            .await
            .expect("alert should create an investigation")
            .0;
        assert!(!resp.id.is_empty(), "investigation has an id");
        assert!(resp.filename.contains(".md"), "filename: {}", resp.filename);
        let written =
            std::fs::read_to_string(temp.path().join("queue").join(&resp.filename)).unwrap();
        assert!(written.contains("severity: S1-major"), "{written}");
        assert!(written.contains("source: alert"), "{written}");
        assert!(written.contains("Raised by pagerduty"), "{written}");
    }

    #[tokio::test]
    async fn test_create_reports_no_kanban_when_unconfigured() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let request = CreateTicketRequest {
            issue_type: "FEAT".to_string(),
            project: Some("gamesvc".to_string()),
            summary: Some("Add pagination".to_string()),
            ..Default::default()
        };
        let resp = create(State(state), Json(request)).await.unwrap().0;
        assert_eq!(resp.kanban, crate::rest::dto::KanbanPushOutcome::NotConfigured);
    }

    #[tokio::test]
    async fn test_preview_returns_field_errors_instead_of_failing() {
        let temp = tempfile::TempDir::new().unwrap();
        let state = make_state_in(temp.path());
        let request = CreateTicketRequest {
            issue_type: "FEAT".to_string(),
            project: Some("gamesvc".to_string()),
            ..Default::default()
        };
        let resp = preview(State(state), Json(request)).await.unwrap().0;
        assert_eq!(resp.errors.len(), 1);
        assert_eq!(resp.errors[0].field, "summary");
        assert!(resp.markdown.is_empty());
    }

    fn write_ticket(root: &std::path::Path, column: &str, name: &str, frontmatter: &str) -> Ticket {
        let dir = root.join(column);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(
            &path,
            format!("---\n{frontmatter}\n---\n\n# Body of {name}\n"),
        )
        .unwrap();
        Ticket::from_file(&path).unwrap()
    }

    fn agent_for(ticket_id: &str) -> crate::state::AgentState {
        serde_json::from_value(serde_json::json!({
            "id": "agent-1", "ticket_id": ticket_id, "ticket_type": "FEAT", "project": "api",
            "status": "running", "started_at": "2026-01-01T00:00:00Z",
            "last_activity": "2026-01-01T00:00:00Z", "paired": false,
            "session_wrapper": "tmux", "session_name": "op-feat-2",
            "current_step": "build", "completed_steps": ["plan"],
            "pr_url": "https://example.test/pr/9", "pr_number": 9
        }))
        .unwrap()
    }

    #[test]
    fn test_ticket_detail_resolves_dependencies_and_dependents() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let done = write_ticket(
            root,
            "completed",
            "20260101-0000-FEAT-api-a.md",
            "id: FEAT-1\nstatus: completed",
        );
        let me = write_ticket(
            root,
            "in-progress",
            "20260101-0001-FEAT-api-b.md",
            "id: FEAT-2\ncampaign: CAMP-0001\ndepends_on:\n  - FEAT-1\n  - jira:PROJ-9",
        );
        let child = write_ticket(
            root,
            "queue",
            "20260101-0002-FEAT-api-c.md",
            "id: FEAT-3\ndepends_on:\n  - FEAT-2",
        );
        let all = vec![done, me.clone(), child];

        let detail = ticket_detail(&Config::default(), &me, &all, None);

        assert_eq!(detail.body, "# Body of 20260101-0001-FEAT-api-b.md\n");
        assert_eq!(detail.column, "in-progress");
        assert_eq!(detail.campaign.as_deref(), Some("CAMP-0001"));
        assert_eq!(detail.depends_on, vec!["FEAT-1", "jira:PROJ-9"]);
        assert_eq!(
            detail.dependencies,
            vec![
                TicketDependency {
                    reference: "FEAT-1".into(),
                    ticket_id: Some("FEAT-1".into()),
                    column: Some("completed".into()),
                    satisfied: true,
                },
                TicketDependency {
                    reference: "jira:PROJ-9".into(),
                    ticket_id: None,
                    column: None,
                    satisfied: false,
                },
            ]
        );
        assert_eq!(detail.dependents, vec!["FEAT-3"]);
        assert!(detail.agent.is_none());
    }

    #[test]
    fn test_ticket_detail_carries_the_live_agent() {
        let temp = tempfile::TempDir::new().unwrap();
        let me = write_ticket(
            temp.path(),
            "in-progress",
            "20260101-0001-FEAT-api-b.md",
            "id: FEAT-2",
        );
        let agent = agent_for("FEAT-2");

        let detail = ticket_detail(
            &Config::default(),
            &me,
            std::slice::from_ref(&me),
            Some(&agent),
        );

        let live = detail.agent.expect("agent");
        assert_eq!(live.id, "agent-1");
        assert_eq!(live.session_wrapper.as_deref(), Some("tmux"));
        assert_eq!(live.session_name.as_deref(), Some("op-feat-2"));
        assert_eq!(live.completed_steps, vec!["plan"]);
        assert_eq!(live.pr_number, Some(9));
        assert_eq!(
            live.attach_command.as_deref(),
            Some("tmux attach -t op-feat-2")
        );
    }

    #[test]
    fn test_attach_command_uses_the_operator_socket_when_configured() {
        let temp = tempfile::TempDir::new().unwrap();
        let mut config = Config::default();
        config.paths.tickets = temp.path().to_string_lossy().into_owned();
        config.tmux.config_generated = true;
        let conf = config.tmux_config_path();
        std::fs::create_dir_all(conf.parent().unwrap()).unwrap();
        std::fs::write(conf, "").unwrap();
        assert_eq!(
            attach_command(&config, &agent_for("FEAT-2")).as_deref(),
            Some("tmux -L operator attach -t op-feat-2")
        );
    }

    #[test]
    fn test_attach_command_is_absent_for_other_wrappers() {
        let mut agent = agent_for("FEAT-2");
        agent.session_wrapper = Some("zellij".into());
        assert_eq!(attach_command(&Config::default(), &agent), None);
        agent.session_wrapper = Some("cmux".into());
        assert_eq!(attach_command(&Config::default(), &agent), None);
    }

    fn write_ticket(root: &std::path::Path, column: &str, name: &str, frontmatter: &str) -> Ticket {
        let dir = root.join(column);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(
            &path,
            format!("---\n{frontmatter}\n---\n\n# Body of {name}\n"),
        )
        .unwrap();
        Ticket::from_file(&path).unwrap()
    }

    fn agent_for(ticket_id: &str) -> crate::state::AgentState {
        serde_json::from_value(serde_json::json!({
            "id": "agent-1", "ticket_id": ticket_id, "ticket_type": "FEAT", "project": "api",
            "status": "running", "started_at": "2026-01-01T00:00:00Z",
            "last_activity": "2026-01-01T00:00:00Z", "paired": false,
            "session_wrapper": "tmux", "session_name": "op-feat-2",
            "current_step": "build", "completed_steps": ["plan"],
            "pr_url": "https://example.test/pr/9", "pr_number": 9
        }))
        .unwrap()
    }

    #[test]
    fn test_ticket_detail_resolves_dependencies_and_dependents() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        let done = write_ticket(
            root,
            "completed",
            "20260101-0000-FEAT-api-a.md",
            "id: FEAT-1\nstatus: completed",
        );
        let me = write_ticket(
            root,
            "in-progress",
            "20260101-0001-FEAT-api-b.md",
            "id: FEAT-2\ncampaign: CAMP-0001\ndepends_on:\n  - FEAT-1\n  - jira:PROJ-9",
        );
        let child = write_ticket(
            root,
            "queue",
            "20260101-0002-FEAT-api-c.md",
            "id: FEAT-3\ndepends_on:\n  - FEAT-2",
        );
        let all = vec![done, me.clone(), child];

        let detail = ticket_detail(&Config::default(), &me, &all, None);

        assert_eq!(detail.body, "# Body of 20260101-0001-FEAT-api-b.md\n");
        assert_eq!(detail.column, "in-progress");
        assert_eq!(detail.campaign.as_deref(), Some("CAMP-0001"));
        assert_eq!(detail.depends_on, vec!["FEAT-1", "jira:PROJ-9"]);
        assert_eq!(
            detail.dependencies,
            vec![
                TicketDependency {
                    reference: "FEAT-1".into(),
                    ticket_id: Some("FEAT-1".into()),
                    column: Some("completed".into()),
                    satisfied: true,
                },
                TicketDependency {
                    reference: "jira:PROJ-9".into(),
                    ticket_id: None,
                    column: None,
                    satisfied: false,
                },
            ]
        );
        assert_eq!(detail.dependents, vec!["FEAT-3"]);
        assert!(detail.agent.is_none());
    }

    #[test]
    fn test_ticket_detail_carries_the_live_agent() {
        let temp = tempfile::TempDir::new().unwrap();
        let me = write_ticket(
            temp.path(),
            "in-progress",
            "20260101-0001-FEAT-api-b.md",
            "id: FEAT-2",
        );
        let agent = agent_for("FEAT-2");

        let detail = ticket_detail(
            &Config::default(),
            &me,
            std::slice::from_ref(&me),
            Some(&agent),
        );

        let live = detail.agent.expect("agent");
        assert_eq!(live.id, "agent-1");
        assert_eq!(live.session_wrapper.as_deref(), Some("tmux"));
        assert_eq!(live.session_name.as_deref(), Some("op-feat-2"));
        assert_eq!(live.completed_steps, vec!["plan"]);
        assert_eq!(live.pr_number, Some(9));
        assert_eq!(
            live.attach_command.as_deref(),
            Some("tmux attach -t op-feat-2")
        );
    }

    #[test]
    fn test_attach_command_uses_the_operator_socket_when_configured() {
        let temp = tempfile::TempDir::new().unwrap();
        let mut config = Config::default();
        config.paths.tickets = temp.path().to_string_lossy().into_owned();
        config.tmux.config_generated = true;
        let conf = config.tmux_config_path();
        std::fs::create_dir_all(conf.parent().unwrap()).unwrap();
        std::fs::write(conf, "").unwrap();
        assert_eq!(
            attach_command(&config, &agent_for("FEAT-2")).as_deref(),
            Some("tmux -L operator attach -t op-feat-2")
        );
    }

    #[test]
    fn test_attach_command_is_absent_for_other_wrappers() {
        let mut agent = agent_for("FEAT-2");
        agent.session_wrapper = Some("zellij".into());
        assert_eq!(attach_command(&Config::default(), &agent), None);
        agent.session_wrapper = Some("cmux".into());
        assert_eq!(attach_command(&Config::default(), &agent), None);
    }
}
