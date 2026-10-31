//! MCP tools for ticket batches and campaigns; each delegates to its REST handler.

use crate::rest::state::RequestState as State;
use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

use super::args::{parse, reply, required_str};
use super::tools::McpToolDefinition;
use crate::campaigns::batch::BatchRequest;
use crate::campaigns::service::NewCampaignInput;
use crate::rest::dto::{AssignCampaignRequest, LaunchCampaignRequest};
use crate::rest::routes::campaigns as routes;
use crate::rest::state::ApiState;

const WORKFLOW: &str = "Workflow: operator_list_issue_types -> operator_preview_ticket_batch -> operator_create_campaign -> operator_create_tickets_bulk (with campaign) -> operator_campaign_status -> operator_launch_campaign with dry_run, then for real.";

fn schema<T: schemars::JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).unwrap_or_else(|_| json!({"type": "object"}))
}

fn with_id(mut body: Value, id_description: &str) -> Value {
    if let Some(props) = body.get_mut("properties").and_then(Value::as_object_mut) {
        props.insert(
            "id".into(),
            json!({"type": "string", "description": id_description}),
        );
    }
    let required = body
        .get_mut("required")
        .and_then(Value::as_array_mut)
        .map(std::mem::take)
        .unwrap_or_default();
    body["required"] = Value::Array(std::iter::once(json!("id")).chain(required).collect());
    body
}

pub fn definitions() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "operator_preview_ticket_batch".into(),
            description: format!("Preview a batch of related tickets without writing: allocated ids, dependencies resolved (depends_on takes batch refs, existing ids or provider:key refs), dependency waves, and per-item and cycle errors. Free. {WORKFLOW}"),
            input_schema: schema::<BatchRequest>(),
        },
        McpToolDefinition {
            name: "operator_create_tickets_bulk".into(),
            description: format!("Create every ticket in a batch or none: validates first, then writes, rolling back on failure. Batch refs resolve to allocated ids. Set campaign to create inside one. Free. Disabled unless [mcp].expose_ticket_write_tools = true. {WORKFLOW}"),
            input_schema: schema::<BatchRequest>(),
        },
        McpToolDefinition {
            name: "operator_create_campaign".into(),
            description: format!("Create a draft campaign (Premium). {WORKFLOW}"),
            input_schema: schema::<NewCampaignInput>(),
        },
        McpToolDefinition {
            name: "operator_assign_ticket_campaign".into(),
            description: "Move a ticket into a campaign (or out, with campaign null) and optionally replace its depends_on; an edit that creates a cycle is rejected (Premium).".into(),
            input_schema: with_id(schema::<AssignCampaignRequest>(), "Ticket id (e.g. FEAT-0012)"),
        },
        McpToolDefinition {
            name: "operator_launch_campaign".into(),
            description: "Launch a campaign (Premium). dry_run returns the waves, effective governance and violations without starting. A real launch starts the run and launches the first wave; `operator api` drives later waves.".into(),
            input_schema: with_id(schema::<LaunchCampaignRequest>(), "Campaign id (e.g. CAMP-0001)"),
        },
        McpToolDefinition {
            name: "operator_campaign_status".into(),
            description: "Campaign run status (Premium): per-ticket column and launches, waves, blocked reasons, tickets awaiting the operator, stalled tickets, and counters against governance caps.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string", "description": "Campaign id (e.g. CAMP-0001)"}},
                "required": ["id"]
            }),
        },
    ]
}

fn id(args: &Value) -> Result<String, String> {
    required_str(args, "id").map(String::from)
}

pub async fn preview(args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::batch_preview(State(state.clone()), Json(parse(&args)?)).await)
}

pub async fn create_bulk(args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::batch_create(State(state.clone()), Json(parse(&args)?)).await)
}

pub async fn create_campaign(args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::create(State(state.clone()), Json(parse(&args)?)).await)
}

pub async fn assign(args: Value, state: &ApiState) -> Result<Value, String> {
    let ticket = id(&args)?;
    let request: AssignCampaignRequest = parse(&args)?;
    reply(routes::assign(State(state.clone()), Path(ticket), Json(request)).await)
}

pub async fn launch(args: Value, state: &ApiState) -> Result<Value, String> {
    let campaign = id(&args)?;
    let request: LaunchCampaignRequest = parse(&args)?;
    reply(routes::launch(State(state.clone()), Path(campaign), Json(request)).await)
}

pub async fn status(args: Value, state: &ApiState) -> Result<Value, String> {
    reply(routes::status(State(state.clone()), Path(id(&args)?)).await)
}
