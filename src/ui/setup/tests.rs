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

#[test]
fn test_setup_navigation_tmux_path() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::SessionWrapperChoice;
    screen.selected_wrapper = SessionWrapperType::Tmux;
    screen.wrapper_state.select(Some(0)); // Select tmux

    screen.confirm();
    assert_eq!(screen.step, SetupStep::TmuxOnboarding);
}

#[test]
fn test_setup_navigation_vscode_path() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::SessionWrapperChoice;
    screen.wrapper_state.select(Some(1)); // Select vscode (index 1)

    screen.confirm();
    assert_eq!(screen.step, SetupStep::VSCodeSetup);
    assert_eq!(screen.selected_wrapper, SessionWrapperType::Vscode);
}

#[test]
fn test_setup_tmux_onboarding_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::TmuxOnboarding;

    screen.go_back();
    assert_eq!(screen.step, SetupStep::SessionWrapperChoice);
}

#[test]
fn test_setup_vscode_setup_go_back() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
    screen.step = SetupStep::VSCodeSetup;

    screen.go_back();
    assert_eq!(screen.step, SetupStep::SessionWrapperChoice);
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

    // Should proceed to KanbanInfo because tmux is available
    screen.confirm();
    assert_eq!(screen.step, SetupStep::KanbanInfo);
}

#[test]
fn test_kanban_info_go_back_respects_wrapper_choice() {
    let mut screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());

    // Test tmux path
    screen.step = SetupStep::KanbanInfo;
    screen.selected_wrapper = SessionWrapperType::Tmux;
    screen.go_back();
    assert_eq!(screen.step, SetupStep::TmuxOnboarding);

    // Test vscode path
    screen.step = SetupStep::KanbanInfo;
    screen.selected_wrapper = SessionWrapperType::Vscode;
    screen.go_back();
    assert_eq!(screen.step, SetupStep::VSCodeSetup);
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
