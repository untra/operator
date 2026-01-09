use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::centered_rect;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
