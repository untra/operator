//! Tests for the setup wizard

use super::types::*;
use super::SetupScreen;
use crate::api::providers::model_server::ModelServerKind;
use crate::config::SessionWrapperType;
use std::collections::HashMap;

#[test]
fn test_detected_tool_info_creation() {
    let info = DetectedToolInfo {
        name: "claude".to_string(),
        version: "2.0.76".to_string(),
        model_count: 3,
    };
    assert_eq!(info.name, "claude");
    assert_eq!(info.version, "2.0.76");
    assert_eq!(info.model_count, 3);
}

#[test]
fn test_setup_screen_new_with_detected_tools() {
    let tools = vec![DetectedToolInfo {
        name: "claude".to_string(),
        version: "2.0.76".to_string(),
        model_count: 3,
    }];
    let mut projects = HashMap::new();
    projects.insert("claude".to_string(), vec!["project-a".to_string()]);

    let screen = SetupScreen::new(".tickets".to_string(), tools, projects);

    assert!(screen.visible);
    assert_eq!(screen.step, SetupStep::Welcome);
}

#[test]
fn test_setup_screen_with_no_detected_tools() {
    let screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    assert!(screen.visible);
    assert_eq!(screen.step, SetupStep::Welcome);
}

#[test]
fn test_setup_screen_with_multiple_tools() {
    let tools = vec![
        DetectedToolInfo {
            name: "claude".to_string(),
            version: "2.0.0".to_string(),
            model_count: 3,
        },
        DetectedToolInfo {
            name: "gemini".to_string(),
            version: "1.0.0".to_string(),
            model_count: 2,
        },
    ];
    let mut projects = HashMap::new();
    projects.insert(
        "claude".to_string(),
        vec!["api".to_string(), "web".to_string()],
    );
    projects.insert("gemini".to_string(), vec!["api".to_string()]);

    let screen = SetupScreen::new(".tickets".to_string(), tools, projects);
    assert!(screen.visible);
    assert_eq!(screen.step, SetupStep::Welcome);
}

// ─── Session Wrapper Selection Tests ────────────────────────────────────────

#[test]
fn test_setup_default_wrapper_is_tmux() {
    let screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    assert_eq!(screen.selected_wrapper, SessionWrapperType::Tmux);
}

#[test]
fn test_setup_tmux_status_default_is_not_checked() {
    let screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    assert_eq!(screen.tmux_status, TmuxDetectionStatus::NotChecked);
}

#[test]
fn test_setup_vscode_status_default_is_not_checked() {
    let screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    assert_eq!(screen.vscode_status, VSCodeDetectionStatus::NotChecked);
}

#[test]
fn test_setup_wrapper_navigation_flow() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());

    // Navigate to SessionWrapperChoice
    screen.step = SetupStep::TaskFieldConfig;
    screen.confirm(); // Should go to SessionWrapperChoice
    assert_eq!(screen.step, SetupStep::SessionWrapperChoice);
}

// ─── Hosted Collection Picker Tests ─────────────────────────────────────────

#[test]
fn test_collection_source_browse_enters_fetch_step() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.enter_collection_source();
    // Select the "Browse Hosted Collections" option.
    let idx = screen
        .source_options
        .iter()
        .position(|o| *o == CollectionSourceOption::Browse)
        .unwrap();
    screen.source_state.select(Some(idx));

    screen.confirm();
    assert_eq!(screen.step, SetupStep::HostedCollectionFetch);
    assert!(!screen.hosted_loaded);
}

#[tokio::test]
async fn test_hosted_picker_offline_fallback_and_commit() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::HostedCollectionFetch;

    // Offline (no URL) -> embedded fallback; picker is never empty.
    screen.load_hosted_collections(None, 1).await;
    assert!(screen.hosted_loaded);
    assert!(!screen.hosted_resolved.is_empty());

    // Highlight dev_kanban and commit.
    let idx = screen
        .hosted_resolved
        .iter()
        .position(|r| r.manifest.id == "dev_kanban")
        .expect("dev_kanban present in embedded fallback");
    screen.hosted_state.select(Some(idx));

    screen.confirm();
    assert_eq!(screen.step, SetupStep::TaskFieldConfig);
    assert_eq!(screen.selected_hosted_id.as_deref(), Some("dev_kanban"));
    // default_selected seeds the custom collection.
    assert_eq!(screen.collection(), vec!["TASK", "FEAT", "FIX"]);
}

