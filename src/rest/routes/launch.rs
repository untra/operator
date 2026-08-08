//! Ticket launch endpoint for the REST API.
//!
//! Provides the launch endpoint for starting agents via external clients
//! like the VS Code extension.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::agents::delegator_resolution::{self, AgentContext};
use crate::agents::{LaunchOptions, Launcher, PreparedLaunch, RelaunchOptions};
use crate::queue::Queue;
use crate::rest::dto::{
    LaunchTicketRequest, LaunchTicketResponse, NextStepInfo, StepCompleteRequest,
    StepCompleteResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// If the sub-agent identified by `request.session_id` (or by ticket fallback)
/// belongs to a multi-agent group, write its individual output artifact to
/// `{worktree}/.tickets/steps/{step_name}/{agent_id}.json` and return a
/// `group_partial` / `group_complete` response. Returns `Ok(None)` when this
/// is a normal single-agent completion and the caller should fall through to
/// existing logic.
fn handle_multi_agent_completion(
    state: &ApiState,
    ticket: &crate::queue::Ticket,
    step_name: &str,
    request: &StepCompleteRequest,
) -> Result<Option<StepCompleteResponse>, ApiError> {
    let mut app_state = crate::state::State::load(&state.config)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Resolve the sub-agent: prefer session-id lookup, fall back to ticket.
    let agent_id = request
        .session_id
        .as_deref()
        .and_then(|sid| app_state.agent_by_session(sid))
        .or_else(|| app_state.agent_by_ticket(&ticket.id))
        .map(|a| a.id.clone());

    let Some(agent_id) = agent_id else {
        return Ok(None);
    };

    // If this agent is not in a group, fall through.
    if app_state.get_group_for_agent(&agent_id).is_none() {
        return Ok(None);
    }

    // Build the per-sub-agent output payload from the POSTed OperatorOutput.
    let output_payload = request
        .output
        .as_ref()
        .map(|o| serde_json::to_value(o).unwrap_or(serde_json::Value::Null))
        .unwrap_or(serde_json::Value::Null);

    // Persist the per-sub-agent file — the sync loop picks it up.
    crate::steps::manager::StepManager::write_agent_step_output(
        ticket,
        step_name,
        &agent_id,
        &output_payload,
    )
    .map_err(|e| ApiError::InternalError(format!("write sub-agent output: {e}")))?;

    // Preview whether this was the final sub-agent for the group. The actual
    // all-done decision is made by the sync loop when it calls record_agent_output.
    let all_done = app_state
        .get_group_for_agent(&agent_id)
        .map(|g| g.individual_outputs.len() + 1 >= g.expected_total)
        .unwrap_or(false);

    // Mark the sub-agent as completing so the sync loop stops polling.
    let _ = app_state.update_agent_status(
        &agent_id,
        "completing",
        Some("sub-agent complete".to_string()),
    );

    // Build a minimal response — the group aggregation/advancement happens
    // in the sync loop, not here.
    let (previous_summary, previous_recommendation, cumulative_files_modified, cumulative_errors) =
        request.output.as_ref().map_or((None, None, 0, 0), |o| {
            (
                o.summary.clone(),
                o.recommendation.clone(),
                o.files_modified.unwrap_or(0),
                o.error_count.unwrap_or(0),
            )
        });

    Ok(Some(StepCompleteResponse {
        status: if all_done {
            "group_complete".to_string()
        } else {
            "group_partial".to_string()
        },
        next_step: None,
        auto_proceed: false,
        next_command: None,
        output_valid: request.output.is_some(),
        should_iterate: false,
        iteration_count: 1,
        circuit_state: "closed".to_string(),
        previous_summary,
        previous_recommendation,
        cumulative_files_modified,
        cumulative_errors,
    }))
}

/// Convert `PreparedLaunch` to `LaunchTicketResponse`
fn prepared_launch_to_response(prepared: PreparedLaunch) -> LaunchTicketResponse {
    LaunchTicketResponse {
        executed_server_side: false,
        agent_id: prepared.agent_id,
        ticket_id: prepared.ticket_id,
        working_directory: prepared.working_directory.to_string_lossy().to_string(),
        command: prepared.command,
        terminal_name: prepared.terminal_name.clone(),
        tmux_session_name: prepared.terminal_name,
        session_wrapper: prepared.session_wrapper,
        session_window_ref: prepared.session_window_ref,
        session_context_ref: prepared.session_context_ref,
        session_id: prepared.session_id,
        worktree_created: prepared.worktree_created,
        branch: prepared.branch,
    }
}

/// Launch a ticket from the queue
///
/// Claims the ticket, sets up worktree if needed, generates the LLM command,
/// and returns all details needed to execute in an external terminal (VS Code, etc.).
#[utoipa::path(
    operation_id = "launch_launch_ticket",
    post,
    path = "/api/v1/tickets/{id}/launch",
    tag = "Launch",
    params(
        ("id" = String, Path, description = "Ticket ID to launch")
    ),
    request_body = LaunchTicketRequest,
    responses(
        (status = 200, description = "Ticket launched successfully", body = LaunchTicketResponse),
        (status = 404, description = "Ticket not found"),
        (status = 409, description = "Ticket already in progress"),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn launch_ticket(
    State(state): State<ApiState>,
    Path(ticket_id): Path<String>,
    Json(request): Json<LaunchTicketRequest>,
) -> Result<Json<LaunchTicketResponse>, ApiError> {
    // Create a queue to find the ticket
    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Find the ticket by ID
    let ticket = queue
        .find_ticket(&ticket_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Ticket '{ticket_id}' not found")))?;

    // Resolve issuetype agent context for delegator layering
    let agent_context = {
        let registry = state.registry.read().await;
        registry
            .get(&ticket.ticket_type.to_uppercase())
            .map(|issue_type| {
                let step_agent = if ticket.step.is_empty() {
                    issue_type.first_step().and_then(|s| s.agent.clone())
                } else {
                    issue_type
                        .get_step(&ticket.step)
                        .and_then(|s| s.agent.clone())
                };
                AgentContext {
                    step_agent,
                    issuetype_agent: issue_type.agent.clone(),
                }
            })
    };

    // Check if ticket is in-progress directory
    let in_progress_path = state
        .config
        .tickets_path()
        .join("in-progress")
        .join(&ticket.filename);

    // Create launcher
    let launcher =
        Launcher::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Non-local targets (docker/coder/ssh) execute SERVER-SIDE: workspace
    // lifecycle and remote session orchestration belong to the server, and a
    // prepared command handed to a remote client could not run them. Local
    // targets keep the prepared-handoff (the client owns the terminal).
    let response = if in_progress_path.exists() {
        // Ticket is in-progress - use relaunch flow (no claim needed)
        let mut relaunch_options =
            build_relaunch_options(&state, &request, agent_context.as_ref())?;
        apply_request_target(&state, &request, &mut relaunch_options.launch_options)?;
        if relaunch_options.launch_options.target.kind == crate::config::TargetKind::Local {
            let prepared = launcher
                .prepare_relaunch(&ticket, relaunch_options)
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            prepared_launch_to_response(prepared)
        } else {
            launcher
                .relaunch(&ticket, relaunch_options)
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            server_side_response(&state, &ticket)?
        }
    } else {
        // New launch - claim ticket from queue
        let mut launch_options = build_launch_options(&state, &request, agent_context.as_ref())?;
        apply_request_target(&state, &request, &mut launch_options)?;
        if launch_options.target.kind == crate::config::TargetKind::Local {
            let prepared = launcher
                .prepare_launch(&ticket, launch_options)
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            prepared_launch_to_response(prepared)
        } else {
            launcher
                .launch_with_options(&ticket, launch_options)
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            server_side_response(&state, &ticket)?
        }
    };

    Ok(Json(response))
}

/// Apply the request's per-launch target override, with the same remote
/// constraints delegator resolution enforces.
fn apply_request_target(
    state: &ApiState,
    request: &LaunchTicketRequest,
    options: &mut LaunchOptions,
) -> Result<(), ApiError> {
    if let Some(ref name) = request.target {
        let target = delegator_resolution::resolve_named_target(&state.config, name)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        delegator_resolution::apply_target_to_options(options, target, &state.config)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(())
}

/// Response for a launch the server executed itself: `command` is empty and
/// the client must not run anything; details come from the agent record.
fn server_side_response(
    state: &ApiState,
    ticket: &crate::queue::Ticket,
) -> Result<LaunchTicketResponse, ApiError> {
    let app_state = crate::state::State::load(&state.config)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let agent = app_state
        .agents
        .iter()
        .rfind(|a| a.ticket_id == ticket.id)
        .ok_or_else(|| {
            ApiError::InternalError("launch succeeded but no agent record found".to_string())
        })?;
    Ok(LaunchTicketResponse {
        executed_server_side: true,
        agent_id: agent.id.clone(),
        ticket_id: ticket.id.clone(),
        working_directory: agent.worktree_path.clone().unwrap_or_else(|| {
            state
                .config
                .projects_path()
                .join(&ticket.project)
                .to_string_lossy()
                .to_string()
        }),
        command: String::new(),
        terminal_name: agent.session_name.clone().unwrap_or_default(),
        tmux_session_name: agent.session_name.clone().unwrap_or_default(),
        session_wrapper: agent.session_wrapper.clone(),
        session_window_ref: agent.session_window_ref.clone(),
        session_context_ref: agent.session_context_ref.clone(),
        session_id: String::new(),
        worktree_created: agent.worktree_path.is_some(),
        branch: ticket.branch.clone(),
    })
}

/// Build `LaunchOptions` from the request, delegating to the shared resolution module.
fn build_launch_options(
    state: &ApiState,
    request: &LaunchTicketRequest,
    agent_context: Option<&AgentContext>,
) -> Result<LaunchOptions, ApiError> {
    delegator_resolution::resolve_launch_options(
        &state.config,
        request.delegator.as_deref(),
        request.provider.as_deref(),
        request.model.as_deref(),
        request.model_server.as_deref(),
        request.yolo_mode,
        agent_context,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Build `RelaunchOptions` from the request
fn build_relaunch_options(
    state: &ApiState,
    request: &LaunchTicketRequest,
    agent_context: Option<&AgentContext>,
) -> Result<RelaunchOptions, ApiError> {
    let launch_options = build_launch_options(state, request, agent_context)?;

    Ok(RelaunchOptions {
        launch_options,
        resume_session_id: request.resume_session_id.clone(),
        retry_reason: request.retry_reason.clone(),
    })
}

/// Pick the agent that ran the just-completed step: prefer a match on the
/// persisted launch-context session id (opr8r's `--session-id`, the LLM
/// session that ran the step) over the first agent on the ticket. With
/// multiple agents on one ticket (retries, manual relaunch) a plain
/// ticket-id match can pick the wrong one's launch context.
fn find_completing_agent<'a>(
    agents: &'a [crate::state::AgentState],
    ticket_id: &str,
    session_id: Option<&str>,
) -> Option<&'a crate::state::AgentState> {
    session_id
        .and_then(|sid| {
            agents.iter().find(|a| {
                a.ticket_id == ticket_id
                    && a.step_launch_context
                        .as_ref()
                        .and_then(|c| c.session_id.as_deref())
                        == Some(sid)
            })
        })
        .or_else(|| agents.iter().find(|a| a.ticket_id == ticket_id))
}

/// Build the next step's opr8r-wrapped command from the launch context
/// persisted with the agent record. Agents launched before contexts were
/// persisted fall back to a baseline reconstructed from per-agent fields.
fn build_next_step_command(
    state: &ApiState,
    ticket: &crate::queue::Ticket,
    next_step: &crate::templates::schema::StepSchema,
    request: &StepCompleteRequest,
) -> anyhow::Result<crate::agents::launcher::step_command::BuiltStepCommand> {
    use crate::agents::launcher::step_command::{self, StepLaunchContext};

    let config: &crate::config::Config = &state.config;
    let app_state = crate::state::State::load(config)?;
    let agent = find_completing_agent(&app_state.agents, &ticket.id, request.session_id.as_deref());

    let ctx = agent
        .and_then(|a| a.step_launch_context.clone())
        .unwrap_or_else(|| StepLaunchContext {
            delegator: None,
            tool: agent
                .and_then(|a| a.llm_tool.clone())
                .unwrap_or_else(|| "claude".to_string()),
            model: agent
                .and_then(|a| a.llm_model.clone())
                .unwrap_or_else(|| "sonnet".to_string()),
            yolo: agent
                .and_then(|a| a.launch_mode.as_deref())
                .is_some_and(|m| m.contains("yolo")),
            session_id: None,
            opr8r: "opr8r".to_string(),
            operator_relay: None,
            extra_flags: vec![],
        });

    // Working directory: the agent's worktree when one exists, else the project
    let project_path = agent
        .and_then(|a| a.worktree_path.clone())
        .unwrap_or_else(|| {
            config
                .projects_path()
                .join(&ticket.project)
                .to_string_lossy()
                .to_string()
        });

    let (previous_summary, previous_recommendation) =
        request.output.as_ref().map_or((None, None), |o| {
            (o.summary.clone(), o.recommendation.clone())
        });

    step_command::build_step_command(
        config,
        ticket,
        next_step,
        &ctx,
        &project_path,
        previous_summary.as_deref(),
        previous_recommendation.as_deref(),
    )
}

/// Advance the ticket file to the next step and persist the minted session id
/// and agent step, so chain bookkeeping matches what will execute. Best-effort:
/// failures are logged, not fatal — opr8r already holds the command.
///
/// opr8r retries the completion POST up to 3 times, and the artifact-sync
/// loop (src/agents/sync.rs) can also advance the ticket independently, so a
/// re-entrant call must not advance twice: re-read the ticket fresh and only
/// call `advance_step()` when it is still sitting on `completed_step`. On a
/// duplicate (already advanced), still record the session id for the next
/// step — that part is idempotent.
fn record_step_transition(
    state: &ApiState,
    ticket: &crate::queue::Ticket,
    completed_step: &str,
    next_step: &crate::templates::schema::StepSchema,
    next_session_id: &str,
    request: &StepCompleteRequest,
) {
    let mut advanced = match crate::queue::Ticket::from_file(std::path::Path::new(&ticket.filepath))
    {
        Ok(fresh) => fresh,
        Err(e) => {
            tracing::warn!(ticket = %ticket.id, error = %e, "Failed to re-read ticket for advance guard; using stale copy");
            ticket.clone()
        }
    };

    // Empty step means "sitting on the schema's first step" (the convention
    // used everywhere else `ticket.step` is read before a step is chosen).
    let first_step_name = advanced
        .template_schema()
        .and_then(|t| t.first_step().map(|s| s.name.clone()));
    let at_completed_step = if advanced.step.is_empty() {
        first_step_name.as_deref() == Some(completed_step)
    } else {
        advanced.step == completed_step
    };

    if at_completed_step {
        if let Err(e) = advanced.advance_step() {
            tracing::warn!(ticket = %ticket.id, error = %e, "Failed to advance ticket step");
        }
    } else {
        tracing::debug!(
            ticket = %ticket.id,
            completed_step = %completed_step,
            current_step = %advanced.step,
            "Ticket already advanced past completed step; skipping duplicate advance"
        );
    }

    if let Err(e) = advanced.set_session_id(&next_step.name, next_session_id) {
        tracing::warn!(ticket = %ticket.id, error = %e, "Failed to store next step session id");
    }
    match crate::state::State::load(&state.config) {
        Ok(mut app_state) => {
            let matched =
                find_completing_agent(&app_state.agents, &ticket.id, request.session_id.as_deref());
            let agent_id = matched.map(|a| a.id.clone());
            // Re-point the persisted context at the uuid minted for the next
            // step; without it the session arm of `find_completing_agent`
            // stops matching from the second transition onward.
            let next_ctx = matched
                .and_then(|a| a.step_launch_context.clone())
                .map(|mut c| {
                    c.session_id = Some(next_session_id.to_string());
                    c
                });
            if let Some(id) = agent_id {
                if let Err(e) = app_state.update_agent_step(&id, &next_step.name) {
                    tracing::warn!(ticket = %ticket.id, error = %e, "Failed to update agent step");
                }
                if let Some(ctx) = next_ctx {
                    if let Err(e) = app_state.update_agent_step_launch_context(&id, ctx) {
                        tracing::warn!(ticket = %ticket.id, error = %e, "Failed to update agent step launch context");
                    }
                }
            }
        }
        Err(e) => tracing::warn!(ticket = %ticket.id, error = %e, "Failed to load state"),
    }
}

/// Report step completion from opr8r wrapper
///
/// Called by the opr8r wrapper when an LLM command completes.
/// Returns next step info and whether to auto-proceed.
#[utoipa::path(
    operation_id = "launch_complete_step",
    post,
    path = "/api/v1/tickets/{id}/steps/{step}/complete",
    tag = "Launch",
    params(
        ("id" = String, Path, description = "Ticket ID"),
        ("step" = String, Path, description = "Step name that completed")
    ),
    request_body = StepCompleteRequest,
    responses(
        (status = 200, description = "Step completion recorded", body = StepCompleteResponse),
        (status = 404, description = "Ticket not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn complete_step(
    State(state): State<ApiState>,
    Path((ticket_id, step_name)): Path<(String, String)>,
    Json(request): Json<StepCompleteRequest>,
) -> Result<Json<StepCompleteResponse>, ApiError> {
    // Create a queue to find the ticket
    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Find the ticket by ID
    let ticket = queue
        .find_ticket(&ticket_id)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Ticket '{ticket_id}' not found")))?;

    // Get the issue type to find step info
    let registry = state.registry.read().await;
    let issue_type = registry
        .get(&ticket.ticket_type.to_uppercase())
        .ok_or_else(|| {
            ApiError::NotFound(format!("Issue type '{}' not found", ticket.ticket_type))
        })?;

    // Find the current step
    let current_step = issue_type.get_step(&step_name).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Step '{}' not found in '{}'",
            step_name, ticket.ticket_type
        ))
    })?;

    // Multi-agent branch: if the calling sub-agent belongs to a group,
    // write its individual output file and return a group_* status.
    // The sync loop owns aggregation, advancement, and artifact writing.
    if let Some(response) = handle_multi_agent_completion(&state, &ticket, &step_name, &request)? {
        return Ok(Json(response));
    }

    // Determine status based on exit code and validation
    let status = if request.exit_code != 0 {
        "failed".to_string()
    } else if current_step.review_type != crate::templates::schema::ReviewType::None {
        "awaiting_review".to_string()
    } else {
        "completed".to_string()
    };

    // Fire-and-forget: push step-completed activity log to upstream kanban provider.
    if status == "completed" {
        if let Some(ref ks) = state.kanban_sync {
            let ks = Arc::clone(ks);
            let ticket_clone = ticket.clone();
            let step = step_name.clone();
            let summary = request.output.as_ref().and_then(|o| o.summary.clone());
            tokio::spawn(async move {
                ks.on_step_completed(&ticket_clone, &step, "unknown", summary.as_deref())
                    .await;
            });
        }
    }

    // Find next step info
    let next_step_info = current_step.next_step.as_ref().and_then(|next_name| {
        issue_type.get_step(next_name).map(|step| NextStepInfo {
            name: step.name.clone(),
            display_name: step.display_name.clone().unwrap_or(step.name.clone()),
            review_type: format!("{:?}", step.review_type).to_lowercase(),
            prompt: Some(step.prompt.clone()),
        })
    });

    // Determine if we should auto-proceed
    let auto_proceed = status == "completed"
        && next_step_info.is_some()
        && current_step.review_type == crate::templates::schema::ReviewType::None;

    // Build next command if auto-proceeding: the same builder the launcher
    // uses for step one, fed by the launch context persisted with the agent.
    // Never target-wrapped — exec() happens inside the already-wrapped
    // environment (see step_command module docs).
    let next_command = if auto_proceed {
        match current_step
            .next_step
            .as_ref()
            .and_then(|n| issue_type.get_step(n).cloned())
        {
            Some(next_schema) => {
                match build_next_step_command(&state, &ticket, &next_schema, &request) {
                    Ok(built) => {
                        record_step_transition(
                            &state,
                            &ticket,
                            &step_name,
                            &next_schema,
                            &built.session_id,
                            &request,
                        );
                        Some(built.command)
                    }
                    Err(e) => {
                        tracing::warn!(
                            ticket = %ticket.id,
                            step = %step_name,
                            error = %e,
                            "Failed to build next step command; chain will not auto-proceed"
                        );
                        None
                    }
                }
            }
            None => None,
        }
    } else {
        None
    };

    // Extract data from OperatorOutput if provided
    let (output_valid, should_iterate, previous_summary, previous_recommendation) =
        if let Some(ref output) = request.output {
            (
                true,
                !output.exit_signal, // should_iterate when exit_signal is false
                output.summary.clone(),
                output.recommendation.clone(),
            )
        } else {
            (false, false, None, None)
        };

    // Calculate cumulative values (for now, just use current values)
    let cumulative_files_modified = request
        .output
        .as_ref()
        .and_then(|o| o.files_modified)
        .unwrap_or(0);
    let cumulative_errors = request
        .output
        .as_ref()
        .and_then(|o| o.error_count)
        .unwrap_or(0);

    Ok(Json(StepCompleteResponse {
        status,
        next_step: next_step_info,
        auto_proceed,
        next_command,
        output_valid,
        should_iterate,
        iteration_count: 1,                  // TODO: Track across iterations
        circuit_state: "closed".to_string(), // TODO: Implement circuit breaker
        previous_summary,
        previous_recommendation,
        cumulative_files_modified,
        cumulative_errors,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test-launch"))
    }

    #[test]
    fn test_build_launch_options_default() {
        let state = make_state();
        let request = LaunchTicketRequest {
            target: None,
            delegator: None,
            provider: None,
            model: None,
            model_server: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request, None);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(!options.yolo_mode);
        assert!(options.provider.is_none());
    }

    #[test]
    fn test_build_launch_options_yolo() {
        let state = make_state();
        let request = LaunchTicketRequest {
            target: None,
            delegator: None,
            provider: None,
            model: None,
            model_server: None,
            yolo_mode: true,
            wrapper: Some("vscode".to_string()),
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request, None);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(options.yolo_mode);
    }

    #[test]
    fn test_build_launch_options_unknown_provider() {
        let state = make_state();
        let request = LaunchTicketRequest {
            target: None,
            delegator: None,
            provider: Some("unknown-provider".to_string()),
            model: None,
            model_server: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_relaunch_options() {
        let state = make_state();
        let request = LaunchTicketRequest {
            target: None,
            delegator: None,
            provider: None,
            model: None,
            model_server: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: Some("Previous attempt timed out".to_string()),
            resume_session_id: Some("abc-123".to_string()),
        };

        let result = build_relaunch_options(&state, &request, None);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(!options.launch_options.yolo_mode);
        assert_eq!(
            options.retry_reason,
            Some("Previous attempt timed out".to_string())
        );
        assert_eq!(options.resume_session_id, Some("abc-123".to_string()));
    }

    #[test]
    fn test_build_launch_options_delegator_propagates_all_fields() {
        let mut config = Config::default();
        config.delegators.push(crate::config::Delegator {
            name: "full-delegator".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: Some(crate::config::DelegatorLaunchConfig {
                yolo: true,
                permission_mode: Some("accept-edits".to_string()),
                flags: vec!["--verbose".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(false),
                docker: Some(true),
                prompt_prefix: Some("PREFIX".to_string()),
                prompt_suffix: Some("SUFFIX".to_string()),
                operator_relay: None,
                host: None,
                target: None,
            }),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-launch"));

        let request = LaunchTicketRequest {
            target: None,
            delegator: Some("full-delegator".to_string()),
            provider: None,
            model: None,
            model_server: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request, None);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(options.yolo_mode);
        assert!(options.is_docker());
        assert_eq!(options.use_worktrees_override, Some(true));
        assert_eq!(options.create_branch_override, Some(false));
        assert_eq!(options.prompt_prefix.as_deref(), Some("PREFIX"));
        assert_eq!(options.prompt_suffix.as_deref(), Some("SUFFIX"));
        assert_eq!(options.extra_flags, vec!["--verbose".to_string()]);
        assert_eq!(options.delegator_name.as_deref(), Some("full-delegator"));
    }

    #[test]
    fn test_build_launch_options_delegator_none_overrides_inherit() {
        let mut config = Config::default();
        config.delegators.push(crate::config::Delegator {
            name: "minimal".to_string(),
            llm_tool: "claude".to_string(),
            model: "sonnet".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: Some(crate::config::DelegatorLaunchConfig::default()),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-launch"));

        let request = LaunchTicketRequest {
            target: None,
            delegator: Some("minimal".to_string()),
            provider: None,
            model: None,
            model_server: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request, None);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(!options.yolo_mode);
        assert!(!options.is_docker());
        assert!(options.use_worktrees_override.is_none());
        assert!(options.create_branch_override.is_none());
        assert!(options.prompt_prefix.is_none());
        assert!(options.prompt_suffix.is_none());
    }

    // --- Layered delegator resolution tests ---

    fn make_state_with_delegators(delegators: Vec<crate::config::Delegator>) -> ApiState {
        let config = Config {
            delegators,
            ..Default::default()
        };
        ApiState::new(config, PathBuf::from("/tmp/test-launch"))
    }

    fn make_delegator(name: &str, tool: &str, model: &str) -> crate::config::Delegator {
        crate::config::Delegator {
            name: name.to_string(),
            llm_tool: tool.to_string(),
            model: model.to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        }
    }

    #[test]
    fn test_apply_request_target_unknown_is_bad_request() {
        let state = make_state();
        let mut request = empty_request();
        request.target = Some("nope".to_string());
        let mut options = LaunchOptions::default();
        let err = apply_request_target(&state, &request, &mut options).unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn test_apply_request_target_docker_overrides_local() {
        let state = make_state();
        let mut request = empty_request();
        request.target = Some("docker".to_string());
        let mut options = LaunchOptions::default();
        apply_request_target(&state, &request, &mut options).unwrap();
        assert!(options.is_docker());
    }

    #[test]
    fn test_apply_request_target_none_keeps_resolved_target() {
        let state = make_state();
        let request = empty_request();
        let mut options = LaunchOptions::default();
        apply_request_target(&state, &request, &mut options).unwrap();
        assert_eq!(options.target.kind, crate::config::TargetKind::Local);
    }

    fn empty_request() -> LaunchTicketRequest {
        LaunchTicketRequest {
            target: None,
            delegator: None,
            provider: None,
            model: None,
            model_server: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        }
    }

    #[test]
    fn test_build_launch_options_step_agent_resolves() {
        let state =
            make_state_with_delegators(vec![make_delegator("claude-opus", "claude", "opus")]);
        let ctx = AgentContext {
            step_agent: Some("claude-opus".to_string()),
            issuetype_agent: None,
        };

        let options = build_launch_options(&state, &empty_request(), Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "claude");
        assert_eq!(provider.model, "opus");
        assert_eq!(options.delegator_name.as_deref(), Some("claude-opus"));
    }

    #[test]
    fn test_build_launch_options_issuetype_agent_fallback() {
        let state =
            make_state_with_delegators(vec![make_delegator("claude-opus", "claude", "opus")]);
        let ctx = AgentContext {
            step_agent: None,
            issuetype_agent: Some("claude-opus".to_string()),
        };

        let options = build_launch_options(&state, &empty_request(), Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "claude");
        assert_eq!(provider.model, "opus");
    }

    #[test]
    fn test_build_launch_options_step_agent_overrides_issuetype() {
        let state = make_state_with_delegators(vec![
            make_delegator("claude-opus", "claude", "opus"),
            make_delegator("claude-sonnet", "claude", "sonnet"),
        ]);
        let ctx = AgentContext {
            step_agent: Some("claude-opus".to_string()),
            issuetype_agent: Some("claude-sonnet".to_string()),
        };

        let options = build_launch_options(&state, &empty_request(), Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.model, "opus");
        assert_eq!(options.delegator_name.as_deref(), Some("claude-opus"));
    }

    #[test]
    fn test_build_launch_options_request_delegator_overrides_context() {
        let state = make_state_with_delegators(vec![
            make_delegator("claude-opus", "claude", "opus"),
            make_delegator("gemini-pro", "gemini", "pro"),
        ]);
        let ctx = AgentContext {
            step_agent: Some("claude-opus".to_string()),
            issuetype_agent: Some("claude-opus".to_string()),
        };
        let request = LaunchTicketRequest {
            target: None,
            delegator: Some("gemini-pro".to_string()),
            ..empty_request()
        };

        let options = build_launch_options(&state, &request, Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "gemini");
        assert_eq!(provider.model, "pro");
        assert_eq!(options.delegator_name.as_deref(), Some("gemini-pro"));
    }

    #[test]
    fn test_build_launch_options_unknown_step_agent_falls_through() {
        let state =
            make_state_with_delegators(vec![make_delegator("claude-opus", "claude", "opus")]);
        let ctx = AgentContext {
            step_agent: Some("nonexistent-delegator".to_string()),
            issuetype_agent: Some("claude-opus".to_string()),
        };

        let options = build_launch_options(&state, &empty_request(), Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.model, "opus");
        assert_eq!(options.delegator_name.as_deref(), Some("claude-opus"));
    }

    #[test]
    fn test_build_launch_options_no_context_preserves_existing() {
        let state =
            make_state_with_delegators(vec![make_delegator("claude-opus", "claude", "opus")]);

        // With a single delegator and no context, should resolve to default delegator
        let options = build_launch_options(&state, &empty_request(), None).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "claude");
        assert_eq!(provider.model, "opus");
    }

    #[test]
    fn test_build_launch_options_step_agent_applies_launch_config() {
        let state = make_state_with_delegators(vec![crate::config::Delegator {
            name: "codex-auto".to_string(),
            llm_tool: "codex".to_string(),
            model: "o3".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: Some(crate::config::DelegatorLaunchConfig {
                yolo: true,
                permission_mode: None,
                flags: vec!["--full-auto".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(true),
                docker: Some(false),
                prompt_prefix: Some("BEGIN".to_string()),
                prompt_suffix: Some("END".to_string()),
                operator_relay: None,
                host: None,
                target: None,
            }),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        }]);
        let ctx = AgentContext {
            step_agent: Some("codex-auto".to_string()),
            issuetype_agent: None,
        };

        let options = build_launch_options(&state, &empty_request(), Some(&ctx)).unwrap();
        assert!(options.yolo_mode);
        assert!(!options.is_docker());
        assert_eq!(options.use_worktrees_override, Some(true));
        assert_eq!(options.create_branch_override, Some(true));
        assert_eq!(options.extra_flags, vec!["--full-auto".to_string()]);
        assert_eq!(options.prompt_prefix.as_deref(), Some("BEGIN"));
        assert_eq!(options.prompt_suffix.as_deref(), Some("END"));
    }

    // ─── Multi-agent grouped completion tests ───────────────────────────

    use crate::queue::Ticket;
    use crate::rest::dto::OperatorOutput;
    use crate::state::{PendingSubAgent, State};
    use tempfile::TempDir;

    fn make_state_with_temp(temp_dir: &TempDir) -> ApiState {
        let state_path = temp_dir.path().join("state");
        std::fs::create_dir_all(&state_path).unwrap();
        let mut config = Config::default();
        config.paths.state = state_path.to_string_lossy().to_string();
        ApiState::new(config, temp_dir.path().to_path_buf())
    }

    fn make_multi_agent_ticket(temp_dir: &TempDir) -> Ticket {
        let worktree = temp_dir.path().join("worktree");
        std::fs::create_dir_all(&worktree).unwrap();
        Ticket {
            filename: "multi.md".to_string(),
            filepath: worktree.join("multi.md").to_string_lossy().to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "TASK".to_string(),
            project: "test".to_string(),
            id: "TASK-555".to_string(),
            summary: "Multi-agent ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "review".to_string(),
            content: "# test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: Some(worktree.to_string_lossy().to_string()),
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        }
    }

    fn make_complete_request(session_id: &str) -> StepCompleteRequest {
        StepCompleteRequest {
            exit_code: 0,
            output_valid: true,
            output_schema_errors: None,
            session_id: Some(session_id.to_string()),
            duration_secs: 10,
            output_sample: None,
            output: Some(OperatorOutput {
                status: "complete".to_string(),
                exit_signal: true,
                summary: Some("done".to_string()),
                ..Default::default()
            }),
        }
    }

    #[test]
    fn test_handle_multi_agent_completion_returns_none_when_no_group() {
        let temp_dir = TempDir::new().unwrap();
        let api_state = make_state_with_temp(&temp_dir);
        let ticket = make_multi_agent_ticket(&temp_dir);

        // Fresh state — no groups, no agents.
        let req = StepCompleteRequest {
            exit_code: 0,
            output_valid: true,
            output_schema_errors: None,
            session_id: None,
            duration_secs: 0,
            output_sample: None,
            output: None,
        };

        let response = handle_multi_agent_completion(&api_state, &ticket, "review", &req).unwrap();
        assert!(
            response.is_none(),
            "no group → fall through to single-agent path"
        );
    }

    #[test]
    fn test_handle_multi_agent_completion_partial_writes_file_and_returns_group_partial() {
        let temp_dir = TempDir::new().unwrap();
        let api_state = make_state_with_temp(&temp_dir);
        let ticket = make_multi_agent_ticket(&temp_dir);

        // Build a group with 2 expected sub-agents; launch one (mark_launched).
        let (agent_id, session_name) = {
            let mut state = State::load(&api_state.config).unwrap();
            let group_id = state
                .create_multi_agent_group(
                    &ticket.id,
                    "review",
                    "multi_model",
                    vec![
                        PendingSubAgent {
                            delegator_name: "d1".to_string(),
                            prompt: "p".to_string(),
                            variant_key: "d1".to_string(),
                        },
                        PendingSubAgent {
                            delegator_name: "d2".to_string(),
                            prompt: "p".to_string(),
                            variant_key: "d2".to_string(),
                        },
                    ],
                )
                .unwrap();

            // Add one agent, record its session id, and mark it launched.
            let agent_id = state
                .add_agent_with_options(
                    ticket.id.clone(),
                    ticket.ticket_type.clone(),
                    ticket.project.clone(),
                    false,
                    Some("claude".to_string()),
                    Some("default".to_string()),
                )
                .unwrap();
            let session_name = "op-TASK-555-d1".to_string();
            state
                .update_agent_session(&agent_id, &session_name)
                .unwrap();
            state.mark_launched(&group_id, "d1", &agent_id).unwrap();
            (agent_id, session_name)
        };

        let req = make_complete_request(&session_name);
        let response = handle_multi_agent_completion(&api_state, &ticket, "review", &req)
            .unwrap()
            .expect("group member → returns Some");

        assert_eq!(response.status, "group_partial");
        assert!(!response.auto_proceed);
        assert!(response.next_step.is_none());

        // Per-sub-agent file written at the expected path
        let expected = temp_dir
            .path()
            .join("worktree")
            .join(".tickets")
            .join("steps")
            .join("review")
            .join(format!("{agent_id}.json"));
        assert!(
            expected.exists(),
            "sub-agent output file should exist at {expected:?}"
        );
    }

    #[test]
    fn test_handle_multi_agent_completion_final_returns_group_complete() {
        let temp_dir = TempDir::new().unwrap();
        let api_state = make_state_with_temp(&temp_dir);
        let ticket = make_multi_agent_ticket(&temp_dir);

        // 2 sub-agents, both launched; the FIRST has already recorded its output.
        let (second_agent_id, session_name) = {
            let mut state = State::load(&api_state.config).unwrap();
            let group_id = state
                .create_multi_agent_group(
                    &ticket.id,
                    "review",
                    "multi_model",
                    vec![
                        PendingSubAgent {
                            delegator_name: "d1".to_string(),
                            prompt: "p".to_string(),
                            variant_key: "d1".to_string(),
                        },
                        PendingSubAgent {
                            delegator_name: "d2".to_string(),
                            prompt: "p".to_string(),
                            variant_key: "d2".to_string(),
                        },
                    ],
                )
                .unwrap();

            let a1 = state
                .add_agent_with_options(
                    ticket.id.clone(),
                    ticket.ticket_type.clone(),
                    ticket.project.clone(),
                    false,
                    Some("claude".to_string()),
                    Some("default".to_string()),
                )
                .unwrap();
            state.update_agent_session(&a1, "op-TASK-555-d1").unwrap();
            state.mark_launched(&group_id, "d1", &a1).unwrap();
            // Simulate first sub-agent already recorded (as if sync had processed it)
            state
                .record_agent_output(&a1, serde_json::json!({"summary": "first"}))
                .unwrap();

            let a2 = state
                .add_agent_with_options(
                    ticket.id.clone(),
                    ticket.ticket_type.clone(),
                    ticket.project.clone(),
                    false,
                    Some("claude".to_string()),
                    Some("default".to_string()),
                )
                .unwrap();
            let session_name = "op-TASK-555-d2".to_string();
            state.update_agent_session(&a2, &session_name).unwrap();
            state.mark_launched(&group_id, "d2", &a2).unwrap();
            (a2, session_name)
        };

        let req = make_complete_request(&session_name);
        let response = handle_multi_agent_completion(&api_state, &ticket, "review", &req)
            .unwrap()
            .expect("group member → returns Some");

        assert_eq!(
            response.status, "group_complete",
            "last sub-agent should return group_complete"
        );
        assert!(
            !response.auto_proceed,
            "sync loop handles advancement, not REST"
        );

        // Our sub-agent's file is written
        let expected = temp_dir
            .path()
            .join("worktree")
            .join(".tickets")
            .join("steps")
            .join("review")
            .join(format!("{second_agent_id}.json"));
        assert!(expected.exists());
    }

    // ─── complete_step chain-hardening tests ────────────────────────────
    //
    // Uses the builtin SYNC issuetype (scan -> validate -> update): "scan"
    // and "validate" both have review_type None (auto_proceed), which lets
    // duplicate-advance be reproduced (a 2-step type has nowhere further to
    // advance to, so it can't show the defect).

    use crate::agents::launcher::step_command::StepLaunchContext;
    use crate::config::{DetectedTool, ToolCapabilities};
    use std::path::Path;

    fn make_chain_detected_tool() -> DetectedTool {
        DetectedTool {
            name: "claude".to_string(),
            path: "/usr/bin/claude".to_string(),
            version: "1.0.0".to_string(),
            min_version: Some("1.0.0".to_string()),
            version_ok: true,
            model_aliases: vec!["sonnet".to_string(), "opus".to_string()],
            command_template: "claude {{config_flags}}{{model_flag}}--session-id {{session_id}} --print-prompt-path {{prompt_file}}".to_string(),
            capabilities: ToolCapabilities {
                supports_sessions: true,
                supports_headless: true,
            },
            yolo_flags: vec!["--dangerously-skip-permissions".to_string()],
            health_ok: true,
        }
    }

    /// Temp `.tickets/{queue,in-progress,...}` + state tree wired for the
    /// full `complete_step` route, with "claude" detected so
    /// `build_step_command` renders a real (non-error) command.
    struct ChainFixture {
        _temp: TempDir,
        state: ApiState,
    }

    fn make_chain_fixture() -> ChainFixture {
        let temp = TempDir::new().unwrap();
        let tickets_path = temp.path().join(".tickets");
        for d in ["queue", "in-progress", "completed", "templates"] {
            std::fs::create_dir_all(tickets_path.join(d)).unwrap();
        }
        let state_path = temp.path().join("state");
        std::fs::create_dir_all(&state_path).unwrap();

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.state = state_path.to_string_lossy().to_string();
        config.paths.projects = temp.path().join("projects").to_string_lossy().to_string();
        config.llm_tools.detected = vec![make_chain_detected_tool()];

        let state = ApiState::new(config, tickets_path);
        ChainFixture { _temp: temp, state }
    }

    /// Write a SYNC ticket into `in-progress/` so `Queue::find_ticket` sees it.
    fn write_sync_ticket(state: &ApiState, id: &str, step: &str) -> Ticket {
        let filename = format!("20260807-0900-SYNC-chainproj-{}.md", id.to_lowercase());
        let content =
            format!("---\nid: {id}\nstatus: running\nstep: {step}\n---\n\n# Chain ticket\n");
        let path = state
            .config
            .tickets_path()
            .join("in-progress")
            .join(filename);
        std::fs::write(&path, content).unwrap();
        Ticket::from_file(&path).unwrap()
    }

    fn make_launch_context(model: &str, session_id: &str) -> StepLaunchContext {
        StepLaunchContext {
            delegator: None,
            tool: "claude".to_string(),
            model: model.to_string(),
            yolo: false,
            session_id: Some(session_id.to_string()),
            opr8r: "opr8r".to_string(),
            operator_relay: None,
            extra_flags: vec![],
        }
    }

    fn make_chain_complete_request(session_id: &str) -> StepCompleteRequest {
        StepCompleteRequest {
            exit_code: 0,
            output_valid: true,
            output_schema_errors: None,
            session_id: Some(session_id.to_string()),
            duration_secs: 5,
            output_sample: None,
            output: Some(OperatorOutput {
                status: "complete".to_string(),
                exit_signal: true,
                summary: Some("done".to_string()),
                ..Default::default()
            }),
        }
    }

    /// Add an agent for `ticket` carrying the given persisted launch context.
    fn add_chain_agent(state: &ApiState, ticket: &Ticket, model: &str, session_id: &str) -> String {
        let mut app_state = State::load(&state.config).unwrap();
        let agent_id = app_state
            .add_agent_with_options(
                ticket.id.clone(),
                ticket.ticket_type.clone(),
                ticket.project.clone(),
                false,
                Some("claude".to_string()),
                None,
            )
            .unwrap();
        app_state
            .update_agent_step_launch_context(&agent_id, make_launch_context(model, session_id))
            .unwrap();
        agent_id
    }

    fn persisted_context_session_id(state: &ApiState, agent_id: &str) -> Option<String> {
        State::load(&state.config)
            .unwrap()
            .agents
            .iter()
            .find(|a| a.id == agent_id)
            .and_then(|a| a.step_launch_context.as_ref())
            .and_then(|c| c.session_id.clone())
    }

    #[tokio::test]
    async fn test_complete_step_next_command_uses_persisted_context() {
        let fixture = make_chain_fixture();
        let ticket = write_sync_ticket(&fixture.state, "SYNC-9001", "scan");
        add_chain_agent(&fixture.state, &ticket, "opus", "session-current");

        let response = complete_step(
            State(fixture.state.clone()),
            Path((ticket.id.clone(), "scan".to_string())),
            Json(make_chain_complete_request("session-current")),
        )
        .await
        .unwrap()
        .0;

        assert!(
            response.auto_proceed,
            "scan has review_type None, so it should auto-proceed"
        );
        let next_command = response.next_command.expect("next_command present");
        assert!(
            next_command.contains("--model opus"),
            "persisted launch-context model must reach the command: {next_command}"
        );
        assert!(
            next_command.contains(&format!("--ticket-id={}", ticket.id)),
            "opr8r wrapper must carry the ticket id: {next_command}"
        );
        assert!(
            !next_command.contains("docker run"),
            "next_command must never be target-wrapped: {next_command}"
        );
    }

    #[test]
    fn test_complete_step_next_command_mints_fresh_session_uuid() {
        let fixture = make_chain_fixture();
        let ticket = write_sync_ticket(&fixture.state, "SYNC-9002", "scan");
        add_chain_agent(&fixture.state, &ticket, "sonnet", "session-scan");

        let next_step = ticket
            .template_schema()
            .unwrap()
            .get_step("validate")
            .unwrap()
            .clone();
        let request = make_chain_complete_request("session-scan");

        let built = build_next_step_command(&fixture.state, &ticket, &next_step, &request)
            .expect("build_next_step_command");
        assert_ne!(
            built.session_id, "session-scan",
            "the next step must mint a fresh session id, not reuse the completed step's"
        );
        assert!(built
            .command
            .contains(&format!("--session-id={}", built.session_id)));

        record_step_transition(
            &fixture.state,
            &ticket,
            "scan",
            &next_step,
            &built.session_id,
            &request,
        );

        let reloaded = Ticket::from_file(Path::new(&ticket.filepath)).unwrap();
        assert_eq!(
            reloaded.sessions.get("validate"),
            Some(&built.session_id),
            "minted session id must be persisted in the ticket's session map"
        );
    }

    #[tokio::test]
    async fn test_complete_step_duplicate_post_does_not_double_advance() {
        let fixture = make_chain_fixture();
        let ticket = write_sync_ticket(&fixture.state, "SYNC-9003", "scan");
        add_chain_agent(&fixture.state, &ticket, "sonnet", "session-scan");

        let first = complete_step(
            State(fixture.state.clone()),
            Path((ticket.id.clone(), "scan".to_string())),
            Json(make_chain_complete_request("session-scan")),
        )
        .await
        .unwrap();
        assert!(first.0.auto_proceed);

        let after_first = Ticket::from_file(Path::new(&ticket.filepath)).unwrap();
        assert_eq!(
            after_first.step, "validate",
            "first POST advances scan -> validate"
        );

        // opr8r retries the same completion POST (duplicate delivery).
        let second = complete_step(
            State(fixture.state.clone()),
            Path((ticket.id.clone(), "scan".to_string())),
            Json(make_chain_complete_request("session-scan")),
        )
        .await
        .unwrap();
        assert!(second.0.auto_proceed);

        let after_second = Ticket::from_file(Path::new(&ticket.filepath)).unwrap();
        assert_eq!(
            after_second.step, "validate",
            "duplicate POST for an already-completed step must not advance a second time"
        );
    }

    #[test]
    fn test_complete_step_agent_lookup_prefers_session_id() {
        let fixture = make_chain_fixture();
        let ticket = write_sync_ticket(&fixture.state, "SYNC-9004", "scan");
        add_chain_agent(&fixture.state, &ticket, "opus", "session-agent-1");
        add_chain_agent(&fixture.state, &ticket, "sonnet", "session-agent-2");

        let next_step = ticket
            .template_schema()
            .unwrap()
            .get_step("validate")
            .unwrap()
            .clone();
        let request = make_chain_complete_request("session-agent-2");

        let built = build_next_step_command(&fixture.state, &ticket, &next_step, &request)
            .expect("build_next_step_command");
        assert!(
            built.command.contains("--model sonnet"),
            "must prefer the agent whose persisted session id matches the request, \
             not the first agent found by ticket id: {}",
            built.command
        );
    }

    #[test]
    fn test_find_completing_agent_session_arm_requires_ticket_match() {
        let fixture = make_chain_fixture();
        let other = write_sync_ticket(&fixture.state, "SYNC-9005", "scan");
        let target = write_sync_ticket(&fixture.state, "SYNC-9006", "scan");
        // The other ticket's agent is registered FIRST and carries the session
        // id being reported, so a ticket-blind session arm would pick it.
        add_chain_agent(&fixture.state, &other, "opus", "shared-session");
        let target_agent = add_chain_agent(&fixture.state, &target, "sonnet", "target-session");

        let app_state = State::load(&fixture.state.config).unwrap();
        let found = find_completing_agent(&app_state.agents, &target.id, Some("shared-session"))
            .expect("falls back to a ticket match");
        assert_eq!(
            found.id, target_agent,
            "a session match on another ticket must not win"
        );
    }

    #[tokio::test]
    async fn test_transition_repoints_context_session_id_for_next_completion() {
        let fixture = make_chain_fixture();
        let ticket = write_sync_ticket(&fixture.state, "SYNC-9007", "scan");
        // Registered first: the ticket-id fallback would pick this one.
        add_chain_agent(&fixture.state, &ticket, "sonnet", "session-other");
        let chain_agent = add_chain_agent(&fixture.state, &ticket, "opus", "session-scan");

        let first = complete_step(
            State(fixture.state.clone()),
            Path((ticket.id.clone(), "scan".to_string())),
            Json(make_chain_complete_request("session-scan")),
        )
        .await
        .unwrap()
        .0;
        assert!(first.auto_proceed);

        let validate_session = Ticket::from_file(Path::new(&ticket.filepath))
            .unwrap()
            .sessions
            .get("validate")
            .cloned()
            .expect("validate session id persisted on the ticket");
        assert_eq!(
            persisted_context_session_id(&fixture.state, &chain_agent),
            Some(validate_session.clone()),
            "the transition must re-point the agent's context at the next step's uuid"
        );

        // Second transition: the session arm must still find the chain agent.
        let second = complete_step(
            State(fixture.state.clone()),
            Path((ticket.id.clone(), "validate".to_string())),
            Json(make_chain_complete_request(&validate_session)),
        )
        .await
        .unwrap()
        .0;
        let next_command = second.next_command.expect("next_command present");
        assert!(
            next_command.contains("--model opus"),
            "session arm must still match the chain agent on transition 2: {next_command}"
        );
    }
}
