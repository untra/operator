//! Tests for the launcher module

use std::sync::Arc;

use tempfile::TempDir;
use uuid::Uuid;

use crate::agents::tmux::{sanitize_session_name, MockTmuxClient, TmuxError};
use crate::config::{Config, DetectedTool, PathsConfig};
use crate::queue::Ticket;

use super::prompt::{generate_session_uuid, shell_escape};
use super::{LaunchOptions, Launcher, SESSION_PREFIX};

/// Helper to read the command file content from a bash command sent to tmux.
/// The command format is: `bash /path/to/script.sh`
fn read_command_file_content(sent_cmd: &str) -> Option<String> {
    // Extract the script path from "bash /path/to/script.sh [Enter]"
    let cmd = sent_cmd.trim_end_matches(" [Enter]");
    let path = cmd.strip_prefix("bash ")?;
    std::fs::read_to_string(path).ok()
}

fn make_test_config(temp_dir: &TempDir) -> Config {
    let projects_path = temp_dir.path().join("projects");
    let tickets_path = temp_dir.path().join("tickets");
    let state_path = temp_dir.path().join("state");
    std::fs::create_dir_all(&projects_path).unwrap();
    std::fs::create_dir_all(tickets_path.join("queue")).unwrap();
    std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
    std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
    std::fs::create_dir_all(tickets_path.join("operator/prompts")).unwrap();
    std::fs::create_dir_all(&state_path).unwrap();

    // Create a test project
    let test_project = projects_path.join("test-project");
    std::fs::create_dir_all(&test_project).unwrap();
    std::fs::write(test_project.join("CLAUDE.md"), "# Test Project").unwrap();

    // Create mock detected tool for tests
    let detected_tool = DetectedTool {
        name: "claude".to_string(),
        path: "/usr/bin/claude".to_string(),
        version: "1.0.0".to_string(),
        min_version: Some("1.0.0".to_string()),
        version_ok: true,
        model_aliases: vec!["sonnet".to_string()],
        command_template: "claude {{config_flags}}{{model_flag}}--session-id {{session_id}} --print-prompt-path {{prompt_file}}".to_string(),
        capabilities: crate::config::ToolCapabilities {
            supports_sessions: true,
            supports_headless: true,
        },
        yolo_flags: vec!["--dangerously-skip-permissions".to_string()],
    };

    Config {
        paths: PathsConfig {
            tickets: tickets_path.to_string_lossy().to_string(),
            projects: projects_path.to_string_lossy().to_string(),
            state: state_path.to_string_lossy().to_string(),
            worktrees: state_path.join("worktrees").to_string_lossy().to_string(),
        },
        projects: vec!["test-project".to_string()],
        llm_tools: crate::config::LlmToolsConfig {
            detected: vec![detected_tool],
            providers: vec![crate::config::LlmProvider {
                tool: "claude".to_string(),
                model: "sonnet".to_string(),
                display_name: None,
                ..Default::default()
            }],
            detection_complete: true,
            skill_directory_overrides: std::collections::HashMap::new(),
        },
        // Disable notifications in tests to avoid DBus requirement on Linux CI
        notifications: crate::config::NotificationsConfig {
            enabled: false,
            os: crate::config::OsNotificationConfig {
                enabled: false,
                sound: false,
                events: vec![],
            },
            webhook: None,
            webhooks: vec![],
            // Legacy fields
            on_agent_start: false,
            on_agent_complete: false,
            on_agent_needs_input: false,
            on_pr_created: false,
            on_investigation_created: false,
            sound: false,
        },
        ..Default::default()
    }
}

#[test]
fn test_shell_escape_simple() {
    assert_eq!(shell_escape("hello"), "'hello'");
}

#[test]
fn test_shell_escape_with_quotes() {
    assert_eq!(shell_escape("it's"), "'it'\"'\"'s'");
}

#[test]
fn test_shell_escape_multiline() {
    let input = "line1\nline2";
    let escaped = shell_escape(input);
    assert!(escaped.starts_with('\''));
    assert!(escaped.ends_with('\''));
    assert!(escaped.contains('\n'));
}

#[test]
fn test_attach_command() {
    assert_eq!(
        Launcher::attach_command("op-TASK-123"),
        "tmux attach -t op-TASK-123"
    );
}

#[test]
fn test_check_tmux_available() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    assert!(launcher.check_tmux().is_ok());
}

