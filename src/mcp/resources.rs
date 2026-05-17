//! MCP resources — exposes tickets as URI-addressable resources.
//!
//! Each ticket is reachable at `operator://tickets/{status}/{id}` where status
//! is one of `queue`, `in-progress`, `completed`. Resource reads return the
//! raw markdown body.

use serde_json::{json, Value};

use crate::queue::Queue;
use crate::rest::state::ApiState;

pub async fn list_resources(state: &ApiState) -> Result<Vec<Value>, String> {
    let config = (*state.config).clone();
    tokio::task::spawn_blocking(move || -> Result<Vec<Value>, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        let mut all = Vec::new();
        for (status, list) in [
            ("queue", queue.list_queue()),
            ("in-progress", queue.list_in_progress()),
            ("completed", queue.list_completed()),
        ] {
            for t in list.map_err(|e| e.to_string())? {
                all.push(json!({
                    "uri": format!("operator://tickets/{status}/{}", t.id),
                    "name": t.filename,
                    "mimeType": "text/markdown",
                    "description": t.summary,
                }));
            }
        }
        Ok(all)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn read_resource(uri: &str, state: &ApiState) -> Result<String, String> {
    let prefix = "operator://tickets/";
    let rest = uri
        .strip_prefix(prefix)
        .ok_or_else(|| format!("Unknown URI scheme: {uri}"))?;
    let (status, id) = rest
        .split_once('/')
        .ok_or_else(|| format!("Malformed URI: {uri}"))?;

    let config = (*state.config).clone();
    let status = status.to_string();
    let id = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        let list = match status.as_str() {
            "queue" => queue.list_queue(),
            "in-progress" => queue.list_in_progress(),
            "completed" => queue.list_completed(),
            other => return Err(format!("Unknown status: {other}")),
        }
        .map_err(|e| e.to_string())?;
        let ticket = list
            .into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Ticket {id} not found"))?;
        std::fs::read_to_string(&ticket.filepath).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_state() -> ApiState {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.keep();
        let mut config = Config::default();
        config.paths.tickets = path.to_string_lossy().into_owned();
        ApiState::new(config, path)
    }

    #[tokio::test]
    async fn test_list_resources_empty() {
        let state = test_state();
        let resources = list_resources(&state).await.unwrap();
        assert!(resources.is_empty());
    }

    #[tokio::test]
    async fn test_read_resource_unknown_scheme() {
        let state = test_state();
        let err = read_resource("file:///tmp/x", &state).await.unwrap_err();
        assert!(err.contains("Unknown URI scheme"));
    }

    #[tokio::test]
    async fn test_read_resource_malformed() {
        let state = test_state();
        let err = read_resource("operator://tickets/queue", &state)
            .await
            .unwrap_err();
        assert!(err.contains("Malformed URI"));
    }
}
