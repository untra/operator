//! Integration tests for REST API server lifecycle
//!
//! These tests verify that:
//! - REST API server starts and binds to the configured port
//! - Session file (api-session.json) is written on startup
//! - Health endpoint responds correctly
//! - Session file is removed on shutdown
//!
//! ## Environment Variables
//!
//! - `OPERATOR_REST_API_TEST_ENABLED=true` - Required to run these tests
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all REST API integration tests
//! OPERATOR_REST_API_TEST_ENABLED=true cargo test --test rest_api_integration -- --nocapture --test-threads=1
//!
//! # Run a specific test
//! OPERATOR_REST_API_TEST_ENABLED=true cargo test --test rest_api_integration test_api_writes_session_file -- --nocapture
//! ```
//!
//! ## Notes
//!
//! - Tests use high port numbers (17000+) to avoid conflicts
//! - Tests are sequential to avoid port conflicts
//! - Each test uses a unique port

use std::env;
use std::fs;
use std::time::Duration;

use serde::Deserialize;
use tempfile::TempDir;

use operator::config::Config;
use operator::rest::server::{ApiSessionInfo, RestApiServer};

// ─── Configuration ────────────────────────────────────────────────────────────

/// Check if REST API integration tests are enabled
fn rest_api_tests_enabled() -> bool {
    env::var("OPERATOR_REST_API_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Macro to skip tests if not configured
macro_rules! skip_if_not_configured {
    () => {
        if !rest_api_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_REST_API_TEST_ENABLED not set to true");
            return;
        }
    };
}

// ─── Test Context ─────────────────────────────────────────────────────────────

/// Test context holding temporary directories and configuration
struct RestApiTestContext {
    temp_dir: TempDir,
    config: Config,
    port: u16,
}

impl RestApiTestContext {
    /// Create a new test context with isolated directories
    fn new(test_name: &str, port: u16) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create directory structure
        let tickets_path = temp_dir.path().join("tickets");
        let projects_path = temp_dir.path().join("projects");
        let state_path = temp_dir.path().join("state");

        fs::create_dir_all(tickets_path.join("queue")).unwrap();
        fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        fs::create_dir_all(tickets_path.join("completed")).unwrap();
        fs::create_dir_all(&state_path).unwrap();
        fs::create_dir_all(&projects_path).unwrap();

        eprintln!(
            "[{}] Created test directories at: {}",
            test_name,
            temp_dir.path().display()
        );

        // Start with default config and modify paths/port
        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.projects = projects_path.to_string_lossy().to_string();
        config.paths.state = state_path.to_string_lossy().to_string();
        config.rest_api.enabled = true;
        config.rest_api.port = port;

        Self {
            temp_dir,
            config,
            port,
        }
    }

    /// Get path to session file
    fn session_file_path(&self) -> std::path::PathBuf {
        self.temp_dir
            .path()
            .join("tickets")
            .join("operator")
            .join("api-session.json")
    }

    /// Read session file contents
    fn read_session_file(&self) -> Option<ApiSessionInfo> {
        let path = self.session_file_path();
        if path.exists() {
            let content = fs::read_to_string(&path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    /// Check if session file exists
    fn session_file_exists(&self) -> bool {
        self.session_file_path().exists()
    }

    /// Make HTTP request to health endpoint
    async fn check_health(&self) -> Result<HealthResponse, String> {
        let url = format!("http://localhost:{}/api/v1/health", self.port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response
                .json::<HealthResponse>()
                .await
                .map_err(|e| e.to_string())
        } else {
            Err(format!(
                "Health check failed with status: {}",
                response.status()
            ))
        }
    }
}

/// Health response from API
#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// ─── Test Cases ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_api_server_starts_and_responds() {
    skip_if_not_configured!();

    let ctx = RestApiTestContext::new("starts_and_responds", 17001);
    let server = RestApiServer::new(ctx.config.clone(), ctx.port);

    // Start server
    let result = server.start();
    assert!(result.is_ok(), "Server should start successfully");

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify server is running
    assert!(server.is_running(), "Server should report as running");

    // Check health endpoint
    let health = ctx.check_health().await;
    assert!(
        health.is_ok(),
        "Health check should succeed. Error: {:?}",
        health.err()
    );

    let health = health.unwrap();
    assert_eq!(health.status, "ok", "Health status should be 'ok'");
    assert!(!health.version.is_empty(), "Version should not be empty");

    // Stop server
    server.stop();

    // Give it time to stop
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify server is stopped
    assert!(!server.is_running(), "Server should report as stopped");
}

#[tokio::test]
async fn test_api_writes_session_file() {
    skip_if_not_configured!();

    let ctx = RestApiTestContext::new("writes_session_file", 17002);
    let server = RestApiServer::new(ctx.config.clone(), ctx.port);

    // Verify session file doesn't exist yet
    assert!(
        !ctx.session_file_exists(),
        "Session file should not exist before server starts"
    );

    // Start server
    let result = server.start();
    assert!(result.is_ok(), "Server should start successfully");

    // Give the server time to start and write session file
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify session file exists
    assert!(
        ctx.session_file_exists(),
        "Session file should exist after server starts"
    );

    // Verify session file contents
    let session = ctx.read_session_file();
    assert!(session.is_some(), "Should be able to parse session file");

    let session = session.unwrap();
    assert_eq!(
        session.port, ctx.port,
        "Session file should have correct port"
    );
    assert!(session.pid > 0, "Session file should have valid PID");
    assert!(
        !session.version.is_empty(),
        "Session file should have version"
    );
    assert!(
        !session.started_at.is_empty(),
        "Session file should have started_at timestamp"
    );

    // Stop server
    server.stop();
}

#[tokio::test]
async fn test_api_removes_session_file_on_stop() {
    skip_if_not_configured!();

    let ctx = RestApiTestContext::new("removes_session_file", 17003);
    let server = RestApiServer::new(ctx.config.clone(), ctx.port);

    // Start server
    server.start().expect("Server should start");
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify session file exists
    assert!(
        ctx.session_file_exists(),
        "Session file should exist after start"
    );

    // Stop server
    server.stop();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify session file is removed
    assert!(
        !ctx.session_file_exists(),
        "Session file should be removed after stop"
    );
}

#[tokio::test]
async fn test_api_session_file_matches_health_endpoint() {
    skip_if_not_configured!();

    let ctx = RestApiTestContext::new("session_matches_health", 17004);
    let server = RestApiServer::new(ctx.config.clone(), ctx.port);

    // Start server
    server.start().expect("Server should start");
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Get session file info
    let session = ctx.read_session_file().expect("Should read session file");

    // Get health endpoint info
    let health = ctx
        .check_health()
        .await
        .expect("Health check should succeed");

    // Verify version matches
    assert_eq!(
        session.version, health.version,
        "Session file version should match health endpoint version"
    );

    // Stop server
    server.stop();
}

#[tokio::test]
async fn test_api_port_in_use_detection() {
    skip_if_not_configured!();

    let port = 17005;

    // Start first server
    let ctx1 = RestApiTestContext::new("port_in_use_1", port);
    let server1 = RestApiServer::new(ctx1.config.clone(), port);
    server1.start().expect("First server should start");
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Try to start second server on same port
    let ctx2 = RestApiTestContext::new("port_in_use_2", port);
    let server2 = RestApiServer::new(ctx2.config.clone(), port);

    // Check if port is in use
    let port_in_use = server2.is_port_in_use().await;
    assert!(port_in_use, "Port should be detected as in use");

    // Stop first server
    server1.stop();
}

#[tokio::test]
async fn test_api_creates_operator_directory() {
    skip_if_not_configured!();

    let ctx = RestApiTestContext::new("creates_operator_dir", 17006);

    // Verify operator directory doesn't exist yet
    let operator_dir = ctx.temp_dir.path().join("tickets").join("operator");
    assert!(
        !operator_dir.exists(),
        "Operator directory should not exist before server starts"
    );

    let server = RestApiServer::new(ctx.config.clone(), ctx.port);

    // Start server
    server.start().expect("Server should start");
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify operator directory was created
    assert!(
        operator_dir.exists(),
        "Operator directory should be created on server start"
    );

    // Verify session file is inside operator directory
    assert!(
        ctx.session_file_exists(),
        "Session file should exist in operator directory"
    );

    // Stop server
    server.stop();
}
