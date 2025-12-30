#![allow(dead_code)]

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::config::LlmProvider;
use crate::queue::Ticket;
use crate::ui::keybindings::{shortcuts_by_category_for_context, ShortcutContext};

/// Selection state for the confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmSelection {
    Yes = 0,
    View = 1,
    No = 2,
}

impl ConfirmSelection {
    fn next(self) -> Self {
        match self {
            Self::Yes => Self::View,
            Self::View => Self::No,
            Self::No => Self::Yes,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Yes => Self::No,
            Self::View => Self::Yes,
            Self::No => Self::View,
        }
    }
}

/// Focus area in the confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmDialogFocus {
    /// Focused on the Yes/View/No buttons (default)
    Buttons,
    /// Focused on the options section (provider, project)
    Options,
}

/// Which option is currently selected in the options section
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedOption {
    Provider = 0,
    Project = 1,
}

impl SelectedOption {
    fn next(self) -> Self {
        match self {
            Self::Provider => Self::Project,
            Self::Project => Self::Provider,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Provider => Self::Project,
            Self::Project => Self::Provider,
        }
    }
}

pub struct ConfirmDialog {
    pub visible: bool,
    pub ticket: Option<Ticket>,
    pub selection: ConfirmSelection,

    // Focus and navigation state
    /// Current focus area (buttons or options)
    pub focus: ConfirmDialogFocus,
    /// Currently selected option when in Options focus
    pub selected_option: SelectedOption,

    // Launch options (only shown if config enables them)
    /// Available LLM providers (tool + model combinations)
    pub provider_options: Vec<LlmProvider>,
    /// Currently selected provider index
    pub selected_provider: usize,
    /// Whether docker mode option is available
    pub docker_enabled: bool,
    /// Whether docker mode is selected
    pub docker_selected: bool,
    /// Whether YOLO mode option is available
    pub yolo_enabled: bool,
    /// Whether YOLO mode is selected
    pub yolo_selected: bool,

    // Project override options
    /// Available projects for override (includes "global" option)
    pub project_options: Vec<String>,
    /// Currently selected project index (0 = ticket's original project)
    pub selected_project: usize,
    /// The original project from the ticket (for display reference)
    pub original_project: String,
}

