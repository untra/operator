//! Startup setup screen when .tickets/ directory is not found

use std::collections::HashMap;

use crate::agents::{SystemTmuxClient, TmuxClient, TmuxError};
use crate::config::{CollectionPreset, SessionWrapperType};
use ratatui::{widgets::ListState, Frame};

pub mod steps;
pub mod types;

pub use types::*;

#[cfg(test)]
mod tests;

/// Setup screen shown when .tickets/ directory doesn't exist
pub struct SetupScreen {
    /// Whether the screen is visible
    pub visible: bool,
    /// Current step in the setup process
    pub step: SetupStep,
    /// Current selection for confirmation: true = Initialize, false = Cancel
    pub confirm_selected: bool,
    /// Path where tickets directory will be created
    pub(crate) tickets_path: String,
    /// Detected LLM tools (from LlmToolsConfig)
    pub(crate) detected_tools: Vec<DetectedToolInfo>,
    /// Projects grouped by tool
    pub(crate) projects_by_tool: HashMap<String, Vec<String>>,
    /// Selected collection preset
    pub selected_preset: CollectionPreset,
    /// Custom issuetype collection (only used when preset is Custom)
    pub custom_collection: Vec<String>,
    /// List state for collection source selection
    pub(crate) source_state: ListState,
    /// List state for custom collection selection
    pub(crate) collection_state: ListState,
    /// Whether we came from custom selection (for back navigation)
    pub(crate) from_custom: bool,
    /// Selected optional fields to include in TASK (and other types)
    pub task_optional_fields: Vec<String>,
    /// List state for field configuration selection
    pub(crate) field_state: ListState,
    /// Startup ticket options (ASSESS, AGENT-SETUP, PROJECT-INIT)
    pub startup_ticket_options: Vec<StartupTicketOption>,
    /// List state for startup ticket selection
    pub(crate) startup_state: ListState,
    /// Acceptance criteria text (editable during setup)
    pub acceptance_criteria_text: String,
    // ─── Kanban Setup State ─────────────────────────────────────────────────────
    /// Detected kanban providers from environment variables
    pub detected_kanban_providers: Vec<crate::api::providers::kanban::DetectedKanbanProvider>,
    /// Indices of providers with valid credentials
    pub valid_kanban_providers: Vec<usize>,
    /// Projects fetched from current provider being configured
    pub kanban_projects:
        super::paginated_list::PaginatedList<crate::api::providers::kanban::ProjectInfo>,
    /// Issue types for the currently selected project
    pub kanban_issue_types: Vec<String>,
    /// Member count for the currently selected project
    pub kanban_member_count: usize,
    /// Whether kanban detection/testing has run
    pub kanban_detection_complete: bool,
    /// Whether the user chose to skip kanban setup
    pub kanban_skipped: bool,
    // ─── Session Wrapper Setup State ────────────────────────────────────────────
    /// Selected session wrapper type
    pub selected_wrapper: SessionWrapperType,
    /// List state for wrapper selection
    pub(crate) wrapper_state: ListState,
    /// Tmux availability status (checked during TmuxOnboarding step)
    pub tmux_status: TmuxDetectionStatus,
    /// VS Code extension status (checked during VSCodeSetup step)
    pub vscode_status: VSCodeDetectionStatus,
    // ─── Git Worktree Setup State ──────────────────────────────────────────────
    /// Whether to use git worktrees for ticket isolation (default: false)
    pub use_worktrees: bool,
    /// List state for worktree option selection
    pub(crate) worktree_state: ListState,
}

impl SetupScreen {
    /// Create a new setup screen
    pub fn new(
        tickets_path: String,
        detected_tools: Vec<DetectedToolInfo>,
        projects_by_tool: HashMap<String, Vec<String>>,
    ) -> Self {
        let mut source_state = ListState::default();
        source_state.select(Some(0));

        let mut collection_state = ListState::default();
        collection_state.select(Some(0));

        let mut field_state = ListState::default();
        field_state.select(Some(0));

        let mut startup_state = ListState::default();
        startup_state.select(Some(0));

        let mut wrapper_state = ListState::default();
        wrapper_state.select(Some(0));

        let mut worktree_state = ListState::default();
        worktree_state.select(Some(0));

        Self {
            visible: true,
            step: SetupStep::Welcome,
            confirm_selected: true, // Default to Initialize
            tickets_path,
            detected_tools,
            projects_by_tool,
            selected_preset: CollectionPreset::DevopsKanban,
            custom_collection: ALL_ISSUE_TYPES.iter().map(|s| s.to_string()).collect(),
            source_state,
            collection_state,
            from_custom: false,
            // Default: all optional fields enabled
            task_optional_fields: TASK_OPTIONAL_FIELDS
                .iter()
                .map(|(name, _)| name.to_string())
                .collect(),
            field_state,
            startup_ticket_options: StartupTicketOption::all(),
            startup_state,
            acceptance_criteria_text: include_str!("../../templates/ACCEPTANCE_CRITERIA.md")
                .to_string(),
            // Kanban setup state
            detected_kanban_providers: Vec::new(),
            valid_kanban_providers: Vec::new(),
            kanban_projects: super::paginated_list::PaginatedList::new(8),
            kanban_issue_types: Vec::new(),
            kanban_member_count: 0,
            kanban_detection_complete: false,
            kanban_skipped: false,
            // Session wrapper state
            selected_wrapper: SessionWrapperType::Tmux,
            wrapper_state,
            tmux_status: TmuxDetectionStatus::NotChecked,
            vscode_status: VSCodeDetectionStatus::NotChecked,
            // Git worktree state
            use_worktrees: false,
            worktree_state,
        }
    }