#[tokio::test]
async fn test_hosted_picker_multi_select_merges_issue_types() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::HostedCollectionFetch;
    screen.load_hosted_collections(None, 1).await;

    // Check both `simple` (TASK) and `dev_kanban` (TASK, FEAT, FIX).
    for id in ["simple", "dev_kanban"] {
        let idx = screen
            .hosted_resolved
            .iter()
            .position(|r| r.manifest.id == id)
            .expect("collection present in embedded fallback");
        screen.hosted_state.select(Some(idx));
        screen.toggle_selection();
    }

    screen.confirm();
    assert_eq!(screen.step, SetupStep::TaskFieldConfig);
    // Several collections merged -> no single committed id.
    assert!(screen.selected_hosted_id.is_none());
    // Union in first-seen order, de-duplicated.
    assert_eq!(screen.collection(), vec!["TASK", "FEAT", "FIX"]);
}

#[test]
fn test_hosted_fetch_go_back_returns_to_collection_source() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::HostedCollectionFetch;
    screen.go_back();
    assert_eq!(screen.step, SetupStep::CollectionSource);
}

#[test]
fn test_setup_navigation_to_worktree_preference() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::SessionWrapperChoice;
    screen.selected_wrapper = SessionWrapperType::Tmux;
    screen.wrapper_state.select(Some(0)); // Select tmux

    // SessionWrapperChoice -> ExecutionTarget
    screen.confirm();
    assert_eq!(screen.step, SetupStep::ExecutionTarget);
}

#[test]
fn test_setup_navigation_tmux_path() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;
    screen.selected_wrapper = SessionWrapperType::Tmux;

    // WorktreePreference -> AdminPassword -> TmuxOnboarding (tmux selected).
    // The optional password step now sits between them; skipping it reaches the same wrapper step.
    screen.confirm();
    assert_eq!(screen.step, SetupStep::AdminPassword);
    screen.confirm();
    assert_eq!(screen.step, SetupStep::TmuxOnboarding);
}

#[test]
fn test_setup_navigation_vscode_path() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;
    screen.selected_wrapper = SessionWrapperType::Vscode;

    // WorktreePreference -> AdminPassword -> VSCodeSetup (vscode selected).
    screen.confirm();
    assert_eq!(screen.step, SetupStep::AdminPassword);
    screen.confirm();
    assert_eq!(screen.step, SetupStep::VSCodeSetup);
}

#[test]
fn test_setup_worktree_preference_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;

    screen.go_back();
    assert_eq!(screen.step, SetupStep::ExecutionTarget);
}

#[test]
fn test_execution_target_local_is_the_default() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::ExecutionTarget;

    assert_eq!(
        screen.selected_execution_target().kind,
        crate::config::TargetKind::Local
    );
    screen.confirm();
    assert_eq!(screen.step, SetupStep::WorktreePreference);
}

#[test]
fn test_execution_target_coder_requires_template() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::ExecutionTarget;
    screen.execution_target_state.select(Some(1));
    screen.coder_template.clear();

    screen.confirm();

    assert_eq!(screen.step, SetupStep::ExecutionTarget);
    assert!(screen
        .execution_target_error
        .as_deref()
        .unwrap_or_default()
        .contains("template"));
}

#[test]
fn test_execution_target_coder_builds_target_and_disables_worktrees() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::ExecutionTarget;
    screen.execution_target_state.select(Some(1));
    screen.coder_template = "operator-agent".to_string();
    screen.use_worktrees = true;

    screen.confirm();

    assert_eq!(screen.step, SetupStep::WorktreePreference);
    assert!(!screen.use_worktrees);
    assert!(matches!(
        screen.selected_execution_target().kind,
        crate::config::TargetKind::Coder(_)
    ));
}

