use super::*;
use tempfile::TempDir;

use crate::config::{DetectedTool, PathsConfig};
use crate::queue::{Queue, Ticket};
use crate::state::State;
use crate::ui::ConfirmSelection;

/// Create a test configuration with isolated temporary directories
fn make_test_config(temp_dir: &TempDir) -> Config {
    let projects_path = temp_dir.path().join("projects");
    let tickets_path = temp_dir.path().join("tickets");
    let state_path = temp_dir.path().join("state");

    std::fs::create_dir_all(&projects_path).unwrap();
    std::fs::create_dir_all(tickets_path.join("queue")).unwrap();
    std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
    std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
    std::fs::create_dir_all(tickets_path.join("operator")).unwrap();
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
            default_tool: None,
            default_model: None,
        },
        // Disable notifications in tests
        notifications: crate::config::NotificationsConfig {
            enabled: false,
            os: crate::config::OsNotificationConfig {
                enabled: false,
                sound: false,
                events: vec![],
            },
            webhook: None,
            webhooks: vec![],
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

// ============================================
// State Transition Tests
// ============================================

mod state_transitions {
    use super::*;

    #[test]
    fn test_pause_queue_sets_state_paused() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Initialize state file
        let mut state = State::load(&config).unwrap();
        state.set_paused(false).unwrap();

        // Reload and verify initial state
        let state = State::load(&config).unwrap();
        assert!(!state.paused);

        // Simulate pause_queue logic
        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();

        // Verify state is now paused
        let reloaded = State::load(&config).unwrap();
        assert!(reloaded.paused);
    }

    #[test]
    fn test_resume_queue_sets_state_resumed() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Initialize state as paused
        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();

        // Reload and verify
        let state = State::load(&config).unwrap();
        assert!(state.paused);

        // Simulate resume_queue logic
        let mut state = State::load(&config).unwrap();
        state.set_paused(false).unwrap();

        // Verify state is now resumed
        let reloaded = State::load(&config).unwrap();
        assert!(!reloaded.paused);
    }

    #[test]
    fn test_pause_persists_to_disk() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Pause and verify persistence
        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();

        // Create a completely new State instance (simulates app restart)
        let fresh_state = State::load(&config).unwrap();
        assert!(fresh_state.paused, "Paused state should persist to disk");

        // Resume and verify persistence
        let mut state = State::load(&config).unwrap();
        state.set_paused(false).unwrap();

        let fresh_state = State::load(&config).unwrap();
        assert!(!fresh_state.paused, "Resumed state should persist to disk");
    }

    #[test]
    fn test_ctrl_c_once_enters_confirmation_mode() {
        // Test the logic without full App instantiation
        let mut exit_confirmation_mode = false;
        let mut exit_confirmation_time: Option<std::time::Instant> = None;

        // Simulate first Ctrl+C
        if !exit_confirmation_mode {
            exit_confirmation_mode = true;
            exit_confirmation_time = Some(std::time::Instant::now());
        }

        assert!(exit_confirmation_mode);
        assert!(exit_confirmation_time.is_some());
    }

    #[test]
    fn test_ctrl_c_timeout_clears_confirmation() {
        let mut exit_confirmation_mode = true;
        // Set a time in the past (simulating timeout)
        let mut exit_confirmation_time = Some(
            std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(2))
                .unwrap(),
        );

        // Simulate the timeout check logic from run()
        if exit_confirmation_mode {
            if let Some(start_time) = exit_confirmation_time {
                if start_time.elapsed() > std::time::Duration::from_secs(1) {
                    exit_confirmation_mode = false;
                    exit_confirmation_time = None;
                }
            }
        }

        assert!(!exit_confirmation_mode);
        assert!(exit_confirmation_time.is_none());
    }

    #[test]
    fn test_ctrl_c_twice_sets_should_quit() {
        let mut should_quit = false;
        let exit_confirmation_mode = true; // Already in confirmation mode

        // Simulate second Ctrl+C
        if exit_confirmation_mode {
            should_quit = true;
        }

        assert!(should_quit);
    }

    #[test]
    fn test_exit_confirmation_resets_on_other_key() {
        // Simulate: first Ctrl+C enters confirmation mode, then any other key resets it
        let mut exit_confirmation_mode = true;
        let mut exit_confirmation_time = Some(std::time::Instant::now());

        // Simulate any non-Ctrl+C key press (the reset logic from run())
        if exit_confirmation_mode {
            exit_confirmation_mode = false;
            exit_confirmation_time = None;
        }

        assert!(
            !exit_confirmation_mode,
            "Any other key should reset exit confirmation"
        );
        assert!(
            exit_confirmation_time.is_none(),
            "Confirmation time should be cleared"
        );
    }
}

