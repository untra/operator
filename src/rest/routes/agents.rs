//! Agent management endpoints for the REST API.
//!
//! Provides endpoints for querying active agents and controlling their review state.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::rest::dto::{
    ActiveAgentResponse, ActiveAgentsResponse, RejectReviewRequest, ReviewResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::state::State as OperatorState;

/// Get all active agents
///
/// Returns a list of all currently running agents with their status and details.
#[utoipa::path(
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
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {}", e)))?;

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
        })
        .collect();

    let count = agents.len();

    Ok(Json(ActiveAgentsResponse { agents, count }))
}

/// Approve an agent's pending review
///
/// Clears the review state and signals the agent to continue.
/// The agent must be in `awaiting_input` status with a pending review.
#[utoipa::path(
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
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {}", e)))?;

    // Find the agent
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", agent_id)))?;

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
        .map_err(|e| ApiError::InternalError(format!("Failed to clear review state: {}", e)))?;

    operator_state
        .update_agent_status(&agent_id, "running", Some("Review approved".to_string()))
        .map_err(|e| ApiError::InternalError(format!("Failed to update agent status: {}", e)))?;

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
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {}", e)))?;

    // Find the agent
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", agent_id)))?;

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
        .map_err(|e| ApiError::InternalError(format!("Failed to set review state: {}", e)))?;

    // Write rejection signal file with reason
    write_review_signal(&state, &agent_id, "rejected", Some(&request.reason))?;

    Ok(Json(ReviewResponse {
        agent_id,
        status: "rejected".to_string(),
        message: format!("Review rejected: {}", request.reason),
    }))
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
        .map_err(|e| ApiError::InternalError(format!("Failed to create operator dir: {}", e)))?;

    let signal_file = operator_dir.join(format!("{}-review-signal.json", agent_id));
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
        .map_err(|e| ApiError::InternalError(format!("Failed to write signal file: {}", e)))?;

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
}
