//! MCP tool definitions and execution.
//!
//! Defines read-only and write tools that wrap existing REST API route handlers.
//! Each tool calls the handler directly (no internal HTTP round-trip).
//! Write tools are gated behind `[mcp].expose_ticket_write_tools`.

use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{campaigns, catalog, control, tickets};
use crate::rest::dto::auth::Scope;
use crate::rest::state::ApiState;

/// Definition of a single MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Returns all available MCP tool definitions
pub fn all_tool_definitions() -> Vec<McpToolDefinition> {
    let mut tools = vec![
        McpToolDefinition {
            name: "operator_health".to_string(),
            description: "Check Operator API health status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_status".to_string(),
            description: "Get Operator server status including registry info (issue type count, collection count, active collection)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_list_issue_types".to_string(),
            description: "List all available issue types (FEAT, FIX, SPIKE, etc.)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_get_issue_type".to_string(),
            description: "Get detailed information about a specific issue type by key".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Issue type key (e.g., FEAT, FIX, SPIKE)"
                    }
                },
                "required": ["key"]
            }),
        },
        McpToolDefinition {
            name: "operator_list_collections".to_string(),
            description: "List all issue type collections".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_get_collection".to_string(),
            description: "Get detailed information about a specific collection by name".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["name"]
            }),
        },
        McpToolDefinition {
            name: "operator_list_skills".to_string(),
            description: "List all discovered skills across LLM tools".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_list_tickets".to_string(),
            description: "List tickets in the operator queue. Filter by status: queue, in-progress, completed. Returns id, project, type, summary, priority, branch, and external links - not body content.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["queue", "in-progress", "completed"],
                        "default": "queue",
                        "description": "Which directory to list (defaults to queue)"
                    }
                },
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_claim_ticket".to_string(),
            description: "Move a ticket from queue to in-progress. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Ticket id (e.g. FEAT-1234)" }
                },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_complete_ticket".to_string(),
            description: "Move a ticket from in-progress to completed. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Ticket id" }
                },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_return_to_queue".to_string(),
            description: "Move a ticket from in-progress back to queue (un-claim). Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Ticket id" }
                },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_create_ticket".to_string(),
            description: "Create a new ticket and write it to the queue - the same path as POST /api/v1/tickets. `issue_type` is a registry key (see operator_list_issue_types); `values` holds that issue type's user-editable fields. When the project maps to a bidirectional kanban project the ticket is also pushed there unless sync_to_kanban is false. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: serde_json::to_value(schemars::schema_for!(
                crate::rest::dto::CreateTicketRequest
            ))
            .unwrap_or_else(|_| json!({ "type": "object" })),
        },
        McpToolDefinition {
            name: "operator_launch_ticket".to_string(),
            description: "Launch/start a ticket by claiming it and preparing an agent. Returns agent details and launch command. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Ticket ID to launch (e.g. FEAT-1234)"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (default: sonnet)",
                        "default": "sonnet"
                    },
                    "wrapper": {
                        "type": "string",
                        "description": "Session wrapper type: vscode, tmux, cmux, terminal (default: terminal)",
                        "default": "terminal"
                    }
                },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_pause_queue".to_string(),
            description: "Pause queue processing, stopping automatic ticket launches. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_resume_queue".to_string(),
            description: "Resume queue processing, re-enabling automatic ticket launches. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_sync_kanban".to_string(),
            description: "Sync kanban collections from external providers (Jira, Linear, etc.) and create local tickets. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "operator_approve_agent".to_string(),
            description: "Approve an agent's pending review, signaling it to continue. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Agent ID to approve"
                    }
                },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_reject_agent".to_string(),
            description: "Reject an agent's pending review with a reason. The agent will re-do the work based on the feedback. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Agent ID to reject"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for rejection (feedback for the agent)"
                    }
                },
                "required": ["id", "reason"]
            }),
        },
    ];
    tools.extend(crate::mcp::campaigns::definitions());
    tools
}