// ============================================
// Launch Validation Tests
// ============================================

mod launch_validation {
    use super::*;

    #[test]
    fn test_try_launch_allowed_when_not_paused_and_under_max() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // State not paused, no agents running
        let state = State::load(&config).unwrap();
        let dashboard_paused = state.paused;
        let running_count = state.running_agents().len();
        let max_agents = config.effective_max_agents();
        let project_count = state.project_agent_count("test-project");
        let max_per_repo = config.effective_max_agents_per_repo();

        // All conditions for launch should be met
        let can_launch =
            !dashboard_paused && running_count < max_agents && project_count < max_per_repo;

        assert!(
            can_launch,
            "Should be allowed to launch when not paused, under max, and project under cap"
        );
    }

    #[test]
    fn test_try_launch_blocked_when_paused() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Set up state as paused
        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();

        // Simulate try_launch check
        let dashboard_paused = true;
        let can_launch = !dashboard_paused;

        assert!(!can_launch, "Should not launch when paused");
    }

    #[test]
    fn test_try_launch_blocked_at_max_agents() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Add max agents to state
        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        // Reload state
        let state = State::load(&config).unwrap();
        let running_count = state.running_agents().len();
        let max_agents = config.effective_max_agents();

        // Test with max_agents = 1 (default)
        let can_launch = running_count < max_agents;

        // With one agent running and max_agents = 1, should be blocked
        assert!(!can_launch || max_agents > 1);
    }

    #[test]
    fn test_try_launch_blocked_project_at_capacity() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Add an agent for test-project
        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        let state = State::load(&config).unwrap();
        let project_count = state.project_agent_count("test-project");
        let max_per_repo = config.effective_max_agents_per_repo();

        assert!(
            project_count >= max_per_repo,
            "Project should be at capacity with running agent"
        );
    }

    #[test]
    fn test_try_launch_project_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let state = State::load(&config).unwrap();
        let project_count = state.project_agent_count("test-project");

        assert_eq!(project_count, 0, "Project should have no agents");
    }

    #[test]
    fn test_try_launch_allowed_when_under_per_repo_cap() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = make_test_config(&temp_dir);
        config.agents.max_agents_per_repo = 2;

        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        let state = State::load(&config).unwrap();
        let project_count = state.project_agent_count("test-project");
        let max_per_repo = config.effective_max_agents_per_repo();

        assert_eq!(project_count, 1);
        assert_eq!(max_per_repo, 2);
        assert!(
            project_count < max_per_repo,
            "Should allow second agent when cap is 2"
        );
    }

    #[test]
    fn test_try_launch_blocked_at_per_repo_cap() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = make_test_config(&temp_dir);
        config.agents.max_agents_per_repo = 2;

        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();
        state
            .add_agent(
                "TASK-002".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        let state = State::load(&config).unwrap();
        let project_count = state.project_agent_count("test-project");
        let max_per_repo = config.effective_max_agents_per_repo();

        assert!(
            project_count >= max_per_repo,
            "Should block third agent when cap is 2"
        );
    }

    #[test]
    fn test_try_launch_with_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert!(tickets.is_empty(), "Queue should be empty initially");
    }

    #[test]
    fn test_try_launch_with_ticket_in_queue() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create a ticket file in the queue
        let ticket_content = r"---
priority: P2-medium
---
# Test ticket

Test content
";
        let ticket_filename = "20241225-1200-TASK-test-project-test.md";
        let ticket_path = config.tickets_path().join("queue").join(ticket_filename);
        std::fs::write(&ticket_path, ticket_content).unwrap();

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 1, "Queue should have one ticket");
    }
}

// ============================================
// Modal Dispatch Tests
// ============================================

mod modal_dispatch {
    use super::*;

    #[test]
    fn test_help_dialog_visibility_toggle() {
        let mut help_visible = false;

        // Toggle on
        help_visible = !help_visible;
        assert!(help_visible);

        // Toggle off
        help_visible = !help_visible;
        assert!(!help_visible);
    }

    #[test]
    fn test_help_dialog_closes_on_key() {
        let mut help_visible = true;

        // Simulate any key press when help is visible
        // In app.rs, any key closes the help dialog
        if help_visible {
            help_visible = false;
        }

        assert!(!help_visible);
    }

