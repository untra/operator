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

    let prepared = if in_progress_path.exists() {
        // Ticket is in-progress - use relaunch flow (no claim needed)
        let relaunch_options = build_relaunch_options(&state, &request, agent_context.as_ref())?;
        launcher
            .prepare_relaunch(&ticket, relaunch_options)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?
    } else {
        // New launch - claim ticket from queue
        let launch_options = build_launch_options(&state, &request, agent_context.as_ref())?;
        launcher
            .prepare_launch(&ticket, launch_options)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?
    };

    Ok(Json(prepared_launch_to_response(prepared)))
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

    // Build next command if auto-proceeding
    // For now, return a placeholder - actual implementation would build the full opr8r command
    let next_command = if auto_proceed {
        next_step_info.as_ref().map(|next| {
            format!(
                "opr8r --ticket-id={} --step={} -- claude --prompt 'Continue with step {}'",
                ticket_id, next.name, next.name
            )
        })
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
            }),
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-launch"));

        let request = LaunchTicketRequest {
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
        assert!(options.docker_mode);
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
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-launch"));

        let request = LaunchTicketRequest {
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
        assert!(!options.docker_mode);
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
        }
    }

    fn empty_request() -> LaunchTicketRequest {
        LaunchTicketRequest {
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
            }),
        }]);
        let ctx = AgentContext {
            step_agent: Some("codex-auto".to_string()),
            issuetype_agent: None,
        };

        let options = build_launch_options(&state, &empty_request(), Some(&ctx)).unwrap();
        assert!(options.yolo_mode);
        assert!(!options.docker_mode);
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
}