#[test]
fn test_check_tmux_not_installed() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::not_installed());

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    let result = launcher.check_tmux();
    assert!(matches!(result, Err(TmuxError::NotInstalled)));
}

#[test]
fn test_check_tmux_version_too_old() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = MockTmuxClient::new();
    *mock.version.lock().unwrap() = Some(crate::agents::tmux::TmuxVersion {
        major: 1,
        minor: 9,
        raw: "tmux 1.9".to_string(),
    });

    let launcher = Launcher::with_tmux_client(&config, Arc::new(mock)).unwrap();
    let result = launcher.check_tmux();
    assert!(matches!(result, Err(TmuxError::VersionTooOld(_, _))));
}

#[test]
fn test_list_sessions_empty() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    let sessions = launcher.list_sessions().unwrap();
    assert!(sessions.is_empty());
}

#[test]
fn test_list_sessions_filters_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    // Add some sessions
    mock.add_session("op-TASK-123", "/tmp");
    mock.add_session("op-FEAT-456", "/tmp");
    mock.add_session("other-session", "/tmp");

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    let sessions = launcher.list_sessions().unwrap();

    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().all(|s| s.starts_with("op-")));
}

#[test]
fn test_list_sessions_tmux_not_installed() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::not_installed());

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    let sessions = launcher.list_sessions().unwrap();

    // Should return empty list, not error
    assert!(sessions.is_empty());
}

#[test]
fn test_session_alive() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    mock.add_session("op-TASK-123", "/tmp");

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();

    assert!(launcher.session_alive("op-TASK-123"));
    assert!(!launcher.session_alive("op-TASK-456"));
}

#[test]
fn test_kill_session() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    mock.add_session("op-TASK-123", "/tmp");

    let launcher = Launcher::with_tmux_client(&config, mock.clone()).unwrap();

    assert!(launcher.session_alive("op-TASK-123"));
    launcher.kill_session("op-TASK-123").unwrap();
    assert!(!launcher.session_alive("op-TASK-123"));
}

#[test]
fn test_kill_session_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    let result = launcher.kill_session("nonexistent");

    assert!(result.is_err());
}

#[test]
fn test_capture_session_content() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    mock.add_session("op-TASK-123", "/tmp");
    mock.set_session_content("op-TASK-123", "Hello from Claude!");

    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
    let content = launcher.capture_session_content("op-TASK-123").unwrap();

    assert_eq!(content, "Hello from Claude!");
}

#[test]
fn test_session_name_sanitization() {
    // Test that session names are properly sanitized
    assert_eq!(
        format!("{}{}", SESSION_PREFIX, sanitize_session_name("TASK-123")),
        "op-TASK-123"
    );
    assert_eq!(
        format!("{}{}", SESSION_PREFIX, sanitize_session_name("FEAT-123.1")),
        "op-FEAT-123-1"
    );
    assert_eq!(
        format!(
            "{}{}",
            SESSION_PREFIX,
            sanitize_session_name("INV:critical")
        ),
        "op-INV-critical"
    );
}

#[test]
fn test_generate_session_uuid_is_valid() {
    let uuid_str = generate_session_uuid();

    // Should be a valid UUID format (36 chars with hyphens)
    assert_eq!(uuid_str.len(), 36);
    assert!(uuid_str.contains('-'));

    // Should parse as a valid UUID
    let parsed = Uuid::parse_str(&uuid_str);
    assert!(parsed.is_ok());
}

#[test]
fn test_generate_session_uuid_is_unique() {
    let uuid1 = generate_session_uuid();
    let uuid2 = generate_session_uuid();
    let uuid3 = generate_session_uuid();

    // Each UUID should be unique
    assert_ne!(uuid1, uuid2);
    assert_ne!(uuid2, uuid3);
    assert_ne!(uuid1, uuid3);
}

fn make_test_ticket(project: &str) -> Ticket {
    Ticket {
        filename: format!("20241225-1200-TASK-{}-test.md", project),
        filepath: format!("/tmp/tickets/queue/20241225-1200-TASK-{}-test.md", project),
        timestamp: "20241225-1200".to_string(),
        ticket_type: "TASK".to_string(),
        project: project.to_string(),
        id: "TASK-1234".to_string(),
        summary: "Test ticket".to_string(),
        priority: "P2-medium".to_string(),
        status: "queued".to_string(),
        step: String::new(),
        content: "Test content".to_string(),
        sessions: std::collections::HashMap::new(),
        llm_task: crate::queue::LlmTask::default(),
        worktree_path: None,
        branch: None,
        external_id: None,
        external_url: None,
        external_provider: None,
    }
}