    #[test]
    fn test_confirm_dialog_y_launches() {
        // Test the confirm dialog selection logic
        let selection = ConfirmSelection::Yes;
        let should_launch = matches!(selection, ConfirmSelection::Yes);

        assert!(should_launch);
    }

    #[test]
    fn test_confirm_dialog_n_closes() {
        let selection = ConfirmSelection::No;
        let should_close = matches!(selection, ConfirmSelection::No);

        assert!(should_close);
    }

    #[test]
    fn test_confirm_dialog_view_option() {
        let selection = ConfirmSelection::View;
        let should_view = matches!(selection, ConfirmSelection::View);

        assert!(should_view);
    }

    #[test]
    fn test_session_recovery_resume_selection() {
        use crate::ui::SessionRecoverySelection;

        let selection = SessionRecoverySelection::ResumeSession;
        let is_resume = matches!(selection, SessionRecoverySelection::ResumeSession);

        assert!(is_resume);
    }

    #[test]
    fn test_session_recovery_fresh_selection() {
        use crate::ui::SessionRecoverySelection;

        let selection = SessionRecoverySelection::StartFresh;
        let is_fresh = matches!(selection, SessionRecoverySelection::StartFresh);

        assert!(is_fresh);
    }

    #[test]
    fn test_session_recovery_return_selection() {
        use crate::ui::SessionRecoverySelection;

        let selection = SessionRecoverySelection::ReturnToQueue;
        let is_return = matches!(selection, SessionRecoverySelection::ReturnToQueue);

        assert!(is_return);
    }
}

// ============================================
// Review Signal Tests
// ============================================

mod review_signals {
    #[test]
    fn test_review_approval_requires_pending_state() {
        // Test the condition check without full App
        let review_state: Option<&str> = Some("pending_plan");

        let can_approve = matches!(review_state, Some("pending_plan" | "pending_visual"));

        assert!(can_approve);
    }

    #[test]
    fn test_review_approval_pending_visual() {
        // Symmetric test for the pending_visual match arm
        let review_state: Option<&str> = Some("pending_visual");

        let can_approve = matches!(review_state, Some("pending_plan" | "pending_visual"));

        assert!(can_approve, "pending_visual should also be approvable");
    }

    #[test]
    fn test_review_approval_blocked_for_other_states() {
        let review_state: Option<&str> = Some("running");

        let can_approve = matches!(review_state, Some("pending_plan" | "pending_visual"));

        assert!(!can_approve);
    }

    #[test]
    fn test_review_rejection_blocked_for_running_state() {
        // Mirrors approval tests but for rejection path — same guard logic applies
        let review_state: Option<&str> = Some("running");

        let can_reject = matches!(review_state, Some("pending_plan" | "pending_visual"));

        assert!(
            !can_reject,
            "Rejection should be blocked for non-pending states"
        );

        // Also verify None is blocked
        let review_state: Option<&str> = None;
        let can_reject = matches!(review_state, Some("pending_plan" | "pending_visual"));
        assert!(
            !can_reject,
            "Rejection should be blocked when no review state"
        );

        // And verify pending states ARE rejectable
        let review_state: Option<&str> = Some("pending_plan");
        let can_reject = matches!(review_state, Some("pending_plan" | "pending_visual"));
        assert!(can_reject, "pending_plan should be rejectable");

        let review_state: Option<&str> = Some("pending_visual");
        let can_reject = matches!(review_state, Some("pending_plan" | "pending_visual"));
        assert!(can_reject, "pending_visual should be rejectable");
    }

    #[test]
    fn test_review_approval_blocked_for_none() {
        let review_state: Option<&str> = None;

        let can_approve = matches!(review_state, Some("pending_plan" | "pending_visual"));

        assert!(!can_approve);
    }

    #[test]
    fn test_review_signal_file_path() {
        let session_name = "op-TASK-123";
        let signal_file = format!("/tmp/operator-detach-{session_name}.signal");

        assert_eq!(signal_file, "/tmp/operator-detach-op-TASK-123.signal");
    }
}

// ============================================
// Return to Queue Tests
// ============================================

mod return_to_queue {
    use super::*;

    #[test]
    fn test_return_ticket_removes_agent_from_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Add an agent and set its session name
        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        // Set the session name
        let session_name = "op-TASK-001".to_string();
        state
            .update_agent_session(&agent_id, &session_name)
            .unwrap();

