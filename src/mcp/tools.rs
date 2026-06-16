//! MCP tool definitions and execution.
//!
//! Defines read-only and write tools that wrap existing REST API route handlers.
//! Each tool calls the handler directly (no internal HTTP round-trip).
//! Write tools are gated behind `[mcp].expose_ticket_write_tools`.

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::rest::dto::{LaunchTicketRequest, RejectReviewRequest};
use crate::rest::routes;
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
    vec![
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
            description: "List tickets in the operator queue. Filter by status: queue, in-progress, completed. Returns id, project, type, summary, priority, branch, and external links — not body content.".to_string(),
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
            description: "Create a new ticket from a template (feature, fix, spike, investigation, task) and write it to the queue. Returns the filename. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "template": {
                        "type": "string",
                        "description": "Template type key (feature, fix, spike, investigation, task)"
                    },
                    "values": {
                        "type": "object",
                        "description": "Handlebars values for the template (project, summary, id, etc.)"
                    }
                },
                "required": ["template"]
            }),
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
    ]
}

fn require_write_tools(state: &ApiState) -> Result<(), String> {
    if state.config.mcp.expose_ticket_write_tools {
        Ok(())
    } else {
        Err(
            "Ticket write tools disabled in config ([mcp].expose_ticket_write_tools = true to enable)"
                .to_string(),
        )
    }
}

/// Execute an MCP tool by name with the given arguments
pub async fn execute_tool(name: &str, args: Value, state: &ApiState) -> Result<Value, String> {
    match name {
        "operator_health" => {
            let resp = routes::health::health(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_status" => {
            let resp = routes::health::status(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_list_issue_types" => {
            let resp = routes::issuetypes::list(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_get_issue_type" => {
            let key = args
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: key".to_string())?;
            let result =
                routes::issuetypes::get_one(State(state.clone()), Path(key.to_string())).await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(_e) => Err(format!("Issue type '{key}' not found")),
            }
        }
        "operator_list_collections" => {
            let resp = routes::collections::list(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_get_collection" => {
            let name_param = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: name".to_string())?;
            let result =
                routes::collections::get_one(State(state.clone()), Path(name_param.to_string()))
                    .await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(_e) => Err(format!("Collection '{name_param}' not found")),
            }
        }
        "operator_list_skills" => {
            let resp = routes::skills::list(State(state.clone())).await;
            serde_json::to_value(&*resp).map_err(|e| e.to_string())
        }
        "operator_list_tickets" => crate::mcp::tickets::list_tickets(args, state).await,
        "operator_claim_ticket" => {
            require_write_tools(state)?;
            crate::mcp::tickets::claim_ticket(args, state).await
        }
        "operator_complete_ticket" => {
            require_write_tools(state)?;
            crate::mcp::tickets::complete_ticket(args, state).await
        }
        "operator_return_to_queue" => {
            require_write_tools(state)?;
            crate::mcp::tickets::return_to_queue(args, state).await
        }
        "operator_create_ticket" => {
            require_write_tools(state)?;
            crate::mcp::tickets::create_ticket(args, state).await
        }
        "operator_launch_ticket" => {
            require_write_tools(state)?;
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: id".to_string())?;
            let model = args
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("sonnet");
            let wrapper = args
                .get("wrapper")
                .and_then(|v| v.as_str())
                .unwrap_or("terminal");
            let request = LaunchTicketRequest {
                delegator: None,
                provider: None,
                model: Some(model.to_string()),
                model_server: None,
                yolo_mode: false,
                wrapper: Some(wrapper.to_string()),
                retry_reason: None,
                resume_session_id: None,
            };
            let result = routes::launch::launch_ticket(
                State(state.clone()),
                Path(id.to_string()),
                Json(request),
            )
            .await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        "operator_pause_queue" => {
            require_write_tools(state)?;
            let result = routes::queue::pause(State(state.clone())).await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        "operator_resume_queue" => {
            require_write_tools(state)?;
            let result = routes::queue::resume(State(state.clone())).await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        "operator_sync_kanban" => {
            require_write_tools(state)?;
            let result = routes::queue::sync(State(state.clone())).await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        "operator_approve_agent" => {
            require_write_tools(state)?;
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: id".to_string())?;
            let result =
                routes::agents::approve_review(State(state.clone()), Path(id.to_string())).await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        "operator_reject_agent" => {
            require_write_tools(state)?;
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: id".to_string())?;
            let reason = args
                .get("reason")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: reason".to_string())?;
            let request = RejectReviewRequest {
                reason: reason.to_string(),
            };
            let result = routes::agents::reject_review(
                State(state.clone()),
                Path(id.to_string()),
                Json(request),
            )
            .await;
            match result {
                Ok(resp) => serde_json::to_value(&*resp).map_err(|e| e.to_string()),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        _ => Err(format!("Unknown tool: {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_all_tool_definitions_count() {
        let tools = all_tool_definitions();
        assert_eq!(tools.len(), 18);
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

        let result = execute_tool("operator_health", json!({}), &state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn test_execute_status() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_status", json!({}), &state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn test_execute_list_issue_types() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_list_issue_types", json!({}), &state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_array());
        assert!(value.as_array().unwrap().len() >= 5); // At least builtins
    }

    #[tokio::test]
    async fn test_execute_get_issue_type_missing_key() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_get_issue_type", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required parameter"));
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("nonexistent_tool", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    // --- Write tool gate tests (write tools disabled by default) ---

    #[tokio::test]
    async fn test_execute_launch_ticket_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_launch_ticket", json!({"id": "FEAT-1"}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_pause_queue_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_pause_queue", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_resume_queue_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_resume_queue", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_sync_kanban_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_sync_kanban", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ticket write tools disabled"));
    }

    #[tokio::test]
    async fn test_execute_approve_agent_disabled() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = execute_tool("operator_approve_agent", json!({"id": "agent-1"}), &state).await;
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

        let result = execute_tool("operator_launch_ticket", json!({}), &state).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Missing required parameter: id"));
    }
}
