//! Integration tests for ticket launching via zellij
//!
//! These tests verify the full launch flow using the zellij wrapper:
//! - Zellij tab creation
//! - Prompt file generation
//! - LLM command structure
//! - Tab lifecycle
//!
//! ## Environment Variables
//!
//! - `OPERATOR_LAUNCH_TEST_ENABLED=true` - Required to run these tests
//! - `OPERATOR_ZELLIJ_TEST_ENABLED=true` - Required for zellij-specific tests
//! - `ZELLIJ` - Must be set (running inside a zellij session)
//!
//! ## Running Tests
//!
//! ```bash
//! # Run inside a zellij session:
//! OPERATOR_LAUNCH_TEST_ENABLED=true OPERATOR_ZELLIJ_TEST_ENABLED=true \
//!   cargo test --test launch_integration_zellij -- --nocapture --test-threads=1
//! ```
//!
//! ## Notes
//!
//! - Tests must run inside a zellij session (ZELLIJ env var set)
//! - Tests are sequential (`--test-threads=1`) to avoid tab conflicts
//! - Tabs are cleaned up automatically via Drop

#![allow(dead_code)]

mod launch_common;

use std::env;
use std::process::Command;
use std::time::Duration;

use launch_common::{LaunchTestContext, WrapperTestMode};

// ─── Zellij-Specific Helpers ────────────────────────────────────────────────

