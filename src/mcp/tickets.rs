//! Ticket-queue MCP tools.
//!
//! Reads/writes via `crate::queue::Queue` which uses blocking `std::fs`,
//! so all calls are wrapped in `tokio::task::spawn_blocking`.

use serde_json::{json, Value};

use crate::queue::{Queue, Ticket};
use crate::rest::state::ApiState;

fn ticket_to_json(t: &Ticket) -> Value {
    json!({
        "id": t.id,
        "filename": t.filename,
        "project": t.project,
        "ticket_type": t.ticket_type,
        "summary": t.summary,
        "priority": t.priority,
        "status": t.status,
        "branch": t.branch,
        "external_id": t.external_id,
        "external_url": t.external_url,
        "external_provider": t.external_provider,
    })
}

pub async fn list_tickets(args: Value, state: &ApiState) -> Result<Value, String> {
    let status = args
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("queue")
        .to_string();
    let config = (*state.config).clone();
    let tickets = tokio::task::spawn_blocking(move || -> Result<Vec<Ticket>, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        match status.as_str() {
            "queue" => queue.list_queue().map_err(|e| e.to_string()),
            "in-progress" => queue.list_in_progress().map_err(|e| e.to_string()),
            "completed" => queue.list_completed().map_err(|e| e.to_string()),
            other => Err(format!("Unknown ticket status: {other}")),
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let json_tickets: Vec<Value> = tickets.iter().map(ticket_to_json).collect();
    Ok(json!({ "tickets": json_tickets, "count": json_tickets.len() }))
}

async fn find_ticket(state: &ApiState, id: &str, in_status: &str) -> Result<Ticket, String> {
    let id = id.to_string();
    let in_status = in_status.to_string();
    let config = (*state.config).clone();
    tokio::task::spawn_blocking(move || -> Result<Ticket, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        let list = match in_status.as_str() {
            "queue" => queue.list_queue(),
            "in-progress" => queue.list_in_progress(),
            "completed" => queue.list_completed(),
            other => return Err(format!("Unknown status: {other}")),
        }
        .map_err(|e| e.to_string())?;
        list.into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Ticket {id} not found in {in_status}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn claim_ticket(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing required arg: id")?;
    let ticket = find_ticket(state, id, "queue").await?;
    let config = (*state.config).clone();
    let id_str = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        queue.claim_ticket(&ticket).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(json!({ "id": id_str, "moved_to": "in-progress" }))
}

pub async fn complete_ticket(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing required arg: id")?;
    let ticket = find_ticket(state, id, "in-progress").await?;
    let config = (*state.config).clone();
    let id_str = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        queue.complete_ticket(&ticket).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(json!({ "id": id_str, "moved_to": "completed" }))
}

pub async fn return_to_queue(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing required arg: id")?;
    let ticket = find_ticket(state, id, "in-progress").await?;
    let config = (*state.config).clone();
    let id_str = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        queue.return_to_queue(&ticket).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(json!({ "id": id_str, "moved_to": "queue" }))
}

pub async fn create_ticket(args: Value, state: &ApiState) -> Result<Value, String> {
    use crate::queue::creator::TicketCreator;
    use crate::templates::TemplateType;
    use std::collections::HashMap;

    let template_str = args
        .get("template")
        .and_then(|v| v.as_str())
        .ok_or("Missing required arg: template")?;
    let template_type = TemplateType::from_key(template_str)
        .ok_or_else(|| format!("Unknown template type: {template_str}"))?;

    let mut values: HashMap<String, String> = HashMap::new();
    if let Some(obj) = args.get("values").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                values.insert(k.clone(), s.to_string());
            }
        }
    }

    let config = (*state.config).clone();
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf, String> {
        let creator = TicketCreator::new(&config);
        creator
            .create_ticket_headless(template_type, &values)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(json!({
        "path": path.to_string_lossy(),
        "filename": path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn test_state_with_config(mut config: Config) -> (ApiState, PathBuf) {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.keep();
        // Point the config's tickets path at the tempdir so Queue::new resolves there.
        config.paths.tickets = path.to_string_lossy().into_owned();
        let state = ApiState::new(config, path.clone());
        (state, path)
    }

    fn test_state() -> ApiState {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        test_state_with_config(config).0
    }

    /// Write a fake ticket file into `<tickets>/queue/` and create the
    /// `in-progress` and `completed` sibling dirs (Queue::{claim,complete,return}
    /// use fs::rename which requires the target dir to exist). Returns the id.
    fn seed_queue_ticket(tickets_path: &std::path::Path, id: &str) -> String {
        let queue_dir = tickets_path.join("queue");
        std::fs::create_dir_all(&queue_dir).unwrap();
        std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
        let filename = format!("20260516-1200-FEAT-demo-test-{id}.md");
        let body = format!(
            "---\nid: {id}\npriority: P2-medium\nstatus: queued\n---\n\n# Task: Test {id}\n"
        );
        std::fs::write(queue_dir.join(&filename), body).unwrap();
        id.to_string()
    }

    #[tokio::test]
    async fn test_list_tickets_empty_queue() {
        let state = test_state();
        let result = list_tickets(json!({}), &state).await.unwrap();
        assert_eq!(result["count"], 0);
    }

    #[tokio::test]
    async fn test_list_tickets_unknown_status_errors() {
        let state = test_state();
        let err = list_tickets(json!({ "status": "bogus" }), &state)
            .await
            .unwrap_err();
        assert!(err.contains("Unknown ticket status"));
    }

    #[tokio::test]
    async fn test_claim_ticket_moves_file() {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        let (state, tickets_path) = test_state_with_config(config);
        let id = seed_queue_ticket(&tickets_path, "FEAT-1234");

        let result = claim_ticket(json!({ "id": id }), &state).await.unwrap();
        assert_eq!(result["moved_to"], "in-progress");

        let in_progress = tickets_path.join("in-progress");
        let entries: Vec<_> = std::fs::read_dir(&in_progress).unwrap().collect();
        assert_eq!(entries.len(), 1);

        let queue_dir = tickets_path.join("queue");
        let entries: Vec<_> = std::fs::read_dir(&queue_dir).unwrap().collect();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_complete_then_return_ticket() {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        let (state, tickets_path) = test_state_with_config(config);
        let id = seed_queue_ticket(&tickets_path, "FEAT-5678");

        claim_ticket(json!({ "id": &id }), &state).await.unwrap();
        let res = complete_ticket(json!({ "id": &id }), &state).await.unwrap();
        assert_eq!(res["moved_to"], "completed");
        let completed = tickets_path.join("completed");
        assert_eq!(std::fs::read_dir(&completed).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn test_return_to_queue_moves_back() {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        let (state, tickets_path) = test_state_with_config(config);
        let id = seed_queue_ticket(&tickets_path, "FEAT-9999");

        claim_ticket(json!({ "id": &id }), &state).await.unwrap();
        let res = return_to_queue(json!({ "id": &id }), &state).await.unwrap();
        assert_eq!(res["moved_to"], "queue");
        let queue_dir = tickets_path.join("queue");
        assert_eq!(std::fs::read_dir(&queue_dir).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn test_create_ticket_writes_file_to_queue() {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        let (state, tickets_path) = test_state_with_config(config);

        let result = create_ticket(
            json!({
                "template": "FEAT",
                "values": { "project": "demo", "summary": "from mcp test" }
            }),
            &state,
        )
        .await
        .unwrap();
        let filename = result["filename"].as_str().unwrap();
        assert!(
            filename.contains("FEAT-demo"),
            "filename should contain FEAT-demo, got: {filename}"
        );

        let queue_dir = tickets_path.join("queue");
        let entries: Vec<_> = std::fs::read_dir(&queue_dir).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_create_ticket_unknown_template_errors() {
        let mut config = Config::default();
        config.mcp.expose_ticket_write_tools = true;
        let (state, _) = test_state_with_config(config);

        let err = create_ticket(json!({ "template": "nope" }), &state)
            .await
            .unwrap_err();
        assert!(err.contains("Unknown template type"));
    }

    #[tokio::test]
    async fn test_claim_ticket_gate_blocks_when_disabled() {
        // Default config has expose_ticket_write_tools = false.
        let config = Config::default();
        let (state, tickets_path) = test_state_with_config(config);
        seed_queue_ticket(&tickets_path, "FEAT-0001");

        let err = crate::mcp::tools::execute_tool(
            "operator_claim_ticket",
            json!({ "id": "FEAT-0001" }),
            &state,
        )
        .await
        .unwrap_err();
        assert!(err.contains("disabled in config"));
    }
}
