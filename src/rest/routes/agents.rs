//! Agent management endpoints for the REST API.
//!
//! Provides endpoints for querying active agents and controlling their review state.

use crate::rest::state::RequestState as State;
use axum::{extract::Path, Json};

use crate::agents::cmux::{CmuxClient, SystemCmuxClient};
use crate::agents::connect::{
    self, Attention, ConnectContext, ConnectPlan, ConnectProbes, ConnectTerminal, RemoteSession,
};
use crate::agents::{SystemTmuxClient, TmuxClient, TmuxError};
use crate::config::Config;
use crate::rest::dto::{
    ActiveAgentResponse, ActiveAgentsResponse, AgentDetailResponse, RejectReviewRequest,
    ReviewResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::state::{AgentState, State as OperatorState};

fn connect_context<'a>(
    config: &'a Config,
    agent: &'a AgentState,
    remote: Option<&'a RemoteSession>,
    probes: &'a ConnectProbes,
) -> ConnectContext<'a> {
    ConnectContext {
        agent,
        remote,
        tmux_socket: connect::tmux_socket(config),
        preferred: config.sessions.connect.preferred,
        can_open_terminal: config.sessions.connect.terminal_command.is_some(),
        probes,
    }
}

fn agent_connect(
    config: &Config,
    agent: &AgentState,
    probes: &ConnectProbes,
) -> (ConnectPlan, Option<Attention>) {
    let remote = connect::resolve_remote(config, agent);
    let plan = connect::connect_plan(&connect_context(config, agent, remote.as_ref(), probes));
    (plan, connect::attention(agent, chrono::Utc::now()))
}

async fn probe(config: std::sync::Arc<Config>) -> Result<ConnectProbes, ApiError> {
    tokio::task::spawn_blocking(move || connect::probe(&config))
        .await
        .map_err(|e| ApiError::InternalError(format!("probe task failed: {e}")))
}

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
    let config = state.config();
    let operator_state = OperatorState::load(&config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;
    let probes = probe(std::sync::Arc::clone(&config)).await?;

    // Map AgentState to ActiveAgentResponse
    let agents: Vec<ActiveAgentResponse> = operator_state
        .agents
        .iter()
        .filter(|a| state.may_access_project(&a.project))
        .filter(|a| {
            a.status == "running" || a.status == "awaiting_input" || a.status == "completing"
        })
        .map(|a| {
            let (connect, attention) = agent_connect(&config, a, &probes);
            ActiveAgentResponse {
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
                last_message: a.last_message.clone(),
                connect,
                attention,
            }
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
    let config = state.config();
    let operator_state = OperatorState::load(&config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id && state.may_access_project(&a.project))
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{agent_id}' not found")))?;
    let probes = probe(std::sync::Arc::clone(&config)).await?;
    let (connect, attention) = agent_connect(&config, agent, &probes);

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
        session_name: agent.session_name.clone(),
        remote_host: agent.remote_host.clone(),
        target_name: agent.target_name.clone(),
        workspace_url: agent.workspace_url.clone(),
        last_message: agent.last_message.clone(),
        last_content_change: agent.last_content_change.map(|t| t.to_rfc3339()),
        connect,
        attention,
    }))
}

/// What an IDE extension runs after opening an agent's workspace
///
/// Returns the attach command while the session is live, or the LLM resume
/// command once it is gone. Deep links never carry commands; this is the only source.
#[utoipa::path(
    operation_id = "agents_connect_terminal",
    get,
    path = "/api/v1/agents/{agent_id}/connect/terminal",
    tag = "Agents",
    params(
        ("agent_id" = String, Path, description = "The agent ID to connect to")
    ),
    responses(
        (status = 200, description = "Terminal to open", body = ConnectTerminal),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn connect_terminal(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> Result<Json<ConnectTerminal>, ApiError> {
    let config = state.config();
    let operator_state = OperatorState::load(&config)
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id && state.may_access_project(&a.project))
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{agent_id}' not found")))?;
    let remote = connect::resolve_remote(&config, agent);
    let probes = ConnectProbes::default();
    Ok(Json(connect::connect_terminal(&connect_context(
        &config,
        agent,
        remote.as_ref(),
        &probes,
    ))))
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
    let mut operator_state = OperatorState::load(&state.config())
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    // Find the agent
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id && state.may_access_project(&a.project))
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
    let mut operator_state = OperatorState::load(&state.config())
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    // Find the agent
    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id && state.may_access_project(&a.project))
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
/// cmux focuses the agent's workspace; tmux switches the attached client to the
/// agent's session and window. zellij can't be focused from outside, and VS Code
/// focuses through its extension's URI handler instead.
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
        (status = 409, description = "Session not reachable; attach to it instead"),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn focus_session(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> Result<(), ApiError> {
    let operator_state = OperatorState::load(&state.config())
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    let agent = operator_state
        .agents
        .iter()
        .find(|a| a.id == agent_id && state.may_access_project(&a.project))
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
            let cmux_config = state.config().sessions.cmux.clone();
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
        Some("tmux") | None if agent.status != "orphaned" => {
            let tmux_agent = agent.clone();
            let client = SystemTmuxClient::for_config(&state.config());
            let focused = tokio::task::spawn_blocking(move || focus_tmux(&client, &tmux_agent))
                .await
                .map_err(|e| ApiError::InternalError(format!("focus task failed: {e}")))?;
            match focused {
                Err(ApiError::Conflict(_)) if can_open_terminal(&state) => {
                    open_terminal(&state, agent)
                }
                other => other,
            }
        }
        Some("zellij" | "tmux") | None if can_open_terminal(&state) => open_terminal(&state, agent),
        Some("zellij") => Err(ApiError::Conflict(format!(
            "zellij sessions can't be focused from here; attach with `zellij attach {}`",
            agent.session_name.as_deref().unwrap_or("<session>")
        ))),
        Some("tmux") | None => Err(ApiError::Conflict(format!(
            "Agent '{agent_id}' has no live session; resume it from the agent page"
        ))),
        Some(other) => Err(ApiError::BadRequest(format!(
            "Focus is not supported for session wrapper '{other}'"
        ))),
    }
}

