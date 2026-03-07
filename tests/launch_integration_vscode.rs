//! Integration tests for the VS Code wrapper launch path
//!
//! These tests verify the `prepare_launch()` path used by the VS Code extension,
//! which generates launch commands without executing them in a terminal multiplexer.
//!
//! The tests verify:
//! - Config parsing with `wrapper = "vscode"`
//! - `PreparedLaunch` response contains correct terminal name, command, session ID
//! - Prompt file generation
//! - Command file creation
//! - Ticket state transitions
//!
//! No VS Code instance is required — these test the Rust API path only.
//!
//! ## Environment Variables
//!
//! - `OPERATOR_VSCODE_TEST_ENABLED=true` - Required to run these tests
//!
//! ## Running Tests
//!
//! ```bash
//! OPERATOR_VSCODE_TEST_ENABLED=true cargo test --test launch_integration_vscode -- --nocapture --test-threads=1
//! ```

#![allow(dead_code)]

mod launch_common;

use std::env;

use launch_common::LaunchTestContext;
use operator::agents::{LaunchOptions, Launcher, PreparedLaunch};
use operator::config::Config;
use operator::queue::Ticket;

// ─── Configuration ──────────────────────────────────────────────────────────

fn vscode_tests_enabled() -> bool {
    env::var("OPERATOR_VSCODE_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

macro_rules! skip_if_not_configured {
    () => {
        if !vscode_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_VSCODE_TEST_ENABLED not set to true");
            return;
        }
    };
}

/// Create a LaunchTestContext with vscode config
fn setup_vscode_test(test_name: &str) -> (LaunchTestContext, Config) {
    let ctx = LaunchTestContext::new_with_sessions_toml(
        test_name,
        r#"
[sessions]
wrapper = "vscode"
"#,
    );
    let config =
        Config::load(Some(ctx.config_path.to_str().unwrap())).expect("Failed to load test config");
    (ctx, config)
}

/// Load ticket from the queue directory
fn load_ticket_from_queue(ctx: &LaunchTestContext) -> Ticket {
    let queue_dir = ctx.tickets_path.join("queue");
    let ticket_file = std::fs::read_dir(&queue_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .expect("Should have a ticket file");
    Ticket::from_file(&ticket_file.path()).expect("Should parse ticket")
}

// ─── Test Cases ─────────────────────────────────────────────────────────────

#[test]
fn test_vscode_config_parsing() {
    skip_if_not_configured!();

    let (_ctx, config) = setup_vscode_test("config_parsing");

    assert_eq!(
        config.sessions.wrapper,
        operator::config::SessionWrapperType::Vscode,
        "Wrapper type should be vscode"
    );
}

#[tokio::test]
async fn test_vscode_prepare_launch_returns_prepared_launch() {
    skip_if_not_configured!();

    let (ctx, config) = setup_vscode_test("prepare_launch");

    let ticket_content = r#"---
id: TASK-V01
priority: P2-medium
status: queued
---

# Task: Test VS Code prepare_launch

## Context
This is a test task to verify the prepare_launch path for VS Code.
"#;
    ctx.create_ticket("TASK", "TASK-V01", ticket_content);

    let launcher = Launcher::new(&config).expect("Failed to create launcher");
    let ticket = load_ticket_from_queue(&ctx);

    let result = launcher
        .prepare_launch(&ticket, LaunchOptions::default())
        .await;

    assert!(
        result.is_ok(),
        "prepare_launch should succeed. Error: {:?}",
        result.err()
    );

    let prepared: PreparedLaunch = result.unwrap();

    // Verify terminal name contains ticket ID
    assert!(
        prepared.terminal_name.contains("TASK-V01"),
        "Terminal name should contain ticket ID. Got: {}",
        prepared.terminal_name
    );

    // Verify command is not empty
    assert!(
        !prepared.command.is_empty(),
        "Command should not be empty"
    );

    // Verify session ID is a valid UUID
    assert!(
        uuid::Uuid::parse_str(&prepared.session_id).is_ok(),
        "Session ID should be valid UUID: {}",
        prepared.session_id
    );

    // Verify agent ID is not empty
    assert!(
        !prepared.agent_id.is_empty(),
        "Agent ID should not be empty"
    );

    // Verify ticket ID matches
    assert_eq!(
        prepared.ticket_id, "TASK-V01",
        "Ticket ID should match"
    );

    // Verify working directory contains testproject
    assert!(
        prepared
            .working_directory
            .to_string_lossy()
            .contains("testproject"),
        "Working directory should contain project name. Got: {}",
        prepared.working_directory.display()
    );
}

#[tokio::test]
async fn test_vscode_prepare_launch_writes_prompt_file() {
    skip_if_not_configured!();

    let (ctx, config) = setup_vscode_test("prompt_file");

    let ticket_content = r#"---
id: TASK-V02
priority: P2-medium
status: queued
---

# Task: Test VS Code prompt file

## Context
VSCODE_PROMPT_MARKER_88888
"#;
    ctx.create_ticket("TASK", "TASK-V02", ticket_content);

    let launcher = Launcher::new(&config).expect("Failed to create launcher");
    let ticket = load_ticket_from_queue(&ctx);

    let _ = launcher
        .prepare_launch(&ticket, LaunchOptions::default())
        .await;

    let prompts = ctx.read_prompt_files();
    assert!(!prompts.is_empty(), "Should have at least one prompt file");

    let has_reference = prompts
        .iter()
        .any(|p| p.contains("TASK-V02") || p.contains("task_v02"));
    assert!(
        has_reference,
        "Prompt file should reference ticket. Prompts: {:?}",
        prompts
    );
}

#[tokio::test]
async fn test_vscode_prepare_launch_moves_ticket() {
    skip_if_not_configured!();

    let (ctx, config) = setup_vscode_test("ticket_state");

    let ticket_content = r#"---
id: TASK-V03
priority: P2-medium
status: queued
---

# Task: Test VS Code ticket state transition
"#;
    ctx.create_ticket("TASK", "TASK-V03", ticket_content);

    let launcher = Launcher::new(&config).expect("Failed to create launcher");
    let ticket = load_ticket_from_queue(&ctx);

    let _ = launcher
        .prepare_launch(&ticket, LaunchOptions::default())
        .await;

    assert!(
        ctx.ticket_in_progress(),
        "Ticket should be moved to in-progress"
    );
}

#[tokio::test]
async fn test_vscode_prepare_launch_command_contains_mock_llm() {
    skip_if_not_configured!();

    let (ctx, config) = setup_vscode_test("command_content");

    let ticket_content = r#"---
id: TASK-V04
priority: P2-medium
status: queued
---

# Task: Test VS Code command content
"#;
    ctx.create_ticket("TASK", "TASK-V04", ticket_content);

    let launcher = Launcher::new(&config).expect("Failed to create launcher");
    let ticket = load_ticket_from_queue(&ctx);

    let result = launcher
        .prepare_launch(&ticket, LaunchOptions::default())
        .await;

    assert!(result.is_ok(), "prepare_launch should succeed");
    let prepared = result.unwrap();

    // Command should reference the mock LLM (our test config points to it)
    assert!(
        prepared.command.contains("mock-claude"),
        "Command should reference the mock LLM. Got: {}",
        prepared.command
    );

    // Command should contain session ID
    assert!(
        prepared.command.contains(&prepared.session_id),
        "Command should contain session ID. Got: {}",
        prepared.command
    );
}