#[test]
fn test_get_project_path_for_project_ticket() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();

    let ticket = make_test_ticket("test-project");
    let path = launcher.get_project_path(&ticket).unwrap();

    // Should be projects_root/project_name
    let expected = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    assert_eq!(path, expected);
}

#[test]
fn test_get_project_path_for_global_ticket() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();

    let ticket = make_test_ticket("global");
    let path = launcher.get_project_path(&ticket).unwrap();

    // Should be projects_root (not projects_root/global)
    let expected = temp_dir
        .path()
        .join("projects")
        .to_string_lossy()
        .to_string();
    assert_eq!(path, expected);
}

#[test]
fn test_get_project_path_nonexistent_project_fails() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let launcher = Launcher::with_tmux_client(&config, mock).unwrap();

    let ticket = make_test_ticket("nonexistent-project");
    let result = launcher.get_project_path(&ticket);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_launch_creates_session_with_correct_working_dir() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    // Create the ticket file in the queue
    let ticket = make_test_ticket("test-project");
    let queue_path = temp_dir.path().join("tickets").join("queue");
    let ticket_path = queue_path.join(&ticket.filename);
    std::fs::write(
        &ticket_path,
        "---\npriority: P2-medium\n---\n# Test\n\nTest content",
    )
    .unwrap();

    let launcher = Launcher::with_tmux_client(&config, mock.clone()).unwrap();
    let result = launcher.launch(&ticket).await;

    // The launch should succeed
    assert!(result.is_ok(), "Launch failed: {:?}", result.err());

    // Verify the tmux session was created with the correct working directory
    let session_name = format!("op-{}", ticket.id);
    let working_dir = mock.get_session_working_dir(&session_name);
    assert!(working_dir.is_some(), "Session should have been created");

    let expected_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    assert_eq!(working_dir.unwrap(), expected_path);
}

#[tokio::test]
async fn test_launch_command_includes_cd_to_project() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    // Create the ticket file in the queue
    let ticket = make_test_ticket("test-project");
    let queue_path = temp_dir.path().join("tickets").join("queue");
    let ticket_path = queue_path.join(&ticket.filename);
    std::fs::write(
        &ticket_path,
        "---\npriority: P2-medium\n---\n# Test\n\nTest content",
    )
    .unwrap();

    let launcher = Launcher::with_tmux_client(&config, mock.clone()).unwrap();
    let result = launcher.launch(&ticket).await;
    assert!(result.is_ok(), "Launch failed: {:?}", result.err());

    // Verify the command sent includes cd to the project directory
    let session_name = format!("op-{}", ticket.id);
    let keys_sent = mock.get_session_keys_sent(&session_name);
    assert!(keys_sent.is_some(), "Keys should have been sent");

    let keys = keys_sent.unwrap();
    assert!(!keys.is_empty(), "At least one command should be sent");

    // The sent command should be a bash script call, read the script content
    let sent_cmd = &keys[0];
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");

    // The script should contain cd to the project path
    let expected_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    assert!(
        script_content.contains(&format!("cd '{}'", expected_path)),
        "Command file should include cd to project path, got: {}",
        script_content
    );
}

#[tokio::test]
async fn test_launch_global_ticket_uses_root() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());

    // Create the ticket file in the queue for global project
    let ticket = make_test_ticket("global");
    let queue_path = temp_dir.path().join("tickets").join("queue");
    let ticket_path = queue_path.join(&ticket.filename);
    std::fs::write(
        &ticket_path,
        "---\npriority: P2-medium\n---\n# Test\n\nTest content",
    )
    .unwrap();

    let launcher = Launcher::with_tmux_client(&config, mock.clone()).unwrap();
    let result = launcher.launch(&ticket).await;
    assert!(result.is_ok(), "Launch failed: {:?}", result.err());

    // Verify the tmux session was created with the projects root
    let session_name = format!("op-{}", ticket.id);
    let working_dir = mock.get_session_working_dir(&session_name);
    assert!(working_dir.is_some(), "Session should have been created");

    let expected_path = temp_dir
        .path()
        .join("projects")
        .to_string_lossy()
        .to_string();
    assert_eq!(working_dir.unwrap(), expected_path);
}

// ========================================
// launch_in_tmux_with_options() tests
// ========================================