        // Reload and verify agent exists
        let state = State::load(&config).unwrap();
        assert!(state.agent_by_session(&session_name).is_some());

        // Remove agent by session
        let mut state = State::load(&config).unwrap();
        state.remove_agent_by_session(&session_name).unwrap();

        // Verify agent is removed
        let state = State::load(&config).unwrap();
        assert!(state.agent_by_session(&session_name).is_none());
    }

    #[test]
    fn test_queue_return_moves_ticket_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create a ticket in in-progress
        let ticket_content = r"---
priority: P2-medium
status: in-progress
---
# Test ticket

Test content
";
        let ticket_filename = "20241225-1200-TASK-test-project-test.md";
        let in_progress_path = config
            .tickets_path()
            .join("in-progress")
            .join(ticket_filename);
        std::fs::write(&in_progress_path, ticket_content).unwrap();

        // Verify file is in in-progress
        assert!(in_progress_path.exists());

        // Load queue and get ticket
        let queue = Queue::new(&config).unwrap();

        // Create ticket struct for return_to_queue
        let ticket = Ticket {
            filename: ticket_filename.to_string(),
            filepath: in_progress_path.to_string_lossy().to_string(),
            timestamp: "20241225-1200".to_string(),
            ticket_type: "TASK".to_string(),
            project: "test-project".to_string(),
            id: "TASK-test".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "in-progress".to_string(),
            step: String::new(),
            content: "Test content".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        };

        // Return to queue
        queue.return_to_queue(&ticket).unwrap();

        // Verify file moved to queue
        let queue_path = config.tickets_path().join("queue").join(ticket_filename);
        assert!(queue_path.exists(), "Ticket should be moved to queue");
        assert!(
            !in_progress_path.exists(),
            "Ticket should be removed from in-progress"
        );
    }
}

// ============================================
// Dashboard State Tests
// ============================================

mod dashboard_state {
    use super::*;

    #[test]
    fn test_dashboard_paused_reflects_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Set state to paused
        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();

        // Create dashboard and update from state
        let mut dashboard = Dashboard::new(&config);
        let state = State::load(&config).unwrap();
        dashboard.paused = state.paused;

        assert!(dashboard.paused);
    }

    #[test]
    fn test_dashboard_agents_update() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Add agent to state
        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        // Create dashboard and update agents
        let mut dashboard = Dashboard::new(&config);
        let state = State::load(&config).unwrap();
        let agents: Vec<_> = state.agents.clone();
        dashboard.update_agents(agents);

        // Verify running agents count via state (dashboard reflects state)
        assert_eq!(state.running_agents().len(), 1);
    }

    #[test]
    fn test_refresh_data_updates_queue_and_agents() {
        // Simulate the refresh_data logic from data_sync.rs:
        // load queue, load state, update dashboard fields
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Set up state with a paused flag and an agent
        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();
        state
            .add_agent(
                "TASK-002".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        // Create a ticket in queue
        let ticket_content = "---\npriority: P2-medium\n---\n# Refresh test\n\nContent\n";
        let ticket_filename = "20241225-1200-TASK-test-project-refresh.md";
        let ticket_path = config.tickets_path().join("queue").join(ticket_filename);
        std::fs::write(&ticket_path, ticket_content).unwrap();

        // Now simulate refresh_data: load queue, load state, update dashboard
        let mut dashboard = Dashboard::new(&config);

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();
        dashboard.update_queue(tickets);

        let state = State::load(&config).unwrap();
        dashboard.paused = state.paused;
        let agents: Vec<_> = state.agents.clone();
        dashboard.update_agents(agents);

        // Verify dashboard reflects the data
        assert!(dashboard.paused, "Dashboard should reflect paused state");
        assert_eq!(
            state.running_agents().len(),
            1,
            "Should have one running agent"
        );
    }
}

// ============================================
// Kanban Sync Tests
// ============================================

mod kanban_sync {
    #[test]
    fn test_show_kanban_view_no_providers_sets_message() {
        // Simulate the logic from show_kanban_view():
        // when collections is empty, sync_status_message should be set
        let collections: Vec<String> = vec![];
        let mut sync_status_message: Option<String> = None;

        if collections.is_empty() {
            sync_status_message = Some(
                "No kanban providers configured. Add [kanban] section to config.toml".to_string(),
            );
        }

        assert_eq!(
            sync_status_message.as_deref(),
            Some("No kanban providers configured. Add [kanban] section to config.toml"),
            "Empty collections should set a status message"
        );
    }