/// Gate the ticket-mutating tools.
fn require_write_tools(state: &ApiState, scopes: &[Scope]) -> Result<(), String> {
    if state.authorization.is_none() && !scopes.contains(&Scope::Write) {
        return Err("This tool requires the `write` scope".to_string());
    }
    if !state.config().mcp.expose_ticket_write_tools {
        return Err(
            "Ticket write tools disabled in config ([mcp].expose_ticket_write_tools = true to enable)"
                .to_string(),
        );
    }
    Ok(())
}
type ToolFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Access {
    Read,
    Write,
}

/// Map a tool name to its access level and boxed future. Every tool is boxed
/// here exactly once, so `execute_tool` stays small however large a tool grows.
fn dispatch<'a>(name: &str, args: Value, state: &'a ApiState) -> Option<(Access, ToolFuture<'a>)> {
    use Access::{Read, Write};
    Some(match name {
        "operator_health" => (Read, Box::pin(catalog::health(args, state))),
        "operator_status" => (Read, Box::pin(catalog::status(args, state))),
        "operator_list_issue_types" => (Read, Box::pin(catalog::list_issue_types(args, state))),
        "operator_get_issue_type" => (Read, Box::pin(catalog::get_issue_type(args, state))),
        "operator_list_collections" => (Read, Box::pin(catalog::list_collections(args, state))),
        "operator_get_collection" => (Read, Box::pin(catalog::get_collection(args, state))),
        "operator_list_skills" => (Read, Box::pin(catalog::list_skills(args, state))),
        "operator_list_tickets" => (Read, Box::pin(tickets::list_tickets(args, state))),
        "operator_claim_ticket" => (Write, Box::pin(tickets::claim_ticket(args, state))),
        "operator_complete_ticket" => (Write, Box::pin(tickets::complete_ticket(args, state))),
        "operator_return_to_queue" => (Write, Box::pin(tickets::return_to_queue(args, state))),
        "operator_create_ticket" => (Write, Box::pin(tickets::create_ticket(args, state))),
        "operator_preview_ticket_batch" => (Read, Box::pin(campaigns::preview(args, state))),
        "operator_create_tickets_bulk" => (Write, Box::pin(campaigns::create_bulk(args, state))),
        "operator_create_campaign" => (Write, Box::pin(campaigns::create_campaign(args, state))),
        "operator_assign_ticket_campaign" => (Write, Box::pin(campaigns::assign(args, state))),
        "operator_launch_campaign" => (Write, Box::pin(campaigns::launch(args, state))),
        "operator_campaign_status" => (Read, Box::pin(campaigns::status(args, state))),
        "operator_launch_ticket" => (Write, Box::pin(control::launch_ticket(args, state))),
        "operator_pause_queue" => (Write, Box::pin(control::pause_queue(args, state))),
        "operator_resume_queue" => (Write, Box::pin(control::resume_queue(args, state))),
        "operator_sync_kanban" => (Write, Box::pin(control::sync_kanban(args, state))),
        "operator_approve_agent" => (Write, Box::pin(control::approve_agent(args, state))),
        "operator_reject_agent" => (Write, Box::pin(control::reject_agent(args, state))),
        _ => return None,
    })
}