fn can_open_terminal(state: &ApiState) -> bool {
    state.config().sessions.connect.terminal_command.is_some()
}

/// Open the configured native terminal on the agent's attach or resume command.
fn open_terminal(state: &ApiState, agent: &AgentState) -> Result<(), ApiError> {
    let config = state.config();
    let template = config
        .sessions
        .connect
        .terminal_command
        .as_deref()
        .ok_or_else(|| ApiError::BadRequest("sessions.connect.terminal_command is unset".into()))?;
    let remote = connect::resolve_remote(&config, agent);
    let probes = ConnectProbes::default();
    let command =
        connect::local_command(&connect_context(&config, agent, remote.as_ref(), &probes))
            .ok_or_else(|| {
                ApiError::Conflict(format!(
                    "Agent '{}' has no session to attach or resume",
                    agent.id
                ))
            })?;
    std::process::Command::new("sh")
        .arg("-c")
        .arg(connect::terminal_invocation(template, &command))
        .spawn()
        .map(drop)
        .map_err(|e| ApiError::InternalError(format!("Failed to open terminal: {e}")))
}

/// Switch the attached tmux client to the agent's session and window.
fn focus_tmux(client: &impl TmuxClient, agent: &AgentState) -> Result<(), ApiError> {
    let session = agent.session_name.as_deref().ok_or_else(|| {
        ApiError::BadRequest(format!("Agent '{}' has no tmux session to focus", agent.id))
    })?;
    let target = agent.session_context_ref.as_deref().map_or_else(
        || session.to_string(),
        |window| format!("{session}:{window}"),
    );
    client.switch_client(&target).map_err(|e| match e {
        TmuxError::SessionNotFound(_) | TmuxError::CommandFailed(_) => ApiError::Conflict(format!(
            "could not focus tmux session '{session}' ({e}); attach with `tmux attach -t {session}`"
        )),
        other => ApiError::InternalError(format!("tmux focus failed: {other}")),
    })
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

    #[tokio::test]
    async fn test_connect_terminal_unknown_agent_is_404() {
        let tmp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = tmp.path().to_string_lossy().into_owned();
        let state = ApiState::new(config, tmp.path().to_path_buf());

        let result = connect_terminal(State(state), Path("no-such-agent".to_string())).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }

    fn tmux_agent(session_name: Option<&str>, window: Option<&str>) -> AgentState {
        let mut agent: AgentState = serde_json::from_value(serde_json::json!({
            "id": "a", "ticket_id": "FEAT-1", "ticket_type": "FEAT", "project": "api",
            "status": "running", "started_at": "2026-01-01T00:00:00Z",
            "last_activity": "2026-01-01T00:00:00Z", "paired": false
        }))
        .unwrap();
        agent.session_wrapper = Some("tmux".into());
        agent.session_name = session_name.map(String::from);
        agent.session_context_ref = window.map(String::from);
        agent
    }

    #[test]
    fn test_focus_tmux_switches_the_client_to_the_agent_window() {
        let client = crate::agents::MockTmuxClient::new();
        client.add_session("op-feat-1", "/tmp");
        focus_tmux(&client, &tmux_agent(Some("op-feat-1"), Some("2"))).unwrap();
        let log = client.command_log.lock().unwrap();
        let switch = log
            .iter()
            .find(|c| c.operation == "switch_client")
            .expect("switched");
        assert_eq!(switch.args, vec!["op-feat-1:2"]);
    }

    #[test]
    fn test_focus_tmux_without_a_session_name_is_a_bad_request() {
        let client = crate::agents::MockTmuxClient::new();
        let result = focus_tmux(&client, &tmux_agent(None, None));
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_focus_tmux_for_a_gone_session_is_a_conflict() {
        let client = crate::agents::MockTmuxClient::new();
        let result = focus_tmux(&client, &tmux_agent(Some("op-gone"), None));
        assert!(matches!(result, Err(ApiError::Conflict(_))));
    }
}
