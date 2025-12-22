use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::queue::Ticket;

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
}

impl ConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            ticket: None,
            selection: ConfirmSelection::Yes,
        }
    }

    pub fn show(&mut self, ticket: Ticket) {
        self.ticket = Some(ticket);
        self.visible = true;
        self.selection = ConfirmSelection::Yes; // Default to Yes
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

        // Center the dialog
        let area = centered_rect(60, 45, frame.area());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Launch Agent? ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Dialog content - reordered: Type/ID, Summary, Project, Priority, Buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Type/ID
                Constraint::Min(3),    // Summary
                Constraint::Length(2), // Project
                Constraint::Length(2), // Priority
                Constraint::Length(3), // Buttons
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
        frame.render_widget(buttons_para, chunks[4]);
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

        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let help_text = vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("q      ", Style::default().fg(Color::Yellow)),
                Span::raw("Quit Operator"),
            ]),
            Line::from(vec![
                Span::styled("Tab    ", Style::default().fg(Color::Yellow)),
                Span::raw("Switch between panels"),
            ]),
            Line::from(vec![
                Span::styled("j/k    ", Style::default().fg(Color::Yellow)),
                Span::raw("Navigate within panel"),
            ]),
            Line::from(vec![
                Span::styled("Enter  ", Style::default().fg(Color::Yellow)),
                Span::raw("Select / confirm"),
            ]),
            Line::from(vec![
                Span::styled("Esc    ", Style::default().fg(Color::Yellow)),
                Span::raw("Cancel / close dialog"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("L      ", Style::default().fg(Color::Yellow)),
                Span::raw("Launch selected ticket"),
            ]),
            Line::from(vec![
                Span::styled("C      ", Style::default().fg(Color::Yellow)),
                Span::raw("Create new ticket"),
            ]),
            Line::from(vec![
                Span::styled("J      ", Style::default().fg(Color::Yellow)),
                Span::raw("Open Projects menu"),
            ]),
            Line::from(vec![
                Span::styled("P      ", Style::default().fg(Color::Yellow)),
                Span::raw("Pause queue processing"),
            ]),
            Line::from(vec![
                Span::styled("R      ", Style::default().fg(Color::Yellow)),
                Span::raw("Resume queue processing"),
            ]),
            Line::from(vec![
                Span::styled("?      ", Style::default().fg(Color::Yellow)),
                Span::raw("Toggle this help"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "In Launch Dialog:",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )),
            Line::from(vec![
                Span::styled("Y      ", Style::default().fg(Color::Yellow)),
                Span::raw("Launch agent"),
            ]),
            Line::from(vec![
                Span::styled("V      ", Style::default().fg(Color::Yellow)),
                Span::raw("View ticket ($VISUAL or open)"),
            ]),
            Line::from(vec![
                Span::styled("E      ", Style::default().fg(Color::Yellow)),
                Span::raw("Edit ticket ($EDITOR)"),
            ]),
            Line::from(vec![
                Span::styled("N      ", Style::default().fg(Color::Yellow)),
                Span::raw("Cancel"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to close",
                Style::default().fg(Color::Gray),
            )),
        ];

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
