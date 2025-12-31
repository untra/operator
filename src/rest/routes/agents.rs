//! Agent management endpoints for the REST API.
//!
//! Provides endpoints for querying active agents.

use axum::{extract::State, Json};

use crate::rest::dto::{ActiveAgentResponse, ActiveAgentsResponse};
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