#[test]
fn test_setup_tmux_onboarding_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::TmuxOnboarding;

    // TmuxOnboarding -> AdminPassword (the step it now came from)
    screen.go_back();
    assert_eq!(screen.step, SetupStep::AdminPassword);
}

#[test]
fn test_setup_vscode_setup_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::VSCodeSetup;

    // VSCodeSetup -> AdminPassword (the step it now came from)
    screen.go_back();
    assert_eq!(screen.step, SetupStep::AdminPassword);
}

#[test]
fn test_setup_wrapper_selection_toggle() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::SessionWrapperChoice;

    // Start at tmux (default)
    assert_eq!(screen.selected_wrapper, SessionWrapperType::Tmux);

    // Navigate down to vscode
    screen.select_next();
    assert_eq!(screen.wrapper_state.selected(), Some(1));

    // Toggle selection
    screen.toggle_selection();
    assert_eq!(screen.selected_wrapper, SessionWrapperType::Vscode);
}

#[test]
fn test_tmux_onboarding_blocks_if_not_available() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::TmuxOnboarding;
    screen.tmux_status = TmuxDetectionStatus::NotInstalled;

    // Should stay on TmuxOnboarding because tmux isn't available
    screen.confirm();
    assert_eq!(screen.step, SetupStep::TmuxOnboarding);
}

#[test]
fn test_tmux_onboarding_proceeds_if_available() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::TmuxOnboarding;
    screen.tmux_status = TmuxDetectionStatus::Available {
        version: "3.3a".to_string(),
    };

    // Wrapper setup now precedes acceptance criteria (kanban moved earlier).
    screen.confirm();
    assert_eq!(screen.step, SetupStep::AcceptanceCriteria);
}

#[test]
fn test_kanban_info_go_back_returns_to_welcome() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    // Kanban setup is now the first step after Welcome.
    screen.step = SetupStep::KanbanInfo;
    screen.go_back();
    assert_eq!(screen.step, SetupStep::Welcome);
}

#[test]
fn test_welcome_advances_to_kanban_info() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    assert_eq!(screen.step, SetupStep::Welcome);
    screen.confirm();
    assert_eq!(screen.step, SetupStep::KanbanInfo);
    assert!(screen.kanban_detection_complete);
}

#[test]
fn test_kanban_skip_advances_to_curated_collection_source() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::KanbanInfo;
    screen.select_next(); // "Skip for now"

    screen.confirm(); // -> ModelServer
    screen.confirm(); // -> GitProvider
    skip_git_provider(&mut screen);

    assert_eq!(screen.step, SetupStep::CollectionSource);
    // Curated options only (no per-provider import options).
    assert_eq!(screen.source_options, CollectionSourceOption::curated());
}

#[test]
fn test_collection_source_lists_import_option_per_configured_provider() {
    use crate::api::providers::kanban::{
        DetectedKanbanProvider, KanbanProviderType, ProviderStatus,
    };
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.detected_kanban_providers = vec![DetectedKanbanProvider {
        provider_type: KanbanProviderType::Linear,
        domain: "acme".to_string(),
        env_vars_found: vec!["OPERATOR_LINEAR_API_KEY".to_string()],
        email: None,
        status: ProviderStatus::Valid,
    }];
    screen.enter_collection_source();

    let import = screen
        .source_options
        .iter()
        .find(|o| matches!(o, CollectionSourceOption::ImportFromProvider(_)))
        .expect("an import option for the configured provider");
    assert_eq!(import.label(), "Import from Linear (acme)");

    // Selecting it stays on the step and surfaces a deferred notice.
    let idx = screen
        .source_options
        .iter()
        .position(|o| matches!(o, CollectionSourceOption::ImportFromProvider(_)))
        .unwrap();
    screen.source_state.select(Some(idx));
    screen.confirm();
    assert_eq!(screen.step, SetupStep::CollectionSource);
    assert!(screen.import_notice.is_some());
}

