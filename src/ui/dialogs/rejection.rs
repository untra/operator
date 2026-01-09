#![allow(dead_code)]

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::centered_rect;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