    #[test]
    fn test_run_kanban_sync_all_no_providers_sets_message() {
        // Simulate the logic from run_kanban_sync_all():
        // when total == 0, sync_status_message should be set
        let total = 0;
        let mut sync_status_message: Option<String> = None;

        if total == 0 {
            sync_status_message = Some("No kanban providers configured".to_string());
        }

        assert_eq!(
            sync_status_message.as_deref(),
            Some("No kanban providers configured"),
        );
    }
}

mod agent_switches {
    use super::*;

    #[test]
    fn test_switch_marker_detected_on_agent_with_review_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-switch-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        state
            .set_agent_review_state(&agent_id, "switching_agent:my-delegator")
            .unwrap();

        let switches: Vec<String> = state
            .agents
            .iter()
            .filter_map(|agent| {
                let rs = agent.review_state.as_ref()?;
                rs.strip_prefix("switching_agent:").map(str::to_string)
            })
            .collect();

        assert_eq!(switches, vec!["my-delegator"]);
    }

    #[test]
    fn test_no_switches_when_agents_have_non_switch_review_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-plan-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        state
            .set_agent_review_state(&agent_id, "pending_plan")
            .unwrap();

        let switches: Vec<String> = state
            .agents
            .iter()
            .filter_map(|agent| {
                let rs = agent.review_state.as_ref()?;
                rs.strip_prefix("switching_agent:").map(str::to_string)
            })
            .collect();

        assert!(
            switches.is_empty(),
            "pending_plan review state should not trigger an agent switch"
        );
    }

    #[test]
    fn test_no_switches_when_all_agents_have_no_review_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-noreview".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        let switches: Vec<String> = state
            .agents
            .iter()
            .filter_map(|agent| {
                let rs = agent.review_state.as_ref()?;
                rs.strip_prefix("switching_agent:").map(str::to_string)
            })
            .collect();

        assert!(
            switches.is_empty(),
            "Agents with no review_state should not produce switches"
        );
    }

    #[test]
    fn test_only_switch_marked_agents_collected_when_mixed() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();

        let id_a = state
            .add_agent(
                "TASK-a".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();
        let id_b = state
            .add_agent(
                "TASK-b".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();
        let _id_c = state
            .add_agent(
                "TASK-c".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        state
            .set_agent_review_state(&id_a, "switching_agent:delegator-x")
            .unwrap();
        state.set_agent_review_state(&id_b, "pending_plan").unwrap();
        // _id_c has no review_state

        let switches: Vec<String> = state
            .agents
            .iter()
            .filter_map(|agent| {
                let rs = agent.review_state.as_ref()?;
                rs.strip_prefix("switching_agent:").map(str::to_string)
            })
            .collect();

        assert_eq!(
            switches,
            vec!["delegator-x"],
            "Only the switch-marked agent should appear"
        );
    }
}

// ============================================
// Ticket Creation Tests
// ============================================

mod ticket_creation {
    use super::*;
    use crate::queue::TicketCreator;
    use crate::templates::TemplateType;
    use std::collections::HashMap;

    #[test]
    fn test_headless_ticket_creation_writes_file_to_queue() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let creator = TicketCreator::new(&config);
        let mut values = HashMap::new();
        values.insert("project".to_string(), "test-project".to_string());
        values.insert("summary".to_string(), "Add user auth".to_string());

        let result = creator.create_ticket_headless(TemplateType::Task, &values);
        assert!(result.is_ok(), "Ticket creation should succeed");

        let filepath = result.unwrap();
        assert!(filepath.exists(), "Ticket file should exist on disk");
        assert!(
            filepath.to_string_lossy().contains("queue"),
            "Ticket should be created in the queue directory"
        );
    }

    #[test]
    fn test_headless_ticket_filename_contains_type_and_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let creator = TicketCreator::new(&config);
        let mut values = HashMap::new();
        values.insert("project".to_string(), "test-project".to_string());

        let filepath = creator
            .create_ticket_headless(TemplateType::Feature, &values)
            .unwrap();
        let filename = filepath.file_name().unwrap().to_string_lossy();

        assert!(
            filename.contains("FEAT"),
            "Filename should contain the ticket type: {filename}"
        );
        assert!(
            filename.contains("test-project"),
            "Filename should contain the project: {filename}"
        );
    }

    #[test]
    fn test_headless_ticket_defaults_project_to_global() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let creator = TicketCreator::new(&config);
        let values = HashMap::new(); // No project specified

        let filepath = creator
            .create_ticket_headless(TemplateType::Task, &values)
            .unwrap();
        let filename = filepath.file_name().unwrap().to_string_lossy();

        assert!(
            filename.contains("global"),
            "Filename should default to 'global' when no project specified: {filename}"
        );
    }

    #[test]
    fn test_headless_ticket_content_is_not_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let creator = TicketCreator::new(&config);
        let mut values = HashMap::new();
        values.insert("project".to_string(), "test-project".to_string());

        let filepath = creator
            .create_ticket_headless(TemplateType::Task, &values)
            .unwrap();
        let content = std::fs::read_to_string(&filepath).unwrap();

        assert!(!content.is_empty(), "Ticket file should have content");
    }

    #[test]
    fn test_created_ticket_appears_in_queue_listing() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let creator = TicketCreator::new(&config);
        let mut values = HashMap::new();
        values.insert("project".to_string(), "test-project".to_string());
        values.insert("summary".to_string(), "Test listing".to_string());

        creator
            .create_ticket_headless(TemplateType::Task, &values)
            .unwrap();

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_queue().unwrap();

        assert_eq!(tickets.len(), 1, "Queue should have the created ticket");
        assert_eq!(tickets[0].ticket_type, "TASK");
    }

    #[test]
    fn test_multiple_ticket_types_created_and_sorted_by_priority() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create tickets of different types
        let types_and_projects = [
            (TemplateType::Feature, "test-project"),
            (TemplateType::Fix, "test-project"),
            (TemplateType::Task, "test-project"),
        ];

        for (template_type, project) in &types_and_projects {
            let creator = TicketCreator::new(&config);
            let mut values = HashMap::new();
            values.insert("project".to_string(), project.to_string());
            creator
                .create_ticket_headless(*template_type, &values)
                .unwrap();
        }

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 3, "Queue should have 3 tickets");
    }
}