/// Check if zellij binary is available
fn zellij_available() -> bool {
    Command::new("zellij")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if we're running inside a zellij session
fn in_zellij() -> bool {
    env::var("ZELLIJ").is_ok()
}

/// Check if zellij-specific tests are enabled
fn zellij_tests_enabled() -> bool {
    env::var("OPERATOR_ZELLIJ_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Macro to skip tests if not configured for zellij
macro_rules! skip_if_not_configured {
    () => {
        if !launch_common::launch_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_LAUNCH_TEST_ENABLED not set to true");
            return;
        }
        if !zellij_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_ZELLIJ_TEST_ENABLED not set to true");
            return;
        }
        if !zellij_available() {
            eprintln!("Skipping test: zellij not available");
            return;
        }
        if !in_zellij() {
            eprintln!(
                "Skipping test: not running inside a zellij session (ZELLIJ env var not set)"
            );
            return;
        }
    };
}

/// List zellij tabs with the operator prefix
fn list_operator_tabs() -> Vec<String> {
    let output = Command::new("zellij")
        .args(["action", "query-tab-names"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|s| s.starts_with("op-"))
            .map(std::string::ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// Close a zellij tab by name
fn close_zellij_tab(name: &str) {
    // Focus the tab first, then close it
    let _ = Command::new("zellij")
        .args(["action", "go-to-tab-name", name])
        .output();
    let _ = Command::new("zellij")
        .args(["action", "close-tab"])
        .output();
}

/// Clean up all operator tabs
fn cleanup_operator_tabs() {
    for tab in list_operator_tabs() {
        close_zellij_tab(&tab);
    }
}

/// Zellij test wrapper providing tab management on top of `LaunchTestContext`
struct ZellijTestContext {
    ctx: LaunchTestContext,
}

impl ZellijTestContext {
    fn new(test_name: &str) -> Self {
        Self {
            ctx: LaunchTestContext::new(test_name, WrapperTestMode::Zellij),
        }
    }
}

impl Drop for ZellijTestContext {
    fn drop(&mut self) {
        cleanup_operator_tabs();
    }
}

// ─── Test Cases ─────────────────────────────────────────────────────────────

#[test]
fn test_launch_creates_zellij_tab() {
    skip_if_not_configured!();

    let zctx = ZellijTestContext::new("creates_tab");

    let ticket_content = r"---
id: TASK-Z01
priority: P2-medium
status: queued
---

# Task: Test zellij tab creation

## Context
This is a test task to verify zellij tab creation.
";
    zctx.ctx.create_ticket("TASK", "TASK-Z01", ticket_content);

    // Run launch
    let output = zctx.ctx.run_launch(&[]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stdout: {stdout}");
    eprintln!("stderr: {stderr}");

    // Wait for tab to be created
    std::thread::sleep(Duration::from_secs(2));

    // Verify tab exists
    let tabs = list_operator_tabs();
    assert!(
        tabs.iter().any(|t| t.contains("TASK-Z01")),
        "Zellij tab should be created. Tabs: {tabs:?}"
    );

    // Verify ticket was moved to in-progress
    assert!(
        zctx.ctx.ticket_in_progress(),
        "Ticket should be in in-progress"
    );
}

#[test]
fn test_zellij_prompt_file_contains_ticket_content() {
    skip_if_not_configured!();

    let zctx = ZellijTestContext::new("prompt_content");

    let ticket_content = r"---
id: TASK-Z02
priority: P2-medium
status: queued
---

# Task: Unique marker for zellij testing

## Context
This content should appear in the prompt file: ZELLIJ_MARKER_99999
";
    zctx.ctx.create_ticket("TASK", "TASK-Z02", ticket_content);

    zctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    let invocations = zctx.ctx.get_invocations();
    assert!(
        !invocations.is_empty(),
        "Should have at least one invocation"
    );

    let inv = &invocations[0];
    assert!(
        inv.prompt_content.contains("ZELLIJ_MARKER_99999")
            || inv.prompt_content.contains("TASK-Z02"),
        "Prompt should contain ticket content. Got: {}",
        inv.prompt_content
    );
}

#[test]
fn test_zellij_llm_command_has_session_id_and_model() {
    skip_if_not_configured!();

    let zctx = ZellijTestContext::new("command_structure");

    let ticket_content = r"---
id: TASK-Z03
priority: P2-medium
status: queued
---

# Task: Test command structure in zellij
";
    zctx.ctx.create_ticket("TASK", "TASK-Z03", ticket_content);

    zctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    let invocations = zctx.ctx.get_invocations();
    assert!(!invocations.is_empty(), "Should have invocation");

    let inv = &invocations[0];

    // Verify session ID is a valid UUID
    assert!(
        !inv.session_id.is_empty(),
        "Should have session ID. Args: {:?}",
        inv.args
    );
    assert!(
        uuid::Uuid::parse_str(&inv.session_id).is_ok(),
        "Session ID should be valid UUID: {}",
        inv.session_id
    );

    // Verify model is set
    assert_eq!(inv.model, "sonnet", "Should use configured model");

    // Verify prompt file path is set
    assert!(
        !inv.prompt_file.is_empty(),
        "Should have prompt file path. Args: {:?}",
        inv.args
    );

    // Verify working directory is project path
    assert!(
        inv.cwd.contains("testproject"),
        "Should be in project directory. Got: {}",
        inv.cwd
    );
}

#[test]
fn test_zellij_command_file_is_created() {
    skip_if_not_configured!();

    let zctx = ZellijTestContext::new("command_file");

    let ticket_content = r"---
id: TASK-Z04
priority: P2-medium
status: queued
---

# Task: Test command file creation in zellij
";
    zctx.ctx.create_ticket("TASK", "TASK-Z04", ticket_content);

    zctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    let command_files = zctx.ctx.read_command_files();
    assert!(
        !command_files.is_empty(),
        "Should have at least one command file"
    );

    let (_path, content) = &command_files[0];

    // Should have shebang
    assert!(
        content.starts_with("#!/bin/bash"),
        "Command file should start with shebang. Got: {}",
        &content[..std::cmp::min(50, content.len())]
    );

    // Should have cd command
    assert!(
        content.contains("cd "),
        "Command file should contain cd command"
    );

    // Should have exec command with LLM tool
    assert!(
        content.contains("exec "),
        "Command file should contain exec command"
    );
}

#[test]
fn test_zellij_prompt_file_is_written_to_disk() {
    skip_if_not_configured!();

    let zctx = ZellijTestContext::new("prompt_file");

    let ticket_content = r"---
id: TASK-Z05
priority: P2-medium
status: queued
---

# Task: Test prompt file in zellij

## Context
ZELLIJ_PROMPT_MARKER_54321
";
    zctx.ctx.create_ticket("TASK", "TASK-Z05", ticket_content);

    zctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    let prompts = zctx.ctx.read_prompt_files();
    assert!(!prompts.is_empty(), "Should have at least one prompt file");

    let has_reference = prompts
        .iter()
        .any(|p| p.contains("TASK-Z05") || p.contains("task_z05"));
    assert!(
        has_reference,
        "Prompt file should reference ticket. Prompts: {prompts:?}"
    );
}

#[test]
fn test_zellij_ticket_moved_to_in_progress() {
    skip_if_not_configured!();

    let zctx = ZellijTestContext::new("in_progress");

    let ticket_content = r"---
id: TASK-Z06
priority: P2-medium
status: queued
---

# Task: Test ticket state transition in zellij
";
    zctx.ctx.create_ticket("TASK", "TASK-Z06", ticket_content);

    zctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    assert!(
        zctx.ctx.ticket_in_progress(),
        "Ticket should be moved to in-progress"
    );
}
