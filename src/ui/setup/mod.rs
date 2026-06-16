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
    /// Detected LLM tools (from `LlmToolsConfig`)
    pub(crate) detected_tools: Vec<DetectedToolInfo>,
    /// Projects grouped by tool
    pub(crate) projects_by_tool: HashMap<String, Vec<String>>,
    /// Selected collection preset
    pub selected_preset: CollectionPreset,
    /// Effective issuetype collection (used when preset is Custom, e.g. from the
    /// hosted browser's merged selection)
    pub custom_collection: Vec<String>,
    /// Dynamic collection-source options (curated + per-provider imports),
    /// rebuilt on entering the collection-source step
    pub(crate) source_options: Vec<CollectionSourceOption>,
    /// List state for collection source selection
    pub(crate) source_state: ListState,
    /// Transient notice shown on the collection-source step (e.g. a deferred
    /// kanban import message)
    pub(crate) import_notice: Option<String>,
    // ─── Hosted Collection State ───────────────────────────────────────────────
    /// Collections resolved for the hosted picker (hosted + embedded fallback)
    pub hosted_resolved: Vec<crate::collections::fetch::ResolvedCollection>,
    /// List state for the hosted collection picker (highlight cursor)
    pub(crate) hosted_state: ListState,
    /// Ids of collections checked in the multi-select hosted picker
    pub(crate) hosted_selected_ids: Vec<String>,
    /// Whether the hosted collection list has been loaded (fetch attempted)
    pub(crate) hosted_loaded: bool,
    /// Id of the chosen hosted collection (set when a hosted collection is picked)
    pub selected_hosted_id: Option<String>,
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
    /// Tmux availability status (checked during `TmuxOnboarding` step)
    pub tmux_status: TmuxDetectionStatus,
    /// VS Code extension status (checked during `VSCodeSetup` step)
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

        let mut field_state = ListState::default();
        field_state.select(Some(0));

        let mut startup_state = ListState::default();
        startup_state.select(Some(0));

        let mut wrapper_state = ListState::default();
        wrapper_state.select(Some(0));

        let mut worktree_state = ListState::default();
        worktree_state.select(Some(0));

        let mut hosted_state = ListState::default();
        hosted_state.select(Some(0));

        Self {
            visible: true,
            step: SetupStep::Welcome,
            confirm_selected: true, // Default to Initialize
            tickets_path,
            detected_tools,
            projects_by_tool,
            selected_preset: CollectionPreset::DevopsKanban,
            custom_collection: Vec::new(),
            source_options: CollectionSourceOption::curated(),
            source_state,
            import_notice: None,
            hosted_resolved: Vec::new(),
            hosted_state,
            hosted_selected_ids: Vec::new(),
            hosted_loaded: false,
            selected_hosted_id: None,
            // Default: all optional fields enabled
            task_optional_fields: TASK_OPTIONAL_FIELDS
                .iter()
                .map(|(name, _)| (*name).to_string())
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

    /// The resolved hosted collection currently highlighted in the picker.
    pub(crate) fn highlighted_hosted(
        &self,
    ) -> Option<&crate::collections::fetch::ResolvedCollection> {
        let i = self.hosted_state.selected()?;
        self.hosted_resolved.get(i)
    }

    /// The resolved hosted collection the user committed to (by id), for scaffolding.
    pub fn selected_hosted_collections(
        &self,
    ) -> Vec<&crate::collections::fetch::ResolvedCollection> {
        if !self.hosted_selected_ids.is_empty() {
            self.hosted_resolved
                .iter()
                .filter(|r| self.hosted_selected_ids.contains(&r.manifest.id))
                .collect()
        } else if let Some(id) = self.selected_hosted_id.as_deref() {
            self.hosted_resolved
                .iter()
                .filter(|r| r.manifest.id == id)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Load the hosted collection picker list (hosted manifest + embedded fallback).
    ///
    /// Always populates at least the embedded collections, so the picker is never
    /// empty even offline. `manifest_url` should be `None` when fetching is disabled.
    pub async fn load_hosted_collections(&mut self, manifest_url: Option<&str>, timeout_secs: u64) {
        self.hosted_resolved =
            crate::collections::fetch::resolve_for_setup(manifest_url, timeout_secs).await;
        self.hosted_state
            .select((!self.hosted_resolved.is_empty()).then_some(0));
        self.hosted_loaded = true;
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
            .and_then(|i| self.source_options.get(i).cloned())
    }

    /// Enter the collection-source step, rebuilding the dynamic option list from
    /// the kanban providers detected/configured earlier in the wizard.
    fn enter_collection_source(&mut self) {
        self.source_options =
            CollectionSourceOption::with_providers(&self.detected_kanban_providers);
        self.source_state.select(Some(0));
        self.import_notice = None;
        self.step = SetupStep::CollectionSource;
    }

    /// Commit the hosted-picker selection: union the issue types of every checked
    /// collection (or the highlighted one if none are checked), in first-seen
    /// order, and advance to the field-config step.
    fn commit_hosted_selection(&mut self) {
        // Resolve the chosen collections by id (checked set, else highlighted).
        let chosen: Vec<&crate::collections::fetch::ResolvedCollection> =
            if self.hosted_selected_ids.is_empty() {
                self.highlighted_hosted().into_iter().collect()
            } else {
                self.hosted_resolved
                    .iter()
                    .filter(|r| self.hosted_selected_ids.contains(&r.manifest.id))
                    .collect()
            };
        if chosen.is_empty() {
            return;
        }

        let mut merged: Vec<String> = Vec::new();
        for r in &chosen {
            let keys = if r.manifest.default_selected.is_empty() {
                r.manifest.type_keys()
            } else {
                r.manifest.default_selected.clone()
            };
            for k in keys {
                if !merged.contains(&k) {
                    merged.push(k);
                }
            }
        }

        // Record the single committed id when exactly one collection is chosen
        // (drives back-navigation + scaffolding); None when several are merged.
        self.selected_hosted_id = (chosen.len() == 1).then(|| chosen[0].manifest.id.clone());
        self.selected_preset = CollectionPreset::Custom;
        self.custom_collection = merged;
        self.step = SetupStep::TaskFieldConfig;
    }

    /// Toggle selection (Space key)
    pub fn toggle_selection(&mut self) {
        match self.step {
            SetupStep::HostedCollectionFetch => {
                // Toggle the highlighted collection in the multi-select picker.
                if let Some(r) = self.highlighted_hosted() {
                    let id = r.manifest.id.clone();
                    if let Some(pos) = self.hosted_selected_ids.iter().position(|x| x == &id) {
                        self.hosted_selected_ids.remove(pos);
                    } else {
                        self.hosted_selected_ids.push(id);
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
                let len = self.source_options.len();
                if len > 0 {
                    let i = self.source_state.selected().map_or(0, |i| (i + 1) % len);
                    self.source_state.select(Some(i));
                }
            }
            SetupStep::HostedCollectionFetch => {
                let len = self.hosted_resolved.len();
                if len > 0 {
                    let i = self.hosted_state.selected().map_or(0, |i| (i + 1) % len);
                    self.hosted_state.select(Some(i));
                }
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
                let len = self.source_options.len();
                if len > 0 {
                    let i = self.source_state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.source_state.select(Some(i));
                }
            }
            SetupStep::HostedCollectionFetch => {
                let len = self.hosted_resolved.len();
                if len > 0 {
                    let i = self.hosted_state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.hosted_state.select(Some(i));
                }
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
                // Kanban setup runs first so the collection step can offer
                // "import from a configured provider" options. Detect providers
                // from environment variables on the way into the kanban step.
                if !self.kanban_detection_complete {
                    self.detected_kanban_providers =
                        crate::api::providers::kanban::detect_kanban_env_vars();
                    self.kanban_detection_complete = true;
                }
                self.step = SetupStep::KanbanInfo;
                SetupResult::Continue
            }
            SetupStep::KanbanInfo => {
                // Configure valid providers, otherwise proceed to the collection step.
                if self.valid_kanban_providers.is_empty() || self.kanban_skipped {
                    self.enter_collection_source();
                } else {
                    self.step = SetupStep::KanbanProviderSetup { provider_index: 0 };
                }
                SetupResult::Continue
            }
            SetupStep::KanbanProviderSetup { provider_index } => {
                // Move to the next provider or on to the collection step.
                let next_index = provider_index + 1;
                if next_index < self.valid_kanban_providers.len() {
                    self.step = SetupStep::KanbanProviderSetup {
                        provider_index: next_index,
                    };
                } else {
                    self.enter_collection_source();
                }
                SetupResult::Continue
            }
            SetupStep::CollectionSource => {
                if let Some(source) = self.selected_source() {
                    match source {
                        CollectionSourceOption::Simple => {
                            self.selected_preset = CollectionPreset::Simple;
                            self.selected_hosted_id = None;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::DevKanban => {
                            self.selected_preset = CollectionPreset::DevKanban;
                            self.selected_hosted_id = None;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::DevopsKanban => {
                            self.selected_preset = CollectionPreset::DevopsKanban;
                            self.selected_hosted_id = None;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::Browse => {
                            // The async list load is triggered by the key handler on
                            // entering this step (see handle_key); reset prior state.
                            self.hosted_loaded = false;
                            self.selected_hosted_id = None;
                            self.hosted_selected_ids.clear();
                            self.step = SetupStep::HostedCollectionFetch;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::ImportFromProvider(r) => {
                            // Import is scaffolded: structural conversion is deferred.
                            // Surface a provider-specific notice and stay on the step.
                            self.import_notice = Some(format!(
                                "Importing from {} is coming soon.",
                                r.author_attribution()
                            ));
                            SetupResult::Continue
                        }
                    }
                } else {
                    SetupResult::Continue
                }
            }
            SetupStep::HostedCollectionFetch => {
                // Commit the checked collections (or the highlighted one).
                self.commit_hosted_selection();
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
                    SessionWrapperType::Cmux => {
                        self.step = SetupStep::CmuxSetup;
                    }
                    SessionWrapperType::Zellij => {
                        self.step = SetupStep::ZellijSetup;
                    }
                }
                SetupResult::Continue
            }
            SetupStep::TmuxOnboarding => {
                // Only allow proceeding if tmux is available
                if matches!(self.tmux_status, TmuxDetectionStatus::Available { .. }) {
                    self.step = SetupStep::AcceptanceCriteria;
                }
                // If tmux not available, stay on this step (user must install or go back)
                SetupResult::Continue
            }
            SetupStep::VSCodeSetup => {
                // For now, allow proceeding (extension check will be added later)
                self.step = SetupStep::AcceptanceCriteria;
                SetupResult::Continue
            }
            SetupStep::CmuxSetup => {
                self.step = SetupStep::AcceptanceCriteria;
                SetupResult::Continue
            }
            SetupStep::ZellijSetup => {
                self.step = SetupStep::AcceptanceCriteria;
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
            SetupStep::KanbanInfo => {
                self.step = SetupStep::Welcome;
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
            SetupStep::CollectionSource => {
                // Return to the kanban step that preceded the collection step.
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
            SetupStep::HostedCollectionFetch => {
                self.enter_collection_source();
                SetupResult::Continue
            }
            SetupStep::TaskFieldConfig => {
                // A Custom preset means the hosted browser produced the selection.
                if matches!(self.selected_preset, CollectionPreset::Custom) {
                    self.step = SetupStep::HostedCollectionFetch;
                } else {
                    self.enter_collection_source();
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
            SetupStep::CmuxSetup => {
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::ZellijSetup => {
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::AcceptanceCriteria => {
                // Go back to the wrapper setup step that preceded this one.
                match self.selected_wrapper {
                    SessionWrapperType::Tmux => self.step = SetupStep::TmuxOnboarding,
                    SessionWrapperType::Vscode => self.step = SetupStep::VSCodeSetup,
                    SessionWrapperType::Cmux => self.step = SetupStep::CmuxSetup,
                    SessionWrapperType::Zellij => self.step = SetupStep::ZellijSetup,
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
            SetupStep::HostedCollectionFetch => self.render_hosted_collection_step(frame),
            SetupStep::TaskFieldConfig => self.render_task_field_config_step(frame),
            SetupStep::SessionWrapperChoice => self.render_session_wrapper_choice_step(frame),
            SetupStep::WorktreePreference => self.render_worktree_preference_step(frame),
            SetupStep::TmuxOnboarding => self.render_tmux_onboarding_step(frame),
            SetupStep::VSCodeSetup => self.render_vscode_setup_step(frame),
            SetupStep::CmuxSetup => self.render_cmux_setup_step(frame),
            SetupStep::ZellijSetup => self.render_zellij_setup_step(frame),
            SetupStep::KanbanInfo => self.render_kanban_info_step(frame),
            SetupStep::KanbanProviderSetup { provider_index } => {
                self.render_kanban_provider_setup_step(frame, provider_index);
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
                        required: format!("{MIN_MAJOR}.{MIN_MINOR}"),
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
}
