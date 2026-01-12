//! Integration tests for opr8r CLI
//!
//! These tests verify opr8r can:
//! - Execute commands with proper exit codes
//! - Handle dry-run mode correctly
//! - Report version information
//!
//! ## Environment Variables
//!
//! - `OPERATOR_OPR8R_TEST_ENABLED=true` - Required to run these tests
//! - `OPR8R_PATH` - Optional path to opr8r binary (defaults to searching in target/release)

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn opr8r_tests_enabled() -> bool {
    env::var("OPERATOR_OPR8R_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

macro_rules! skip_if_not_configured {
    () => {
        if !opr8r_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_OPR8R_TEST_ENABLED not set");
            return;
        }
    };
}

fn get_opr8r_path() -> PathBuf {
    // Check environment variable first
    if let Ok(path) = env::var("OPR8R_PATH") {
        return PathBuf::from(path);
    }

    // Try target/release directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    #[cfg(windows)]
    let binary_name = "opr8r.exe";
    #[cfg(not(windows))]
    let binary_name = "opr8r";

    // Check opr8r subproject target
    let opr8r_path = PathBuf::from(&manifest_dir)
        .join("opr8r")
        .join("target")
        .join("release")
        .join(binary_name);
    if opr8r_path.exists() {
        return opr8r_path;
    }

    // Fallback to main target
    PathBuf::from(&manifest_dir)
        .join("target")
        .join("release")
        .join(binary_name)
}

#[test]
fn test_opr8r_version() {
    skip_if_not_configured!();

    let opr8r_path = get_opr8r_path();
    eprintln!("Testing opr8r at: {:?}", opr8r_path);

    let output = Command::new(&opr8r_path)
        .arg("--version")
        .output()
        .expect("Failed to execute opr8r");

    assert!(output.status.success(), "opr8r --version should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("opr8r"),
        "Version output should contain 'opr8r'"
    );
}

#[test]
fn test_opr8r_help() {
    skip_if_not_configured!();

    let opr8r_path = get_opr8r_path();

    let output = Command::new(&opr8r_path)
        .arg("--help")
        .output()
        .expect("Failed to execute opr8r");

    assert!(output.status.success(), "opr8r --help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ticket-id") || stdout.contains("TICKET_ID"),
        "Help should mention ticket-id"
    );
}

#[test]
fn test_opr8r_dry_run() {
    skip_if_not_configured!();

    let opr8r_path = get_opr8r_path();

    // Dry run should show what would be executed without actually running
    let output = Command::new(&opr8r_path)
        .args([
            "--ticket-id",
            "TEST-001",
            "--step",
            "plan",
            "--dry-run",
            "--",
            "echo",
            "hello",
        ])
        .output()
        .expect("Failed to execute opr8r");

    assert!(output.status.success(), "Dry run should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dry-run") || stdout.contains("Would execute"),
        "Dry run should indicate it's not actually executing"
    );
}

#[test]
fn test_opr8r_missing_required_args() {
    skip_if_not_configured!();

    let opr8r_path = get_opr8r_path();

    // Missing --ticket-id should fail
    let output = Command::new(&opr8r_path)
        .args(["--step", "plan", "--", "echo", "hello"])
        .output()
        .expect("Failed to execute opr8r");

    assert!(
        !output.status.success(),
        "Missing required args should fail"
    );
}

#[test]
fn test_opr8r_missing_command() {
    skip_if_not_configured!();

    let opr8r_path = get_opr8r_path();

    // Missing command after -- should fail
    let output = Command::new(&opr8r_path)
        .args(["--ticket-id", "TEST-001", "--step", "plan"])
        .output()
        .expect("Failed to execute opr8r");

    assert!(!output.status.success(), "Missing command should fail");
}
