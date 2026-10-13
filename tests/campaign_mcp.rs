//! The MCP planning flow an outside agent drives: preview, bulk create, then campaign tools.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use operator::campaigns::{CampaignStore, NewCampaign};
use operator::config::Config;
use operator::mcp::tools::execute_tool;
use operator::rest::dto::auth::Scope;
use operator::rest::state::ApiState;

struct Workspace {
    state: ApiState,
    config: Config,
    _dir: tempfile::TempDir,
}

impl Workspace {
    fn new(write_tools: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let tickets = dir.path().join("tickets");
        for sub in ["queue", "in-progress", "completed"] {
            fs::create_dir_all(tickets.join(sub)).unwrap();
        }
        let mut config = Config::default();
        config.paths.tickets = tickets.to_string_lossy().into_owned();
        config.paths.state = dir.path().join("state").to_string_lossy().into_owned();
        config.paths.projects = dir.path().to_string_lossy().into_owned();
        config.profile.id = uuid::Uuid::new_v4();
        config.mcp.expose_ticket_write_tools = write_tools;
        Self {
            state: ApiState::new(config.clone(), tickets),
            config,
            _dir: dir,
        }
    }

    async fn call(&self, tool: &str, args: Value) -> Result<Value, String> {
        execute_tool(tool, args, &self.state, &Scope::ALL).await
    }

    fn queued(&self) -> Vec<String> {
        let mut names: Vec<String> =
            fs::read_dir(Path::new(&self.config.paths.tickets).join("queue"))
                .unwrap()
                .filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
        names.sort();
        names
    }
}

fn diamond(campaign: Option<&str>) -> Value {
    json!({
        "campaign": campaign,
        "tickets": [
            {"ref": "schema", "template": "FEAT", "project": "api", "summary": "Pricing schema"},
            {"ref": "api", "template": "FEAT", "project": "api", "summary": "Pricing API", "depends_on": ["schema"]},
            {"ref": "ui", "template": "FIX", "project": "web", "summary": "Pricing page", "depends_on": ["schema"]},
            {"ref": "docs", "template": "TASK", "project": "web", "summary": "Pricing docs", "depends_on": ["api", "ui"]},
            {"ref": "rollout", "template": "TASK", "project": "api", "summary": "Roll out", "depends_on": ["docs", "jira:PRICE-1"]}
        ]
    })
}

#[tokio::test]
async fn preview_allocates_ids_and_waves_without_writing() {
    let ws = Workspace::new(false);
    let preview = ws
        .call("operator_preview_ticket_batch", diamond(None))
        .await
        .unwrap();
    assert_eq!(preview["valid"], true, "{preview}");
    let ids: Vec<&str> = preview["tickets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "FEAT-0001",
            "FEAT-0002",
            "FIX-0001",
            "TASK-0001",
            "TASK-0002"
        ]
    );
    assert_eq!(
        preview["waves"],
        json!([
            ["FEAT-0001"],
            ["FIX-0001", "FEAT-0002"],
            ["TASK-0001"],
            ["TASK-0002"]
        ])
    );
    assert_eq!(
        preview["tickets"][4]["depends_on"],
        json!(["TASK-0001", "jira:PRICE-1"])
    );
    assert!(ws.queued().is_empty());
}

#[tokio::test]
async fn bulk_create_needs_write_tools() {
    let ws = Workspace::new(false);
    let err = ws
        .call("operator_create_tickets_bulk", diamond(None))
        .await
        .unwrap_err();
    assert!(err.contains("expose_ticket_write_tools"), "{err}");
    assert!(ws.queued().is_empty());
}

#[tokio::test]
async fn bulk_create_is_all_or_nothing() {
    let ws = Workspace::new(true);
    let mut batch = diamond(None);
    batch["tickets"]
        .as_array_mut()
        .unwrap()
        .push(json!({"ref": "bad", "template": "NOPE"}));
    let err = ws
        .call("operator_create_tickets_bulk", batch)
        .await
        .unwrap_err();
    assert!(err.starts_with("validation_error"), "{err}");
    assert!(err.contains("unknown template"), "{err}");
    assert!(ws.queued().is_empty(), "nothing is written");
}

#[tokio::test]
async fn bulk_create_inside_a_campaign_wires_dependencies() {
    let ws = Workspace::new(true);
    CampaignStore::new(&ws.config)
        .create(NewCampaign {
            title: "Pricing".into(),
            ..NewCampaign::default()
        })
        .unwrap();
    let created = ws
        .call("operator_create_tickets_bulk", diamond(Some("CAMP-0001")))
        .await
        .unwrap();
    assert_eq!(created["tickets"].as_array().unwrap().len(), 5);
    assert_eq!(ws.queued().len(), 5);

    let queue = operator::queue::Queue::new(&ws.config).unwrap();
    let docs = queue.find_ticket("TASK-0001").unwrap().unwrap();
    assert_eq!(docs.campaign.as_deref(), Some("CAMP-0001"));
    assert_eq!(docs.depends_on, ["FEAT-0002", "FIX-0001"]);
    assert_eq!(docs.project, "web");

    let again = ws
        .call("operator_preview_ticket_batch", diamond(None))
        .await
        .unwrap();
    assert_eq!(
        again["tickets"][0]["id"], "FEAT-0003",
        "ids continue past the batch"
    );
}

#[tokio::test]
async fn campaign_tools_are_premium_and_launch_touches_nothing() {
    let ws = Workspace::new(true);
    CampaignStore::new(&ws.config)
        .create(NewCampaign {
            title: "Pricing".into(),
            ..NewCampaign::default()
        })
        .unwrap();
    ws.call("operator_create_tickets_bulk", diamond(Some("CAMP-0001")))
        .await
        .unwrap();

    for (tool, args) in [
        ("operator_create_campaign", json!({"title": "Another"})),
        (
            "operator_assign_ticket_campaign",
            json!({"id": "FEAT-0001", "campaign": null}),
        ),
        ("operator_launch_campaign", json!({"id": "CAMP-0001"})),
        ("operator_campaign_status", json!({"id": "CAMP-0001"})),
    ] {
        let err = ws.call(tool, args).await.unwrap_err();
        assert!(err.starts_with("not_entitled"), "{tool}: {err}");
    }
    assert_eq!(ws.queued().len(), 5, "no ticket was claimed");
    let state = operator::state::State::load(&ws.config).unwrap();
    assert!(state.agents.is_empty());
    assert!(state.campaign_runs.is_empty());
}

#[test]
fn campaign_tool_schemas_are_objects() {
    for tool in operator::mcp::tools::all_tool_definitions()
        .into_iter()
        .filter(|t| {
            t.name.contains("campaign") || t.name.contains("batch") || t.name.contains("bulk")
        })
    {
        assert_eq!(tool.input_schema["type"], "object", "{}", tool.name);
    }
}