use super::options::RelaunchOptions;
use super::tmux_session::{launch_in_tmux_with_options, launch_in_tmux_with_relaunch_options};
use crate::agents::tmux::TmuxClient;

fn make_test_config_with_docker(temp_dir: &TempDir, image: &str) -> Config {
    let mut config = make_test_config(temp_dir);
    config.launch.docker.image = image.to_string();
    config.launch.docker.mount_path = "/workspace".to_string();
    config
}

fn make_test_config_no_yolo(temp_dir: &TempDir) -> Config {
    let mut config = make_test_config(temp_dir);
    config.llm_tools.detected[0].yolo_flags = vec![];
    config
}

#[test]
fn test_launch_in_tmux_session_uses_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions::default();

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok(), "Launch failed: {:?}", result.err());
    let session_name = result.unwrap();
    assert!(
        session_name.starts_with("op-"),
        "Session should use op- prefix, got: {}",
        session_name
    );
}

#[test]
fn test_launch_in_tmux_existing_session_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions::default();

    // Pre-add a session with the expected name
    let expected_session = format!("op-{}", ticket.id);
    mock.add_session(&expected_session, &project_path);

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("already exists"),
        "Error should mention session exists, got: {}",
        err
    );
}

#[test]
fn test_launch_in_tmux_sends_cd_command() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions::default();

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    assert!(keys_sent.is_some(), "Keys should have been sent");
    let sent_cmd = &keys_sent.unwrap()[0];

    // Command should be a bash script execution
    assert!(
        sent_cmd.starts_with("bash "),
        "Command should be a bash script execution, got: {}",
        sent_cmd
    );

    // Read the script content and verify it contains cd
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("cd "),
        "Command file should contain cd, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_sends_llm_command() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions::default();

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains the LLM command
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("claude"),
        "Command file should contain claude, got: {}",
        script_content
    );
    assert!(
        script_content.contains("--session-id"),
        "Command file should contain --session-id, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_yolo_mode_applies_flags() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions {
        yolo_mode: true,
        ..Default::default()
    };

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains the YOLO flag
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("--dangerously-skip-permissions"),
        "Command file should contain YOLO flag, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_yolo_mode_disabled_no_flags() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions {
        yolo_mode: false,
        ..Default::default()
    };

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it does NOT contain the YOLO flag
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        !script_content.contains("--dangerously-skip-permissions"),
        "Command file should NOT contain YOLO flag when disabled, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_docker_mode_wraps() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config_with_docker(&temp_dir, "my-claude:latest");
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions {
        docker_mode: true,
        ..Default::default()
    };

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains docker run
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("docker run"),
        "Command file should contain docker run, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_both_modes() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config_with_docker(&temp_dir, "my-claude:latest");
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions {
        yolo_mode: true,
        docker_mode: true,
        ..Default::default()
    };

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains both docker and YOLO flag
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("docker run"),
        "Command file should contain docker run, got: {}",
        script_content
    );
    assert!(
        script_content.contains("--dangerously-skip-permissions"),
        "Command file should contain YOLO flag, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_uses_provider_from_options() {
    let temp_dir = TempDir::new().unwrap();
    // Add a second tool
    let mut config = make_test_config(&temp_dir);
    config.llm_tools.detected.push(crate::config::DetectedTool {
        name: "gemini".to_string(),
        path: "/usr/bin/gemini".to_string(),
        version: "1.0.0".to_string(),
        min_version: None,
        version_ok: true,
        model_aliases: vec!["pro".to_string()],
        command_template:
            "gemini {{config_flags}}{{model_flag}}--session {{session_id}} --prompt {{prompt_file}}"
                .to_string(),
        capabilities: crate::config::ToolCapabilities::default(),
        yolo_flags: vec![],
    });
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions {
        provider: Some(crate::config::LlmProvider {
            tool: "gemini".to_string(),
            model: "pro".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it uses the gemini tool
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("gemini"),
        "Command file should use gemini tool, got: {}",
        script_content
    );
    assert!(
        script_content.contains("--model pro"),
        "Command file should use pro model, got: {}",
        script_content
    );
}

#[test]
fn test_launch_in_tmux_writes_prompt_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions::default();

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt content",
        &options,
    );

    assert!(result.is_ok());

    // Verify a prompt file was created
    let prompts_dir = temp_dir
        .path()
        .join("tickets")
        .join("operator")
        .join("prompts");
    let prompt_files: Vec<_> = std::fs::read_dir(&prompts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !prompt_files.is_empty(),
        "At least one prompt file should have been created"
    );
}