    /// Get the selected collection preset
    pub fn preset(&self) -> CollectionPreset {
        self.selected_preset
    }

    /// Get the effective issuetype collection based on preset
    pub fn collection(&self) -> Vec<String> {
        match self.selected_preset {
            CollectionPreset::Custom => self.custom_collection.clone(),
            _ => self.selected_preset.issue_types(),
        }
    }

    /// Get the configured optional fields for TASK (and propagation to other types)
    pub fn configured_task_fields(&self) -> Vec<String> {
        self.task_optional_fields.clone()
    }

    /// Get the selected startup ticket types to create
    pub fn selected_startup_tickets(&self) -> Vec<String> {
        self.startup_ticket_options
            .iter()
            .filter(|opt| opt.enabled)
            .map(|opt| opt.key.to_string())
            .collect()
    }

    /// Get the currently selected source option
    fn selected_source(&self) -> Option<CollectionSourceOption> {
        self.source_state
            .selected()
            .map(|i| CollectionSourceOption::all()[i])
    }

    /// Toggle selection (Space key)
    pub fn toggle_selection(&mut self) {
        match self.step {
            SetupStep::CustomCollection => {
                // Toggle the currently highlighted collection item
                if let Some(i) = self.collection_state.selected() {
                    let types = ALL_ISSUE_TYPES;
                    if i < types.len() {
                        let type_str = types[i].to_string();
                        if self.custom_collection.contains(&type_str) {
                            self.custom_collection.retain(|t| t != &type_str);
                        } else {
                            self.custom_collection.push(type_str);
                        }
                    }
                }
            }
            SetupStep::TaskFieldConfig => {
                // Toggle the currently highlighted field
                if let Some(i) = self.field_state.selected() {
                    if i < TASK_OPTIONAL_FIELDS.len() {
                        let field_name = TASK_OPTIONAL_FIELDS[i].0.to_string();
                        if self.task_optional_fields.contains(&field_name) {
                            self.task_optional_fields.retain(|f| f != &field_name);
                        } else {
                            self.task_optional_fields.push(field_name);
                        }
                    }
                }
            }
            SetupStep::SessionWrapperChoice => {
                // Select the currently highlighted wrapper option
                if let Some(i) = self.wrapper_state.selected() {
                    let options = SessionWrapperOption::all();
                    if i < options.len() {
                        self.selected_wrapper = options[i].to_wrapper_type();
                    }
                }
            }
            SetupStep::WorktreePreference => {
                // Select the currently highlighted worktree option
                if let Some(i) = self.worktree_state.selected() {
                    let options = WorktreeOption::all();
                    if i < options.len() {
                        self.use_worktrees = options[i].to_use_worktrees();
                    }
                }
            }
            SetupStep::StartupTickets => {
                // Toggle the currently highlighted startup ticket option
                if let Some(i) = self.startup_state.selected() {
                    if i < self.startup_ticket_options.len() {
                        self.startup_ticket_options[i].enabled =
                            !self.startup_ticket_options[i].enabled;
                    }
                }
            }
            SetupStep::Confirm => {
                self.confirm_selected = !self.confirm_selected;
            }
            _ => {}
        }
    }