/// Execute an MCP tool by name with the given arguments
pub async fn execute_tool(
    name: &str,
    args: Value,
    state: &ApiState,
    scopes: &[Scope],
) -> Result<Value, String> {
    if !crate::auth::policy::tool_allowed(state.authorization.as_deref(), name) {
        return Err("Tool not authorized".into());
    }
    let (access, run) =
        dispatch(name, args, state).ok_or_else(|| format!("Unknown tool: {name}"))?;
    if access == Access::Write {
        require_write_tools(state, scopes)?;
    }
    run.await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    const DISPATCH_FUTURE_BUDGET_BYTES: usize = 512;

    #[test]
    fn test_execute_tool_future_stays_within_budget() {
        let state = ApiState::new(Config::default(), PathBuf::from("/tmp/test"));
        let fut = execute_tool("operator_health", json!({}), &state, &Scope::ALL);
        let size = std::mem::size_of_val(&fut);
        assert!(
            size <= DISPATCH_FUTURE_BUDGET_BYTES,
            "execute_tool future is {size} bytes; box tool futures in `dispatch`, not inline"
        );
    }

    #[test]
    fn test_every_defined_tool_dispatches() {
        let state = ApiState::new(Config::default(), PathBuf::from("/tmp/test"));
        for tool in all_tool_definitions() {
            assert!(
                dispatch(&tool.name, json!({}), &state).is_some(),
                "{} is defined but not dispatched",
                tool.name
            );
        }
    }

    #[test]
    fn test_all_tool_definitions_count() {
        let tools = all_tool_definitions();
        assert_eq!(tools.len(), 24);
    }

    #[test]
    fn test_all_tool_definitions_names() {
        let tools = all_tool_definitions();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"operator_health"));
        assert!(names.contains(&"operator_status"));
        assert!(names.contains(&"operator_list_issue_types"));
        assert!(names.contains(&"operator_get_issue_type"));
        assert!(names.contains(&"operator_list_collections"));
        assert!(names.contains(&"operator_get_collection"));
        assert!(names.contains(&"operator_list_skills"));
        assert!(names.contains(&"operator_list_tickets"));
        assert!(names.contains(&"operator_launch_ticket"));
        assert!(names.contains(&"operator_pause_queue"));
        assert!(names.contains(&"operator_resume_queue"));
        assert!(names.contains(&"operator_sync_kanban"));
        assert!(names.contains(&"operator_approve_agent"));
        assert!(names.contains(&"operator_reject_agent"));
        for tool in [
            "operator_preview_ticket_batch",
            "operator_create_tickets_bulk",
            "operator_create_campaign",
            "operator_assign_ticket_campaign",
            "operator_launch_campaign",
            "operator_campaign_status",
        ] {
            assert!(names.contains(&tool), "{tool}");
        }
    }

    #[test]
    fn test_tool_schemas_are_valid_objects() {
        let tools = all_tool_definitions();
        for tool in &tools {
            assert!(
                tool.input_schema.is_object(),
                "Tool '{}' schema should be an object",
                tool.name
            );
            assert_eq!(
                tool.input_schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Tool '{}' schema type should be 'object'",
                tool.name
            );
        }
    }

    #[test]
    fn test_parameterized_tools_have_required_fields() {
        let tools = all_tool_definitions();

        let get_issue_type = tools
            .iter()
            .find(|t| t.name == "operator_get_issue_type")
            .unwrap();
        let required = get_issue_type.input_schema.get("required").unwrap();
        assert!(required.as_array().unwrap().contains(&json!("key")));

        let get_collection = tools
            .iter()
            .find(|t| t.name == "operator_get_collection")
            .unwrap();
        let required = get_collection.input_schema.get("required").unwrap();
        assert!(required.as_array().unwrap().contains(&json!("name")));
    }

    #[tokio::test]
    async fn test_execute_health() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_health", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn test_execute_status() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_status", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn test_execute_list_issue_types() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result =
            execute_tool("operator_list_issue_types", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_array());
        assert!(value.as_array().unwrap().len() >= 5); // At least builtins
    }

    #[tokio::test]
    async fn test_execute_get_issue_type_missing_key() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_get_issue_type", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required parameter"));
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("nonexistent_tool", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    // --- Write tool gate tests (write tools disabled by default) ---

    #[tokio::test]
    async fn test_execute_launch_ticket_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool(
            "operator_launch_ticket",
            json!({"id": "FEAT-1"}),
            &state,
            &Scope::ALL,
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_pause_queue_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_pause_queue", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_resume_queue_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_resume_queue", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_sync_kanban_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_sync_kanban", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_approve_agent_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool(
            "operator_approve_agent",
            json!({"id": "agent-1"}),
            &state,
            &Scope::ALL,
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_reject_agent_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool(
            "operator_reject_agent",
            json!({"id": "agent-1", "reason": "bad"}),
            &state,
            &Scope::ALL,
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_launch_ticket_requires_id() {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_launch_ticket", json!({}), &state, &Scope::ALL).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Missing required parameter: id"));
    }
}
