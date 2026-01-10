//! Startup setup screen when .tickets/ directory is not found

use std::collections::HashMap;

use crate::agents::{SystemTmuxClient, TmuxClient, TmuxError};
use crate::config::{CollectionPreset, SessionWrapperType};
use crate::projects::TOOL_MARKERS;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Simplified tool info for display on the welcome screen
#[derive(Debug, Clone)]
pub struct DetectedToolInfo {
    pub name: String,
    pub version: String,
    pub model_count: usize,
}

/// Available issuetype collections (all known types)
pub const ALL_ISSUE_TYPES: &[&str] = &["TASK", "FEAT", "FIX", "SPIKE", "INV"];

/// Optional fields that can be configured for TASK (and propagated to other types)
/// Note: 'summary' and 'description' remain required, 'id' is auto-generated
pub const TASK_OPTIONAL_FIELDS: &[(&str, &str)] = &[
    ("priority", "Priority level (P0-critical to P3-low)"),
    ("points", "Story points estimate (0 or greater)"),
    ("user_story", "User story or background context"),
];

/// Collection source options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSourceOption {
    Simple,
    DevKanban,
    DevopsKanban,
    ImportJira,
    ImportNotion,
    CustomSelection,
}

impl CollectionSourceOption {
    pub fn all() -> &'static [CollectionSourceOption] {
        &[
            CollectionSourceOption::Simple,
            CollectionSourceOption::DevKanban,
            CollectionSourceOption::DevopsKanban,
            CollectionSourceOption::ImportJira,
            CollectionSourceOption::ImportNotion,
            CollectionSourceOption::CustomSelection,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Simple",
            CollectionSourceOption::DevKanban => "Dev Kanban",
            CollectionSourceOption::DevopsKanban => "DevOps Kanban",
            CollectionSourceOption::ImportJira => "Import from Jira",
            CollectionSourceOption::ImportNotion => "Import from Notion",
            CollectionSourceOption::CustomSelection => "Custom Selection",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Just TASK - minimal setup for general work",
            CollectionSourceOption::DevKanban => "3 issue types: TASK, FEAT, FIX",
            CollectionSourceOption::DevopsKanban => "5 issue types: TASK, SPIKE, INV, FEAT, FIX",
            CollectionSourceOption::ImportJira => "(Coming soon)",
            CollectionSourceOption::ImportNotion => "(Coming soon)",
            CollectionSourceOption::CustomSelection => "Choose individual issue types",
        }
    }

    pub fn is_unimplemented(&self) -> bool {
        matches!(
            self,
            CollectionSourceOption::ImportJira | CollectionSourceOption::ImportNotion
        )
    }
}

/// Result of setup screen actions
#[derive(Debug, Clone)]
pub enum SetupResult {
    /// Continue to next step
    Continue,
    /// Cancel/quit setup
    Cancel,
    /// Exit with unimplemented message
    ExitUnimplemented(String),
    /// Setup complete, initialize
    Initialize,
}

/// Setup screen shown when .tickets/ directory doesn't exist
pub struct SetupScreen {
    /// Whether the screen is visible
    pub visible: bool,
    /// Current step in the setup process
    pub step: SetupStep,
    /// Current selection for confirmation: true = Initialize, false = Cancel
    pub confirm_selected: bool,
    /// Path where tickets directory will be created
    tickets_path: String,
    /// Detected LLM tools (from LlmToolsConfig)
    detected_tools: Vec<DetectedToolInfo>,
    /// Projects grouped by tool
    projects_by_tool: HashMap<String, Vec<String>>,
    /// Selected collection preset
    pub selected_preset: CollectionPreset,
    /// Custom issuetype collection (only used when preset is Custom)
    pub custom_collection: Vec<String>,
    /// List state for collection source selection
    source_state: ListState,
    /// List state for custom collection selection
    collection_state: ListState,
    /// Whether we came from custom selection (for back navigation)
    from_custom: bool,
    /// Selected optional fields to include in TASK (and other types)
    pub task_optional_fields: Vec<String>,
    /// List state for field configuration selection
    field_state: ListState,
    /// Startup ticket options (ASSESS, AGENT-SETUP, PROJECT-INIT)
    pub startup_ticket_options: Vec<StartupTicketOption>,
    /// List state for startup ticket selection
    startup_state: ListState,
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
    wrapper_state: ListState,
    /// Tmux availability status (checked during TmuxOnboarding step)
    pub tmux_status: TmuxDetectionStatus,
    /// VS Code extension status (checked during VSCodeSetup step)
    pub vscode_status: VSCodeDetectionStatus,
}

