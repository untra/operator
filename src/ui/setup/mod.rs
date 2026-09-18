//! Startup setup screen when .tickets/ directory is not found

use std::collections::HashMap;

use crate::agents::{SystemTmuxClient, TmuxClient, TmuxError};
use crate::api::providers::model_server::ModelServerKind;
use crate::config::{CollectionPreset, SessionWrapperType};
use crate::integrations::catalog::{onboardable, CatalogEntry, Vertical};
use crate::ui::masked_input::MaskedInput;
use ratatui::{widgets::ListState, Frame};

pub mod steps;
pub mod types;

pub use types::*;

#[cfg(test)]
mod tests;

pub(crate) const LOCAL_TARGET_OPTION_INDEX: usize = 0;
pub(crate) const CODER_TARGET_OPTION_INDEX: usize = 1;
const EXECUTION_TARGET_OPTION_COUNT: usize = 2;

pub(crate) const LOCAL_EXECUTION_OPTION_INDEX: usize = 0;
pub(crate) const REMOTE_EXECUTION_OPTION_INDEX: usize = 1;
const EXECUTION_MODE_OPTION_COUNT: usize = 2;

/// Setup screen shown when .tickets/ directory doesn't exist
pub struct SetupScreen {
    /// Whether the screen is visible
    pub visible: bool,
    /// Current step in the setup process
    pub step: SetupStep,
    /// Current selection for confirmation: true = Initialize, false = Cancel
    pub confirm_selected: bool,
    pub configuration_name: String,
    pub(crate) configuration_name_error: Option<String>,
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
    /// Startup ticket options (ASSESS, `AGENT_SETUP`, `PROJECT_INIT`)
    pub startup_ticket_options: Vec<StartupTicketOption>,
    /// List state for startup ticket selection
    pub(crate) startup_state: ListState,
    /// Acceptance criteria text (editable during setup)
    pub acceptance_criteria_text: String,
    // ─── Kanban Setup State ─────────────────────────────────────────────────────
    /// Detected kanban providers from environment variables
    pub detected_kanban_providers: Vec<crate::api::providers::kanban::DetectedKanbanProvider>,
    /// Indices of providers with valid credentials
    /// Highlighted row on the kanban info step (0 = connect, 1 = skip)
    pub(crate) kanban_choice_state: ListState,
    /// Set when the user chose "connect"; the key handler opens the dialog
    pub(crate) kanban_dialog_requested: bool,
    // ─── Model Server State ─────────────────────────────────────────────────
    /// Cursor over `model_providers()`
    pub(crate) model_server_state: ListState,
    /// Live probe result per kind slug, filled on entering the step
    pub(crate) model_server_probes: std::collections::HashMap<String, String>,
    /// Kind slugs the user declared, written as `[[model_servers]]` entries
    pub model_servers_declared: Vec<String>,
    /// Whether the probe pass has run
    pub model_servers_probed: bool,
    // ─── Git Provider State ─────────────────────────────────────────────────
    /// Cursor over `git_providers()`
    pub(crate) git_provider_state: ListState,
    /// Slug the user asked to connect; the key handler resolves onboarding
    pub(crate) git_connect_requested: Option<String>,
    /// Per-slug outcome line rendered next to the provider
    pub(crate) git_provider_status: std::collections::HashMap<String, String>,
    /// Shell line that persists the connected provider's token
    pub(crate) git_export_hint: Option<String>,
    /// Whether kanban detection/testing has run
    pub kanban_detection_complete: bool,
    // ─── Session Wrapper Setup State ────────────────────────────────────────────
    /// Selected session wrapper type
    pub selected_wrapper: SessionWrapperType,
    /// List state for wrapper selection
    pub(crate) wrapper_state: ListState,
    // ─── Licence State ──────────────────────────────────────────────────────
    /// Verified licence status for this configuration; the app refreshes it.
    pub license: Option<crate::licensing::LicenseResponse>,
    /// Licence key entry field.
    pub(crate) license_input: MaskedInput,
    /// Set when the user submitted a key; the app performs the install.
    pub(crate) license_install_requested: bool,
    /// Inline outcome of the last install attempt.
    pub(crate) license_error: Option<String>,
    // ─── Execution Mode State ───────────────────────────────────────────────
    pub(crate) execution_mode_state: ListState,
    /// Whether agents run on remote targets; gates the execution-target step.
    pub remote_execution: bool,
    // ─── Execution Target State ─────────────────────────────────────────────
    pub(crate) execution_target_state: ListState,
    pub(crate) coder_target_name: String,
    pub(crate) coder_template: String,
    pub(crate) coder_field: CoderSetupField,
    pub(crate) execution_target_error: Option<String>,
    /// Tmux availability status (checked during `TmuxOnboarding` step)
    pub tmux_status: TmuxDetectionStatus,
    /// VS Code extension status (checked during `VSCodeSetup` step)
    pub vscode_status: VSCodeDetectionStatus,
    // ─── Git Worktree Setup State ──────────────────────────────────────────────
    /// Whether to use git worktrees for ticket isolation (default: false)
    pub use_worktrees: bool,
    /// List state for worktree option selection
    pub(crate) worktree_state: ListState,
    // ─── Admin Password State ─────────────────────────────────────────────────
    /// The admin password field.
    pub(crate) password: MaskedInput,
    /// The confirmation field.
    pub(crate) password_confirm: MaskedInput,
    /// Which field Tab currently targets.
    pub(crate) password_field_focused: PasswordField,
    /// Inline validation message; `Some` keeps the step from advancing.
    pub(crate) password_error: Option<String>,
    /// The accepted password, applied at initialization. `None` means skipped.
    pub admin_password: Option<String>,
    /// Whether an admin account already exists, in which case the step is skipped.
    pub admin_password_configured: bool,
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