#[test]
fn test_launch_in_tmux_tmux_not_installed() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::not_installed());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = LaunchOptions::default();

    let result = launch_in_tmux_with_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("tmux is not installed"),
        "Error should mention tmux not installed, got: {}",
        err
    );
}

// ========================================
// launch_in_tmux_with_relaunch_options() tests
// ========================================

#[test]
fn test_relaunch_fresh_start_new_uuid() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = RelaunchOptions {
        resume_session_id: None, // Fresh start
        ..Default::default()
    };

    let result = launch_in_tmux_with_relaunch_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    assert!(session_name.starts_with("op-"));
}

#[test]
fn test_relaunch_inherits_yolo_mode() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = RelaunchOptions {
        launch_options: LaunchOptions {
            yolo_mode: true,
            ..Default::default()
        },
        resume_session_id: None,
        retry_reason: None,
    };

    let result = launch_in_tmux_with_relaunch_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains the YOLO flag
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("--dangerously-skip-permissions"),
        "Relaunch command file should apply YOLO flags, got: {}",
        script_content
    );
}

#[test]
fn test_relaunch_inherits_docker_mode() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config_with_docker(&temp_dir, "my-claude:latest");
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();
    let options = RelaunchOptions {
        launch_options: LaunchOptions {
            docker_mode: true,
            ..Default::default()
        },
        resume_session_id: None,
        retry_reason: None,
    };

    let result = launch_in_tmux_with_relaunch_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains docker run
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("docker run"),
        "Relaunch command file should apply Docker wrapping, got: {}",
        script_content
    );
}

#[test]
fn test_relaunch_existing_session_errors() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();

    // Pre-add a session with the expected name
    let expected_session = format!("op-{}", ticket.id);
    mock.add_session(&expected_session, &project_path);

    let options = RelaunchOptions::default();

    let result = launch_in_tmux_with_relaunch_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("already exists"),
        "Relaunch should fail if session exists, got: {}",
        err
    );
}

#[test]
fn test_relaunch_with_resume_adds_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();

    // Create a prompt file for the resume session
    let resume_uuid = "test-resume-uuid-12345";
    let prompts_dir = temp_dir
        .path()
        .join("tickets")
        .join("operator")
        .join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(
        prompts_dir.join(format!("{}.txt", resume_uuid)),
        "Previous prompt",
    )
    .unwrap();

    let options = RelaunchOptions {
        launch_options: LaunchOptions::default(),
        resume_session_id: Some(resume_uuid.to_string()),
        retry_reason: None,
    };

    let result = launch_in_tmux_with_relaunch_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Test prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it contains the resume flag
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    assert!(
        script_content.contains("--resume"),
        "Resume mode command file should add --resume flag, got: {}",
        script_content
    );
    assert!(
        script_content.contains(resume_uuid),
        "Resume should use the provided session ID, got: {}",
        script_content
    );
}

#[test]
fn test_relaunch_missing_prompt_fresh_start() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_test_config(&temp_dir);
    let mock = Arc::new(MockTmuxClient::new());
    let tmux: Arc<dyn TmuxClient> = mock.clone();
    let ticket = make_test_ticket("test-project");
    let project_path = temp_dir
        .path()
        .join("projects")
        .join("test-project")
        .to_string_lossy()
        .to_string();

    // Don't create the prompt file - it should fall back to fresh start
    let resume_uuid = "nonexistent-uuid-12345";
    let options = RelaunchOptions {
        launch_options: LaunchOptions::default(),
        resume_session_id: Some(resume_uuid.to_string()),
        retry_reason: None,
    };

    let result = launch_in_tmux_with_relaunch_options(
        &config,
        &tmux,
        &ticket,
        &project_path,
        "Fallback prompt",
        &options,
    );

    assert!(result.is_ok());
    let session_name = result.unwrap();
    let keys_sent = mock.get_session_keys_sent(&session_name);
    let sent_cmd = &keys_sent.unwrap()[0];

    // Read the script content and verify it does NOT have resume flag
    let script_content =
        read_command_file_content(sent_cmd).expect("Should be able to read command file content");
    // Should NOT have resume flag since prompt file doesn't exist
    assert!(
        !script_content.contains("--resume"),
        "Should fall back to fresh start when prompt file missing, got: {}",
        script_content
    );
}