/// Startup ticket options for project initialization
#[derive(Debug, Clone)]
pub struct StartupTicketOption {
    /// Key identifier for the ticket type (e.g., "assess", "agent_setup")
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub enabled: bool,
}

impl StartupTicketOption {
    pub fn all() -> Vec<StartupTicketOption> {
        vec![
            StartupTicketOption {
                key: "assess",
                name: "ASSESS tickets",
                description: "Scan projects for catalog-info.yaml, create if missing",
                enabled: true,
            },
            StartupTicketOption {
                key: "agent_setup",
                name: "AGENT-SETUP tickets",
                description: "Configure Claude agents for each project",
                enabled: false,
            },
            StartupTicketOption {
                key: "project_init",
                name: "PROJECT-INIT tickets",
                description: "Run both ASSESS and AGENT-SETUP for each project",
                enabled: false,
            },
        ]
    }
}

/// Tmux availability detection status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmuxDetectionStatus {
    /// Not yet checked
    NotChecked,
    /// Tmux is available with the given version
    Available { version: String },
    /// Tmux is not installed
    NotInstalled,
    /// Tmux is installed but version is too old
    VersionTooOld { current: String, required: String },
}

impl Default for TmuxDetectionStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// VS Code extension detection status
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants will be used when VS Code extension support is implemented
pub enum VSCodeDetectionStatus {
    /// Not yet checked
    NotChecked,
    /// Currently checking connection
    Checking,
    /// Connected to extension with the given version
    Connected { version: String },
    /// Extension not reachable
    NotReachable,
}

impl Default for VSCodeDetectionStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// Session wrapper options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionWrapperOption {
    Tmux,
    VSCode,
}

impl SessionWrapperOption {
    pub fn all() -> &'static [SessionWrapperOption] {
        &[SessionWrapperOption::Tmux, SessionWrapperOption::VSCode]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SessionWrapperOption::Tmux => "Tmux (default)",
            SessionWrapperOption::VSCode => "VS Code Integrated Terminal",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SessionWrapperOption::Tmux => "Run agents in standalone tmux sessions",
            SessionWrapperOption::VSCode => {
                "Run agents in VS Code terminal panels (requires extension)"
            }
        }
    }

    pub fn to_wrapper_type(self) -> SessionWrapperType {
        match self {
            SessionWrapperOption::Tmux => SessionWrapperType::Tmux,
            SessionWrapperOption::VSCode => SessionWrapperType::Vscode,
        }
    }
}

