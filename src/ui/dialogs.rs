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

pub struct ConfirmDialog {
    pub visible: bool,
    pub ticket: Option<Ticket>,
    pub selection: ConfirmSelection,

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
}

impl ConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            ticket: None,
            selection: ConfirmSelection::Yes,
            provider_options: Vec::new(),
            selected_provider: 0,
            docker_enabled: false,
            docker_selected: false,
            yolo_enabled: false,
            yolo_selected: false,
        }
    }

    /// Configure the dialog with available options from config
    pub fn configure(
        &mut self,
        providers: Vec<LlmProvider>,
        docker_enabled: bool,
        yolo_enabled: bool,
    ) {
        self.provider_options = providers;
        self.docker_enabled = docker_enabled;
        self.yolo_enabled = yolo_enabled;
    }

    pub fn show(&mut self, ticket: Ticket) {
        self.ticket = Some(ticket);
        self.visible = true;
        self.selection = ConfirmSelection::Yes; // Default to Yes
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
        self.provider_options.len() > 1 || self.docker_enabled || self.yolo_enabled
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

        // Buttons - Yes, View, No
        let yes_style = if self.selection == ConfirmSelection::Yes {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let view_style = if self.selection == ConfirmSelection::View {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Blue)
        };

        let no_style = if self.selection == ConfirmSelection::No {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let buttons = Line::from(vec![
            Span::raw("   "),
            Span::styled(" [Y]es ", yes_style),
            Span::raw("   "),
            Span::styled(" [V]iew ", view_style),
            Span::raw("   "),
            Span::styled(" [N]o ", no_style),
        ]);

        let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[5]);
    }

    /// Render the options section (provider, docker, yolo toggles)
    fn render_options(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();

        // Separator line
        lines.push(Line::from(vec![Span::styled(
            "── Options ──",
            Style::default().fg(Color::DarkGray),
        )]));

        // Provider option (if multiple providers)
        if self.provider_options.len() > 1 {
            let provider_display = self
                .selected_provider()
                .map(|p| {
                    p.display_name
                        .clone()
                        .unwrap_or_else(|| format!("{} {}", p.tool, p.model))
                })
                .unwrap_or_else(|| "Default".to_string());

            lines.push(Line::from(vec![
                Span::styled("[M] ", Style::default().fg(Color::Yellow)),
                Span::styled("Provider: ", Style::default().fg(Color::Gray)),
                Span::styled(provider_display, Style::default().fg(Color::White)),
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
