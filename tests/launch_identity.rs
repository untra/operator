//! Every launch surface must tell the agent which configuration it belongs to.
//!
//! An agent that reports to the wrong configuration is harder to diagnose than one with no identity at all:
//! the callback succeeds, against the wrong queue. `opr8r` resolves the id from `--profile-id`, then `OPERATOR_PROFILE_ID`, then `api-session.json`
use std::path::PathBuf;

use operator::agents::{LaunchOptions, Launcher};
use operator::config::{Config, TargetDef};
use operator::queue::Ticket;

const TICKET: &str = r"---
id: TASK-700
priority: P2-medium
status: queued
---

# Task: Carry the configuration id

## Context
Launched locally; the assertion is about the environment, not the agent.
";

struct Workspace {
    config: Config,
    tickets: PathBuf,
    _directory: tempfile::TempDir,
}

fn workspace() -> Workspace {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let tickets = root.join("tickets");
    for sub in ["queue", "in-progress", "done", "operator"] {
        std::fs::create_dir_all(tickets.join(sub)).unwrap();
    }
    std::fs::create_dir_all(root.join("testproject")).unwrap();

    let mut config = Config::default();
    config.paths.tickets = tickets.to_string_lossy().into_owned();
    config.paths.state = tickets.join("operator").to_string_lossy().into_owned();
    config.paths.projects = root.to_string_lossy().into_owned();
    config.paths.worktrees = root.join("worktrees").to_string_lossy().into_owned();
    config.projects = vec!["testproject".to_string()];
    config.profile.id = uuid::Uuid::new_v4();
    // A stand-in tool: named for a real provider so the permission translator
    // resolves, but pointed at a harmless binary. `prepare_launch` builds a
    // command without executing it.
    config.llm_tools.detected = vec![operator::config::DetectedTool {
        name: "claude".to_string(),
        path: "/bin/cat".to_string(),
        version: "0.0.0-test".to_string(),
        min_version: None,
        version_ok: true,
        model_aliases: vec!["sonnet".to_string()],
        command_template: "cat {{prompt_file}}".to_string(),
        capabilities: operator::config::ToolCapabilities::default(),
        yolo_flags: Vec::new(),
        health_ok: true,
    }];
    config.llm_tools.detection_complete = true;
    config.llm_tools.default_tool = Some("claude".to_string());
    // Built from TOML so serde supplies every defaulted field; the struct gains
    // fields often enough that spelling them out here would rot.
    config.delegators = vec![toml::from_str(
        r#"
name = "test-agent"
llm_tool = "claude"
model = "sonnet"
"#,
    )
    .expect("delegator fixture parses")];

    let filename = format!(
        "{}-TASK-testproject-task_700.md",
        chrono::Local::now().format("%Y%m%d-%H%M")
    );
    std::fs::write(tickets.join("queue").join(&filename), TICKET).unwrap();

    Workspace {
        config,
        tickets,
        _directory: directory,
    }
}

fn queued_ticket(workspace: &Workspace) -> Ticket {
    let entry = std::fs::read_dir(workspace.tickets.join("queue"))
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| e.path().extension().is_some_and(|x| x == "md"))
        .expect("the queued ticket");
    Ticket::from_file(&entry.path()).expect("ticket parses")
}

/// `PreparedLaunch.env_vars` feeds the non-shell launch surfaces. It carried
/// the agent, ticket and API url but not the configuration id, so those agents
/// fell back to whichever configuration answered first.
#[tokio::test]
async fn a_prepared_launch_carries_the_configuration_id() {
    let workspace = workspace();
    let expected = workspace.config.profile.id.to_string();
    let launcher = Launcher::new(&workspace.config).expect("launcher");
    let ticket = queued_ticket(&workspace);

    let prepared = launcher
        .prepare_launch(
            &ticket,
            LaunchOptions {
                target: TargetDef::local(),
                ..LaunchOptions::default()
            },
        )
        .await
        .expect("a local launch needs no licence");

    assert_eq!(
        prepared.env_vars.get("OPERATOR_PROFILE_ID"),
        Some(&expected),
        "env_vars must name the configuration this launch belongs to"
    );
}
