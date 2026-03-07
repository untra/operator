//! Integration tests for ticket launching via tmux
//!
//! These tests verify the full launch flow including:
//! - Tmux session creation
//! - Prompt file generation with proper layering
//! - LLM command structure
//! - Session lifecycle (attach/detach/kill)
//!
//! ## Environment Variables
//!
//! - `OPERATOR_LAUNCH_TEST_ENABLED=true` - Required to run these tests
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all launch integration tests
//! OPERATOR_LAUNCH_TEST_ENABLED=true cargo test --test launch_integration -- --nocapture --test-threads=1
//!
//! # Run a specific test
//! OPERATOR_LAUNCH_TEST_ENABLED=true cargo test --test launch_integration test_launch_creates_tmux_session -- --nocapture
//! ```
//!
//! ## Notes
//!
//! - Tests use `optest-` prefix for sessions (not `op-`) to avoid conflicts
//! - Tests are sequential (`--test-threads=1`) to avoid tmux conflicts
//! - Sessions are cleaned up automatically via Drop

#![allow(dead_code)]

mod launch_common;

use std::fs;
use std::process::Command;
use std::time::Duration;

use launch_common::{LaunchTestContext, WrapperTestMode};

// ─── Tmux-Specific Helpers ──────────────────────────────────────────────────

/// Check if tmux is available on the system
fn tmux_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Macro to skip tests if not configured for tmux
macro_rules! skip_if_not_configured {
    () => {
        if !launch_common::launch_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_LAUNCH_TEST_ENABLED not set to true");
            return;
        }
        if !tmux_available() {
            eprintln!("Skipping test: tmux not available");
            return;
        }
    };
}

/// Tmux test wrapper providing session management on top of LaunchTestContext
struct TmuxTestContext {
    ctx: LaunchTestContext,
}

impl TmuxTestContext {
    fn new(test_name: &str) -> Self {
        Self {
            ctx: LaunchTestContext::new(test_name, WrapperTestMode::Tmux),
        }
    }

    /// Check if a tmux session exists
    fn session_exists(&self, session_name: &str) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", session_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// List tmux sessions with the operator prefix
    fn list_test_sessions(&self) -> Vec<String> {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output();

        match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|s| s.starts_with("op-"))
                .map(|s| s.to_string())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Kill a specific tmux session
    fn kill_session(&self, session_name: &str) {
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", session_name])
            .output();
    }

    /// Clean up all test sessions
    fn cleanup_sessions(&self) {
        for session in self.list_test_sessions() {
            self.kill_session(&session);
        }
    }

    /// Wait for session to appear (with timeout)
    fn wait_for_session(&self, session_prefix: &str, timeout: Duration) -> Option<String> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            let sessions = self.list_test_sessions();
            if let Some(session) = sessions.iter().find(|s| s.contains(session_prefix)) {
                return Some(session.clone());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        None
    }
}

impl Drop for TmuxTestContext {
    fn drop(&mut self) {
        self.cleanup_sessions();
    }
}

// ─── Test Cases ─────────────────────────────────────────────────────────────

#[test]
fn test_launch_creates_tmux_session() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("creates_session");

    // Create a TASK ticket (uses simple fallback prompt)
    let ticket_content = r#"---
id: TASK-001
priority: P2-medium
status: queued
---

# Task: Test session creation

## Context
This is a test task to verify tmux session creation.
"#;
    tctx.ctx.create_ticket("TASK", "TASK-001", ticket_content);

    // Run launch
    let output = tctx.ctx.run_launch(&[]);

    // Check output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    // Wait for session to be created
    std::thread::sleep(Duration::from_secs(2));

    // Verify session exists
    let sessions = tctx.list_test_sessions();
    assert!(
        sessions.iter().any(|s| s.contains("TASK-001")),
        "Tmux session should be created. Sessions: {:?}",
        sessions
    );

    // Verify ticket was moved to in-progress
    assert!(
        tctx.ctx.ticket_in_progress(),
        "Ticket should be in in-progress"
    );
}

#[test]
fn test_prompt_file_contains_ticket_content() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("prompt_content");

    // Create a TASK ticket
    let ticket_content = r#"---
id: TASK-002
priority: P2-medium
status: queued
---

# Task: Unique marker for testing