#[test]
fn test_collection_source_skips_provider_without_required_env_vars() {
    use crate::api::providers::kanban::{
        DetectedKanbanProvider, KanbanProviderType, ProviderStatus,
    };
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    // Jira needs domain + email + key; only a domain here -> no import option.
    screen.detected_kanban_providers = vec![DetectedKanbanProvider {
        provider_type: KanbanProviderType::Jira,
        domain: "acme.atlassian.net".to_string(),
        env_vars_found: vec!["OPERATOR_JIRA_DOMAIN".to_string()],
        email: None,
        status: ProviderStatus::Untested,
    }];
    screen.enter_collection_source();
    assert_eq!(screen.source_options, CollectionSourceOption::curated());
}

// ─── Worktree Preference Tests ────────────────────────────────────────────────

#[test]
fn test_setup_default_worktrees_is_false() {
    let screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    assert!(!screen.use_worktrees);
}

#[test]
fn test_setup_worktree_selection_toggle() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;

    // Start with worktrees disabled (default)
    assert!(!screen.use_worktrees);

    // Navigate down to worktrees option
    screen.select_next();
    assert_eq!(screen.worktree_state.selected(), Some(1));

    // Toggle selection
    screen.toggle_selection();
    assert!(screen.use_worktrees);
}

#[test]
fn test_worktree_option_labels() {
    assert_eq!(
        WorktreeOption::InPlace.label(),
        "Work in project directory (recommended)"
    );
    assert_eq!(WorktreeOption::Worktrees.label(), "Use isolated worktrees");
}

#[test]
fn test_worktree_option_to_use_worktrees() {
    assert!(!WorktreeOption::InPlace.to_use_worktrees());
    assert!(WorktreeOption::Worktrees.to_use_worktrees());
}

#[test]
fn test_worktree_option_from_use_worktrees() {
    assert_eq!(
        WorktreeOption::from_use_worktrees(false),
        WorktreeOption::InPlace
    );
    assert_eq!(
        WorktreeOption::from_use_worktrees(true),
        WorktreeOption::Worktrees
    );
}

#[test]
fn test_session_wrapper_option_labels() {
    assert_eq!(SessionWrapperOption::Tmux.label(), "Tmux (default)");
    assert_eq!(
        SessionWrapperOption::VSCode.label(),
        "VS Code Integrated Terminal"
    );
}

#[test]
fn test_session_wrapper_option_to_wrapper_type() {
    assert_eq!(
        SessionWrapperOption::Tmux.to_wrapper_type(),
        SessionWrapperType::Tmux
    );
    assert_eq!(
        SessionWrapperOption::VSCode.to_wrapper_type(),
        SessionWrapperType::Vscode
    );
}

// =============================================================================
// Admin password step
// =============================================================================

/// Drive the two password fields the way the key handler does.
fn type_password(screen: &mut SetupScreen, password: &str, confirm: &str) {
    use ratatui::crossterm::event::KeyCode;

    screen.password_field_focused = PasswordField::Password;
    for c in password.chars() {
        screen.handle_password_key(KeyCode::Char(c));
    }
    screen.password_field_focused = PasswordField::Confirm;
    for c in confirm.chars() {
        screen.handle_password_key(KeyCode::Char(c));
    }
}

fn at_admin_password() -> SetupScreen {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::AdminPassword;
    screen.selected_wrapper = SessionWrapperType::Vscode;
    screen
}

#[test]
fn test_admin_password_step_follows_worktree_preference() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;

    screen.confirm();
    assert_eq!(screen.step, SetupStep::AdminPassword);
}

#[test]
fn test_admin_password_skipped_when_admin_exists() {
    // Forward: straight past the step to the wrapper.
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.admin_password_configured = true;
    screen.step = SetupStep::WorktreePreference;
    screen.selected_wrapper = SessionWrapperType::Vscode;
    screen.confirm();
    assert_eq!(
        screen.step,
        SetupStep::VSCodeSetup,
        "an existing admin must not be offered a second bootstrap"
    );

    // Backward: the wrapper step returns past it too, or Esc would strand the
    // operator on a step that cannot be completed.
    screen.go_back();
    assert_eq!(screen.step, SetupStep::WorktreePreference);
}

