//! Integration tests for ticket launching via cmux (mock-based)
//!
//! Since cmux is a macOS-only desktop application that cannot run headlessly
//! in CI, these tests use `MockCmuxClient` to verify the full config-to-launch
//! pipeline at the library level.
//!
//! The tests verify:
//! - Config parsing with `wrapper = "cmux"`
//! - Launcher wiring with `Launcher::with_cmux_client()`
//! - Prompt file generation
//! - Command file creation
//! - MockCmuxClient workspace creation
//! - Ticket state transitions
//!
//! ## Environment Variables
//!
//! - `OPERATOR_CMUX_TEST_ENABLED=true` - Required to run these tests
//!
//! ## Running Tests
//!
//! ```bash
//! OPERATOR_CMUX_TEST_ENABLED=true cargo test --test launch_integration_cmux -- --nocapture --test-threads=1
//! ```

#![allow(dead_code)]

mod launch_common;

use std::env;
use std::sync::Arc;

use launch_common::{LaunchTestContext, WrapperTestMode};
use operator::agents::{LaunchOptions, Launcher, MockCmuxClient};
use operator::config::Config;
use operator::queue::Ticket;

// ─── Configuration ──────────────────────────────────────────────────────────

fn cmux_tests_enabled() -> bool {
    env::var("OPERATOR_CMUX_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

macro_rules! skip_if_not_configured {
    () => {
        if !cmux_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_CMUX_TEST_ENABLED not set to true");
            return;
        }
    };
}

/// Create a LaunchTestContext with cmux config and load the Config from it
fn setup_cmux_test(test_name: &str) -> (LaunchTestContext, Config) {
    let ctx = LaunchTestContext::new(test_name, WrapperTestMode::Cmux);
    let config =
        Config::load(Some(ctx.config_path.to_str().unwrap())).expect("Failed to load test config");
    (ctx, config)
}

// ─── Test Cases ─────────────────────────────────────────────────────────────

#[test]
fn test_cmux_config_parsing() {
    skip_if_not_configured!();

    let (_ctx, config) = setup_cmux_test("config_parsing");

    assert_eq!(
        config.sessions.wrapper,
        operator::config::SessionWrapperType::Cmux,
        "Wrapper type should be cmux"
    );
    assert!(
        !config.sessions.cmux.require_in_cmux,
        "require_in_cmux should be false in test config"
    );
}

#[test]
fn test_cmux_launcher_with_mock_client() {
    skip_if_not_configured!();

    let (_ctx, config) = setup_cmux_test("launcher_mock");

    let mock = Arc::new(MockCmuxClient::new());
    let launcher = Launcher::with_cmux_client(&config, mock.clone());

    assert!(
        launcher.is_ok(),
        "Should create launcher with mock cmux client"
    );
}

#[tokio::test]
async fn test_cmux_launch_creates_workspace() {
    skip_if_not_configured!();

    let (ctx, config) = setup_cmux_test("creates_workspace");

    let ticket_content = r#"---
id: TASK-C01
priority: P2-medium
status: queued
---

# Task: Test cmux workspace creation

## Context
This is a test task to verify cmux workspace creation via mock client.
"#;
    ctx.create_ticket("TASK", "TASK-C01", ticket_content);

    let mock = Arc::new(MockCmuxClient::new());
    let launcher =
        Launcher::with_cmux_client(&config, mock.clone()).expect("Failed to create launcher");

    // Load the ticket from file
    let queue_dir = ctx.tickets_path.join("queue");
    let ticket_file = std::fs::read_dir(&queue_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .expect("Should have a ticket file");

    let ticket = Ticket::from_file(&ticket_file.path()).expect("Should parse ticket");

    let result = launcher
        .launch_with_options(&ticket, LaunchOptions::default())
        .await;

    // The launch should succeed (mock client accepts all operations)
    assert!(
        result.is_ok(),
        "Launch should succeed with mock client. Error: {:?}",
        result.err()
    );

    // launch_with_options returns the agent_id (a UUID), not the session name
    let agent_id = result.unwrap();
    assert!(
        !agent_id.is_empty(),
        "Should return a non-empty agent ID"
    );

    // Verify ticket was moved to in-progress
    assert!(
        ctx.ticket_in_progress(),
        "Ticket should be moved to in-progress"
    );
}

#[tokio::test]
async fn test_cmux_prompt_file_written() {
    skip_if_not_configured!();

    let (ctx, config) = setup_cmux_test("prompt_file");

    let ticket_content = r#"---
id: TASK-C02
priority: P2-medium
status: queued
---

# Task: Test cmux prompt file

## Context
CMUX_PROMPT_MARKER_77777
"#;
    ctx.create_ticket("TASK", "TASK-C02", ticket_content);

    let mock = Arc::new(MockCmuxClient::new());
    let launcher =
        Launcher::with_cmux_client(&config, mock.clone()).expect("Failed to create launcher");

    let queue_dir = ctx.tickets_path.join("queue");
    let ticket_file = std::fs::read_dir(&queue_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .expect("Should have a ticket file");
    let ticket = Ticket::from_file(&ticket_file.path()).expect("Should parse ticket");

    let _ = launcher
        .launch_with_options(&ticket, LaunchOptions::default())
        .await;

    // Check prompt files
    let prompts = ctx.read_prompt_files();
    assert!(!prompts.is_empty(), "Should have at least one prompt file");

    let has_reference = prompts
        .iter()
        .any(|p| p.contains("TASK-C02") || p.contains("task_c02"));
    assert!(
        has_reference,
        "Prompt file should reference ticket. Prompts: {:?}",
        prompts
    );
}

#[tokio::test]
async fn test_cmux_command_file_created() {
    skip_if_not_configured!();

    let (ctx, config) = setup_cmux_test("command_file");

    let ticket_content = r#"---
id: TASK-C03
priority: P2-medium
status: queued
---

# Task: Test cmux command file creation
"#;
    ctx.create_ticket("TASK", "TASK-C03", ticket_content);

    let mock = Arc::new(MockCmuxClient::new());
    let launcher =
        Launcher::with_cmux_client(&config, mock.clone()).expect("Failed to create launcher");

    let queue_dir = ctx.tickets_path.join("queue");
    let ticket_file = std::fs::read_dir(&queue_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .expect("Should have a ticket file");
    let ticket = Ticket::from_file(&ticket_file.path()).expect("Should parse ticket");

    let _ = launcher
        .launch_with_options(&ticket, LaunchOptions::default())
        .await;

    let command_files = ctx.read_command_files();
    assert!(
        !command_files.is_empty(),
        "Should have at least one command file"
    );

    let (_path, content) = &command_files[0];

    assert!(
        content.starts_with("#!/bin/bash"),
        "Command file should start with shebang"
    );
    assert!(
        content.contains("cd "),
        "Command file should contain cd command"
    );
    assert!(
        content.contains("exec "),
        "Command file should contain exec command"
    );
}

#[tokio::test]
async fn test_cmux_mock_receives_send_text() {
    skip_if_not_configured!();

    let (ctx, config) = setup_cmux_test("send_text");

    let ticket_content = r#"---
id: TASK-C04
priority: P2-medium
status: queued
---

# Task: Test cmux send_text via mock
"#;
    ctx.create_ticket("TASK", "TASK-C04", ticket_content);

    let mock = Arc::new(MockCmuxClient::new());
    let launcher =
        Launcher::with_cmux_client(&config, mock.clone()).expect("Failed to create launcher");

    let queue_dir = ctx.tickets_path.join("queue");
    let ticket_file = std::fs::read_dir(&queue_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .expect("Should have a ticket file");
    let ticket = Ticket::from_file(&ticket_file.path()).expect("Should parse ticket");

    let result = launcher
        .launch_with_options(&ticket, LaunchOptions::default())
        .await;

    assert!(result.is_ok(), "Launch should succeed");

    // Verify the mock received send_text calls (the launch command)
    let sent = mock.sent_texts();
    assert!(
        !sent.is_empty(),
        "MockCmuxClient should have received send_text calls"
    );

    // The sent text should contain the command script path
    let has_command = sent.iter().any(|(_ws, text)| text.contains("bash "));
    assert!(
        has_command,
        "Sent text should contain bash command. Got: {:?}",
        sent
    );
}