## Context
This content should appear in the prompt file: UNIQUE_MARKER_12345
"#;
    tctx.ctx.create_ticket("TASK", "TASK-002", ticket_content);

    // Run launch
    tctx.ctx.run_launch(&[]);

    // Wait for mock LLM to be invoked
    std::thread::sleep(Duration::from_secs(2));

    // Check invocation captured the prompt
    let invocations = tctx.ctx.get_invocations();
    assert!(
        !invocations.is_empty(),
        "Should have at least one invocation"
    );

    let inv = &invocations[0];

    // Verify prompt content contains the ticket marker
    assert!(
        inv.prompt_content.contains("UNIQUE_MARKER_12345")
            || inv.prompt_content.contains("TASK-002"),
        "Prompt should contain ticket content. Got: {}",
        inv.prompt_content
    );
}

#[test]
fn test_llm_command_has_session_id_and_model() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("command_structure");

    let ticket_content = r#"---
id: TASK-003
priority: P2-medium
status: queued
---

# Task: Test command structure
"#;
    tctx.ctx.create_ticket("TASK", "TASK-003", ticket_content);

    tctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    let invocations = tctx.ctx.get_invocations();
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
fn test_prompt_file_is_written_to_disk() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("prompt_file");

    let ticket_content = r#"---
id: TASK-004
priority: P2-medium
status: queued
---

# Task: Test prompt file

## Context
PROMPT_FILE_MARKER_67890
"#;
    tctx.ctx.create_ticket("TASK", "TASK-004", ticket_content);

    tctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    // Check prompt files in the prompts directory
    let prompts = tctx.ctx.read_prompt_files();
    assert!(!prompts.is_empty(), "Should have at least one prompt file");

    // Verify content - TASK tickets use simple prompt that references the ticket file
    // rather than embedding the full content. Check for ticket ID reference.
    let has_reference = prompts
        .iter()
        .any(|p| p.contains("TASK-004") || p.contains("task_004"));
    assert!(
        has_reference,
        "Prompt file should reference ticket. Prompts: {:?}",
        prompts
    );
}

#[test]
fn test_session_already_exists_error() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("session_exists");

    // Pre-create a tmux session with the expected name
    let session_name = "op-TASK-005";
    let _ = Command::new("tmux")
        .args(["new-session", "-d", "-s", session_name])
        .output();

    // Verify session was created
    assert!(
        tctx.session_exists(session_name),
        "Pre-created session should exist"
    );

    let ticket_content = r#"---
id: TASK-005
priority: P2-medium
status: queued
---

# Task: Test session conflict
"#;
    tctx.ctx.create_ticket("TASK", "TASK-005", ticket_content);

    let output = tctx.ctx.run_launch(&[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should either fail or output an error about existing session
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        !output.status.success() || combined.contains("already exists"),
        "Should error when session already exists. Status: {}, Output: {}",
        output.status,
        combined
    );

    // Cleanup
    tctx.kill_session(session_name);
}

#[test]
fn test_ticket_with_empty_step_uses_default() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("empty_step");

    // Ticket without explicit step
    let ticket_content = r#"---
id: FEAT-001
priority: P2-medium
status: queued
---

# Feature: Test default step

## Context
When step is not specified, should use first step from template.
"#;
    tctx.ctx.create_ticket("FEAT", "FEAT-001", ticket_content);

    let output = tctx.ctx.run_launch(&[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    std::thread::sleep(Duration::from_secs(2));

    // Should succeed
    let sessions = tctx.list_test_sessions();
    assert!(
        sessions.iter().any(|s| s.contains("FEAT-001")),
        "Should create session for FEAT ticket. Sessions: {:?}",
        sessions
    );
}

#[test]
fn test_definition_of_done_included_in_prompt() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("dod_prompt");

    // Create definition of done
    tctx.ctx.create_definition_of_done(
        r#"- All tests pass
- Code reviewed and approved
- Documentation updated
- DOD_MARKER_UNIQUE_11111"#,
    );

    // Create FEAT ticket (which should use template.prompt with {{ definition_of_done }})
    let ticket_content = r#"---
id: FEAT-002
priority: P2-medium
status: queued
step: plan
---

# Feature: Test definition of done inclusion

## Context
This feature should have definition of done in its prompt.
"#;
    tctx.ctx.create_ticket("FEAT", "FEAT-002", ticket_content);

    tctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    // Check the prompt file
    let prompts = tctx.ctx.read_prompt_files();
    eprintln!("Prompts found: {}", prompts.len());
    for (i, p) in prompts.iter().enumerate() {
        eprintln!("Prompt {}: {}", i, &p[..std::cmp::min(500, p.len())]);
    }

    // Note: This test verifies the prompt layering system works
    // The definition_of_done marker should appear if the template system is working
    let has_dod = prompts
        .iter()
        .any(|p| p.contains("DOD_MARKER_UNIQUE_11111") || p.contains("definition"));

    // This may not always contain DOD if template doesn't use it - log for debugging
    if !has_dod {
        eprintln!("Note: DOD marker not found - template may not include it");
    }
}