    /// Move to next item in list
    pub fn select_next(&mut self) {
        match self.step {
            SetupStep::CollectionSource => {
                let len = CollectionSourceOption::all().len();
                let i = self.source_state.selected().map_or(0, |i| (i + 1) % len);
                self.source_state.select(Some(i));
            }
            SetupStep::CustomCollection => {
                let len = ALL_ISSUE_TYPES.len();
                let i = self
                    .collection_state
                    .selected()
                    .map_or(0, |i| (i + 1) % len);
                self.collection_state.select(Some(i));
            }
            SetupStep::TaskFieldConfig => {
                let len = TASK_OPTIONAL_FIELDS.len();
                let i = self.field_state.selected().map_or(0, |i| (i + 1) % len);
                self.field_state.select(Some(i));
            }
            SetupStep::SessionWrapperChoice => {
                let len = SessionWrapperOption::all().len();
                let i = self.wrapper_state.selected().map_or(0, |i| (i + 1) % len);
                self.wrapper_state.select(Some(i));
            }
            SetupStep::WorktreePreference => {
                let len = WorktreeOption::all().len();
                let i = self.worktree_state.selected().map_or(0, |i| (i + 1) % len);
                self.worktree_state.select(Some(i));
            }
            SetupStep::StartupTickets => {
                let len = self.startup_ticket_options.len();
                let i = self.startup_state.selected().map_or(0, |i| (i + 1) % len);
                self.startup_state.select(Some(i));
            }
            _ => {}
        }
    }

    /// Move to previous item in list
    pub fn select_prev(&mut self) {
        match self.step {
            SetupStep::CollectionSource => {
                let len = CollectionSourceOption::all().len();
                let i =
                    self.source_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.source_state.select(Some(i));
            }
            SetupStep::CustomCollection => {
                let len = ALL_ISSUE_TYPES.len();
                let i = self.collection_state.selected().map_or(0, |i| {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                });
                self.collection_state.select(Some(i));
            }
            SetupStep::TaskFieldConfig => {
                let len = TASK_OPTIONAL_FIELDS.len();
                let i = self
                    .field_state
                    .selected()
                    .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.field_state.select(Some(i));
            }
            SetupStep::SessionWrapperChoice => {
                let len = SessionWrapperOption::all().len();
                let i =
                    self.wrapper_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.wrapper_state.select(Some(i));
            }
            SetupStep::WorktreePreference => {
                let len = WorktreeOption::all().len();
                let i =
                    self.worktree_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.worktree_state.select(Some(i));
            }
            SetupStep::StartupTickets => {
                let len = self.startup_ticket_options.len();
                let i =
                    self.startup_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.startup_state.select(Some(i));
            }
            _ => {}
        }
    }