// ============================================
// Ticket Directory Initialization Tests
// ============================================

mod ticket_initialization {
    use super::*;

    #[test]
    fn test_queue_directories_exist_after_config_setup() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let tickets_path = config.tickets_path();
        assert!(
            tickets_path.join("queue").exists(),
            "queue directory should exist"
        );
        assert!(
            tickets_path.join("in-progress").exists(),
            "in-progress directory should exist"
        );
        assert!(
            tickets_path.join("completed").exists(),
            "completed directory should exist"
        );
        assert!(
            tickets_path.join("operator").exists(),
            "operator directory should exist"
        );
    }

    #[test]
    fn test_queue_starts_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let queue = Queue::new(&config).unwrap();
        assert!(queue.list_queue().unwrap().is_empty());
        assert!(queue.list_in_progress().unwrap().is_empty());
        assert!(queue.list_completed().unwrap().is_empty());
    }

    #[test]
    fn test_claim_ticket_moves_to_in_progress() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create a ticket file in the queue
        let ticket_content = "---\npriority: P2-medium\n---\n# Test\n\nContent\n";
        let ticket_filename = "20241225-1200-TASK-test-project-test.md";
        let queue_path = config.tickets_path().join("queue").join(ticket_filename);
        std::fs::write(&queue_path, ticket_content).unwrap();

        // Verify it's in the queue
        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_queue().unwrap();
        assert_eq!(tickets.len(), 1);

        // Claim (move to in-progress)
        queue.claim_ticket(&tickets[0]).unwrap();

        // Verify moved
        assert!(queue.list_queue().unwrap().is_empty());
        assert_eq!(queue.list_in_progress().unwrap().len(), 1);
    }
}

// ============================================
// Auto-Launch Decision Logic Tests
// ============================================

mod auto_launch_logic {
    use super::*;

    fn can_auto_launch(config: &Config, state: &State, queue: &Queue) -> bool {
        if state.paused {
            return false;
        }
        let running = state.running_agents().len();
        let max = config.effective_max_agents();
        if running >= max {
            return false;
        }
        let tickets = queue.list_by_priority().unwrap_or_default();
        if tickets.is_empty() {
            return false;
        }
        // Check per-repo cap for the top ticket's project
        let top = &tickets[0];
        let project_count = state.project_agent_count(&top.project);
        let max_per_repo = config.effective_max_agents_per_repo();
        project_count < max_per_repo
    }

