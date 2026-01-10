//! Ticket launch endpoint for the REST API.
//!
//! Provides the launch endpoint for starting agents via external clients
//! like the VS Code extension.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::agents::{LaunchOptions, Launcher, PreparedLaunch};
use crate::config::LlmProvider;
use crate::queue::Queue;
use crate::rest::dto::{LaunchTicketRequest, LaunchTicketResponse};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// Convert PreparedLaunch to LaunchTicketResponse
fn prepared_launch_to_response(prepared: PreparedLaunch) -> LaunchTicketResponse {
    LaunchTicketResponse {
        agent_id: prepared.agent_id,
        ticket_id: prepared.ticket_id,
        working_directory: prepared.working_directory.to_string_lossy().to_string(),
        command: prepared.command,
        terminal_name: prepared.terminal_name,
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

    // Check if ticket is already in progress
    if ticket.status == "running" || ticket.status == "in-progress" {
        // Check if it's actually in the in-progress directory
        let in_progress_path = state
            .config
            .tickets_path()
            .join("in-progress")
            .join(&ticket.filename);
        if in_progress_path.exists() {
            return Err(ApiError::Conflict(format!(
                "Ticket '{}' is already in progress",
                ticket_id
            )));
        }
    }

    // Build launch options from request
    let launch_options = build_launch_options(&state, &request)?;

    // Create launcher and prepare the launch
    let launcher =
        Launcher::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let prepared = launcher
        .prepare_launch(&ticket, launch_options)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

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
        };

        let result = build_launch_options(&state, &request);
        assert!(result.is_err());
    }
}
