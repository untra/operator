//! MCP tools that launch tickets and control the queue and agent reviews.

use crate::rest::state::RequestState as State;
use axum::extract::Path;
use axum::Json;
use serde_json::Value;

use super::args::{reply, required_str};
use crate::rest::dto::{LaunchTicketRequest, RejectReviewRequest};
use crate::rest::routes;
use crate::rest::state::ApiState;

const DEFAULT_LAUNCH_MODEL: &str = "sonnet";
const DEFAULT_LAUNCH_WRAPPER: &str = "terminal";

pub async fn launch_ticket(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = required_str(&args, "id")?;
    let model = args
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_LAUNCH_MODEL);
    let wrapper = args
        .get("wrapper")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_LAUNCH_WRAPPER);
    let request = LaunchTicketRequest {
        target: None,
        delegator: None,
        provider: None,
        model: Some(model.to_string()),
        model_server: None,
        yolo_mode: false,
        wrapper: Some(wrapper.to_string()),
        retry_reason: None,
        resume_session_id: None,
    };
    reply(
        routes::launch::launch_ticket(State(state.clone()), Path(id.to_string()), Json(request))
            .await,
    )
}

pub async fn pause_queue(_args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::queue::pause(State(state.clone())).await)
}

pub async fn resume_queue(_args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::queue::resume(State(state.clone())).await)
}

pub async fn sync_kanban(_args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::queue::sync(State(state.clone())).await)
}

pub async fn approve_agent(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = required_str(&args, "id")?;
    reply(routes::agents::approve_review(State(state.clone()), Path(id.to_string())).await)
}

pub async fn reject_agent(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = required_str(&args, "id")?;
    let request = RejectReviewRequest {
        reason: required_str(&args, "reason")?.to_string(),
    };
    reply(
        routes::agents::reject_review(State(state.clone()), Path(id.to_string()), Json(request))
            .await,
    )
}