#[test]
fn test_admin_password_empty_enter_skips() {
    let mut screen = at_admin_password();

    screen.confirm();

    assert_eq!(screen.step, SetupStep::VSCodeSetup);
    assert!(
        screen.admin_password.is_none(),
        "blank fields mean skip, not an empty password"
    );
    assert!(screen.password_error.is_none());
}

#[test]
fn test_admin_password_too_short_shows_error_and_stays() {
    let mut screen = at_admin_password();
    type_password(&mut screen, "short", "short");

    screen.confirm();

    assert_eq!(screen.step, SetupStep::AdminPassword, "must not advance");
    let error = screen.password_error.as_deref().unwrap_or_default();
    assert!(
        error.contains("12"),
        "error should name the length rule, got {error:?}"
    );
    assert!(screen.admin_password.is_none());
}

#[test]
fn test_admin_password_mismatch_shows_error_and_stays() {
    let mut screen = at_admin_password();
    type_password(
        &mut screen,
        "a properly long password",
        "a properly long passwerd",
    );

    screen.confirm();

    assert_eq!(screen.step, SetupStep::AdminPassword);
    assert_eq!(
        screen.password_error.as_deref(),
        Some("Passwords do not match")
    );
    assert!(screen.admin_password.is_none());
}

#[test]
fn test_admin_password_mismatch_is_reported_before_length() {
    // A mismatched pair that is also too short should say "do not match" -
    // telling someone their password is too short when they simply mistyped the
    // confirmation sends them to fix the wrong thing.
    let mut screen = at_admin_password();
    type_password(&mut screen, "short", "shorter");

    screen.confirm();

    assert_eq!(
        screen.password_error.as_deref(),
        Some("Passwords do not match")
    );
}

#[test]
fn test_admin_password_valid_advances_and_records_value() {
    let mut screen = at_admin_password();
    type_password(
        &mut screen,
        "a properly long password",
        "a properly long password",
    );

    screen.confirm();

    assert_eq!(screen.step, SetupStep::VSCodeSetup);
    assert_eq!(
        screen.admin_password.as_deref(),
        Some("a properly long password")
    );
    assert!(screen.password_error.is_none());
}

#[test]
fn test_admin_password_go_back_returns_to_worktree_preference() {
    let mut screen = at_admin_password();

    screen.go_back();

    assert_eq!(screen.step, SetupStep::WorktreePreference);
}

#[test]
fn test_admin_password_reaches_every_wrapper_step() {
    // The wrapper fan-out moved from WorktreePreference onto this step, so all
    // four destinations must still be reachable.
    for (wrapper, expected) in [
        (SessionWrapperType::Tmux, SetupStep::TmuxOnboarding),
        (SessionWrapperType::Vscode, SetupStep::VSCodeSetup),
        (SessionWrapperType::Cmux, SetupStep::CmuxSetup),
        (SessionWrapperType::Zellij, SetupStep::ZellijSetup),
    ] {
        let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
        screen.step = SetupStep::AdminPassword;
        screen.selected_wrapper = wrapper;

        screen.confirm();

        assert_eq!(screen.step, expected, "wrapper {wrapper:?} lost its step");
    }
}

#[test]
fn test_tab_switches_password_fields() {
    let mut screen = at_admin_password();
    assert_eq!(screen.password_field_focused, PasswordField::Password);

    screen.toggle_selection();
    assert_eq!(screen.password_field_focused, PasswordField::Confirm);

    screen.toggle_selection();
    assert_eq!(screen.password_field_focused, PasswordField::Password);
}

#[test]
fn test_editing_clears_a_previous_error() {
    use ratatui::crossterm::event::KeyCode;

    let mut screen = at_admin_password();
    type_password(&mut screen, "short", "short");
    screen.confirm();
    assert!(screen.password_error.is_some());

    screen.password_field_focused = PasswordField::Password;
    screen.handle_password_key(KeyCode::Char('x'));

    assert!(
        screen.password_error.is_none(),
        "a stale complaint must not sit under freshly typed input"
    );
}

