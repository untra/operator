//! Tests for the setup wizard

use super::types::*;
use super::SetupScreen;
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

    // SessionWrapperChoice -> WorktreePreference
    screen.confirm();
    assert_eq!(screen.step, SetupStep::WorktreePreference);
}

#[test]
fn test_setup_navigation_tmux_path() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;
    screen.selected_wrapper = SessionWrapperType::Tmux;

    // WorktreePreference -> TmuxOnboarding (when tmux selected)
    screen.confirm();
    assert_eq!(screen.step, SetupStep::TmuxOnboarding);
}

#[test]
fn test_setup_navigation_vscode_path() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;
    screen.selected_wrapper = SessionWrapperType::Vscode;

    // WorktreePreference -> VSCodeSetup (when vscode selected)
    screen.confirm();
    assert_eq!(screen.step, SetupStep::VSCodeSetup);
}

#[test]
fn test_setup_worktree_preference_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::WorktreePreference;

    screen.go_back();
    assert_eq!(screen.step, SetupStep::SessionWrapperChoice);
}

#[test]
fn test_setup_tmux_onboarding_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::TmuxOnboarding;

    // TmuxOnboarding -> WorktreePreference
    screen.go_back();
    assert_eq!(screen.step, SetupStep::WorktreePreference);
}

#[test]
fn test_setup_vscode_setup_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::VSCodeSetup;

    // VSCodeSetup -> WorktreePreference
    screen.go_back();
    assert_eq!(screen.step, SetupStep::WorktreePreference);
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
fn test_kanban_info_no_providers_advances_to_collection_source() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::KanbanInfo;
    // No valid providers -> straight to the collection source step.
    screen.confirm();
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
