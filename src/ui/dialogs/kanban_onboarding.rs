//! Kanban onboarding dialog for the TUI.
//!
//! Multi-state wizard: pick provider → collect creds → validate → pick
//! project → write config + set session env + sync issue types → show
//! shell export nudge. All async work (`validate_credentials` /
//! `list_projects` / `write_config` / sync) is dispatched by the `App`
//! event loop calling `services::kanban_onboarding` directly — this
//! dialog is purely UI state + rendering + key handling.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::centered_rect;

/// Provider selection in the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KanbanOnboardingProvider {
    Jira,
    Linear,
}

/// Multi-state wizard state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KanbanOnboardingState {
    /// Initial state — pick Jira or Linear.
    PickProvider,
    /// Collecting Jira domain.
    JiraDomain,
    /// Collecting Jira email.
    JiraEmail,
    /// Collecting Jira token (masked).
    JiraToken,
    /// Collecting Linear API key (masked).
    LinearApiKey,
    /// Calling `validate_credentials` async.
    Validating,
    /// Showing project picker after validation.
    PickProject,
    /// Calling `write_config` + `set_session_env` + `sync_issue_types` async.
    Writing,
    /// Showing the shell export nudge after success.
    EnvExportNudge,
    /// Inline error — user can press Enter to retry from the relevant input.
    Error,
}

/// Action emitted by the dialog after a key press; the App handles async
/// dispatch and updates the dialog via the setters below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KanbanOnboardingAction {
    /// No state-machine transition; just a focus/cursor move.
    None,
    /// User picked a provider — App should advance to the first input step.
    PickedProvider(KanbanOnboardingProvider),
    /// User submitted full Jira credentials — App should call
    /// `services::kanban_onboarding::validate_credentials`.
    SubmitJiraCreds {
        domain: String,
        email: String,
        token: String,
    },
    /// User submitted Linear API key — App should call
    /// `services::kanban_onboarding::validate_credentials`.
    SubmitLinearCreds { api_key: String },
    /// User picked a project — App should call `write_config` +
    /// `set_session_env` + `sync_issue_types`.
    PickedProject {
        provider: KanbanOnboardingProvider,
        project_key: String,
        project_name: String,
    },
    /// User pressed C to copy the export block to clipboard.
    CopyExportBlock,
    /// User dismissed the dialog.
    Cancelled,
    /// Final dismissal after Done.
    Done,
}

/// A project entry shown in the picker. Mirrors `dto::KanbanProjectInfo`
/// but lives here so the dialog has no `rest::dto` dependency.
#[derive(Debug, Clone)]
pub struct KanbanOnboardingProject {
    /// Provider-specific opaque ID. Currently unused by the dialog itself
    /// (we display key + name) but kept on the type because it's part of
    /// the data contract surfaced to App-side handlers.
    #[allow(dead_code)]
    pub id: String,
    pub key: String,
    pub name: String,
}

pub struct KanbanOnboardingDialog {
    pub visible: bool,
    pub state: KanbanOnboardingState,
    pub provider: KanbanOnboardingProvider,
    /// Picker selection on the `PickProvider` step.
    provider_index: usize,

    // Input buffers (separate per field — we don't share across steps)
    domain_buf: String,
    email_buf: String,
    token_buf: String,
    api_key_buf: String,
    cursor_position: usize,

    // Validation results — populated by App after validate_credentials
    pub jira_account_id: String,
    pub jira_display_name: String,
    pub linear_user_id: String,
    pub linear_user_name: String,
    pub linear_org_name: String,

    // Project picker
    projects: Vec<KanbanOnboardingProject>,
    project_list_state: ListState,

    // Result of write_config + set_session_env
    export_block: String,
    success_message: String,

    // Inline error message (shown in Error state)
    error_message: String,
}