    /// Proceed to next step or confirm (Enter key)
    pub fn confirm(&mut self) -> SetupResult {
        match self.step {
            SetupStep::Welcome => {
                self.step = SetupStep::CollectionSource;
                SetupResult::Continue
            }
            SetupStep::CollectionSource => {
                if let Some(source) = self.selected_source() {
                    match source {
                        CollectionSourceOption::Simple => {
                            self.selected_preset = CollectionPreset::Simple;
                            self.from_custom = false;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::DevKanban => {
                            self.selected_preset = CollectionPreset::DevKanban;
                            self.from_custom = false;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::DevopsKanban => {
                            self.selected_preset = CollectionPreset::DevopsKanban;
                            self.from_custom = false;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::ImportJira => SetupResult::ExitUnimplemented(
                            "Jira import is not yet implemented".to_string(),
                        ),
                        CollectionSourceOption::ImportNotion => SetupResult::ExitUnimplemented(
                            "Notion import is not yet implemented".to_string(),
                        ),
                        CollectionSourceOption::CustomSelection => {
                            self.selected_preset = CollectionPreset::Custom;
                            self.from_custom = true;
                            self.step = SetupStep::CustomCollection;
                            SetupResult::Continue
                        }
                    }
                } else {
                    SetupResult::Continue
                }
            }
            SetupStep::CustomCollection => {
                if !self.custom_collection.is_empty() {
                    self.step = SetupStep::TaskFieldConfig;
                }
                SetupResult::Continue
            }
            SetupStep::TaskFieldConfig => {
                self.step = SetupStep::SessionWrapperChoice;
                SetupResult::Continue
            }
            SetupStep::SessionWrapperChoice => {
                // Select the wrapper from the highlighted option
                if let Some(i) = self.wrapper_state.selected() {
                    let options = SessionWrapperOption::all();
                    if i < options.len() {
                        self.selected_wrapper = options[i].to_wrapper_type();
                    }
                }
                // Navigate to worktree preference step
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::WorktreePreference => {
                // Select the worktree option from the highlighted option
                if let Some(i) = self.worktree_state.selected() {
                    let options = WorktreeOption::all();
                    if i < options.len() {
                        self.use_worktrees = options[i].to_use_worktrees();
                    }
                }
                // Navigate to the appropriate next step based on wrapper choice
                match self.selected_wrapper {
                    SessionWrapperType::Tmux => {
                        // Check tmux availability when entering TmuxOnboarding
                        self.check_tmux_availability();
                        self.step = SetupStep::TmuxOnboarding;
                    }
                    SessionWrapperType::Vscode => {
                        self.step = SetupStep::VSCodeSetup;
                    }
                }
                SetupResult::Continue
            }
            SetupStep::TmuxOnboarding => {
                // Only allow proceeding if tmux is available
                if matches!(self.tmux_status, TmuxDetectionStatus::Available { .. }) {
                    // Detect kanban providers if not already done
                    if !self.kanban_detection_complete {
                        self.detected_kanban_providers =
                            crate::api::providers::kanban::detect_kanban_env_vars();
                        self.kanban_detection_complete = true;
                    }
                    self.step = SetupStep::KanbanInfo;
                }
                // If tmux not available, stay on this step (user must install or go back)
                SetupResult::Continue
            }
            SetupStep::VSCodeSetup => {
                // For now, allow proceeding (extension check will be added later)
                // Detect kanban providers if not already done
                if !self.kanban_detection_complete {
                    self.detected_kanban_providers =
                        crate::api::providers::kanban::detect_kanban_env_vars();
                    self.kanban_detection_complete = true;
                }
                self.step = SetupStep::KanbanInfo;
                SetupResult::Continue
            }
            SetupStep::KanbanInfo => {
                // If no valid providers or skipped, go to acceptance criteria
                if self.valid_kanban_providers.is_empty() || self.kanban_skipped {
                    self.step = SetupStep::AcceptanceCriteria;
                } else {
                    // Start with first valid provider
                    self.step = SetupStep::KanbanProviderSetup { provider_index: 0 };
                }
                SetupResult::Continue
            }
            SetupStep::KanbanProviderSetup { provider_index } => {
                // Move to next provider or acceptance criteria
                let next_index = provider_index + 1;
                if next_index < self.valid_kanban_providers.len() {
                    self.step = SetupStep::KanbanProviderSetup {
                        provider_index: next_index,
                    };
                } else {
                    self.step = SetupStep::AcceptanceCriteria;
                }
                SetupResult::Continue
            }
            SetupStep::AcceptanceCriteria => {
                self.step = SetupStep::StartupTickets;
                SetupResult::Continue
            }
            SetupStep::StartupTickets => {
                self.step = SetupStep::Confirm;
                SetupResult::Continue
            }
            SetupStep::Confirm => {
                if self.confirm_selected {
                    SetupResult::Initialize
                } else {
                    SetupResult::Cancel
                }
            }
        }
    }

    /// Go back to previous step (Esc key)
    pub fn go_back(&mut self) -> SetupResult {
        match self.step {
            SetupStep::Welcome => SetupResult::Cancel,
            SetupStep::CollectionSource => {
                self.step = SetupStep::Welcome;
                SetupResult::Continue
            }
            SetupStep::CustomCollection => {
                self.step = SetupStep::CollectionSource;
                SetupResult::Continue
            }
            SetupStep::TaskFieldConfig => {
                if self.from_custom {
                    self.step = SetupStep::CustomCollection;
                } else {
                    self.step = SetupStep::CollectionSource;
                }
                SetupResult::Continue
            }
            SetupStep::SessionWrapperChoice => {
                self.step = SetupStep::TaskFieldConfig;
                SetupResult::Continue
            }
            SetupStep::WorktreePreference => {
                self.step = SetupStep::SessionWrapperChoice;
                SetupResult::Continue
            }
            SetupStep::TmuxOnboarding => {
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::VSCodeSetup => {
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::KanbanInfo => {
                // Go back to the appropriate wrapper setup step
                match self.selected_wrapper {
                    SessionWrapperType::Tmux => self.step = SetupStep::TmuxOnboarding,
                    SessionWrapperType::Vscode => self.step = SetupStep::VSCodeSetup,
                }
                SetupResult::Continue
            }
            SetupStep::KanbanProviderSetup { provider_index } => {
                if provider_index > 0 {
                    self.step = SetupStep::KanbanProviderSetup {
                        provider_index: provider_index - 1,
                    };
                } else {
                    self.step = SetupStep::KanbanInfo;
                }
                SetupResult::Continue
            }
            SetupStep::AcceptanceCriteria => {
                // Go back to last kanban provider setup or kanban info
                if !self.valid_kanban_providers.is_empty() && !self.kanban_skipped {
                    let last_index = self.valid_kanban_providers.len() - 1;
                    self.step = SetupStep::KanbanProviderSetup {
                        provider_index: last_index,
                    };
                } else {
                    self.step = SetupStep::KanbanInfo;
                }
                SetupResult::Continue
            }
            SetupStep::StartupTickets => {
                self.step = SetupStep::AcceptanceCriteria;
                SetupResult::Continue
            }
            SetupStep::Confirm => {
                self.step = SetupStep::StartupTickets;
                SetupResult::Continue
            }
        }
    }

    /// Render the setup screen
    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        match self.step.clone() {
            SetupStep::Welcome => self.render_welcome_step(frame),
            SetupStep::CollectionSource => self.render_collection_source_step(frame),
            SetupStep::CustomCollection => self.render_custom_collection_step(frame),
            SetupStep::TaskFieldConfig => self.render_task_field_config_step(frame),
            SetupStep::SessionWrapperChoice => self.render_session_wrapper_choice_step(frame),
            SetupStep::WorktreePreference => self.render_worktree_preference_step(frame),
            SetupStep::TmuxOnboarding => self.render_tmux_onboarding_step(frame),
            SetupStep::VSCodeSetup => self.render_vscode_setup_step(frame),
            SetupStep::KanbanInfo => self.render_kanban_info_step(frame),
            SetupStep::KanbanProviderSetup { provider_index } => {
                self.render_kanban_provider_setup_step(frame, provider_index)
            }
            SetupStep::AcceptanceCriteria => self.render_acceptance_criteria_step(frame),
            SetupStep::StartupTickets => self.render_startup_tickets_step(frame),
            SetupStep::Confirm => self.render_confirm_step(frame),
        }
    }

    /// Check tmux availability and update status
    pub fn check_tmux_availability(&mut self) {
        let client = SystemTmuxClient::new();
        match client.check_available() {
            Ok(version) => {
                // Minimum version 2.1 for the features we use
                const MIN_MAJOR: u32 = 2;
                const MIN_MINOR: u32 = 1;

                if version.meets_minimum(MIN_MAJOR, MIN_MINOR) {
                    self.tmux_status = TmuxDetectionStatus::Available {
                        version: version.raw,
                    };
                } else {
                    self.tmux_status = TmuxDetectionStatus::VersionTooOld {
                        current: version.raw,
                        required: format!("{}.{}", MIN_MAJOR, MIN_MINOR),
                    };
                }
            }
            Err(TmuxError::NotInstalled) => {
                self.tmux_status = TmuxDetectionStatus::NotInstalled;
            }
            Err(_) => {
                self.tmux_status = TmuxDetectionStatus::NotInstalled;
            }
        }
    }

    /// Re-check tmux availability (for [R] key binding)
    #[allow(dead_code)] // Will be connected to [R] key handler in app event loop
    pub fn recheck_tmux(&mut self) {
        self.check_tmux_availability();
    }

    // ─── Kanban Setup Helper Methods ────────────────────────────────────────────
    // These methods are called from app.rs during async credential testing
    // and project fetching. They are infrastructure for the full kanban setup flow.

    /// Skip kanban setup entirely
    #[allow(dead_code)]
    pub fn skip_kanban(&mut self) {
        self.kanban_skipped = true;
    }

    /// Mark a provider as valid after testing
    #[allow(dead_code)]
    pub fn mark_provider_valid(&mut self, index: usize) {
        use crate::api::providers::kanban::ProviderStatus;

        if let Some(provider) = self.detected_kanban_providers.get_mut(index) {
            provider.status = ProviderStatus::Valid;
            if !self.valid_kanban_providers.contains(&index) {
                self.valid_kanban_providers.push(index);
            }
        }
    }

    /// Mark a provider as failed after testing
    #[allow(dead_code)]
    pub fn mark_provider_failed(&mut self, index: usize, error: String) {
        use crate::api::providers::kanban::ProviderStatus;

        if let Some(provider) = self.detected_kanban_providers.get_mut(index) {
            provider.status = ProviderStatus::Failed { error };
        }
    }

    /// Set the projects for the current kanban provider
    #[allow(dead_code)]
    pub fn set_kanban_projects(
        &mut self,
        projects: Vec<crate::api::providers::kanban::ProjectInfo>,
    ) {
        self.kanban_projects.set_items(projects);
    }

    /// Get the selected kanban project
    #[allow(dead_code)]
    pub fn selected_kanban_project(&self) -> Option<&crate::api::providers::kanban::ProjectInfo> {
        self.kanban_projects.selected_item()
    }

    /// Set the preview info for the selected project
    #[allow(dead_code)]
    pub fn set_kanban_preview(&mut self, issue_types: Vec<String>, member_count: usize) {
        self.kanban_issue_types = issue_types;
        self.kanban_member_count = member_count;
    }
}