#[test]
fn test_password_keys_reach_the_focused_field_only() {
    use ratatui::crossterm::event::KeyCode;

    let mut screen = at_admin_password();
    screen.password_field_focused = PasswordField::Password;
    screen.handle_password_key(KeyCode::Char('a'));
    screen.password_field_focused = PasswordField::Confirm;
    screen.handle_password_key(KeyCode::Char('b'));

    assert_eq!(screen.password.value(), "a");
    assert_eq!(screen.password_confirm.value(), "b");
}

#[test]
fn test_wizard_command_characters_are_typable_in_a_password() {
    use ratatui::crossterm::event::KeyCode;

    // `i` initializes, `c` quits, and `j`/`k`/space navigate when the wizard
    // handles them. Reaching the field they must be plain characters instead.
    let mut screen = at_admin_password();
    screen.password_field_focused = PasswordField::Password;
    for c in "ick j".chars() {
        screen.handle_password_key(KeyCode::Char(c));
    }

    assert_eq!(screen.password.value(), "ick j");
    assert_eq!(screen.step, SetupStep::AdminPassword, "still on the step");
}

// ─── Kanban step (defect A: the step used to be unreachable dead code) ──────

/// Move the git step's cursor to its trailing "continue without" row and
/// confirm, so a walk does not stall on a live connect attempt.
fn skip_git_provider(screen: &mut SetupScreen) {
    for _ in 0..SetupScreen::git_providers().len() {
        screen.select_next();
    }
    screen.confirm();
}

fn at_kanban_info() -> SetupScreen {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::KanbanInfo;
    screen
}

#[test]
fn test_kanban_info_follows_welcome() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::Welcome;

    screen.confirm();
    assert_eq!(screen.step, SetupStep::KanbanInfo);
}

#[test]
fn test_kanban_info_defaults_to_the_connect_row() {
    let screen = at_kanban_info();
    assert_eq!(screen.kanban_choice_state.selected(), Some(0));
}

#[test]
fn test_kanban_connect_requests_the_dialog_without_advancing() {
    let mut screen = at_kanban_info();

    screen.confirm();

    assert_eq!(
        screen.step,
        SetupStep::KanbanInfo,
        "the wizard waits on the dialog rather than moving on"
    );
    assert!(screen.take_kanban_dialog_request());
    assert!(
        !screen.take_kanban_dialog_request(),
        "the request is consumed once, so the dialog opens once"
    );
}

#[test]
fn test_kanban_skip_advances_without_requesting_the_dialog() {
    let mut screen = at_kanban_info();
    screen.select_next();

    screen.confirm();

    assert_eq!(screen.step, SetupStep::ModelServer);
    assert!(!screen.take_kanban_dialog_request());
}

#[test]
fn test_kanban_choice_selection_wraps() {
    let mut screen = at_kanban_info();

    screen.select_next();
    assert_eq!(screen.kanban_choice_state.selected(), Some(1));
    screen.select_next();
    assert_eq!(screen.kanban_choice_state.selected(), Some(0));
    screen.select_prev();
    assert_eq!(screen.kanban_choice_state.selected(), Some(1));
}

#[test]
fn test_collection_source_goes_back_to_git_provider() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::CollectionSource;

    screen.go_back();
    assert_eq!(screen.step, SetupStep::GitProvider);
}

