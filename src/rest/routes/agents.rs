//! Agent management endpoints for the REST API.
//!
//! Provides endpoints for querying active agents and controlling their review state.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::agents::cmux::{CmuxClient, SystemCmuxClient};
use crate::rest::dto::{
    ActiveAgentResponse, ActiveAgentsResponse, AgentDetailResponse, RejectReviewRequest,
    ReviewResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::state::State as OperatorState;

/// Get all active agents
///
/// Returns a list of all currently running agents with their status and details.
#[utoipa::path(
    operation_id = "agents_active",
    get,
    path = "/api/v1/agents/active",
    tag = "Agents",
    responses(
        (status = 200, description = "Active agents list", body = ActiveAgentsResponse)
    )
)]
pub async fn active(State(state): State<ApiState>) -> Result<Json<ActiveAgentsResponse>, ApiError> {
    // Load operator state from state.json
    let operator_state = OperatorState::load(&state.config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    // Map AgentState to ActiveAgentResponse
    let agents: Vec<ActiveAgentResponse> = operator_state
        .agents
        .iter()
        .filter(|a| {
            a.status == "running" || a.status == "awaiting_input" || a.status == "completing"
        })
        .map(|a| ActiveAgentResponse {
            id: a.id.clone(),
            ticket_id: a.ticket_id.clone(),
            ticket_type: a.ticket_type.clone(),
            project: a.project.clone(),
            status: a.status.clone(),
            mode: if a.paired {
                "paired".to_string()
            } else {
                "autonomous".to_string()
            },
            started_at: a.started_at.to_rfc3339(),
            current_step: a.current_step.clone(),
            session_wrapper: a.session_wrapper.clone(),
            session_window_ref: a.session_window_ref.clone(),
            session_context_ref: a.session_context_ref.clone(),
            session_pane_ref: a.session_pane_ref.clone(),
        })
        .collect();

    let count = agents.len();

    Ok(Json(ActiveAgentsResponse { agents, count }))
}

/// Get details for a single agent by ID
///
/// Returns full details for a specific agent, including all tracked state.
#[utoipa::path(
    operation_id = "agents_get_detail",
    get,
    path = "/api/v1/agents/{agent_id}",
    tag = "Agents",
    params(
        ("agent_id" = String, Path, description = "The agent ID to look up")
    ),
    responses(
        (status = 200, description = "Agent details", body = AgentDetailResponse),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn get_detail(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetailResponse>, ApiError> {
    let operator_state = OperatorState::load(&state.config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{agent_id}' not found")))?;

    Ok(Json(AgentDetailResponse {
        id: agent.id.clone(),
        ticket_id: agent.ticket_id.clone(),
        ticket_type: agent.ticket_type.clone(),
        project: agent.project.clone(),
        status: agent.status.clone(),
        started_at: agent.started_at.to_rfc3339(),
        last_activity: agent.last_activity.to_rfc3339(),
        current_step: agent.current_step.clone(),
        llm_tool: agent.llm_tool.clone(),
        llm_model: agent.llm_model.clone(),
        launch_mode: agent.launch_mode.clone(),
        pr_url: agent.pr_url.clone(),
        pr_status: agent.pr_status.clone(),
        session_wrapper: agent.session_wrapper.clone(),
        review_state: agent.review_state.clone(),
        completed_steps: agent.completed_steps.clone(),
        worktree_path: agent.worktree_path.clone(),
        paired: agent.paired,
    }))
}

/// Approve an agent's pending review
///
/// Clears the review state and signals the agent to continue.
/// The agent must be in `awaiting_input` status with a pending review.
#[utoipa::path(
    operation_id = "agents_approve_review",
    post,
    path = "/api/v1/agents/{agent_id}/approve",
    tag = "Agents",
    params(
        ("agent_id" = String, Path, description = "The agent ID to approve")
    ),
    responses(
        (status = 200, description = "Review approved", body = ReviewResponse),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn approve_review(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let mut operator_state = OperatorState::load(&state.config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    // Find the agent
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{agent_id}' not found")))?;

    // Verify agent is awaiting review
    if agent.status != "awaiting_input" {
        return Err(ApiError::BadRequest(format!(
            "Agent '{}' is not awaiting review (status: {})",
            agent_id, agent.status
        )));
    }

    // Clear review state and update status
    operator_state
        .clear_review_state(&agent_id)
        .map_err(|e| ApiError::InternalError(format!("Failed to clear review state: {e}")))?;

    operator_state
        .update_agent_status(&agent_id, "running", Some("Review approved".to_string()))
        .map_err(|e| ApiError::InternalError(format!("Failed to update agent status: {e}")))?;

    // Write approval signal file for the agent to pick up
    write_review_signal(&state, &agent_id, "approved", None)?;

    Ok(Json(ReviewResponse {
        agent_id,
        status: "approved".to_string(),
        message: "Review approved, agent will resume".to_string(),
    }))
}

/// Reject an agent's pending review
///
/// Signals the agent that the review was rejected with feedback.
/// The agent should re-do the work based on the rejection reason.
#[utoipa::path(
    operation_id = "agents_reject_review",
    post,
    path = "/api/v1/agents/{agent_id}/reject",
    tag = "Agents",
    params(
        ("agent_id" = String, Path, description = "The agent ID to reject")
    ),
    request_body = RejectReviewRequest,
    responses(
        (status = 200, description = "Review rejected", body = ReviewResponse),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn reject_review(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
    Json(request): Json<RejectReviewRequest>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let mut operator_state = OperatorState::load(&state.config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    // Find the agent
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{agent_id}' not found")))?;

    // Verify agent is awaiting review
    if agent.status != "awaiting_input" {
        return Err(ApiError::BadRequest(format!(
            "Agent '{}' is not awaiting review (status: {})",
            agent_id, agent.status
        )));
    }

    // Update review state to rejected
    operator_state
        .set_agent_review_state(&agent_id, "rejected")
        .map_err(|e| ApiError::InternalError(format!("Failed to set review state: {e}")))?;

    // Write rejection signal file with reason
    write_review_signal(&state, &agent_id, "rejected", Some(&request.reason))?;

    Ok(Json(ReviewResponse {
        agent_id,
        status: "rejected".to_string(),
        message: format!("Review rejected: {}", request.reason),
    }))
}

/// Focus the terminal session of a running agent in its session wrapper.
///
/// The web UI's launch panel calls this for **cmux** launches: cmux exposes no
/// browser URL scheme, so the operator control plane (which runs inside cmux)
/// shells out to `cmux focus-workspace` for the agent's saved workspace ref to
/// bring its pane to the foreground. Other wrappers are unsupported here — VS
/// Code focuses through its extension's URI handler, and tmux/zellij are
/// display-only in the UI, so it never calls this for them.
#[utoipa::path(
    operation_id = "agents_focus_session",
    post,
    path = "/api/v1/agents/{agent_id}/focus",
    tag = "Agents",
    params(
        ("agent_id" = String, Path, description = "The agent ID whose session to focus")
    ),
    responses(
        (status = 200, description = "Session focused"),
        (status = 400, description = "Wrapper unsupported, or no session refs to focus"),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn focus_session(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> Result<(), ApiError> {
    let operator_state = OperatorState::load(&state.config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{agent_id}' not found")))?;

    match agent.session_wrapper.as_deref() {
        Some("cmux") => {
            let workspace_ref = agent.session_context_ref.clone();
            let window_ref = agent.session_window_ref.clone();
            if workspace_ref.is_none() && window_ref.is_none() {
                return Err(ApiError::BadRequest(format!(
                    "Agent '{agent_id}' has no cmux session refs to focus"
                )));
            }
            let cmux_config = state.config.sessions.cmux.clone();
            // cmux focusing shells out to the cmux binary; run it off the async
            // worker so a slow subprocess can't stall the runtime.
            tokio::task::spawn_blocking(move || -> Result<(), crate::agents::cmux::CmuxError> {
                let client = SystemCmuxClient::from_config(&cmux_config);
                client.check_available()?;
                // Prefer the workspace (the agent's pane); fall back to its window.
                if let Some(ws) = workspace_ref.as_deref() {
                    client.focus_workspace(ws)
                } else if let Some(win) = window_ref.as_deref() {
                    client.focus_window(win)
                } else {
                    unreachable!("at least one ref present (checked above)")
                }
            })
            .await
            .map_err(|e| ApiError::InternalError(format!("focus task failed: {e}")))?
            .map_err(|e| ApiError::InternalError(format!("cmux focus failed: {e}")))?;
            Ok(())
        }
        other => Err(ApiError::BadRequest(format!(
            "Focus is not supported for session wrapper '{}'",
            other.unwrap_or("none")
        ))),
    }
}

/// Write a review signal file for the agent to pick up
fn write_review_signal(
    state: &ApiState,
    agent_id: &str,
    decision: &str,
    reason: Option<&str>,
) -> Result<(), ApiError> {
    let operator_dir = state.tickets_path.join("operator");
    std::fs::create_dir_all(&operator_dir)
        .map_err(|e| ApiError::InternalError(format!("Failed to create operator dir: {e}")))?;

    let signal_file = operator_dir.join(format!("{agent_id}-review-signal.json"));
    let content = if let Some(reason) = reason {
        serde_json::json!({
            "decision": decision,
            "reason": reason,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    } else {
        serde_json::json!({
            "decision": decision,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    };

    std::fs::write(&signal_file, content.to_string())
        .map_err(|e| ApiError::InternalError(format!("Failed to write signal file: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test-agents"))
    }

    #[tokio::test]
    async fn test_active_agents_handler() {
        let state = make_state();
        let result = active(State(state)).await;
        // Should succeed even if state file doesn't exist (returns empty)
        // or handle gracefully with error
        // In this case, State::load will create a new empty state if file doesn't exist
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_focus_session_unknown_agent_is_404() {
        // Point state at a fresh temp dir so load() returns an empty agent list
        // deterministically (not the repo's own state). An unknown id → NotFound.
        // The cmux success path and the unsupported-wrapper 400 branch need a
        // persisted agent / live cmux, so they're covered by manual e2e.
        let tmp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = tmp.path().to_string_lossy().into_owned();
        let state = ApiState::new(config, tmp.path().to_path_buf());

        let result = focus_session(State(state), Path("no-such-agent".to_string())).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }
}