    #[test]
    fn test_auto_launch_blocked_when_paused() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        state.set_paused(true).unwrap();
        let state = State::load(&config).unwrap();

        // Add a ticket to queue
        let ticket_content = "---\npriority: P2-medium\n---\n# Test\n\nContent\n";
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-TASK-test-project-test.md"),
            ticket_content,
        )
        .unwrap();
        let queue = Queue::new(&config).unwrap();

        assert!(
            !can_auto_launch(&config, &state, &queue),
            "Should not auto-launch when paused"
        );
    }

    #[test]
    fn test_auto_launch_blocked_when_queue_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let state = State::load(&config).unwrap();
        let queue = Queue::new(&config).unwrap();

        assert!(
            !can_auto_launch(&config, &state, &queue),
            "Should not auto-launch with empty queue"
        );
    }

    #[test]
    fn test_auto_launch_blocked_at_max_agents() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = make_test_config(&temp_dir);
        config.agents.max_parallel = 1;

        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();
        let state = State::load(&config).unwrap();

        let ticket_content = "---\npriority: P2-medium\n---\n# Test\n\nContent\n";
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-TASK-test-project-launch.md"),
            ticket_content,
        )
        .unwrap();
        let queue = Queue::new(&config).unwrap();

        assert!(
            !can_auto_launch(&config, &state, &queue),
            "Should not auto-launch when at max agents"
        );
    }

    #[test]
    fn test_auto_launch_allowed_when_conditions_met() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let state = State::load(&config).unwrap();

        let ticket_content = "---\npriority: P2-medium\n---\n# Test\n\nContent\n";
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-TASK-test-project-go.md"),
            ticket_content,
        )
        .unwrap();
        let queue = Queue::new(&config).unwrap();

        assert!(
            can_auto_launch(&config, &state, &queue),
            "Should auto-launch when not paused, under capacity, with tickets"
        );
    }

    #[test]
    fn test_auto_launch_blocked_when_project_at_per_repo_cap() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = make_test_config(&temp_dir);
        config.agents.max_parallel = 5;
        config.agents.max_agents_per_repo = 1;

        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-existing".to_string(),
                "TASK".to_string(),
                "testproject".to_string(),
                false,
            )
            .unwrap();
        let state = State::load(&config).unwrap();

        // Project name in filename must be [a-z0-9]+ (no hyphens) to match parser regex
        let ticket_content = "---\npriority: P2-medium\n---\n# Test\n\nContent\n";
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-TASK-testproject-blocked.md"),
            ticket_content,
        )
        .unwrap();
        let queue = Queue::new(&config).unwrap();

        assert!(
            !can_auto_launch(&config, &state, &queue),
            "Should not auto-launch when project is at per-repo capacity"
        );
    }
}

// ============================================
// Status Action Tests
// ============================================

mod status_actions {
    use crate::ui::status_panel::StatusAction;

    #[test]
    fn test_status_action_none_is_noop() {
        let action = StatusAction::None;
        assert!(matches!(action, StatusAction::None));
    }

    #[test]
    fn test_status_action_toggle_section_carries_id() {
        use crate::ui::status_panel::SectionId;
        let action = StatusAction::ToggleSection(SectionId::Configuration);
        if let StatusAction::ToggleSection(id) = action {
            assert!(matches!(id, SectionId::Configuration));
        } else {
            panic!("Expected ToggleSection");
        }
    }

    #[test]
    fn test_open_in_browser_constructs_url() {
        let port = 8080;
        let url = format!("http://localhost:{port}/swagger-ui/");
        assert_eq!(url, "http://localhost:8080/swagger-ui/");
    }
}

// ============================================
// Queue Priority Ordering Tests
// ============================================

mod queue_priority {
    use super::*;