        let mut execution_target_state = ListState::default();
        execution_target_state.select(Some(LOCAL_TARGET_OPTION_INDEX));

        let mut execution_mode_state = ListState::default();
        execution_mode_state.select(Some(LOCAL_EXECUTION_OPTION_INDEX));

        let mut hosted_state = ListState::default();
        hosted_state.select(Some(0));

        Self {
            visible: true,
            step: SetupStep::Welcome,
            confirm_selected: true, // Default to Initialize
            // Empty, not "legacy": the field's validation should be visible
            // rather than pre-satisfied. `App` overwrites this with the real
            // name when the configuration already has one.
            configuration_name: String::new(),
            configuration_name_error: None,
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
            kanban_choice_state: {
                let mut st = ListState::default();
                st.select(Some(0));
                st
            },
            kanban_dialog_requested: false,
            model_server_state: {
                let mut st = ListState::default();
                st.select(Some(0));
                st
            },
            model_server_probes: std::collections::HashMap::new(),
            model_servers_declared: Vec::new(),
            model_servers_probed: false,
            git_provider_state: {
                let mut st = ListState::default();
                st.select(Some(0));
                st
            },
            git_connect_requested: None,
            git_provider_status: std::collections::HashMap::new(),
            git_export_hint: None,
            kanban_detection_complete: false,
            // Session wrapper state
            selected_wrapper: SessionWrapperType::Tmux,
            wrapper_state,
            license: None,
            license_input: MaskedInput::default(),
            license_install_requested: false,
            license_error: None,
            execution_mode_state,
            remote_execution: false,
            execution_target_state,
            coder_target_name: crate::config::DEFAULT_CODER_TARGET_NAME.to_string(),
            coder_template: String::new(),
            coder_field: CoderSetupField::TargetName,
            execution_target_error: None,
            tmux_status: TmuxDetectionStatus::NotChecked,
            vscode_status: VSCodeDetectionStatus::NotChecked,
            // Git worktree state
            use_worktrees: false,
            password: MaskedInput::new(),
            password_confirm: MaskedInput::new(),
            password_field_focused: PasswordField::default(),
            password_error: None,
            admin_password: None,
            admin_password_configured: false,
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
            SetupStep::ModelServer => self.toggle_model_server(),
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
            SetupStep::ExecutionTarget => {
                self.coder_field = self.coder_field.toggled();
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
            SetupStep::AdminPassword => {
                self.password_field_focused = self.password_field_focused.toggled();
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
            SetupStep::KanbanInfo => {
                let i = self
                    .kanban_choice_state
                    .selected()
                    .map_or(0, |i| (i + 1) % 2);
                self.kanban_choice_state.select(Some(i));
            }
            SetupStep::ModelServer => {
                let len = Self::model_providers().len();
                let i = self
                    .model_server_state
                    .selected()
                    .map_or(0, |i| (i + 1) % len);
                self.model_server_state.select(Some(i));
            }
            SetupStep::GitProvider => {
                let len = Self::git_providers().len() + 1;
                let i = self
                    .git_provider_state
                    .selected()
                    .map_or(0, |i| (i + 1) % len);
                self.git_provider_state.select(Some(i));
            }
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
            SetupStep::ExecutionMode => {
                let i = self
                    .execution_mode_state
                    .selected()
                    .map_or(LOCAL_EXECUTION_OPTION_INDEX, |i| {
                        (i + 1) % EXECUTION_MODE_OPTION_COUNT
                    });
                self.execution_mode_state.select(Some(i));
                self.license_error = None;
            }
            SetupStep::ExecutionTarget => {
                let i = self
                    .execution_target_state
                    .selected()
                    .map_or(LOCAL_TARGET_OPTION_INDEX, |i| {
                        (i + 1) % EXECUTION_TARGET_OPTION_COUNT
                    });
                self.execution_target_state.select(Some(i));
                self.execution_target_error = None;
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
            SetupStep::KanbanInfo => {
                let i = self
                    .kanban_choice_state
                    .selected()
                    .map_or(0, |i| (i + 1) % 2);
                self.kanban_choice_state.select(Some(i));
            }
            SetupStep::ModelServer => {
                let len = Self::model_providers().len();
                let i = self.model_server_state.selected().map_or(0, |i| {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                });
                self.model_server_state.select(Some(i));
            }
            SetupStep::GitProvider => {
                let len = Self::git_providers().len() + 1;
                let i = self.git_provider_state.selected().map_or(0, |i| {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                });
                self.git_provider_state.select(Some(i));
            }
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
            SetupStep::ExecutionMode => {
                let i = self
                    .execution_mode_state
                    .selected()
                    .map_or(REMOTE_EXECUTION_OPTION_INDEX, |i| {
                        usize::from(i == LOCAL_EXECUTION_OPTION_INDEX)
                    });
                self.execution_mode_state.select(Some(i));
                self.license_error = None;
            }
            SetupStep::ExecutionTarget => {
                let i = self
                    .execution_target_state
                    .selected()
                    .map_or(CODER_TARGET_OPTION_INDEX, |i| {
                        usize::from(i == LOCAL_TARGET_OPTION_INDEX)
                    });
                self.execution_target_state.select(Some(i));
                self.execution_target_error = None;
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
    fn enter_wrapper_step(&mut self) {
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
    }

    /// Validate the password fields.
    ///
    /// `Ok(None)` means the step was skipped - both fields empty. The step is
    /// optional, so an empty pair is a deliberate choice, not an error.
    /// `Err(message)` is shown inline and keeps the wizard on this step.
    fn validate_admin_password(&self) -> Result<Option<String>, String> {
        let password = self.password.value();
        let confirm = self.password_confirm.value();

        if self.password.is_empty() && self.password_confirm.is_empty() {
            return Ok(None);
        }
        if password != confirm {
            return Err("Passwords do not match".to_string());
        }
        // Reuse the server's rule rather than restating a length here
        crate::auth::password::validate_password(password).map_err(|e| e.to_string())?;

        Ok(Some(password.to_string()))
    }

    /// Route a key to the focused password field.
    ///
    /// Called only for `SetupStep::AdminPassword`; see the guard in
    /// `app::keyboard`, which otherwise consumes `i`, `c`, `j`, `k`, and space
    /// as wizard commands before any character reaches a text field.
    pub fn handle_password_key(&mut self, code: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        let field = match self.password_field_focused {
            PasswordField::Password => &mut self.password,
            PasswordField::Confirm => &mut self.password_confirm,
        };

        match code {
            KeyCode::Char(c) => field.handle_char(c),
            KeyCode::Backspace => field.handle_backspace(),
            KeyCode::Delete => field.handle_delete(),
            KeyCode::Left => field.cursor_left(),
            KeyCode::Right => field.cursor_right(),
            KeyCode::Home => field.cursor_home(),
            KeyCode::End => field.cursor_end(),
            _ => return,
        }

        // Any edit invalidates the previous complaint.
        self.password_error = None;
    }

    pub fn handle_execution_target_key(&mut self, code: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        if self.execution_target_state.selected() != Some(CODER_TARGET_OPTION_INDEX) {
            return;
        }
        let value = match self.coder_field {
            CoderSetupField::TargetName => &mut self.coder_target_name,
            CoderSetupField::Template => &mut self.coder_template,
        };
        match code {
            KeyCode::Char(c) => value.push(c),
            KeyCode::Backspace | KeyCode::Delete => {
                value.pop();
            }
            _ => return,
        }
        self.execution_target_error = None;
    }

    pub fn selected_execution_target(&self) -> crate::config::TargetDef {
        if self.execution_target_state.selected() != Some(CODER_TARGET_OPTION_INDEX) {
            return crate::config::TargetDef::local();
        }
        crate::config::TargetDef {
            name: self.coder_target_name.trim().to_string(),
            display_name: Some("Coder".to_string()),
            kind: crate::config::TargetKind::Coder(crate::config::CoderConfig {
                template: self.coder_template.trim().to_string(),
                ..Default::default()
            }),
        }
    }

    fn coder_target_selected(&self) -> bool {
        self.execution_target_state.selected() == Some(CODER_TARGET_OPTION_INDEX)
    }

    /// Declare or undeclare the highlighted provider. Kinds without a default
    /// base URL need one supplied by hand, so they are not selectable here.
    fn toggle_model_server(&mut self) {
        let Some(kind) = self
            .model_server_state
            .selected()
            .and_then(|i| Self::model_providers().get(i).map(|(_, k)| *k))
        else {
            return;
        };
        if !kind.connectable_from_defaults() {
            return;
        }
        let slug = kind.slug().to_string();
        if let Some(pos) = self.model_servers_declared.iter().position(|s| s == &slug) {
            self.model_servers_declared.remove(pos);
        } else {
            self.model_servers_declared.push(slug);
        }
    }

    /// Probe every connectable kind against its defaults, mirroring what the
    /// web UI's provider cards show.
    pub async fn probe_model_servers(&mut self, config: &crate::config::Config) {
        if self.model_servers_probed {
            return;
        }
        self.model_servers_probed = true;
        let policy = crate::auth::egress::EgressPolicy::from_config(config);
        for (_, kind) in Self::model_providers() {
            if !kind.connectable_from_defaults() {
                continue;
            }
            let server = crate::config::ModelServer {
                name: kind.slug().to_string(),
                kind: kind.slug().to_string(),
                base_url: kind.default_base_url().map(str::to_string),
                api_key_env: kind.default_api_key_env().map(str::to_string),
                extra_env: std::collections::HashMap::new(),
                display_name: None,
            };
            let outcome = crate::api::providers::model_server::probe_models(&server, &policy).await;
            let status = if outcome.reachable {
                format!("{} models", outcome.models.len())
            } else if kind
                .default_api_key_env()
                .is_some_and(|e| std::env::var(e).is_err())
            {
                "key missing".to_string()
            } else {
                outcome
                    .error
                    .unwrap_or_else(|| "unreachable".to_string())
                    .chars()
                    .take(40)
                    .collect()
            };
            self.model_server_probes
                .insert(kind.slug().to_string(), status);
        }
    }

    /// The declared providers, as `[[model_servers]]` entries.
    pub fn declared_model_servers(&self) -> Vec<crate::config::ModelServer> {
        self.model_servers_declared
            .iter()
            .filter_map(|slug| ModelServerKind::from_slug(slug))
            .map(|kind| crate::config::ModelServer {
                name: kind.slug().to_string(),
                kind: kind.slug().to_string(),
                base_url: kind.default_base_url().map(str::to_string),
                api_key_env: kind.default_api_key_env().map(str::to_string),
                extra_env: std::collections::HashMap::new(),
                display_name: Some(kind.display_name().to_string()),
            })
            .collect()
    }

    /// Git providers the wizard offers, from the integration catalog.
    pub(crate) fn git_providers() -> Vec<CatalogEntry> {
        onboardable(Vertical::Git)
    }

    /// Model providers the wizard offers, from the integration catalog. Each
    /// resolves to a `ModelServerKind` (guaranteed by `tests/vertical_parity.rs`).
    pub(crate) fn model_providers() -> Vec<(CatalogEntry, ModelServerKind)> {
        onboardable(Vertical::Model)
            .into_iter()
            .filter_map(|e| ModelServerKind::from_slug(e.slug).map(|k| (e, k)))
            .collect()
    }

    /// The highlighted provider slug, or `None` for the trailing "skip" row.
    pub(crate) fn selected_git_provider(&self) -> Option<String> {
        let providers = Self::git_providers();
        self.git_provider_state
            .selected()
            .and_then(|i| providers.get(i))
            .map(|e| e.slug.to_string())
    }

    /// Whether this configuration currently holds a valid Premium licence.
    pub(crate) fn premium_entitled(&self) -> bool {
        self.license.as_ref().is_some_and(|l| l.premium)
    }

    /// Consume a pending licence key submitted on the licence step.
    pub(crate) fn take_license_install_request(&mut self) -> Option<String> {
        if std::mem::take(&mut self.license_install_requested) {
            return Some(self.license_input.value().to_string());
        }
        None
    }

    /// Record the outcome of an install attempt made by the app.
    pub(crate) fn set_license_outcome(
        &mut self,
        result: Result<crate::licensing::LicenseResponse, String>,
    ) {
        match result {
            Ok(license) => {
                self.license = Some(license);
                self.license_error = None;
                self.license_input.clear();
            }
            // A rejected key returns to the licence screen: the error belongs
            // where the field that produced it is.
            Err(error) => {
                self.license_error = Some(error);
                self.step = SetupStep::License;
            }
        }
    }

    /// Editing keys for the licence key field.
    pub(crate) fn handle_license_key(&mut self, code: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;
        self.license_error = None;
        match code {
            KeyCode::Char(c) => self.license_input.handle_char(c),
            KeyCode::Backspace => self.license_input.handle_backspace(),
            KeyCode::Delete => self.license_input.handle_delete(),
            KeyCode::Left => self.license_input.cursor_left(),
            KeyCode::Right => self.license_input.cursor_right(),
            KeyCode::Home => self.license_input.cursor_home(),
            KeyCode::End => self.license_input.cursor_end(),
            _ => {}
        }
    }

    pub(crate) fn handle_configuration_name_key(
        &mut self,
        code: ratatui::crossterm::event::KeyCode,
    ) {
        use ratatui::crossterm::event::KeyCode;

        self.configuration_name_error = None;
        match code {
            KeyCode::Char(c)
                if self.configuration_name.len() < crate::profiles::MAX_PROFILE_NAME_LENGTH
                    && (c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_')) =>
            {
                self.configuration_name.push(c);
            }
            KeyCode::Backspace => {
                self.configuration_name.pop();
            }
            _ => {}
        }
    }

    /// Consume a pending request to connect a git provider.
    pub fn take_git_connect_request(&mut self) -> Option<String> {
        self.git_connect_requested.take()
    }

    /// Record the outcome of a connect attempt for display.
    pub fn set_git_provider_status(&mut self, slug: &str, status: String) {
        self.git_provider_status.insert(slug.to_string(), status);
    }

    /// Consume a pending request to open the kanban onboarding dialog.
    pub fn take_kanban_dialog_request(&mut self) -> bool {
        std::mem::take(&mut self.kanban_dialog_requested)
    }

    pub fn confirm(&mut self) -> SetupResult {
        match self.step {
            SetupStep::Welcome => {
                if let Err(error) = crate::profiles::validate_name(&self.configuration_name) {
                    self.configuration_name_error = Some(error.to_string());
                    return SetupResult::Continue;
                }
                // Kanban setup runs first so the collection step can offer
                // "import from a configured provider" options. Detect providers
                // from environment variables on the way into the kanban step.
                if !self.kanban_detection_complete {
                    self.detected_kanban_providers =
                        crate::api::providers::kanban::detect_kanban_env_vars();
                    self.kanban_detection_complete = true;
                }
                self.step = SetupStep::License;
                SetupResult::Continue
            }
            SetupStep::License => {
                if !self.license_input.is_empty() {
                    self.license_install_requested = true;
                }
                self.step = SetupStep::ExecutionMode;
                SetupResult::Continue
            }
            SetupStep::ExecutionMode => {
                self.remote_execution =
                    self.execution_mode_state.selected() == Some(REMOTE_EXECUTION_OPTION_INDEX);
                if self.remote_execution && !self.premium_entitled() {
                    self.license_error = Some(
                        "Remote targets require a valid Premium licence for this configuration"
                            .to_string(),
                    );
                    return SetupResult::Continue;
                }
                self.step = SetupStep::KanbanInfo;
                SetupResult::Continue
            }
            SetupStep::KanbanInfo => {
                // Row 0 hands off to the shared onboarding dialog, which
                // collects credentials and writes the provider section.
                if self.kanban_choice_state.selected() == Some(0) {
                    self.kanban_dialog_requested = true;
                } else {
                    self.step = SetupStep::ModelServer;
                }
                SetupResult::Continue
            }
            SetupStep::ModelServer => {
                self.step = SetupStep::GitProvider;
                SetupResult::Continue
            }
            SetupStep::GitProvider => {
                // Row 0..n connect; the last row moves on without a provider.
                match self.selected_git_provider() {
                    Some(slug) => self.git_connect_requested = Some(slug),
                    None => self.enter_collection_source(),
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
                self.step = if self.remote_execution {
                    SetupStep::ExecutionTarget
                } else {
                    SetupStep::WorktreePreference
                };
                SetupResult::Continue
            }
            SetupStep::ExecutionTarget => {
                if self.coder_target_selected() {
                    if self.selected_wrapper == SessionWrapperType::Zellij {
                        self.execution_target_error =
                            Some("Coder targets cannot use the Zellij session wrapper".to_string());
                        return SetupResult::Continue;
                    }
                    if self.coder_target_name.trim().is_empty() {
                        self.execution_target_error =
                            Some("Coder target name is required".to_string());
                        return SetupResult::Continue;
                    }
                    if matches!(
                        self.coder_target_name.trim(),
                        crate::config::TARGET_LOCAL | crate::config::TARGET_DOCKER
                    ) {
                        self.execution_target_error = Some(format!(
                            "'{}' is reserved for a built-in target",
                            self.coder_target_name.trim()
                        ));
                        return SetupResult::Continue;
                    }
                    if self.coder_template.trim().is_empty() {
                        self.execution_target_error =
                            Some("Coder template is required".to_string());
                        return SetupResult::Continue;
                    }
                    self.use_worktrees = false;
                }
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::WorktreePreference => {
                // Select the worktree option from the highlighted option
                if let Some(i) = self.worktree_state.selected() {
                    let options = WorktreeOption::all();
                    if i < options.len() {
                        self.use_worktrees =
                            !self.coder_target_selected() && options[i].to_use_worktrees();
                    }
                }
                // The wrapper fan-out now lives on the AdminPassword arm, so
                // the password step sits between this one and the wrapper step.
                if self.admin_password_configured {
                    self.enter_wrapper_step();
                } else {
                    self.step = SetupStep::AdminPassword;
                }
                SetupResult::Continue
            }
            SetupStep::AdminPassword => {
                match self.validate_admin_password() {
                    Ok(password) => {
                        self.admin_password = password;
                        self.password_error = None;
                        self.enter_wrapper_step();
                    }
                    // `SetupResult` has no "stay and report" variant, so the
                    // message lives on the screen and the step does not change.
                    Err(message) => self.password_error = Some(message),
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
            SetupStep::License => {
                self.step = SetupStep::Welcome;
                SetupResult::Continue
            }
            SetupStep::ExecutionMode => {
                self.license_error = None;
                self.step = SetupStep::License;
                SetupResult::Continue
            }
            SetupStep::KanbanInfo => {
                self.step = SetupStep::ExecutionMode;
                SetupResult::Continue
            }
            SetupStep::ModelServer => {
                self.step = SetupStep::KanbanInfo;
                SetupResult::Continue
            }
            SetupStep::GitProvider => {
                self.step = SetupStep::ModelServer;
                SetupResult::Continue
            }
            SetupStep::CollectionSource => {
                self.step = SetupStep::GitProvider;
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
                self.step = if self.remote_execution {
                    SetupStep::ExecutionTarget
                } else {
                    SetupStep::SessionWrapperChoice
                };
                SetupResult::Continue
            }
            SetupStep::ExecutionTarget => {
                self.step = SetupStep::SessionWrapperChoice;
                SetupResult::Continue
            }
            SetupStep::AdminPassword => {
                self.step = SetupStep::WorktreePreference;
                SetupResult::Continue
            }
            SetupStep::TmuxOnboarding => {
                self.step = if self.admin_password_configured {
                    SetupStep::WorktreePreference
                } else {
                    SetupStep::AdminPassword
                };
                SetupResult::Continue
            }
            SetupStep::VSCodeSetup => {
                self.step = if self.admin_password_configured {
                    SetupStep::WorktreePreference
                } else {
                    SetupStep::AdminPassword
                };
                SetupResult::Continue
            }
            SetupStep::CmuxSetup => {
                self.step = if self.admin_password_configured {
                    SetupStep::WorktreePreference
                } else {
                    SetupStep::AdminPassword
                };
                SetupResult::Continue
            }
            SetupStep::ZellijSetup => {
                self.step = if self.admin_password_configured {
                    SetupStep::WorktreePreference
                } else {
                    SetupStep::AdminPassword
                };
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

        match self.step {
            SetupStep::Welcome => self.render_welcome_step(frame),
            SetupStep::License => self.render_license_step(frame),
            SetupStep::ExecutionMode => self.render_execution_mode_step(frame),
            SetupStep::CollectionSource => self.render_collection_source_step(frame),
            SetupStep::HostedCollectionFetch => self.render_hosted_collection_step(frame),
            SetupStep::TaskFieldConfig => self.render_task_field_config_step(frame),
            SetupStep::SessionWrapperChoice => self.render_session_wrapper_choice_step(frame),
            SetupStep::ExecutionTarget => self.render_execution_target_step(frame),
            SetupStep::WorktreePreference => self.render_worktree_preference_step(frame),
            SetupStep::TmuxOnboarding => self.render_tmux_onboarding_step(frame),
            SetupStep::VSCodeSetup => self.render_vscode_setup_step(frame),
            SetupStep::CmuxSetup => self.render_cmux_setup_step(frame),
            SetupStep::ZellijSetup => self.render_zellij_setup_step(frame),
            SetupStep::KanbanInfo => self.render_kanban_info_step(frame),
            SetupStep::ModelServer => self.render_model_server_step(frame),
            SetupStep::GitProvider => self.render_git_provider_step(frame),
            SetupStep::AdminPassword => self.render_admin_password_step(frame),
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