/// Defect A regression: `KanbanProviderSetup` sat in the step enum, rendered a
/// screen and was never reachable, because the collection it keyed off was
/// never populated. Walking every wrapper branch proves each catalogued step
/// is actually selected by the state machine.
#[test]
fn test_wizard_walk_visits_every_catalog_step() {
    // Steps only reachable from a branch the walk below does not take.
    let conditional = [
        SetupStep::HostedCollectionFetch, // needs the hosted picker chosen
    ];

    let mut visited = std::collections::HashSet::new();
    for wrapper in [
        SessionWrapperType::Tmux,
        SessionWrapperType::Vscode,
        SessionWrapperType::Cmux,
        SessionWrapperType::Zellij,
    ] {
        let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
        screen.tmux_status = TmuxDetectionStatus::Available {
            version: "3.4".to_string(),
        };

        visited.insert(screen.step);
        for _ in 0..SetupStep::ALL.len() * 2 {
            if screen.step == SetupStep::Confirm {
                break;
            }
            // Skip past the kanban hand-off; the dialog is driven by the app.
            if screen.step == SetupStep::KanbanInfo {
                screen.select_next();
            }
            // Connecting shells out to provider CLIs; take the skip row.
            if screen.step == SetupStep::GitProvider {
                for _ in 0..SetupScreen::git_providers().len() {
                    screen.select_next();
                }
            }
            // `confirm` commits the highlighted wrapper, so steer the list.
            if screen.step == SetupStep::SessionWrapperChoice {
                let i = SessionWrapperOption::all()
                    .iter()
                    .position(|o| o.to_wrapper_type() == wrapper)
                    .expect("every wrapper is offered");
                screen.wrapper_state.select(Some(i));
            }
            screen.confirm();
            screen.take_kanban_dialog_request();
            visited.insert(screen.step);
        }
        assert_eq!(
            screen.step,
            SetupStep::Confirm,
            "{wrapper:?} branch never reached Confirm"
        );
    }

    for step in SetupStep::ALL {
        if conditional.contains(&step) {
            continue;
        }
        assert!(
            visited.contains(&step),
            "{step:?} is in the catalog but no wizard path reaches it"
        );
    }
}

// ─── Model server step (D1a) ───────────────────────────────────────────────

fn at_model_server() -> SetupScreen {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::ModelServer;
    screen
}

fn select_kind(screen: &mut SetupScreen, kind: ModelServerKind) {
    let i = SetupScreen::model_providers()
        .iter()
        .position(|(_, k)| *k == kind)
        .expect("kind is offered by the wizard");
    screen.model_server_state.select(Some(i));
}

#[test]
fn test_model_server_step_follows_kanban_skip() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::KanbanInfo;
    screen.select_next(); // "Skip for now"

    screen.confirm();
    assert_eq!(screen.step, SetupStep::ModelServer);
}

#[test]
fn test_model_server_advances_to_git_provider() {
    let mut screen = at_model_server();

    screen.confirm();
    assert_eq!(screen.step, SetupStep::GitProvider);
}

#[test]
fn test_model_server_go_back_returns_to_kanban_info() {
    let mut screen = at_model_server();

    screen.go_back();
    assert_eq!(screen.step, SetupStep::KanbanInfo);
}

#[test]
fn test_model_server_declares_nothing_by_default() {
    let screen = at_model_server();
    assert!(screen.declared_model_servers().is_empty());
}

#[test]
fn test_model_server_toggle_declares_the_highlighted_kind() {
    let mut screen = at_model_server();
    select_kind(&mut screen, ModelServerKind::Ollama);

    screen.toggle_selection();

    let declared = screen.declared_model_servers();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].kind, ModelServerKind::Ollama.slug());
    assert_eq!(
        declared[0].base_url.as_deref(),
        ModelServerKind::Ollama.default_base_url()
    );
}

#[test]
fn test_model_server_toggle_is_reversible() {
    let mut screen = at_model_server();
    select_kind(&mut screen, ModelServerKind::Ollama);

    screen.toggle_selection();
    screen.toggle_selection();

    assert!(screen.declared_model_servers().is_empty());
}

/// The key is referenced by env-var name; the secret never reaches config.
#[test]
fn test_declared_server_records_the_key_env_name_not_a_secret() {
    let mut screen = at_model_server();
    select_kind(&mut screen, ModelServerKind::AnthropicApi);

    screen.toggle_selection();

    let declared = screen.declared_model_servers();
    assert_eq!(
        declared[0].api_key_env.as_deref(),
        ModelServerKind::AnthropicApi.default_api_key_env()
    );
}

/// Declaring writes a `[[model_servers]]` entry from kind defaults, so every
/// offered provider must actually have a default base URL to write.
#[test]
fn test_every_offered_model_provider_is_connectable_from_defaults() {
    for (entry, kind) in SetupScreen::model_providers() {
        assert!(
            kind.connectable_from_defaults(),
            "{} is offered but has no default base URL to declare",
            entry.slug
        );
    }
}

