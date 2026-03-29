//! Ticket launch endpoint for the REST API.
//!
//! Provides the launch endpoint for starting agents via external clients
//! like the VS Code extension.

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

    // Determine status based on exit code and validation
    let status = if request.exit_code != 0 {
        "failed".to_string()
    } else if current_step.review_type != crate::templates::schema::ReviewType::None {
        "awaiting_review".to_string()
    } else {
        "completed".to_string()
    };

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
            launch_config: Some(crate::config::DelegatorLaunchConfig {
                yolo: true,
                permission_mode: Some("accept-edits".to_string()),
                flags: vec!["--verbose".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(false),
                docker: Some(true),
                prompt_prefix: Some("PREFIX".to_string()),
                prompt_suffix: Some("SUFFIX".to_string()),
            }),
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-launch"));

        let request = LaunchTicketRequest {
            delegator: Some("full-delegator".to_string()),
            provider: None,
            model: None,
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
            launch_config: Some(crate::config::DelegatorLaunchConfig::default()),
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-launch"));

        let request = LaunchTicketRequest {
            delegator: Some("minimal".to_string()),
            provider: None,
            model: None,
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
            launch_config: None,
        }
    }

    fn empty_request() -> LaunchTicketRequest {
        LaunchTicketRequest {
            delegator: None,
            provider: None,
            model: None,
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
            launch_config: Some(crate::config::DelegatorLaunchConfig {
                yolo: true,
                permission_mode: None,
                flags: vec!["--full-auto".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(true),
                docker: Some(false),
                prompt_prefix: Some("BEGIN".to_string()),
                prompt_suffix: Some("END".to_string()),
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
}