impl Default for KanbanOnboardingDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl KanbanOnboardingDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            state: KanbanOnboardingState::PickProvider,
            provider: KanbanOnboardingProvider::Jira,
            provider_index: 0,
            domain_buf: String::new(),
            email_buf: String::new(),
            token_buf: String::new(),
            api_key_buf: String::new(),
            cursor_position: 0,
            jira_account_id: String::new(),
            jira_display_name: String::new(),
            linear_user_id: String::new(),
            linear_user_name: String::new(),
            linear_org_name: String::new(),
            projects: Vec::new(),
            project_list_state: ListState::default(),
            export_block: String::new(),
            success_message: String::new(),
            error_message: String::new(),
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.state = KanbanOnboardingState::PickProvider;
        self.provider_index = 0;
        self.domain_buf.clear();
        self.email_buf.clear();
        self.token_buf.clear();
        self.api_key_buf.clear();
        self.cursor_position = 0;
        self.jira_account_id.clear();
        self.jira_display_name.clear();
        self.linear_user_id.clear();
        self.linear_user_name.clear();
        self.linear_org_name.clear();
        self.projects.clear();
        self.project_list_state.select(None);
        self.export_block.clear();
        self.success_message.clear();
        self.error_message.clear();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        // Wipe sensitive fields
        self.domain_buf.clear();
        self.email_buf.clear();
        self.token_buf.clear();
        self.api_key_buf.clear();
    }

    // ─── Setters called by App after async work ─────────────────────────

    pub fn set_validation_jira(&mut self, account_id: String, display_name: String) {
        self.jira_account_id = account_id;
        self.jira_display_name = display_name;
    }

    pub fn set_validation_linear(&mut self, user_id: String, user_name: String, org_name: String) {
        self.linear_user_id = user_id;
        self.linear_user_name = user_name;
        self.linear_org_name = org_name;
    }

    pub fn set_projects(&mut self, projects: Vec<KanbanOnboardingProject>) {
        self.projects = projects;
        if !self.projects.is_empty() {
            self.project_list_state.select(Some(0));
        }
        self.state = KanbanOnboardingState::PickProject;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = msg;
        self.state = KanbanOnboardingState::Error;
    }

    pub fn set_success(&mut self, success_message: String, export_block: String) {
        self.success_message = success_message;
        self.export_block = export_block;
        self.state = KanbanOnboardingState::EnvExportNudge;
    }

    /// Get the shell export block to display. Used by tests and may be used
    /// by future clipboard integration.
    #[allow(dead_code)]
    pub fn export_block(&self) -> &str {
        &self.export_block
    }

    // ─── Input helpers ───────────────────────────────────────────────────

    fn current_buf(&self) -> &str {
        match self.state {
            KanbanOnboardingState::JiraDomain => &self.domain_buf,
            KanbanOnboardingState::JiraEmail => &self.email_buf,
            KanbanOnboardingState::JiraToken => &self.token_buf,
            KanbanOnboardingState::LinearApiKey => &self.api_key_buf,
            _ => "",
        }
    }

    fn current_buf_mut(&mut self) -> Option<&mut String> {
        match self.state {
            KanbanOnboardingState::JiraDomain => Some(&mut self.domain_buf),
            KanbanOnboardingState::JiraEmail => Some(&mut self.email_buf),
            KanbanOnboardingState::JiraToken => Some(&mut self.token_buf),
            KanbanOnboardingState::LinearApiKey => Some(&mut self.api_key_buf),
            _ => None,
        }
    }

    fn input_label(&self) -> &'static str {
        match self.state {
            KanbanOnboardingState::JiraDomain => "Jira domain (e.g. acme.atlassian.net)",
            KanbanOnboardingState::JiraEmail => "Jira email",
            KanbanOnboardingState::JiraToken => "Jira API token",
            KanbanOnboardingState::LinearApiKey => "Linear API key (lin_api_…)",
            _ => "",
        }
    }

    fn is_password_step(&self) -> bool {
        matches!(
            self.state,
            KanbanOnboardingState::JiraToken | KanbanOnboardingState::LinearApiKey
        )
    }

    fn validate_current_input(&self) -> Result<(), &'static str> {
        match self.state {
            KanbanOnboardingState::JiraDomain => {
                if self.domain_buf.is_empty() {
                    Err("Domain is required")
                } else if !self.domain_buf.ends_with(".atlassian.net") {
                    Err("Must end in .atlassian.net")
                } else {
                    Ok(())
                }
            }
            KanbanOnboardingState::JiraEmail => {
                if self.email_buf.is_empty() {
                    Err("Email is required")
                } else if !self.email_buf.contains('@') || !self.email_buf.contains('.') {
                    Err("Enter a valid email")
                } else {
                    Ok(())
                }
            }
            KanbanOnboardingState::JiraToken => {
                if self.token_buf.is_empty() {
                    Err("API token is required")
                } else {
                    Ok(())
                }
            }
            KanbanOnboardingState::LinearApiKey => {
                if self.api_key_buf.is_empty() {
                    Err("API key is required")
                } else if !self.api_key_buf.starts_with("lin_api_") {
                    Err("Linear API keys start with \"lin_api_\"")
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    // ─── Key handling ────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyCode) -> KanbanOnboardingAction {
        // Block input during async ops
        if matches!(
            self.state,
            KanbanOnboardingState::Validating | KanbanOnboardingState::Writing
        ) {
            return KanbanOnboardingAction::None;
        }

        match self.state {
            KanbanOnboardingState::PickProvider => self.handle_pick_provider_key(key),
            KanbanOnboardingState::JiraDomain
            | KanbanOnboardingState::JiraEmail
            | KanbanOnboardingState::JiraToken
            | KanbanOnboardingState::LinearApiKey => self.handle_input_key(key),
            KanbanOnboardingState::PickProject => self.handle_pick_project_key(key),
            KanbanOnboardingState::EnvExportNudge => self.handle_nudge_key(key),
            KanbanOnboardingState::Error => self.handle_error_key(key),
            KanbanOnboardingState::Validating | KanbanOnboardingState::Writing => {
                KanbanOnboardingAction::None
            }
        }
    }

    fn handle_pick_provider_key(&mut self, key: KeyCode) -> KanbanOnboardingAction {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.provider_index > 0 {
                    self.provider_index -= 1;
                }
                KanbanOnboardingAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.provider_index < 1 {
                    self.provider_index += 1;
                }
                KanbanOnboardingAction::None
            }
            KeyCode::Enter => {
                self.provider = if self.provider_index == 0 {
                    KanbanOnboardingProvider::Jira
                } else {
                    KanbanOnboardingProvider::Linear
                };
                self.state = match self.provider {
                    KanbanOnboardingProvider::Jira => KanbanOnboardingState::JiraDomain,
                    KanbanOnboardingProvider::Linear => KanbanOnboardingState::LinearApiKey,
                };
                self.cursor_position = 0;
                self.error_message.clear();
                KanbanOnboardingAction::PickedProvider(self.provider)
            }
            KeyCode::Esc => {
                self.hide();
                KanbanOnboardingAction::Cancelled
            }
            _ => KanbanOnboardingAction::None,
        }
    }

    fn handle_input_key(&mut self, key: KeyCode) -> KanbanOnboardingAction {
        match key {
            KeyCode::Char(c) => {
                let pos = self.cursor_position;
                let inserted = if let Some(buf) = self.current_buf_mut() {
                    if pos <= buf.len() {
                        buf.insert(pos, c);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if inserted {
                    self.cursor_position += 1;
                }
                self.error_message.clear();
                KanbanOnboardingAction::None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    let pos = self.cursor_position;
                    if let Some(buf) = self.current_buf_mut() {
                        if pos < buf.len() {
                            buf.remove(pos);
                        }
                    }
                }
                self.error_message.clear();
                KanbanOnboardingAction::None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                KanbanOnboardingAction::None
            }
            KeyCode::Right => {
                let len = self.current_buf().len();
                if self.cursor_position < len {
                    self.cursor_position += 1;
                }
                KanbanOnboardingAction::None
            }
            KeyCode::Enter => {
                if let Err(msg) = self.validate_current_input() {
                    self.error_message = msg.to_string();
                    return KanbanOnboardingAction::None;
                }
                // Advance to next step
                match self.state {
                    KanbanOnboardingState::JiraDomain => {
                        self.state = KanbanOnboardingState::JiraEmail;
                        self.cursor_position = self.email_buf.len();
                        KanbanOnboardingAction::None
                    }
                    KanbanOnboardingState::JiraEmail => {
                        self.state = KanbanOnboardingState::JiraToken;
                        self.cursor_position = self.token_buf.len();
                        KanbanOnboardingAction::None
                    }
                    KanbanOnboardingState::JiraToken => {
                        // Submit creds — App will dispatch validate
                        self.state = KanbanOnboardingState::Validating;
                        KanbanOnboardingAction::SubmitJiraCreds {
                            domain: self.domain_buf.clone(),
                            email: self.email_buf.clone(),
                            token: self.token_buf.clone(),
                        }
                    }
                    KanbanOnboardingState::LinearApiKey => {
                        self.state = KanbanOnboardingState::Validating;
                        KanbanOnboardingAction::SubmitLinearCreds {
                            api_key: self.api_key_buf.clone(),
                        }
                    }
                    _ => KanbanOnboardingAction::None,
                }
            }
            KeyCode::Esc => {
                // Go back one step
                self.error_message.clear();
                match self.state {
                    KanbanOnboardingState::JiraDomain => {
                        self.state = KanbanOnboardingState::PickProvider;
                    }
                    KanbanOnboardingState::JiraEmail => {
                        self.state = KanbanOnboardingState::JiraDomain;
                        self.cursor_position = self.domain_buf.len();
                    }
                    KanbanOnboardingState::JiraToken => {
                        self.state = KanbanOnboardingState::JiraEmail;
                        self.cursor_position = self.email_buf.len();
                    }
                    KanbanOnboardingState::LinearApiKey => {
                        self.state = KanbanOnboardingState::PickProvider;
                    }
                    _ => {}
                }
                KanbanOnboardingAction::None
            }
            _ => KanbanOnboardingAction::None,
        }
    }

    fn handle_pick_project_key(&mut self, key: KeyCode) -> KanbanOnboardingAction {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                let cur = self.project_list_state.selected().unwrap_or(0);
                if cur > 0 {
                    self.project_list_state.select(Some(cur - 1));
                }
                KanbanOnboardingAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let cur = self.project_list_state.selected().unwrap_or(0);
                if cur + 1 < self.projects.len() {
                    self.project_list_state.select(Some(cur + 1));
                }
                KanbanOnboardingAction::None
            }
            KeyCode::Enter => {
                let idx = self.project_list_state.selected().unwrap_or(0);
                if let Some(p) = self.projects.get(idx) {
                    self.state = KanbanOnboardingState::Writing;
                    KanbanOnboardingAction::PickedProject {
                        provider: self.provider,
                        project_key: p.key.clone(),
                        project_name: p.name.clone(),
                    }
                } else {
                    KanbanOnboardingAction::None
                }
            }
            KeyCode::Esc => {
                self.hide();
                KanbanOnboardingAction::Cancelled
            }
            _ => KanbanOnboardingAction::None,
        }
    }

    fn handle_nudge_key(&mut self, key: KeyCode) -> KanbanOnboardingAction {
        match key {
            KeyCode::Char('c' | 'C') => KanbanOnboardingAction::CopyExportBlock,
            KeyCode::Enter | KeyCode::Esc => {
                self.hide();
                KanbanOnboardingAction::Done
            }
            _ => KanbanOnboardingAction::None,
        }
    }

    fn handle_error_key(&mut self, key: KeyCode) -> KanbanOnboardingAction {
        match key {
            KeyCode::Enter => {
                // Retry from the relevant input step
                self.error_message.clear();
                self.state = match self.provider {
                    KanbanOnboardingProvider::Jira => KanbanOnboardingState::JiraToken,
                    KanbanOnboardingProvider::Linear => KanbanOnboardingState::LinearApiKey,
                };
                KanbanOnboardingAction::None
            }
            KeyCode::Esc => {
                self.hide();
                KanbanOnboardingAction::Cancelled
            }
            _ => KanbanOnboardingAction::None,
        }
    }

    // ─── Rendering ──────────────────────────────────────────────────────

    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let title = match self.provider {
            KanbanOnboardingProvider::Jira => " Onboard: Jira Cloud ",
            KanbanOnboardingProvider::Linear => " Onboard: Linear ",
        };
        let title = if matches!(self.state, KanbanOnboardingState::PickProvider) {
            " Connect Kanban Provider "
        } else {
            title
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match self.state {
            KanbanOnboardingState::PickProvider => self.render_pick_provider(frame, inner),
            KanbanOnboardingState::JiraDomain
            | KanbanOnboardingState::JiraEmail
            | KanbanOnboardingState::JiraToken
            | KanbanOnboardingState::LinearApiKey => self.render_input(frame, inner),
            KanbanOnboardingState::Validating => {
                self.render_progress(frame, inner, "Validating credentials...");
            }
            KanbanOnboardingState::PickProject => self.render_pick_project(frame, inner),
            KanbanOnboardingState::Writing => {
                self.render_progress(frame, inner, "Writing config + syncing issue types...");
            }
            KanbanOnboardingState::EnvExportNudge => self.render_nudge(frame, inner),
            KanbanOnboardingState::Error => self.render_error(frame, inner),
        }
    }

    fn render_pick_provider(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Prompt
                Constraint::Length(1), // Spacer
                Constraint::Min(4),    // Options
                Constraint::Length(2), // Footer
            ])
            .split(area);

        let prompt = Paragraph::new("Which kanban provider do you use?")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(prompt, chunks[0]);

        let options: Vec<Line> = vec![
            self.option_line("Jira Cloud", "Connect with API token", 0),
            self.option_line("Linear", "Connect with API key", 1),
        ];
        let opts_widget = Paragraph::new(options).alignment(Alignment::Center);
        frame.render_widget(opts_widget, chunks[2]);

        let footer = Line::from(vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Confirm  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[3],
        );
    }

    fn option_line(&self, label: &str, desc: &str, index: usize) -> Line<'static> {
        let selected = index == self.provider_index;
        let marker = if selected { "▶ " } else { "  " };
        let style = if selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        Line::from(vec![
            Span::styled(marker.to_string(), style),
            Span::styled(label.to_string(), style),
            Span::raw("  "),
            Span::styled(format!("({desc})"), Style::default().fg(Color::DarkGray)),
        ])
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let has_error = !self.error_message.is_empty();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(if has_error {
                vec![
                    Constraint::Length(2), // Label
                    Constraint::Length(3), // Input
                    Constraint::Length(2), // Error
                    Constraint::Min(0),    // Spacer
                    Constraint::Length(2), // Footer
                ]
            } else {
                vec![
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(2),
                    Constraint::Length(0),
                ]
            })
            .split(area);

        let label = Paragraph::new(Line::from(vec![Span::styled(
            self.input_label().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]));
        frame.render_widget(label, chunks[0]);

        let display: String = if self.is_password_step() {
            "•".repeat(self.current_buf().len())
        } else {
            self.current_buf().to_string()
        };
        let input = Paragraph::new(display)
            .block(Block::default().borders(Borders::ALL).border_style(
                Style::default().fg(if has_error { Color::Red } else { Color::Cyan }),
            ))
            .wrap(Wrap { trim: false });
        frame.render_widget(input, chunks[1]);

        // Cursor
        let input_inner = Block::default().borders(Borders::ALL).inner(chunks[1]);
        frame.set_cursor_position((input_inner.x + self.cursor_position as u16, input_inner.y));

        if has_error {
            let err = Paragraph::new(Line::from(vec![Span::styled(
                self.error_message.clone(),
                Style::default().fg(Color::Red),
            )]));
            frame.render_widget(err, chunks[2]);
        }

        let footer_idx = if has_error { 4 } else { 3 };
        let footer = Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Next  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[footer_idx],
        );
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect, message: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Min(2), // Spacer
                Constraint::Length(2),
                Constraint::Min(2), // Spacer
            ])
            .split(area);
        let p = Paragraph::new(Line::from(vec![Span::styled(
            message.to_string(),
            Style::default().fg(Color::Yellow),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(p, chunks[1]);
    }

    fn render_pick_project(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(5),    // List
                Constraint::Length(2), // Footer
            ])
            .split(area);

        let auth_msg = match self.provider {
            KanbanOnboardingProvider::Jira => format!(
                "Authenticated as {} ({})",
                self.jira_display_name, self.jira_account_id
            ),
            KanbanOnboardingProvider::Linear => format!(
                "Authenticated as {} in {}",
                self.linear_user_name, self.linear_org_name
            ),
        };
        let header = Paragraph::new(Line::from(vec![
            Span::raw("✓ "),
            Span::styled(auth_msg, Style::default().fg(Color::Green)),
        ]));
        frame.render_widget(header, chunks[0]);

        let items: Vec<ListItem> = self
            .projects
            .iter()
            .map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:8}", p.key),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — "),
                    Span::styled(p.name.clone(), Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Projects ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, chunks[1], &mut self.project_list_state);

        let footer = Line::from(vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Confirm  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn render_nudge(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Success
                Constraint::Length(2), // Instructions
                Constraint::Length(3), // Export block
                Constraint::Min(0),
                Constraint::Length(2), // Footer
            ])
            .split(area);

        let success = Paragraph::new(Line::from(vec![
            Span::raw("✓ "),
            Span::styled(
                self.success_message.clone(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        frame.render_widget(success, chunks[0]);

        let instructions = Paragraph::new(
            "Add this to your shell profile (~/.zshrc or ~/.bashrc) for persistence:",
        )
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        let export = Paragraph::new(self.export_block.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(export, chunks[2]);

        let footer = Line::from(vec![
            Span::styled("[C]", Style::default().fg(Color::Yellow)),
            Span::raw(" Copy  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Done"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[4],
        );
    }

    fn render_error(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(2),
                Constraint::Length(2),
            ])
            .split(area);

        let header = Paragraph::new(Line::from(vec![
            Span::raw("✗ "),
            Span::styled(
                "Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        frame.render_widget(header, chunks[0]);

        let body = Paragraph::new(self.error_message.clone())
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: false });
        frame.render_widget(body, chunks[1]);

        let footer = Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Retry  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[2],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_starts_hidden() {
        let dialog = KanbanOnboardingDialog::new();
        assert!(!dialog.visible);
        assert_eq!(dialog.state, KanbanOnboardingState::PickProvider);
    }

    #[test]
    fn test_show_resets_state() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.domain_buf = "stale".to_string();
        dialog.show();
        assert!(dialog.visible);
        assert!(dialog.domain_buf.is_empty());
        assert_eq!(dialog.state, KanbanOnboardingState::PickProvider);
    }

    #[test]
    fn test_pick_jira_advances_to_jira_domain() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        let action = dialog.handle_key(KeyCode::Enter);
        assert_eq!(
            action,
            KanbanOnboardingAction::PickedProvider(KanbanOnboardingProvider::Jira)
        );
        assert_eq!(dialog.state, KanbanOnboardingState::JiraDomain);
    }

    #[test]
    fn test_pick_linear_advances_to_api_key() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Down);
        let action = dialog.handle_key(KeyCode::Enter);
        assert_eq!(
            action,
            KanbanOnboardingAction::PickedProvider(KanbanOnboardingProvider::Linear)
        );
        assert_eq!(dialog.state, KanbanOnboardingState::LinearApiKey);
    }

    #[test]
    fn test_jira_domain_validation_rejects_non_atlassian() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Enter); // pick Jira
        for c in "notjira.example.com".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        let action = dialog.handle_key(KeyCode::Enter);
        assert_eq!(action, KanbanOnboardingAction::None);
        assert_eq!(dialog.state, KanbanOnboardingState::JiraDomain);
        assert!(!dialog.error_message.is_empty());
    }

    #[test]
    fn test_jira_full_flow_to_validating() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Enter); // pick Jira

        // Domain
        for c in "acme.atlassian.net".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        dialog.handle_key(KeyCode::Enter);
        assert_eq!(dialog.state, KanbanOnboardingState::JiraEmail);

        // Email
        for c in "u@acme.com".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        dialog.handle_key(KeyCode::Enter);
        assert_eq!(dialog.state, KanbanOnboardingState::JiraToken);

        // Token
        for c in "secret-token".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        let action = dialog.handle_key(KeyCode::Enter);
        match action {
            KanbanOnboardingAction::SubmitJiraCreds {
                domain,
                email,
                token,
            } => {
                assert_eq!(domain, "acme.atlassian.net");
                assert_eq!(email, "u@acme.com");
                assert_eq!(token, "secret-token");
            }
            other => panic!("expected SubmitJiraCreds, got {other:?}"),
        }
        assert_eq!(dialog.state, KanbanOnboardingState::Validating);
    }

    #[test]
    fn test_linear_validation_rejects_wrong_prefix() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Down);
        dialog.handle_key(KeyCode::Enter); // pick Linear

        for c in "wrong_prefix_xxx".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        let action = dialog.handle_key(KeyCode::Enter);
        assert_eq!(action, KanbanOnboardingAction::None);
        assert_eq!(dialog.state, KanbanOnboardingState::LinearApiKey);
        assert!(dialog.error_message.contains("lin_api_"));
    }

    #[test]
    fn test_linear_full_flow_to_validating() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Down);
        dialog.handle_key(KeyCode::Enter); // pick Linear

        for c in "lin_api_realtoken".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        let action = dialog.handle_key(KeyCode::Enter);
        match action {
            KanbanOnboardingAction::SubmitLinearCreds { api_key } => {
                assert_eq!(api_key, "lin_api_realtoken");
            }
            other => panic!("expected SubmitLinearCreds, got {other:?}"),
        }
        assert_eq!(dialog.state, KanbanOnboardingState::Validating);
    }

    #[test]
    fn test_set_projects_advances_to_pick_project() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.state = KanbanOnboardingState::Validating;
        dialog.set_projects(vec![KanbanOnboardingProject {
            id: "1".to_string(),
            key: "PROJ".to_string(),
            name: "My Project".to_string(),
        }]);
        assert_eq!(dialog.state, KanbanOnboardingState::PickProject);
        assert_eq!(dialog.project_list_state.selected(), Some(0));
    }

    #[test]
    fn test_pick_project_emits_action_with_project_key() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Enter); // pick Jira
        dialog.provider = KanbanOnboardingProvider::Jira;
        dialog.set_projects(vec![
            KanbanOnboardingProject {
                id: "1".to_string(),
                key: "PROJ".to_string(),
                name: "First".to_string(),
            },
            KanbanOnboardingProject {
                id: "2".to_string(),
                key: "OTHER".to_string(),
                name: "Second".to_string(),
            },
        ]);
        // Move to second
        dialog.handle_key(KeyCode::Down);
        let action = dialog.handle_key(KeyCode::Enter);
        match action {
            KanbanOnboardingAction::PickedProject {
                provider,
                project_key,
                project_name,
            } => {
                assert_eq!(provider, KanbanOnboardingProvider::Jira);
                assert_eq!(project_key, "OTHER");
                assert_eq!(project_name, "Second");
            }
            other => panic!("expected PickedProject, got {other:?}"),
        }
        assert_eq!(dialog.state, KanbanOnboardingState::Writing);
    }

    #[test]
    fn test_set_error_transitions_to_error_state() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.set_error("Invalid credentials".to_string());
        assert_eq!(dialog.state, KanbanOnboardingState::Error);
        assert_eq!(dialog.error_message, "Invalid credentials");
    }

    #[test]
    fn test_error_retry_returns_to_token_step_for_jira() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.provider = KanbanOnboardingProvider::Jira;
        dialog.set_error("Auth failed".to_string());
        dialog.handle_key(KeyCode::Enter);
        assert_eq!(dialog.state, KanbanOnboardingState::JiraToken);
        assert!(dialog.error_message.is_empty());
    }

    #[test]
    fn test_set_success_transitions_to_nudge() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.set_success(
            "Jira configured!".to_string(),
            "export OPERATOR_JIRA_API_KEY=\"<your-token>\"".to_string(),
        );
        assert_eq!(dialog.state, KanbanOnboardingState::EnvExportNudge);
        assert!(dialog.export_block().contains("OPERATOR_JIRA_API_KEY"));
    }

    #[test]
    fn test_nudge_copy_emits_action() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.set_success("ok".to_string(), "export FOO=bar".to_string());
        let action = dialog.handle_key(KeyCode::Char('c'));
        assert_eq!(action, KanbanOnboardingAction::CopyExportBlock);
    }

    #[test]
    fn test_nudge_enter_dismisses() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.set_success("ok".to_string(), "export FOO=bar".to_string());
        let action = dialog.handle_key(KeyCode::Enter);
        assert_eq!(action, KanbanOnboardingAction::Done);
        assert!(!dialog.visible);
    }

    #[test]
    fn test_validating_state_blocks_input() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.state = KanbanOnboardingState::Validating;
        let action = dialog.handle_key(KeyCode::Char('x'));
        assert_eq!(action, KanbanOnboardingAction::None);
        // State unchanged
        assert_eq!(dialog.state, KanbanOnboardingState::Validating);
    }

    #[test]
    fn test_backspace_removes_char_from_buffer() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Enter); // pick Jira
        for c in "abc".chars() {
            dialog.handle_key(KeyCode::Char(c));
        }
        dialog.handle_key(KeyCode::Backspace);
        assert_eq!(dialog.domain_buf, "ab");
        assert_eq!(dialog.cursor_position, 2);
    }

    #[test]
    fn test_esc_from_input_goes_back_one_step() {
        let mut dialog = KanbanOnboardingDialog::new();
        dialog.show();
        dialog.handle_key(KeyCode::Enter); // pick Jira
                                           // Now in JiraDomain. Esc goes back to PickProvider.
        dialog.handle_key(KeyCode::Esc);
        assert_eq!(dialog.state, KanbanOnboardingState::PickProvider);
    }
}