/// Steps in the setup process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupStep {
    /// Welcome splash screen with discovered projects
    Welcome,
    /// Select template collection source
    CollectionSource,
    /// Custom issuetype selection (optional)
    CustomCollection,
    /// Configure TASK optional fields
    TaskFieldConfig,
    /// Select session wrapper (tmux or vscode)
    SessionWrapperChoice,
    /// Tmux onboarding/help (only shown if tmux selected)
    TmuxOnboarding,
    /// VS Code extension setup (only shown if vscode selected)
    VSCodeSetup,
    /// Kanban integration info and provider detection
    KanbanInfo,
    /// Per-provider setup with project selection (index into valid_providers)
    KanbanProviderSetup { provider_index: usize },
    /// Review and configure acceptance criteria
    AcceptanceCriteria,
    /// Optional startup tickets creation
    StartupTickets,
    /// Confirm initialization
    Confirm,
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
            acceptance_criteria_text: include_str!("../templates/ACCEPTANCE_CRITERIA.md")
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
                // Navigate to the appropriate next step
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
            SetupStep::TmuxOnboarding => {
                self.step = SetupStep::SessionWrapperChoice;
                SetupResult::Continue
            }
            SetupStep::VSCodeSetup => {
                self.step = SetupStep::SessionWrapperChoice;
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

    fn render_welcome_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Operator!",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Workspace Setup "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Description
                Constraint::Length(1), // Spacer
                Constraint::Length(6), // Detected LLM Tools
                Constraint::Length(1), // Spacer
                Constraint::Min(6),    // Discovered projects by tool
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Path info
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Operator!",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![Line::from("A TUI for orchestrating LLM Code agents.")])
            .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Detected LLM Tools
        let mut tools_text = vec![Line::from(Span::styled(
            "Detected LLM Tools:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        // Show each known tool with detection status
        for (tool_name, _marker) in TOOL_MARKERS {
            let detected = self.detected_tools.iter().find(|t| t.name == *tool_name);

            let line = if let Some(tool) = detected {
                Line::from(vec![
                    Span::styled("  + ", Style::default().fg(Color::Green)),
                    Span::styled(
                        tool_name.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" (v{}) - {} models", tool.version, tool.model_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  - ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tool_name.to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(" - not installed", Style::default().fg(Color::DarkGray)),
                ])
            };
            tools_text.push(line);
        }
        frame.render_widget(Paragraph::new(tools_text), chunks[4]);

        // Discovered projects by tool
        let mut projects_text = vec![Line::from(Span::styled(
            "Discovered Projects:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        let mut has_any_projects = false;
        for (tool_name, _marker) in TOOL_MARKERS {
            if let Some(projects) = self.projects_by_tool.get(*tool_name) {
                if !projects.is_empty() {
                    has_any_projects = true;
                    let project_list = projects.join(", ");
                    projects_text.push(Line::from(vec![
                        Span::styled(
                            format!("  {}: ", tool_name),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(project_list, Style::default().fg(Color::Green)),
                    ]));
                }
            }
        }

        if !has_any_projects {
            projects_text.push(Line::from(Span::styled(
                "  (no projects with marker files found)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        frame.render_widget(Paragraph::new(projects_text), chunks[6]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[8]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[9]);
    }

    fn render_collection_source_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Select Template Collection ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Options list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Choose Template Source",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions =
            Paragraph::new(vec![Line::from("Use arrows to navigate, Enter to select")])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Options list
        let items: Vec<ListItem> = CollectionSourceOption::all()
            .iter()
            .map(|opt| {
                let style = if opt.is_unimplemented() {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        opt.label(),
                        style.add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(opt.description(), Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[2], &mut self.source_state);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    fn render_custom_collection_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Operator!",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Setup - Issue Types "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Collection list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Select Issue Types",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Collection list
        let items: Vec<ListItem> = ALL_ISSUE_TYPES
            .iter()
            .map(|t| {
                let is_selected = self.custom_collection.contains(&t.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                let description = match *t {
                    "TASK" => "Focused task that executes one specific thing",
                    "FEAT" => "New feature or enhancement",
                    "FIX" => "Bug fix, follow-up work, tech debt",
                    "SPIKE" => "Research or exploration (paired mode)",
                    "INV" => "Incident investigation (paired mode)",
                    _ => "",
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            checkbox,
                            Style::default().fg(if is_selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            *t,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(if is_selected {
                                    Color::White
                                } else {
                                    Color::Gray
                                }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[2], &mut self.collection_state);

        // Footer
        let selected_count = self.custom_collection.len();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} selected", selected_count),
                Style::default().fg(if selected_count > 0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    fn render_task_field_config_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Operator!",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Setup - Configure TASK Fields "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Explanation
                Constraint::Length(2), // Instructions
                Constraint::Min(6),    // Field list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Configure TASK Fields",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Explanation
        let explanation = Paragraph::new(vec![
            Line::from("TASK is the foundational issuetype. Configure which optional"),
            Line::from("fields to include. These choices will propagate to other types."),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(explanation, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, chunks[2]);

        // Field list
        let items: Vec<ListItem> = TASK_OPTIONAL_FIELDS
            .iter()
            .map(|(name, description)| {
                let is_selected = self.task_optional_fields.contains(&name.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            checkbox,
                            Style::default().fg(if is_selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            *name,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(if is_selected {
                                    Color::White
                                } else {
                                    Color::Gray
                                }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(*description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[3], &mut self.field_state);

        // Footer
        let selected_count = self.task_optional_fields.len();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(
                    "{}/{} fields enabled",
                    selected_count,
                    TASK_OPTIONAL_FIELDS.len()
                ),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }

    fn render_session_wrapper_choice_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Operator!",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Setup - Session Wrapper "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Description
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Options list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Session Wrapper Configuration",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let description = Paragraph::new(vec![
            Line::from("Operator runs agents in terminal sessions."),
            Line::from("Select your preferred wrapper:"),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(description, chunks[1]);

        // Instructions
        let instructions =
            Paragraph::new(vec![Line::from("Use arrows to navigate, Enter to select")])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, chunks[2]);

        // Options list
        let items: Vec<ListItem> = SessionWrapperOption::all()
            .iter()
            .map(|opt| {
                let is_selected = opt.to_wrapper_type() == self.selected_wrapper;
                let radio = if is_selected { "(o)" } else { "( )" };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            radio,
                            Style::default().fg(if is_selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            opt.label(),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(if is_selected {
                                    Color::White
                                } else {
                                    Color::Gray
                                }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(opt.description(), Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[3], &mut self.wrapper_state);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }

    fn render_tmux_onboarding_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 75, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Tmux Configuration ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Status
                Constraint::Length(1), // Spacer
                Constraint::Min(12),   // Help text or install instructions
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Tmux Session Configuration",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Status indicator
        let status_line = match &self.tmux_status {
            TmuxDetectionStatus::NotChecked => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[?] ", Style::default().fg(Color::Yellow)),
                Span::styled("Not checked", Style::default().fg(Color::Yellow)),
            ]),
            TmuxDetectionStatus::Available { version } => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[+] ", Style::default().fg(Color::Green)),
                Span::styled(
                    format!("Available (v{})", version),
                    Style::default().fg(Color::Green),
                ),
            ]),
            TmuxDetectionStatus::NotInstalled => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[x] ", Style::default().fg(Color::Red)),
                Span::styled("Not installed", Style::default().fg(Color::Red)),
            ]),
            TmuxDetectionStatus::VersionTooOld { current, required } => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[x] ", Style::default().fg(Color::Red)),
                Span::styled(
                    format!("Version too old (v{}, need {}+)", current, required),
                    Style::default().fg(Color::Red),
                ),
            ]),
        };
        let status = Paragraph::new(vec![status_line]).alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);

        // Help text or install instructions
        let help_text = if matches!(self.tmux_status, TmuxDetectionStatus::Available { .. }) {
            vec![
                Line::from(Span::styled(
                    "Essential Commands:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Detach from session:  ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "Ctrl+a",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " (quick, no prefix needed!)",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Fallback detach:      ", Style::default().fg(Color::Gray)),
                    Span::styled("Ctrl+b", Style::default().fg(Color::Cyan)),
                    Span::styled(" then ", Style::default().fg(Color::Gray)),
                    Span::styled("d", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  List sessions:        ", Style::default().fg(Color::Gray)),
                    Span::styled("tmux ls", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Attach to session:    ", Style::default().fg(Color::Gray)),
                    Span::styled("tmux attach -t ", Style::default().fg(Color::Green)),
                    Span::styled("<name>", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Operator session names start with 'op-'",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    "Install tmux:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  macOS:         ", Style::default().fg(Color::Gray)),
                    Span::styled("brew install tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Ubuntu/Debian: ", Style::default().fg(Color::Gray)),
                    Span::styled("sudo apt install tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Fedora/RHEL:   ", Style::default().fg(Color::Gray)),
                    Span::styled("sudo dnf install tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Arch:          ", Style::default().fg(Color::Gray)),
                    Span::styled("sudo pacman -S tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "After installing, press [R] to re-check",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        };
        frame.render_widget(Paragraph::new(help_text), chunks[4]);

        // Footer - different depending on status
        let footer = if matches!(self.tmux_status, TmuxDetectionStatus::Available { .. }) {
            Paragraph::new(Line::from(vec![
                Span::styled("[R]", Style::default().fg(Color::Yellow)),
                Span::raw(" re-check  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" continue  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]))
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled("[R]", Style::default().fg(Color::Green)),
                Span::raw(" re-check tmux  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]))
        };
        frame.render_widget(footer.alignment(Alignment::Center), chunks[5]);
    }

    fn render_vscode_setup_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" VS Code Extension Setup ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Status
                Constraint::Length(1), // Spacer
                Constraint::Min(12),   // Instructions
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "VS Code Integration Setup",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Status indicator
        let status_line = match &self.vscode_status {
            VSCodeDetectionStatus::NotChecked => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[?] ", Style::default().fg(Color::Yellow)),
                Span::styled("Not checked", Style::default().fg(Color::Yellow)),
            ]),
            VSCodeDetectionStatus::Checking => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[~] ", Style::default().fg(Color::Yellow)),
                Span::styled("Checking...", Style::default().fg(Color::Yellow)),
            ]),
            VSCodeDetectionStatus::Connected { version } => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[+] ", Style::default().fg(Color::Green)),
                Span::styled(
                    format!("Connected (v{})", version),
                    Style::default().fg(Color::Green),
                ),
            ]),
            VSCodeDetectionStatus::NotReachable => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[x] ", Style::default().fg(Color::Red)),
                Span::styled("Not detected", Style::default().fg(Color::Red)),
            ]),
        };
        let status = Paragraph::new(vec![status_line]).alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);

        // Instructions
        let instructions = vec![
            Line::from(Span::styled(
                "To use VS Code integration:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  1. ", Style::default().fg(Color::Cyan)),
                Span::raw("Install the Operator extension from:"),
            ]),
            Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    "https://operator.untra.io/vscode",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  2. ", Style::default().fg(Color::Cyan)),
                Span::raw("Restart VS Code after installation"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  3. ", Style::default().fg(Color::Cyan)),
                Span::raw("The extension will start automatically on port 7009"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Note: VS Code extension support is coming soon!",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(instructions), chunks[4]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[T]", Style::default().fg(Color::Yellow)),
            Span::raw(" test connection  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[5]);
    }

    fn render_acceptance_criteria_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Operator!",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Setup - Acceptance Criteria "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Description
                Constraint::Min(8),    // Acceptance criteria content
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Review Acceptance Criteria",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![
            Line::from("These criteria will be used to validate completed work."),
            Line::from("Other template files (Definition of Done, Definition of Ready) will be written from defaults."),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[1]);

        // Acceptance criteria content (read-only preview)
        let content_block = Block::default()
            .title(" Acceptance Criteria Template ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let content = Paragraph::new(self.acceptance_criteria_text.as_str())
            .block(content_block)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(content, chunks[2]);

        // Footer with key hints
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" accept  |  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    fn render_startup_tickets_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Operator!",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Setup - Startup Tickets "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Explanation
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Options list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Create Startup Tickets",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Explanation
        let explanation = Paragraph::new(vec![
            Line::from("Optionally create tickets to bootstrap your projects."),
            Line::from("These help set up catalog entries and agent configurations."),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(explanation, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, chunks[2]);

        // Options list
        let items: Vec<ListItem> = self
            .startup_ticket_options
            .iter()
            .map(|opt| {
                let checkbox = if opt.enabled { "[x]" } else { "[ ]" };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            checkbox,
                            Style::default().fg(if opt.enabled {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            opt.name,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(if opt.enabled {
                                    Color::White
                                } else {
                                    Color::Gray
                                }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(opt.description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[3], &mut self.startup_state);

        // Footer
        let selected_count = self
            .startup_ticket_options
            .iter()
            .filter(|o| o.enabled)
            .count();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ticket types selected", selected_count),
                Style::default().fg(if selected_count > 0 {
                    Color::Green
                } else {
                    Color::Gray
                }),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }

    fn render_confirm_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Confirm Setup ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Description
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Path info
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Selected collection
                Constraint::Min(4),    // What will be created
                Constraint::Length(3), // Buttons
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Ready to Initialize",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![Line::from(
            "Would you like to initialize the ticket queue with these settings?",
        )])
        .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[4]);

        // Selected collection
        let effective_collection = self.collection();
        let collection_text = vec![
            Line::from(Span::styled(
                format!(
                    "Selected issue types ({}):",
                    self.selected_preset.display_name()
                ),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    effective_collection.join(", "),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];
        frame.render_widget(Paragraph::new(collection_text), chunks[6]);

        // What will be created
        let will_create = vec![
            Line::from(Span::styled(
                "This will create:",
                Style::default().fg(Color::Gray),
            )),
            Line::from("  .tickets/queue/  .tickets/in-progress/  .tickets/completed/"),
            Line::from(Span::styled(
                "  .tickets/templates/ (with selected issue type templates)",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(will_create), chunks[7]);

        // Buttons
        let init_style = if self.confirm_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let cancel_style = if !self.confirm_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let buttons = Line::from(vec![
            Span::raw("     "),
            Span::styled(" [I]nitialize ", init_style),
            Span::raw("     "),
            Span::styled(" [C]ancel ", cancel_style),
        ]);

        let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[8]);
    }

    // ─── Kanban Setup Render Methods ────────────────────────────────────────────

    fn render_kanban_info_step(&self, frame: &mut Frame) {
        use crate::api::providers::kanban::{KanbanProviderType, ProviderStatus};

        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Kanban Integration ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Description
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Supported providers header
                Constraint::Length(4), // Supported providers list
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Detected header
                Constraint::Min(6),    // Detected providers list
                Constraint::Length(2), // Footer/help
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "Kanban",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Integration Setup"),
        ])])
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new("Operator can sync issues from external kanban providers.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[1]);

        // Supported providers header
        let supported_header = Paragraph::new(Line::from(vec![Span::styled(
            "Supported Providers:",
            Style::default().fg(Color::Yellow),
        )]));
        frame.render_widget(supported_header, chunks[3]);

        // Supported providers list
        let supported = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Jira Cloud", Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(
                    "OPERATOR_JIRA_API_KEY",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Linear", Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(
                    "OPERATOR_LINEAR_API_KEY",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]),
        ]);
        frame.render_widget(supported, chunks[4]);

        // Detected header
        let detected_header = Paragraph::new(Line::from(vec![Span::styled(
            "Detected Providers:",
            Style::default().fg(Color::Yellow),
        )]));
        frame.render_widget(detected_header, chunks[6]);

        // Detected providers list
        let mut detected_lines = Vec::new();
        if self.detected_kanban_providers.is_empty() {
            detected_lines.push(Line::from(vec![Span::styled(
                "  No providers detected from environment variables",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            for (i, provider) in self.detected_kanban_providers.iter().enumerate() {
                let is_valid = self.valid_kanban_providers.contains(&i);
                let (icon, icon_color) = match &provider.status {
                    ProviderStatus::Untested => ("?", Color::Yellow),
                    ProviderStatus::Testing => ("~", Color::Yellow),
                    ProviderStatus::Valid => ("✓", Color::Green),
                    ProviderStatus::Failed { .. } => ("✗", Color::Red),
                };

                let provider_name = match provider.provider_type {
                    KanbanProviderType::Jira => "Jira",
                    KanbanProviderType::Linear => "Linear",
                };

                let status_text = match &provider.status {
                    ProviderStatus::Untested => "not tested".to_string(),
                    ProviderStatus::Testing => "testing...".to_string(),
                    ProviderStatus::Valid => "valid".to_string(),
                    ProviderStatus::Failed { error } => {
                        format!("failed: {}", error.chars().take(30).collect::<String>())
                    }
                };

                detected_lines.push(Line::from(vec![
                    Span::raw("  ["),
                    Span::styled(icon, Style::default().fg(icon_color)),
                    Span::raw("] "),
                    Span::styled(
                        provider_name,
                        Style::default().fg(if is_valid {
                            Color::White
                        } else {
                            Color::DarkGray
                        }),
                    ),
                    Span::raw(" - "),
                    Span::styled(&provider.domain, Style::default().fg(Color::Cyan)),
                    Span::raw(" ("),
                    Span::styled(status_text, Style::default().fg(icon_color)),
                    Span::raw(")"),
                ]));
            }
        }
        let detected_list = Paragraph::new(detected_lines);
        frame.render_widget(detected_list, chunks[7]);

        // Footer
        let footer = if self.valid_kanban_providers.is_empty() {
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(" Continue  "),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Back"),
            ])
        } else {
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(" Configure providers  "),
                Span::styled("[S]", Style::default().fg(Color::Yellow)),
                Span::raw(" Skip  "),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Back"),
            ])
        };
        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, chunks[8]);
    }

    fn render_kanban_provider_setup_step(&mut self, frame: &mut Frame, provider_index: usize) {
        use crate::api::providers::kanban::KanbanProviderType;

        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        // Get the provider being configured
        let provider_idx = self
            .valid_kanban_providers
            .get(provider_index)
            .copied()
            .unwrap_or(0);
        let provider = self.detected_kanban_providers.get(provider_idx);

        let title = if let Some(p) = provider {
            let provider_name = match p.provider_type {
                KanbanProviderType::Jira => "Jira",
                KanbanProviderType::Linear => "Linear",
            };
            format!(" Setup: {} - {} ", provider_name, p.domain)
        } else {
            " Kanban Provider Setup ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Length(1), // Spacer
                Constraint::Min(10),   // Project list
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Preview info
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let instructions =
            Paragraph::new("Select a project to sync:").style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[0]);

        // Project list
        if self.kanban_projects.is_empty() {
            let loading = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Loading projects...",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "(Projects will be fetched when you enter this step)",
                    Style::default().fg(Color::DarkGray),
                )]),
            ])
            .alignment(Alignment::Center);
            frame.render_widget(loading, chunks[2]);
        } else {
            super::paginated_list::render_paginated_list(
                frame,
                chunks[2],
                &mut self.kanban_projects,
                "Projects",
                |project, _selected| {
                    ratatui::widgets::ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:8}", project.key),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" - "),
                        Span::styled(project.name.clone(), Style::default().fg(Color::White)),
                    ]))
                },
            );
        }

        // Preview info
        let preview = if !self.kanban_issue_types.is_empty() {
            Line::from(vec![
                Span::styled("Issue Types: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.kanban_issue_types.join(", "),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  |  "),
                Span::styled("Members: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.kanban_member_count.to_string(),
                    Style::default().fg(Color::White),
                ),
            ])
        } else {
            Line::from(vec![Span::styled(
                "Select a project to see details",
                Style::default().fg(Color::DarkGray),
            )])
        };
        let preview_para = Paragraph::new(preview);
        frame.render_widget(preview_para, chunks[4]);

        // Footer
        let footer = Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("[n/p]", Style::default().fg(Color::Yellow)),
            Span::raw(" Page  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Skip provider"),
        ]);
        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, chunks[5]);
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

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