impl ConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            ticket: None,
            selection: ConfirmSelection::Yes,
            focus: ConfirmDialogFocus::Buttons,
            selected_option: SelectedOption::Provider,
            provider_options: Vec::new(),
            selected_provider: 0,
            docker_enabled: false,
            docker_selected: false,
            yolo_enabled: false,
            yolo_selected: false,
            project_options: Vec::new(),
            selected_project: 0,
            original_project: String::new(),
        }
    }

    /// Configure the dialog with available options from config
    pub fn configure(
        &mut self,
        providers: Vec<LlmProvider>,
        projects: Vec<String>,
        docker_enabled: bool,
        yolo_enabled: bool,
    ) {
        self.provider_options = providers;
        self.project_options = projects;
        self.docker_enabled = docker_enabled;
        self.yolo_enabled = yolo_enabled;
    }

    pub fn show(&mut self, ticket: Ticket) {
        // Store original project from ticket
        self.original_project = ticket.project.clone();

        // Reset project selection to match ticket's project if it exists in options
        self.selected_project = self
            .project_options
            .iter()
            .position(|p| p == &ticket.project)
            .unwrap_or(0);

        self.ticket = Some(ticket);
        self.visible = true;
        self.selection = ConfirmSelection::Yes; // Default to Yes
        self.focus = ConfirmDialogFocus::Buttons; // Default to buttons
        self.selected_option = SelectedOption::Provider;
        // Reset mode selections but keep provider selection
        self.docker_selected = false;
        self.yolo_selected = false;
    }

    /// Cycle to the next provider
    pub fn cycle_provider(&mut self) {
        if !self.provider_options.is_empty() {
            self.selected_provider = (self.selected_provider + 1) % self.provider_options.len();
        }
    }

    /// Toggle docker mode
    pub fn toggle_docker(&mut self) {
        if self.docker_enabled {
            self.docker_selected = !self.docker_selected;
        }
    }

    /// Toggle YOLO mode
    pub fn toggle_yolo(&mut self) {
        if self.yolo_enabled {
            self.yolo_selected = !self.yolo_selected;
        }
    }

    /// Get the selected provider (if any)
    pub fn selected_provider(&self) -> Option<&LlmProvider> {
        self.provider_options.get(self.selected_provider)
    }

    /// Check if any options are available
    pub fn has_options(&self) -> bool {
        self.provider_options.len() > 1
            || self.project_options.len() > 1
            || self.docker_enabled
            || self.yolo_enabled
    }

    /// Cycle to the next project
    pub fn cycle_project(&mut self) {
        if !self.project_options.is_empty() {
            self.selected_project = (self.selected_project + 1) % self.project_options.len();
        }
    }

    /// Cycle to the previous project
    pub fn cycle_project_prev(&mut self) {
        if !self.project_options.is_empty() {
            if self.selected_project == 0 {
                self.selected_project = self.project_options.len() - 1;
            } else {
                self.selected_project -= 1;
            }
        }
    }

    /// Get the selected project name
    pub fn selected_project_name(&self) -> Option<&String> {
        self.project_options.get(self.selected_project)
    }

    /// Check if the project has been overridden from the original
    pub fn is_project_overridden(&self) -> bool {
        self.selected_project_name()
            .is_some_and(|p| p != &self.original_project)
    }

    /// Move focus to options section
    pub fn focus_options(&mut self) {
        if self.has_options() {
            self.focus = ConfirmDialogFocus::Options;
        }
    }

    /// Move focus to buttons section
    pub fn focus_buttons(&mut self) {
        self.focus = ConfirmDialogFocus::Buttons;
    }

    /// Check if focused on options
    pub fn is_options_focused(&self) -> bool {
        self.focus == ConfirmDialogFocus::Options
    }

    /// Navigate to next option (within options section)
    pub fn next_option(&mut self) {
        self.selected_option = self.selected_option.next();
    }

    /// Navigate to previous option (within options section)
    pub fn prev_option(&mut self) {
        self.selected_option = self.selected_option.prev();
    }

    /// Cycle the currently selected option's value
    pub fn cycle_current_option(&mut self) {
        match self.selected_option {
            SelectedOption::Provider => self.cycle_provider(),
            SelectedOption::Project => self.cycle_project(),
        }
    }

    /// Cycle the currently selected option's value in reverse
    pub fn cycle_current_option_prev(&mut self) {
        match self.selected_option {
            SelectedOption::Provider => {
                // Cycle provider in reverse
                if !self.provider_options.is_empty() {
                    if self.selected_provider == 0 {
                        self.selected_provider = self.provider_options.len() - 1;
                    } else {
                        self.selected_provider -= 1;
                    }
                }
            }
            SelectedOption::Project => self.cycle_project_prev(),
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.ticket = None;
    }

    pub fn select_next(&mut self) {
        self.selection = self.selection.next();
    }

    pub fn select_prev(&mut self) {
        self.selection = self.selection.prev();
    }

    /// Get the ticket filepath for viewing/editing
    pub fn ticket_filepath(&self) -> Option<String> {
        self.ticket.as_ref().map(|t| t.filepath.clone())
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let Some(ticket) = &self.ticket else {
            return;
        };

        // Calculate dialog height based on options
        let has_options = self.has_options();
        let dialog_height = if has_options { 60 } else { 45 };

        // Center the dialog
        let area = centered_rect(60, dialog_height, frame.area());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Launch Agent? ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate options height
        let options_height = if has_options { 6 } else { 0 };

        // Dialog content layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),              // Type/ID
                Constraint::Min(3),                 // Summary
                Constraint::Length(2),              // Project
                Constraint::Length(2),              // Priority
                Constraint::Length(options_height), // Options (if any)
                Constraint::Length(3),              // Buttons
            ])
            .margin(1)
            .split(inner);

        // Type and ID
        let type_line = Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &ticket.ticket_type,
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(&ticket.id, Style::default().add_modifier(Modifier::BOLD)),
        ]);
        frame.render_widget(Paragraph::new(type_line), chunks[0]);

        // Summary (now second, after Type/ID)
        let summary = Paragraph::new(ticket.summary.clone())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        frame.render_widget(summary, chunks[1]);

        // Project
        let project_line = Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &ticket.project,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(project_line), chunks[2]);

        // Priority
        let priority_color = match ticket.priority.as_str() {
            "P0-critical" => Color::Red,
            "P1-high" => Color::Yellow,
            "P2-medium" => Color::White,
            _ => Color::Gray,
        };
        let priority_line = Line::from(vec![
            Span::styled("Priority: ", Style::default().fg(Color::Gray)),
            Span::styled(&ticket.priority, Style::default().fg(priority_color)),
        ]);
        frame.render_widget(Paragraph::new(priority_line), chunks[3]);

        // Render options section if any are available
        if has_options {
            self.render_options(frame, chunks[4]);
        }

        // Dim buttons when options are focused
        let buttons_focused = self.focus == ConfirmDialogFocus::Buttons;

        // Buttons - Yes, View, No
        let yes_style = if self.selection == ConfirmSelection::Yes && buttons_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if buttons_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let view_style = if self.selection == ConfirmSelection::View && buttons_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else if buttons_focused {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let no_style = if self.selection == ConfirmSelection::No && buttons_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else if buttons_focused {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let buttons = Line::from(vec![
            Span::raw("   "),
            Span::styled(" [Y]es ", yes_style),
            Span::raw("   "),
            Span::styled(" [V]iew ", view_style),
            Span::raw("   "),
            Span::styled(" [N]o ", no_style),
        ]);

        // Add hint for navigating to options
        let hint = if has_options && buttons_focused {
            Line::from(vec![Span::styled(
                "↑ to edit launch options",
                Style::default().fg(Color::DarkGray),
            )])
        } else if has_options {
            Line::from(vec![Span::styled(
                "↓ to confirm, ←/→ to change value",
                Style::default().fg(Color::DarkGray),
            )])
        } else {
            Line::from("")
        };

        let buttons_para = Paragraph::new(vec![buttons, hint]).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[5]);
    }

    /// Render the options section (provider, project, docker, yolo toggles)
    fn render_options(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        let options_focused = self.focus == ConfirmDialogFocus::Options;

        // Separator line with focus indicator
        let separator_style = if options_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![Span::styled(
            "── Launch Options ──",
            separator_style,
        )]));

        // Provider option (if multiple providers)
        if self.provider_options.len() > 1 {
            let is_selected = options_focused && self.selected_option == SelectedOption::Provider;
            let provider_display = self
                .selected_provider()
                .map(|p| {
                    p.display_name
                        .clone()
                        .unwrap_or_else(|| format!("{} {}", p.tool, p.model))
                })
                .unwrap_or_else(|| "Default".to_string());

            let prefix = if is_selected { "▶ " } else { "  " };
            let value_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled("Provider: ", Style::default().fg(Color::Gray)),
                Span::styled(provider_display, value_style),
            ]));
        }

        // Project option (if multiple projects)
        if self.project_options.len() > 1 {
            let is_selected = options_focused && self.selected_option == SelectedOption::Project;
            let project_display = self
                .selected_project_name()
                .cloned()
                .unwrap_or_else(|| self.original_project.clone());

            let prefix = if is_selected { "▶ " } else { "  " };

            // Show if project is overridden from original
            let (value_style, suffix) = if self.is_project_overridden() {
                (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    " (override)",
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    "",
                )
            } else {
                (Style::default().fg(Color::White), "")
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled("Project:  ", Style::default().fg(Color::Gray)),
                Span::styled(project_display, value_style),
                Span::styled(suffix, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // Docker option
        if self.docker_enabled {
            let (indicator, color) = if self.docker_selected {
                ("●", Color::Green)
            } else {
                ("○", Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("[D] ", Style::default().fg(Color::Yellow)),
                Span::styled("Docker: ", Style::default().fg(Color::Gray)),
                Span::styled(indicator, Style::default().fg(color)),
                Span::styled(
                    if self.docker_selected { " On" } else { " Off" },
                    Style::default().fg(color),
                ),
            ]));
        }

        // YOLO option (use 'A' for Auto-accept to avoid conflict with 'Y' for Yes)
        if self.yolo_enabled {
            let (indicator, color) = if self.yolo_selected {
                ("●", Color::Red)
            } else {
                ("○", Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("[A] ", Style::default().fg(Color::Yellow)),
                Span::styled("Auto:   ", Style::default().fg(Color::Gray)),
                Span::styled(indicator, Style::default().fg(color)),
                Span::styled(
                    if self.yolo_selected {
                        " On (YOLO mode)"
                    } else {
                        " Off"
                    },
                    Style::default().fg(color),
                ),
            ]));
        }

        let options = Paragraph::new(lines);
        frame.render_widget(options, area);
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

/// Result from rejection dialog
#[derive(Debug, Clone)]
pub struct RejectionResult {
    pub reason: String,
    pub confirmed: bool,
}

/// Dialog for capturing rejection feedback
pub struct RejectionDialog {
    pub visible: bool,
    pub step_name: String,
    pub ticket_id: String,
    pub reason: String,
    pub cursor_position: usize,
}

impl RejectionDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            step_name: String::new(),
            ticket_id: String::new(),
            reason: String::new(),
            cursor_position: 0,
        }
    }

    /// Show the dialog for a specific step rejection
    pub fn show(&mut self, step_name: &str, ticket_id: &str) {
        self.step_name = step_name.to_string();
        self.ticket_id = ticket_id.to_string();
        self.reason.clear();
        self.cursor_position = 0;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.reason.clear();
    }

    /// Handle a character input
    pub fn handle_char(&mut self, c: char) {
        self.reason.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.reason.remove(self.cursor_position);
        }
    }

    /// Handle delete
    pub fn handle_delete(&mut self) {
        if self.cursor_position < self.reason.len() {
            self.reason.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.reason.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.reason.len();
    }

    /// Get the result and confirm
    pub fn confirm(&self) -> RejectionResult {
        RejectionResult {
            reason: self.reason.clone(),
            confirmed: true,
        }
    }

    /// Get the result and cancel
    pub fn cancel(&self) -> RejectionResult {
        RejectionResult {
            reason: String::new(),
            confirmed: false,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = centered_rect(60, 40, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Reject Step - Enter Reason ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Step/Ticket info
                Constraint::Length(2), // Label
                Constraint::Min(4),    // Text input
                Constraint::Length(2), // Instructions
            ])
            .margin(1)
            .split(inner);

        // Step and ticket info
        let info_line = Line::from(vec![
            Span::styled("Step: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &self.step_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Ticket: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &self.ticket_id,
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(info_line), chunks[0]);

        // Label
        let label = Paragraph::new(Line::from(vec![Span::styled(
            "Why is this being rejected?",
            Style::default().fg(Color::White),
        )]));
        frame.render_widget(label, chunks[1]);

        // Text input area with cursor
        let display_text = if self.reason.is_empty() {
            Span::styled(
                "Enter rejection reason...",
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled(&self.reason, Style::default().fg(Color::White))
        };

        let input = Paragraph::new(display_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(input, chunks[2]);

        // Set cursor position
        let input_inner = Block::default().borders(Borders::ALL).inner(chunks[2]);
        frame.set_cursor_position((input_inner.x + self.cursor_position as u16, input_inner.y));

        // Instructions
        let instructions = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" to confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" to cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(instructions).alignment(Alignment::Center),
            chunks[3],
        );
    }
}

pub struct HelpDialog {
    pub visible: bool,
}

impl HelpDialog {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let mut help_text = vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )),
            Line::from(""),
        ];

        // Add global shortcuts grouped by category
        for (category, shortcuts) in shortcuts_by_category_for_context(ShortcutContext::Global) {
            // Add category header (skip for first category to keep it compact)
            if category != crate::ui::keybindings::ShortcutCategory::General {
                help_text.push(Line::from(""));
            }

            for shortcut in shortcuts {
                help_text.push(Line::from(vec![
                    Span::styled(
                        shortcut.key_display_padded(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
        }

        // Add Launch Dialog section
        help_text.push(Line::from(""));
        help_text.push(Line::from(Span::styled(
            "In Launch Dialog:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )));

        for (_, shortcuts) in shortcuts_by_category_for_context(ShortcutContext::LaunchDialog) {
            for shortcut in shortcuts {
                help_text.push(Line::from(vec![
                    Span::styled(
                        shortcut.key_display_padded(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
        }

        // Add Session Preview section
        help_text.push(Line::from(""));
        help_text.push(Line::from(Span::styled(
            "In Session Preview:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )));

        for (_, shortcuts) in shortcuts_by_category_for_context(ShortcutContext::Preview) {
            for shortcut in shortcuts {
                help_text.push(Line::from(vec![
                    Span::styled(
                        shortcut.key_display_padded(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
        }

        // Footer
        help_text.push(Line::from(""));
        help_text.push(Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::Gray),
        )));

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(help, area);
    }
}

/// Selection state for session recovery dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRecoverySelection {
    ResumeSession = 0, // Only shown when session ID exists
    StartFresh = 1,
    ReturnToQueue = 2,
    Cancel = 3,
}

impl SessionRecoverySelection {
    /// Get the next selection (skipping ResumeSession if no session ID)
    fn next(self, has_session_id: bool) -> Self {
        match self {
            Self::ResumeSession => Self::StartFresh,
            Self::StartFresh => Self::ReturnToQueue,
            Self::ReturnToQueue => Self::Cancel,
            Self::Cancel => {
                if has_session_id {
                    Self::ResumeSession
                } else {
                    Self::StartFresh
                }
            }
        }
    }

    /// Get the previous selection (skipping ResumeSession if no session ID)
    fn prev(self, has_session_id: bool) -> Self {
        match self {
            Self::ResumeSession => Self::Cancel,
            Self::StartFresh => {
                if has_session_id {
                    Self::ResumeSession
                } else {
                    Self::Cancel
                }
            }
            Self::ReturnToQueue => Self::StartFresh,
            Self::Cancel => Self::ReturnToQueue,
        }
    }

    /// Get display label for this selection
    fn label(&self) -> &'static str {
        match self {
            Self::ResumeSession => "Resume session",
            Self::StartFresh => "Start fresh",
            Self::ReturnToQueue => "Return to queue",
            Self::Cancel => "Cancel",
        }
    }

    /// Get shortcut key for this selection
    fn key(&self) -> &'static str {
        match self {
            Self::ResumeSession => "R",
            Self::StartFresh => "S",
            Self::ReturnToQueue => "Q",
            Self::Cancel => "Esc",
        }
    }
}

/// Dialog shown when tmux session is not found for an in-progress ticket
pub struct SessionRecoveryDialog {
    pub visible: bool,
    pub ticket_id: String,
    pub session_name: String,
    pub step: String,
    /// The Claude session UUID if available (from ticket.sessions)
    pub claude_session_id: Option<String>,
    pub selection: SessionRecoverySelection,
}

impl SessionRecoveryDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            ticket_id: String::new(),
            session_name: String::new(),
            step: String::new(),
            claude_session_id: None,
            selection: SessionRecoverySelection::StartFresh,
        }
    }

    /// Show the dialog with ticket context
    pub fn show(
        &mut self,
        ticket_id: String,
        session_name: String,
        step: String,
        claude_session_id: Option<String>,
    ) {
        self.ticket_id = ticket_id;
        self.session_name = session_name;
        self.step = step;
        self.claude_session_id = claude_session_id.clone();
        // Default to ResumeSession if session ID exists, otherwise StartFresh
        self.selection = if claude_session_id.is_some() {
            SessionRecoverySelection::ResumeSession
        } else {
            SessionRecoverySelection::StartFresh
        };
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if a Claude session ID is available
    pub fn has_session_id(&self) -> bool {
        self.claude_session_id.is_some()
    }

    pub fn select_next(&mut self) {
        self.selection = self.selection.next(self.has_session_id());
    }

    pub fn select_prev(&mut self) {
        self.selection = self.selection.prev(self.has_session_id());
    }

    /// Get list of available options based on session ID presence
    fn available_options(&self) -> Vec<SessionRecoverySelection> {
        if self.has_session_id() {
            vec![
                SessionRecoverySelection::ResumeSession,
                SessionRecoverySelection::StartFresh,
                SessionRecoverySelection::ReturnToQueue,
                SessionRecoverySelection::Cancel,
            ]
        } else {
            vec![
                SessionRecoverySelection::StartFresh,
                SessionRecoverySelection::ReturnToQueue,
                SessionRecoverySelection::Cancel,
            ]
        }
    }

    /// Make the available_options method accessible for testing
    #[cfg(test)]
    pub fn available_options_for_test(&self) -> Vec<SessionRecoverySelection> {
        self.available_options()
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = centered_rect(55, 50, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Session Not Found ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Message
                Constraint::Length(4), // Ticket info
                Constraint::Min(5),    // Options
                Constraint::Length(2), // Instructions
            ])
            .margin(1)
            .split(inner);

        // Message
        let message = Paragraph::new(vec![Line::from(Span::styled(
            "The tmux session for this ticket no longer exists.",
            Style::default().fg(Color::White),
        ))])
        .wrap(Wrap { trim: true });
        frame.render_widget(message, chunks[0]);

        // Ticket info
        let info_lines = vec![
            Line::from(vec![
                Span::styled("Ticket:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &self.ticket_id,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Step:    ", Style::default().fg(Color::Gray)),
                Span::styled(&self.step, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Session: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.session_name, Style::default().fg(Color::DarkGray)),
            ]),
        ];
        frame.render_widget(Paragraph::new(info_lines), chunks[1]);

        // Options
        let options = self.available_options();
        let mut option_lines = Vec::new();

        for option in &options {
            let is_selected = *option == self.selection;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let suffix = if *option == SessionRecoverySelection::ResumeSession {
                " (session data found)"
            } else {
                ""
            };

            option_lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled(
                    format!("[{}] ", option.key()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(option.label(), style),
                Span::styled(suffix, Style::default().fg(Color::Green)),
            ]));
        }

        let options_widget = Paragraph::new(option_lines);
        frame.render_widget(options_widget, chunks[2]);

        // Instructions
        let instructions = Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(instructions).alignment(Alignment::Center),
            chunks[3],
        );
    }
}

/// Display information for a syncable collection
#[derive(Debug, Clone)]
pub struct SyncableCollectionDisplay {
    pub provider: String,
    pub project_key: String,
    pub collection_name: String,
    pub status_count: usize,
}

impl From<&crate::services::SyncableCollection> for SyncableCollectionDisplay {
    fn from(collection: &crate::services::SyncableCollection) -> Self {
        Self {
            provider: collection.provider.clone(),
            project_key: collection.project_key.clone(),
            collection_name: collection.collection_name.clone(),
            status_count: collection.sync_statuses.len(),
        }
    }
}

/// Result from the sync confirmation dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncConfirmResult {
    /// User confirmed, sync should proceed
    Confirmed,
    /// User cancelled
    Cancelled,
}

/// Dialog for confirming kanban sync of all collections
pub struct SyncConfirmDialog {
    pub visible: bool,
    collections: Vec<SyncableCollectionDisplay>,
    /// Whether sync is in progress
    pub syncing: bool,
    /// Current sync progress (0-indexed)
    current_sync_index: usize,
    /// Total number of collections to sync
    total_collections: usize,
    /// Status message to display
    status_message: Option<String>,
}

impl Default for SyncConfirmDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            collections: Vec::new(),
            syncing: false,
            current_sync_index: 0,
            total_collections: 0,
            status_message: None,
        }
    }

    /// Show the dialog with collections to sync
    pub fn show(&mut self, collections: Vec<crate::services::SyncableCollection>) {
        self.collections = collections
            .iter()
            .map(SyncableCollectionDisplay::from)
            .collect();
        self.total_collections = self.collections.len();
        self.syncing = false;
        self.current_sync_index = 0;
        self.status_message = None;
        self.visible = true;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.collections.clear();
        self.syncing = false;
        self.status_message = None;
    }

    /// Set syncing state with progress
    pub fn set_syncing(&mut self, index: usize, total: usize) {
        self.syncing = true;
        self.current_sync_index = index;
        self.total_collections = total;
        self.status_message = Some(format!("Syncing collection {}/{}...", index + 1, total));
    }

    /// Set completion message
    pub fn set_complete(&mut self, message: &str) {
        self.syncing = false;
        self.status_message = Some(message.to_string());
    }

    /// Handle key input, returns Some(result) if an action was triggered
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Option<SyncConfirmResult> {
        use crossterm::event::KeyCode;

        // Don't handle keys while syncing
        if self.syncing {
            return None;
        }

        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Some(SyncConfirmResult::Confirmed)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.hide();
                Some(SyncConfirmResult::Cancelled)
            }
            _ => None,
        }
    }

    /// Check if dialog has any collections
    pub fn has_collections(&self) -> bool {
        !self.collections.is_empty()
    }

    /// Render the dialog
    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        // Calculate dialog size based on number of collections
        let collection_count = self.collections.len();
        let dialog_height = (40 + collection_count * 3).min(70) as u16;

        let area = centered_rect(55, dialog_height, frame.area());
        frame.render_widget(Clear, area);

        let title = if self.syncing {
            " Kanban Sync [Syncing...] "
        } else {
            " Kanban Sync "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header text
                Constraint::Min(3),    // Collections list
                Constraint::Length(2), // Status message
                Constraint::Length(3), // Footer/buttons
            ])
            .margin(1)
            .split(inner);

        // Header
        let header = Paragraph::new(Line::from(Span::styled(
            "The following collections will be synced:",
            Style::default().fg(Color::White),
        )));
        frame.render_widget(header, chunks[0]);

        // Collections list
        self.render_collections(frame, chunks[1]);

        // Status message
        if let Some(ref message) = self.status_message {
            let status = Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(status, chunks[2]);
        }

        // Footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_collections(&self, frame: &mut Frame, area: Rect) {
        if self.collections.is_empty() {
            let empty_msg = Paragraph::new("No kanban providers configured")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_msg, area);
            return;
        }

        let mut lines = Vec::new();
        for (i, collection) in self.collections.iter().enumerate() {
            // Show syncing indicator for current collection
            let prefix = if self.syncing && i == self.current_sync_index {
                "▶ "
            } else if self.syncing && i < self.current_sync_index {
                "✓ "
            } else {
                "  "
            };

            // Provider badge
            let provider_badge = match collection.provider.as_str() {
                "jira" => Span::styled(" JIRA ", Style::default().bg(Color::Blue).fg(Color::White)),
                "linear" => Span::styled(
                    " LINEAR ",
                    Style::default().bg(Color::Magenta).fg(Color::White),
                ),
                _ => Span::styled(
                    format!(" {} ", collection.provider.to_uppercase()),
                    Style::default().bg(Color::Gray).fg(Color::White),
                ),
            };

            // Status count suffix
            let status_suffix = if collection.status_count > 0 {
                format!(" ({} statuses)", collection.status_count)
            } else {
                " (default)".to_string()
            };

            lines.push(Line::from(vec![
                Span::raw(prefix),
                provider_badge,
                Span::styled(
                    format!(" {} ", collection.project_key),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("→ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &collection.collection_name,
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(status_suffix, Style::default().fg(Color::DarkGray)),
            ]));
        }

        let list = Paragraph::new(lines);
        frame.render_widget(list, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = if self.syncing {
            Line::from(vec![Span::styled(
                " Syncing... Please wait ",
                Style::default().fg(Color::Yellow),
            )])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    " [Y]es, Sync All ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(" [N]o, Cancel ", Style::default().fg(Color::Red)),
            ])
        };

        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
            sessions: HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    // ConfirmSelection tests
    #[test]
    fn test_confirm_selection_next_cycles_correctly() {
        assert_eq!(ConfirmSelection::Yes.next(), ConfirmSelection::View);
        assert_eq!(ConfirmSelection::View.next(), ConfirmSelection::No);
        assert_eq!(ConfirmSelection::No.next(), ConfirmSelection::Yes);
    }

    #[test]
    fn test_confirm_selection_prev_cycles_correctly() {
        assert_eq!(ConfirmSelection::Yes.prev(), ConfirmSelection::No);
        assert_eq!(ConfirmSelection::View.prev(), ConfirmSelection::Yes);
        assert_eq!(ConfirmSelection::No.prev(), ConfirmSelection::View);
    }

    // SelectedOption tests
    #[test]
    fn test_selected_option_next_cycles_correctly() {
        assert_eq!(SelectedOption::Provider.next(), SelectedOption::Project);
        assert_eq!(SelectedOption::Project.next(), SelectedOption::Provider);
    }

    #[test]
    fn test_selected_option_prev_cycles_correctly() {
        assert_eq!(SelectedOption::Provider.prev(), SelectedOption::Project);
        assert_eq!(SelectedOption::Project.prev(), SelectedOption::Provider);
    }

    // ConfirmDialog tests
    #[test]
    fn test_confirm_dialog_new_initializes_correctly() {
        let dialog = ConfirmDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.ticket.is_none());
        assert_eq!(dialog.selection, ConfirmSelection::Yes);
        assert_eq!(dialog.focus, ConfirmDialogFocus::Buttons);
        assert!(dialog.provider_options.is_empty());
        assert!(!dialog.docker_selected);
        assert!(!dialog.yolo_selected);
    }

    #[test]
    fn test_confirm_dialog_configure_sets_options() {
        let mut dialog = ConfirmDialog::new();
        let providers = vec![
            LlmProvider {
                tool: "claude".to_string(),
                model: "opus".to_string(),
                display_name: Some("Claude Opus".to_string()),
                ..Default::default()
            },
            LlmProvider {
                tool: "claude".to_string(),
                model: "sonnet".to_string(),
                display_name: None,
                ..Default::default()
            },
        ];
        let projects = vec!["project-a".to_string(), "project-b".to_string()];

        dialog.configure(providers.clone(), projects.clone(), true, false);

        assert_eq!(dialog.provider_options.len(), 2);
        assert_eq!(dialog.project_options.len(), 2);
        assert!(dialog.docker_enabled);
        assert!(!dialog.yolo_enabled);
    }

    #[test]
    fn test_confirm_dialog_show_sets_ticket_and_resets_state() {
        let mut dialog = ConfirmDialog::new();
        dialog.configure(
            vec![],
            vec!["project-a".to_string(), "project-b".to_string()],
            false,
            false,
        );
        dialog.selection = ConfirmSelection::No;
        dialog.docker_selected = true;

        let ticket = make_test_ticket("project-b");
        dialog.show(ticket);

        assert!(dialog.visible);
        assert!(dialog.ticket.is_some());
        assert_eq!(dialog.selection, ConfirmSelection::Yes);
        assert_eq!(dialog.selected_project, 1); // project-b is at index 1
        assert!(!dialog.docker_selected); // Reset
    }

    #[test]
    fn test_confirm_dialog_cycle_provider() {
        let mut dialog = ConfirmDialog::new();
        dialog.provider_options = vec![
            LlmProvider {
                tool: "a".to_string(),
                model: "1".to_string(),
                display_name: None,
                ..Default::default()
            },
            LlmProvider {
                tool: "b".to_string(),
                model: "2".to_string(),
                display_name: None,
                ..Default::default()
            },
            LlmProvider {
                tool: "c".to_string(),
                model: "3".to_string(),
                display_name: None,
                ..Default::default()
            },
        ];

        assert_eq!(dialog.selected_provider, 0);
        dialog.cycle_provider();
        assert_eq!(dialog.selected_provider, 1);
        dialog.cycle_provider();
        assert_eq!(dialog.selected_provider, 2);
        dialog.cycle_provider();
        assert_eq!(dialog.selected_provider, 0); // Wraps
    }

    #[test]
    fn test_confirm_dialog_toggle_docker_respects_enabled() {
        let mut dialog = ConfirmDialog::new();
        dialog.docker_enabled = false;

        dialog.toggle_docker();
        assert!(!dialog.docker_selected); // No-op when disabled

        dialog.docker_enabled = true;
        dialog.toggle_docker();
        assert!(dialog.docker_selected);
        dialog.toggle_docker();
        assert!(!dialog.docker_selected);
    }

    #[test]
    fn test_confirm_dialog_toggle_yolo_respects_enabled() {
        let mut dialog = ConfirmDialog::new();
        dialog.yolo_enabled = false;

        dialog.toggle_yolo();
        assert!(!dialog.yolo_selected); // No-op when disabled

        dialog.yolo_enabled = true;
        dialog.toggle_yolo();
        assert!(dialog.yolo_selected);
    }

    #[test]
    fn test_confirm_dialog_cycle_project() {
        let mut dialog = ConfirmDialog::new();
        dialog.project_options = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        dialog.cycle_project();
        assert_eq!(dialog.selected_project, 1);
        dialog.cycle_project();
        assert_eq!(dialog.selected_project, 2);
        dialog.cycle_project();
        assert_eq!(dialog.selected_project, 0); // Wraps
    }

    #[test]
    fn test_confirm_dialog_cycle_project_prev() {
        let mut dialog = ConfirmDialog::new();
        dialog.project_options = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        dialog.cycle_project_prev();
        assert_eq!(dialog.selected_project, 2); // Wraps to end
        dialog.cycle_project_prev();
        assert_eq!(dialog.selected_project, 1);
    }

    #[test]
    fn test_confirm_dialog_is_project_overridden() {
        let mut dialog = ConfirmDialog::new();
        dialog.project_options = vec!["a".to_string(), "b".to_string()];
        dialog.original_project = "a".to_string();
        dialog.selected_project = 0;

        assert!(!dialog.is_project_overridden());

        dialog.selected_project = 1;
        assert!(dialog.is_project_overridden());
    }

    #[test]
    fn test_confirm_dialog_has_options() {
        let mut dialog = ConfirmDialog::new();
        assert!(!dialog.has_options());

        dialog.docker_enabled = true;
        assert!(dialog.has_options());

        dialog.docker_enabled = false;
        dialog.yolo_enabled = true;
        assert!(dialog.has_options());

        dialog.yolo_enabled = false;
        dialog.provider_options = vec![
            LlmProvider {
                tool: "a".to_string(),
                model: "1".to_string(),
                display_name: None,
                ..Default::default()
            },
            LlmProvider {
                tool: "b".to_string(),
                model: "2".to_string(),
                display_name: None,
                ..Default::default()
            },
        ];
        assert!(dialog.has_options());
    }

    #[test]
    fn test_confirm_dialog_focus_management() {
        let mut dialog = ConfirmDialog::new();
        dialog.docker_enabled = true; // Enable options

        assert!(!dialog.is_options_focused());
        dialog.focus_options();
        assert!(dialog.is_options_focused());
        dialog.focus_buttons();
        assert!(!dialog.is_options_focused());
    }

    #[test]
    fn test_confirm_dialog_option_navigation() {
        let mut dialog = ConfirmDialog::new();
        assert_eq!(dialog.selected_option, SelectedOption::Provider);

        dialog.next_option();
        assert_eq!(dialog.selected_option, SelectedOption::Project);

        dialog.prev_option();
        assert_eq!(dialog.selected_option, SelectedOption::Provider);
    }

    #[test]
    fn test_confirm_dialog_select_next_prev() {
        let mut dialog = ConfirmDialog::new();
        assert_eq!(dialog.selection, ConfirmSelection::Yes);

        dialog.select_next();
        assert_eq!(dialog.selection, ConfirmSelection::View);

        dialog.select_prev();
        assert_eq!(dialog.selection, ConfirmSelection::Yes);
    }

    #[test]
    fn test_confirm_dialog_hide_clears_state() {
        let mut dialog = ConfirmDialog::new();
        dialog.visible = true;
        dialog.ticket = Some(make_test_ticket("test"));

        dialog.hide();

        assert!(!dialog.visible);
        assert!(dialog.ticket.is_none());
    }

    // RejectionDialog tests
    #[test]
    fn test_rejection_dialog_new_initializes_correctly() {
        let dialog = RejectionDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.reason.is_empty());
        assert_eq!(dialog.cursor_position, 0);
    }

    #[test]
    fn test_rejection_dialog_show_and_hide() {
        let mut dialog = RejectionDialog::new();

        dialog.show("plan", "TICKET-001");
        assert!(dialog.visible);
        assert_eq!(dialog.step_name, "plan");
        assert_eq!(dialog.ticket_id, "TICKET-001");

        dialog.hide();
        assert!(!dialog.visible);
        assert!(dialog.reason.is_empty());
    }

    #[test]
    fn test_rejection_dialog_handle_char() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");

        dialog.handle_char('H');
        dialog.handle_char('i');

        assert_eq!(dialog.reason, "Hi");
        assert_eq!(dialog.cursor_position, 2);
    }

    #[test]
    fn test_rejection_dialog_handle_backspace() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");

        dialog.handle_char('a');
        dialog.handle_char('b');
        dialog.handle_backspace();

        assert_eq!(dialog.reason, "a");
        assert_eq!(dialog.cursor_position, 1);
    }

    #[test]
    fn test_rejection_dialog_handle_backspace_at_start() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");

        dialog.handle_backspace(); // Should be no-op
        assert_eq!(dialog.cursor_position, 0);
        assert!(dialog.reason.is_empty());
    }

    #[test]
    fn test_rejection_dialog_cursor_movement() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");
        dialog.handle_char('a');
        dialog.handle_char('b');
        dialog.handle_char('c');

        dialog.cursor_left();
        assert_eq!(dialog.cursor_position, 2);

        dialog.cursor_right();
        assert_eq!(dialog.cursor_position, 3);

        dialog.cursor_home();
        assert_eq!(dialog.cursor_position, 0);

        dialog.cursor_end();
        assert_eq!(dialog.cursor_position, 3);
    }

    #[test]
    fn test_rejection_dialog_cursor_insert_at_position() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");

        dialog.handle_char('a');
        dialog.handle_char('c');
        dialog.cursor_left();
        dialog.handle_char('b');

        assert_eq!(dialog.reason, "abc");
    }

    #[test]
    fn test_rejection_dialog_handle_delete() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");

        dialog.handle_char('a');
        dialog.handle_char('b');
        dialog.handle_char('c');
        dialog.cursor_home();
        dialog.handle_delete();

        assert_eq!(dialog.reason, "bc");
    }

    #[test]
    fn test_rejection_dialog_confirm_and_cancel() {
        let mut dialog = RejectionDialog::new();
        dialog.show("step", "T1");
        dialog.handle_char('r');
        dialog.handle_char('e');
        dialog.handle_char('a');
        dialog.handle_char('s');
        dialog.handle_char('o');
        dialog.handle_char('n');

        let result = dialog.confirm();
        assert!(result.confirmed);
        assert_eq!(result.reason, "reason");

        let cancel_result = dialog.cancel();
        assert!(!cancel_result.confirmed);
        assert!(cancel_result.reason.is_empty());
    }

    // HelpDialog tests
    #[test]
    fn test_help_dialog_toggle() {
        let mut dialog = HelpDialog::new();
        assert!(!dialog.visible);

        dialog.toggle();
        assert!(dialog.visible);

        dialog.toggle();
        assert!(!dialog.visible);
    }

    // SessionRecoverySelection tests
    #[test]
    fn test_session_recovery_selection_next_with_session_id() {
        let has_session_id = true;

        assert_eq!(
            SessionRecoverySelection::ResumeSession.next(has_session_id),
            SessionRecoverySelection::StartFresh
        );
        assert_eq!(
            SessionRecoverySelection::StartFresh.next(has_session_id),
            SessionRecoverySelection::ReturnToQueue
        );
        assert_eq!(
            SessionRecoverySelection::ReturnToQueue.next(has_session_id),
            SessionRecoverySelection::Cancel
        );
        assert_eq!(
            SessionRecoverySelection::Cancel.next(has_session_id),
            SessionRecoverySelection::ResumeSession
        );
    }

    #[test]
    fn test_session_recovery_selection_next_without_session_id() {
        let has_session_id = false;

        assert_eq!(
            SessionRecoverySelection::Cancel.next(has_session_id),
            SessionRecoverySelection::StartFresh
        ); // Skips ResumeSession
    }

    #[test]
    fn test_session_recovery_selection_prev_with_session_id() {
        let has_session_id = true;

        assert_eq!(
            SessionRecoverySelection::ResumeSession.prev(has_session_id),
            SessionRecoverySelection::Cancel
        );
        assert_eq!(
            SessionRecoverySelection::StartFresh.prev(has_session_id),
            SessionRecoverySelection::ResumeSession
        );
    }

    #[test]
    fn test_session_recovery_selection_prev_without_session_id() {
        let has_session_id = false;

        assert_eq!(
            SessionRecoverySelection::StartFresh.prev(has_session_id),
            SessionRecoverySelection::Cancel
        ); // Skips ResumeSession
    }

    #[test]
    fn test_session_recovery_selection_label_and_key() {
        assert_eq!(
            SessionRecoverySelection::ResumeSession.label(),
            "Resume session"
        );
        assert_eq!(SessionRecoverySelection::StartFresh.label(), "Start fresh");
        assert_eq!(
            SessionRecoverySelection::ReturnToQueue.label(),
            "Return to queue"
        );
        assert_eq!(SessionRecoverySelection::Cancel.label(), "Cancel");

        assert_eq!(SessionRecoverySelection::ResumeSession.key(), "R");
        assert_eq!(SessionRecoverySelection::StartFresh.key(), "S");
        assert_eq!(SessionRecoverySelection::ReturnToQueue.key(), "Q");
        assert_eq!(SessionRecoverySelection::Cancel.key(), "Esc");
    }

    // SessionRecoveryDialog tests
    #[test]
    fn test_session_recovery_dialog_new() {
        let dialog = SessionRecoveryDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.claude_session_id.is_none());
        assert_eq!(dialog.selection, SessionRecoverySelection::StartFresh);
    }

    #[test]
    fn test_session_recovery_dialog_show_with_session_id() {
        let mut dialog = SessionRecoveryDialog::new();

        dialog.show(
            "TICKET-001".to_string(),
            "session-name".to_string(),
            "plan".to_string(),
            Some("uuid-123".to_string()),
        );

        assert!(dialog.visible);
        assert!(dialog.has_session_id());
        assert_eq!(dialog.selection, SessionRecoverySelection::ResumeSession);
    }

    #[test]
    fn test_session_recovery_dialog_show_without_session_id() {
        let mut dialog = SessionRecoveryDialog::new();

        dialog.show(
            "TICKET-001".to_string(),
            "session-name".to_string(),
            "plan".to_string(),
            None,
        );

        assert!(dialog.visible);
        assert!(!dialog.has_session_id());
        assert_eq!(dialog.selection, SessionRecoverySelection::StartFresh);
    }

    #[test]
    fn test_session_recovery_dialog_available_options() {
        let mut dialog = SessionRecoveryDialog::new();

        // Without session ID
        dialog.show("T1".to_string(), "s1".to_string(), "step".to_string(), None);
        let opts = dialog.available_options_for_test();
        assert_eq!(opts.len(), 3);
        assert!(!opts.contains(&SessionRecoverySelection::ResumeSession));

        // With session ID
        dialog.show(
            "T1".to_string(),
            "s1".to_string(),
            "step".to_string(),
            Some("uuid".to_string()),
        );
        let opts = dialog.available_options_for_test();
        assert_eq!(opts.len(), 4);
        assert!(opts.contains(&SessionRecoverySelection::ResumeSession));
    }

    #[test]
    fn test_session_recovery_dialog_navigation() {
        let mut dialog = SessionRecoveryDialog::new();
        dialog.show(
            "T1".to_string(),
            "s1".to_string(),
            "step".to_string(),
            Some("uuid".to_string()),
        );

        assert_eq!(dialog.selection, SessionRecoverySelection::ResumeSession);
        dialog.select_next();
        assert_eq!(dialog.selection, SessionRecoverySelection::StartFresh);
        dialog.select_prev();
        assert_eq!(dialog.selection, SessionRecoverySelection::ResumeSession);
    }

    #[test]
    fn test_session_recovery_dialog_hide() {
        let mut dialog = SessionRecoveryDialog::new();
        dialog.show("T1".to_string(), "s1".to_string(), "step".to_string(), None);

        dialog.hide();
        assert!(!dialog.visible);
    }

    // SyncConfirmDialog tests
    #[test]
    fn test_sync_confirm_dialog_new() {
        let dialog = SyncConfirmDialog::new();
        assert!(!dialog.visible);
        assert!(!dialog.syncing);
        assert!(!dialog.has_collections());
    }

    #[test]
    fn test_sync_confirm_dialog_show_hide() {
        let mut dialog = SyncConfirmDialog::new();

        let collections = vec![crate::services::SyncableCollection {
            provider: "jira".to_string(),
            project_key: "PROJ".to_string(),
            collection_name: "jira-proj".to_string(),
            sync_user_id: "user123".to_string(),
            sync_statuses: vec!["To Do".to_string()],
        }];

        dialog.show(collections);
        assert!(dialog.visible);
        assert!(dialog.has_collections());
        assert!(!dialog.syncing);

        dialog.hide();
        assert!(!dialog.visible);
        assert!(!dialog.has_collections());
    }

    #[test]
    fn test_sync_confirm_dialog_key_handling() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.show(vec![]);

        // Y key should confirm
        let result = dialog.handle_key(crossterm::event::KeyCode::Char('y'));
        assert_eq!(result, Some(SyncConfirmResult::Confirmed));

        // Reset dialog
        dialog.visible = true;

        // N key should cancel
        let result = dialog.handle_key(crossterm::event::KeyCode::Char('n'));
        assert_eq!(result, Some(SyncConfirmResult::Cancelled));
        assert!(!dialog.visible); // Should be hidden
    }

    #[test]
    fn test_sync_confirm_dialog_syncing_blocks_keys() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.show(vec![]);
        dialog.syncing = true;

        // Keys should be blocked while syncing
        let result = dialog.handle_key(crossterm::event::KeyCode::Char('y'));
        assert!(result.is_none());
    }

    #[test]
    fn test_sync_confirm_dialog_set_syncing() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.show(vec![]);

        dialog.set_syncing(1, 3);
        assert!(dialog.syncing);
    }

    #[test]
    fn test_sync_confirm_dialog_set_complete() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.syncing = true;

        dialog.set_complete("Sync complete: 5 tickets created");
        assert!(!dialog.syncing);
    }
}