    #[test]
    fn test_priority_ordering_inv_before_feat() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        // Create tickets with different types
        // INV should sort before FEAT per priority_order config
        let feat_content = "---\npriority: P2-medium\n---\n# Feature\n\nContent\n";
        let inv_content = "---\npriority: P0-critical\n---\n# Investigation\n\nContent\n";

        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-FEAT-test-project-feat.md"),
            feat_content,
        )
        .unwrap();
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1201-INV-test-project-inv.md"),
            inv_content,
        )
        .unwrap();

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 2);
        // INV has higher priority (lower index) in default priority_order
        let first_type = &tickets[0].ticket_type;
        let second_type = &tickets[1].ticket_type;

        let first_idx = config.priority_index(first_type);
        let second_idx = config.priority_index(second_type);
        assert!(
            first_idx <= second_idx,
            "First ticket type '{first_type}' (idx {first_idx}) should have equal or higher priority than '{second_type}' (idx {second_idx})"
        );
    }

    #[test]
    fn test_same_type_tickets_sorted_fifo_by_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let content = "---\npriority: P2-medium\n---\n# Task\n\nContent\n";

        // Earlier timestamp first
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-0800-TASK-test-project-early.md"),
            content,
        )
        .unwrap();
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1400-TASK-test-project-late.md"),
            content,
        )
        .unwrap();

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 2);
        assert!(
            tickets[0].timestamp < tickets[1].timestamp,
            "Earlier timestamp should sort first: {} vs {}",
            tickets[0].timestamp,
            tickets[1].timestamp
        );
    }

    #[test]
    fn test_unknown_ticket_type_sorts_last() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let task_content = "---\npriority: P2-medium\n---\n# Task\n\nContent\n";
        let unknown_content = "---\npriority: P2-medium\n---\n# Custom\n\nContent\n";

        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-TASK-test-project-task.md"),
            task_content,
        )
        .unwrap();
        std::fs::write(
            config
                .tickets_path()
                .join("queue/20241225-1200-CUSTOM-test-project-custom.md"),
            unknown_content,
        )
        .unwrap();

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 2);
        // TASK is in priority_order, CUSTOM is not (sorts to usize::MAX)
        assert_eq!(
            tickets[0].ticket_type, "TASK",
            "Known type should sort before unknown"
        );
    }
}

// ============================================
// Agent State Lifecycle Tests
// ============================================

mod agent_lifecycle {
    use super::*;

    #[test]
    fn test_agent_added_with_correct_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "FEAT-042".to_string(),
                "FEAT".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        let state = State::load(&config).unwrap();
        let agent = state
            .agents
            .iter()
            .find(|a| a.id == agent_id)
            .expect("Agent should exist");

        assert_eq!(agent.ticket_id, "FEAT-042");
        assert_eq!(agent.ticket_type, "FEAT");
        assert_eq!(agent.project, "test-project");
    }

    #[test]
    fn test_multiple_agents_tracked_independently() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let id1 = state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "project-a".to_string(),
                false,
            )
            .unwrap();
        let id2 = state
            .add_agent(
                "FEAT-002".to_string(),
                "FEAT".to_string(),
                "project-b".to_string(),
                false,
            )
            .unwrap();

        let state = State::load(&config).unwrap();
        assert_eq!(state.running_agents().len(), 2);
        assert_ne!(id1, id2, "Agent IDs should be unique");
    }

    #[test]
    fn test_project_agent_count_tracks_per_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        state
            .add_agent(
                "TASK-001".to_string(),
                "TASK".to_string(),
                "project-a".to_string(),
                false,
            )
            .unwrap();
        state
            .add_agent(
                "TASK-002".to_string(),
                "TASK".to_string(),
                "project-a".to_string(),
                false,
            )
            .unwrap();
        state
            .add_agent(
                "TASK-003".to_string(),
                "TASK".to_string(),
                "project-b".to_string(),
                false,
            )
            .unwrap();

        let state = State::load(&config).unwrap();
        assert_eq!(state.project_agent_count("project-a"), 2);
        assert_eq!(state.project_agent_count("project-b"), 1);
        assert_eq!(state.project_agent_count("project-c"), 0);
    }

    #[test]
    fn test_agent_session_update_and_lookup() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-session".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();

        state
            .update_agent_session(&agent_id, "op-TASK-session")
            .unwrap();

        let state = State::load(&config).unwrap();
        let agent = state.agent_by_session("op-TASK-session");
        assert!(agent.is_some(), "Should find agent by session name");
        assert_eq!(agent.unwrap().ticket_id, "TASK-session");
    }

    #[test]
    fn test_remove_agent_by_session_cleans_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);

        let mut state = State::load(&config).unwrap();
        let agent_id = state
            .add_agent(
                "TASK-remove".to_string(),
                "TASK".to_string(),
                "test-project".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-TASK-remove")
            .unwrap();

        assert_eq!(state.running_agents().len(), 1);

        let mut state = State::load(&config).unwrap();
        state.remove_agent_by_session("op-TASK-remove").unwrap();

        let state = State::load(&config).unwrap();
        assert_eq!(state.running_agents().len(), 0);
        assert!(state.agent_by_session("op-TASK-remove").is_none());
    }
}
