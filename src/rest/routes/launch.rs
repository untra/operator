//! Ticket launch endpoint for the REST API.
//!
//! Provides the launch endpoint for starting agents via external clients
//! like the VS Code extension.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::agents::{LaunchOptions, Launcher, PreparedLaunch, RelaunchOptions};
use crate::config::LlmProvider;
use crate::queue::Queue;
use crate::rest::dto::{
    LaunchTicketRequest, LaunchTicketResponse, NextStepInfo, StepCompleteRequest,
    StepCompleteResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// Convert PreparedLaunch to LaunchTicketResponse
fn prepared_launch_to_response(prepared: PreparedLaunch) -> LaunchTicketResponse {
    LaunchTicketResponse {
        agent_id: prepared.agent_id,
        ticket_id: prepared.ticket_id,
        working_directory: prepared.working_directory.to_string_lossy().to_string(),
        command: prepared.command,
        terminal_name: prepared.terminal_name.clone(),
        tmux_session_name: prepared.terminal_name,
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
        .ok_or_else(|| ApiError::NotFound(format!("Ticket '{}' not found", ticket_id)))?;

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
        let relaunch_options = build_relaunch_options(&state, &request)?;
        launcher
            .prepare_relaunch(&ticket, relaunch_options)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?
    } else {
        // New launch - claim ticket from queue
        let launch_options = build_launch_options(&state, &request)?;
        launcher
            .prepare_launch(&ticket, launch_options)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?
    };

    Ok(Json(prepared_launch_to_response(prepared)))
}

/// Build LaunchOptions from the request
fn build_launch_options(
    state: &ApiState,
    request: &LaunchTicketRequest,
) -> Result<LaunchOptions, ApiError> {
    let mut options = LaunchOptions {
        yolo_mode: request.yolo_mode,
        ..Default::default()
    };

    // Set provider/model if specified
    if let Some(ref provider_name) = request.provider {
        // Find the provider in config by tool name
        let provider = state
            .config
            .llm_tools
            .providers
            .iter()
            .find(|p| p.tool == *provider_name)
            .cloned();

        if let Some(p) = provider {
            // Use specified model or default to provider's model
            let model = request.model.clone().unwrap_or(p.model.clone());
            options.provider = Some(LlmProvider {
                tool: p.tool,
                model,
                ..Default::default()
            });
        } else {
            return Err(ApiError::BadRequest(format!(
                "Unknown provider '{}'",
                provider_name
            )));
        }
    } else if let Some(ref model) = request.model {
        // Model specified without provider - use default provider with custom model
        if let Some(p) = state.config.llm_tools.providers.first().cloned() {
            options.provider = Some(LlmProvider {
                tool: p.tool,
                model: model.clone(),
                ..Default::default()
            });
        }
    }

    Ok(options)
}

/// Build RelaunchOptions from the request
fn build_relaunch_options(
    state: &ApiState,
    request: &LaunchTicketRequest,
) -> Result<RelaunchOptions, ApiError> {
    let launch_options = build_launch_options(state, request)?;

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
        .ok_or_else(|| ApiError::NotFound(format!("Ticket '{}' not found", ticket_id)))?;

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

    Ok(Json(StepCompleteResponse {
        status,
        next_step: next_step_info,
        auto_proceed,
        next_command,
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
            provider: None,
            model: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(!options.yolo_mode);
        assert!(options.provider.is_none());
    }

    #[test]
    fn test_build_launch_options_yolo() {
        let state = make_state();
        let request = LaunchTicketRequest {
            provider: None,
            model: None,
            yolo_mode: true,
            wrapper: Some("vscode".to_string()),
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(options.yolo_mode);
    }

    #[test]
    fn test_build_launch_options_unknown_provider() {
        let state = make_state();
        let request = LaunchTicketRequest {
            provider: Some("unknown-provider".to_string()),
            model: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: None,
            resume_session_id: None,
        };

        let result = build_launch_options(&state, &request);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_relaunch_options() {
        let state = make_state();
        let request = LaunchTicketRequest {
            provider: None,
            model: None,
            yolo_mode: false,
            wrapper: None,
            retry_reason: Some("Previous attempt timed out".to_string()),
            resume_session_id: Some("abc-123".to_string()),
        };

        let result = build_relaunch_options(&state, &request);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert!(!options.launch_options.yolo_mode);
        assert_eq!(
            options.retry_reason,
            Some("Previous attempt timed out".to_string())
        );
        assert_eq!(options.resume_session_id, Some("abc-123".to_string()));
    }
}