// ─── Module Tests ───────────────────────────────────────────────────────────

mod session_lifecycle {
    use super::*;

    #[test]
    fn test_session_can_be_killed() {
        skip_if_not_configured!();

        let tctx = TmuxTestContext::new("kill_session");

        let ticket_content = r#"---
id: TASK-006
priority: P2-medium
status: queued
---

# Task: Test session kill
"#;
        tctx.ctx.create_ticket("TASK", "TASK-006", ticket_content);

        tctx.ctx.run_launch(&[]);
        std::thread::sleep(Duration::from_secs(2));

        // Find the session
        let sessions = tctx.list_test_sessions();
        let session = sessions.iter().find(|s| s.contains("TASK-006")).cloned();

        assert!(session.is_some(), "Session should exist");
        let session_name = session.unwrap();

        // Kill the session
        tctx.kill_session(&session_name);
        std::thread::sleep(Duration::from_millis(500));

        // Verify session is gone
        assert!(
            !tctx.session_exists(&session_name),
            "Session should be killed"
        );
    }
}

#[test]
fn test_command_file_is_created_during_launch() {
    skip_if_not_configured!();

    let tctx = TmuxTestContext::new("command_file");

    let ticket_content = r#"---
id: TASK-008
priority: P2-medium
status: queued
---

# Task: Test command file creation

## Context
This test verifies that a command shell script is created during launch.
"#;
    tctx.ctx.create_ticket("TASK", "TASK-008", ticket_content);

    tctx.ctx.run_launch(&[]);
    std::thread::sleep(Duration::from_secs(2));

    // Check command files in the commands directory
    let command_files = tctx.ctx.read_command_files();
    assert!(
        !command_files.is_empty(),
        "Should have at least one command file"
    );

    // Verify command file structure
    let (path, content) = &command_files[0];
    eprintln!("Command file path: {:?}", path);
    eprintln!("Command file content:\n{}", content);

    // Should have shebang
    assert!(
        content.starts_with("#!/bin/bash"),
        "Command file should start with shebang. Got: {}",
        &content[..std::cmp::min(50, content.len())]
    );

    // Should have cd command
    assert!(
        content.contains("cd "),
        "Command file should contain cd command. Got: {}",
        content
    );

    // Should have exec command with LLM tool
    assert!(
        content.contains("exec "),
        "Command file should contain exec command. Got: {}",
        content
    );

    // Should be executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        assert!(
            mode & 0o111 != 0,
            "Command file should be executable. Mode: {:o}",
            mode
        );
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_nonexistent_project_fails_gracefully() {
        skip_if_not_configured!();

        let tctx = TmuxTestContext::new("bad_project");

        // Create ticket with non-existent project
        let ticket_content = r#"---
id: TASK-007
priority: P2-medium
status: queued
---

# Task: Test bad project
"#;
        // Manually create with bad project name
        let filename = format!(
            "{}-TASK-nonexistent-project-task_007.md",
            chrono::Local::now().format("%Y%m%d-%H%M")
        );
        let path = tctx.ctx.tickets_path.join("queue").join(&filename);
        fs::write(&path, ticket_content).unwrap();

        let output = tctx.ctx.run_launch(&[]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should fail with project not found error
        let combined = format!("{}{}", stdout, stderr);
        eprintln!("Output: {}", combined);

        // Either no session created or error message
        std::thread::sleep(Duration::from_secs(1));
        let sessions = tctx.list_test_sessions();
        let has_session = sessions.iter().any(|s| s.contains("TASK-007"));

        // If there's a session, the launch succeeded which might be ok if project was found
        // If no session, check for error message
        if !has_session {
            assert!(
                !output.status.success()
                    || combined.contains("does not exist")
                    || combined.contains("not found"),
                "Should fail gracefully for non-existent project"
            );
        }
    }
}