#[test]
fn test_model_server_navigation_wraps_over_every_offered_provider() {
    let mut screen = at_model_server();
    let len = SetupScreen::model_providers().len();
    assert_eq!(screen.model_server_state.selected(), Some(0));

    for _ in 0..len {
        screen.select_next();
    }
    assert_eq!(screen.model_server_state.selected(), Some(0));

    screen.select_prev();
    assert_eq!(screen.model_server_state.selected(), Some(len - 1));
}

// ─── Git provider step (D1b) ───────────────────────────────────────────────

fn at_git_provider() -> SetupScreen {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::GitProvider;
    screen
}

#[test]
fn test_git_provider_step_follows_model_server() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::ModelServer;

    screen.confirm();
    assert_eq!(screen.step, SetupStep::GitProvider);
}

#[test]
fn test_git_provider_go_back_returns_to_model_server() {
    let mut screen = at_git_provider();

    screen.go_back();
    assert_eq!(screen.step, SetupStep::ModelServer);
}

/// The offered set comes from the integration catalog, so promoting a provider
/// into onboarding is a `SupportStatus` bump rather than an edit here.
#[test]
fn test_git_provider_offers_the_catalog_onboardable_set() {
    let slugs: Vec<&str> = SetupScreen::git_providers()
        .iter()
        .map(|e| e.slug)
        .collect();
    assert_eq!(slugs, vec!["github", "gitlab", "gitea"]);
}

/// Proto entries are unadvertised and undocumented, so onboarding must not
/// surface them - in either provider vertical.
#[test]
fn test_wizard_offers_no_proto_providers() {
    for slug in ["bitbucket", "azure", "forgejo"] {
        assert!(
            !SetupScreen::git_providers().iter().any(|e| e.slug == slug),
            "{slug} is Proto and must not be offered"
        );
    }
    for slug in ["openai-compat", "lmstudio"] {
        assert!(
            !SetupScreen::model_providers()
                .iter()
                .any(|(e, _)| e.slug == slug),
            "{slug} is Proto and must not be offered"
        );
    }
}

/// Every offered provider links out to its docs page from the wizard copy.
#[test]
fn test_offered_providers_are_documented() {
    for entry in SetupScreen::git_providers() {
        assert!(entry.docs_path.is_some(), "{}", entry.slug);
    }
    for (entry, _) in SetupScreen::model_providers() {
        assert!(entry.docs_path.is_some(), "{}", entry.slug);
    }
}

#[test]
fn test_git_provider_enter_requests_the_highlighted_provider() {
    let mut screen = at_git_provider();

    screen.confirm();

    assert_eq!(
        screen.step,
        SetupStep::GitProvider,
        "the wizard waits on the connect attempt rather than moving on"
    );
    assert_eq!(screen.take_git_connect_request().as_deref(), Some("github"));
    assert!(
        screen.take_git_connect_request().is_none(),
        "the request is consumed once"
    );
}

#[test]
fn test_git_provider_last_row_continues_without_a_provider() {
    let mut screen = at_git_provider();
    for _ in 0..SetupScreen::git_providers().len() {
        screen.select_next();
    }

    screen.confirm();

    assert_eq!(screen.step, SetupStep::CollectionSource);
    assert!(screen.take_git_connect_request().is_none());
}

#[test]
fn test_git_provider_navigation_includes_the_skip_row() {
    let mut screen = at_git_provider();
    let rows = SetupScreen::git_providers().len() + 1;

    for _ in 0..rows {
        screen.select_next();
    }
    assert_eq!(screen.git_provider_state.selected(), Some(0));

    screen.select_prev();
    assert_eq!(screen.git_provider_state.selected(), Some(rows - 1));
    assert!(screen.selected_git_provider().is_none());
}

#[test]
fn test_git_provider_status_is_recorded_per_provider() {
    let mut screen = at_git_provider();

    screen.set_git_provider_status("gitlab", "connected as octocat".to_string());

    assert_eq!(
        screen.git_provider_status.get("gitlab").map(String::as_str),
        Some("connected as octocat")
    );
    assert!(!screen.git_provider_status.contains_key("github"));
}
