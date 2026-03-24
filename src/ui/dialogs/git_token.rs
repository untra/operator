//! Git token input dialog with masked display.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::centered_rect;

/// Dialog for collecting a git personal access token with masked input.
pub struct GitTokenDialog {
    pub visible: bool,
    /// Lowercase provider key ("github" or "gitlab").
    pub provider: String,
    /// Display name ("GitHub" or "GitLab").
    pub provider_display: String,
    /// PAT creation URL (opened in browser before dialog is shown).
    pub pat_url: String,
    /// Placeholder text for the input field.
    pub placeholder: String,
    /// Inline error message (shown below input on validation failure).
    pub error: Option<String>,
    token: String,
    cursor_position: usize,
}

impl GitTokenDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            provider: String::new(),
            provider_display: String::new(),
            pat_url: String::new(),
            placeholder: String::new(),
            error: None,
            token: String::new(),
            cursor_position: 0,
        }
    }

    /// Show the dialog for a specific provider.
    pub fn show(
        &mut self,
        provider: &str,
        provider_display: &str,
        pat_url: &str,
        placeholder: &str,
    ) {
        self.provider = provider.to_string();
        self.provider_display = provider_display.to_string();
        self.pat_url = pat_url.to_string();
        self.placeholder = placeholder.to_string();
        self.token.clear();
        self.cursor_position = 0;
        self.error = None;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.token.clear();
        self.cursor_position = 0;
        self.error = None;
    }

    /// Get the current token value.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Set an inline error message.
    pub fn set_error(&mut self, msg: &str) {
        self.error = Some(msg.to_string());
    }

    pub fn handle_char(&mut self, c: char) {
        self.token.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.error = None; // clear error on new input
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.token.remove(self.cursor_position);
            self.error = None;
        }
    }

    pub fn handle_delete(&mut self) {
        if self.cursor_position < self.token.len() {
            self.token.remove(self.cursor_position);
            self.error = None;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.token.len() {
            self.cursor_position += 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor_position = self.token.len();
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = centered_rect(60, 40, frame.area());
        frame.render_widget(Clear, area);

        let title = format!(" {} Authentication ", self.provider_display);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let has_error = self.error.is_some();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_error {
                vec![
                    Constraint::Length(2), // Prompt
                    Constraint::Length(3), // Input
                    Constraint::Length(2), // Error
                    Constraint::Min(0),    // Spacer
                    Constraint::Length(2), // Instructions
                ]
            } else {
                vec![
                    Constraint::Length(2), // Prompt
                    Constraint::Length(3), // Input
                    Constraint::Min(0),    // Spacer
                    Constraint::Length(2), // Instructions
                    Constraint::Length(0), // Unused
                ]
            })
            .margin(1)
            .split(inner);

        // Prompt
        let prompt = Line::from(vec![Span::styled(
            format!(
                "Enter your {} Personal Access Token:",
                self.provider_display
            ),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]);
        frame.render_widget(Paragraph::new(prompt), chunks[0]);

        // Masked input
        let display_text = if self.token.is_empty() {
            Span::styled(&self.placeholder, Style::default().fg(Color::DarkGray))
        } else {
            let masked: String = "•".repeat(self.token.len());
            Span::styled(masked, Style::default().fg(Color::White))
        };

        let input = Paragraph::new(display_text)
            .block(Block::default().borders(Borders::ALL).border_style(
                Style::default().fg(if has_error { Color::Red } else { Color::Cyan }),
            ))
            .wrap(Wrap { trim: false });
        frame.render_widget(input, chunks[1]);

        // Cursor
        let input_inner = Block::default().borders(Borders::ALL).inner(chunks[1]);
        frame.set_cursor_position((input_inner.x + self.cursor_position as u16, input_inner.y));

        // Error message (if present)
        if has_error {
            let error_text = Line::from(vec![Span::styled(
                self.error.as_deref().unwrap_or(""),
                Style::default().fg(Color::Red),
            )]);
            frame.render_widget(Paragraph::new(error_text), chunks[2]);
        }

        // Instructions (last non-empty chunk)
        let instructions_idx = if has_error { 4 } else { 3 };
        let instructions = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" to submit  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" to cancel"),
        ]);
        frame.render_widget(
            Paragraph::new(instructions).alignment(Alignment::Center),
            chunks[instructions_idx],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_token_dialog_new_is_hidden() {
        let dialog = GitTokenDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.token().is_empty());
        assert_eq!(dialog.cursor_position, 0);
        assert!(dialog.error.is_none());
    }

    #[test]
    fn test_git_token_dialog_show_and_hide() {
        let mut dialog = GitTokenDialog::new();

        dialog.show("github", "GitHub", "https://example.com/pat", "ghp_...");
        assert!(dialog.visible);
        assert_eq!(dialog.provider, "github");
        assert_eq!(dialog.provider_display, "GitHub");
        assert_eq!(dialog.pat_url, "https://example.com/pat");
        assert_eq!(dialog.placeholder, "ghp_...");

        dialog.handle_char('t');
        dialog.handle_char('o');
        dialog.handle_char('k');
        assert_eq!(dialog.token(), "tok");

        dialog.hide();
        assert!(!dialog.visible);
        assert!(dialog.token().is_empty());
        assert!(dialog.error.is_none());
    }

    #[test]
    fn test_git_token_dialog_char_input() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");

        dialog.handle_char('g');
        dialog.handle_char('h');
        dialog.handle_char('p');

        assert_eq!(dialog.token(), "ghp");
        assert_eq!(dialog.cursor_position, 3);
    }

    #[test]
    fn test_git_token_dialog_backspace() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");

        dialog.handle_char('a');
        dialog.handle_char('b');
        dialog.handle_backspace();

        assert_eq!(dialog.token(), "a");
        assert_eq!(dialog.cursor_position, 1);
    }

    #[test]
    fn test_git_token_dialog_backspace_at_start() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");

        dialog.handle_backspace();
        assert!(dialog.token().is_empty());
        assert_eq!(dialog.cursor_position, 0);
    }

    #[test]
    fn test_git_token_dialog_cursor_movement() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");
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
    fn test_git_token_dialog_delete() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");
        dialog.handle_char('a');
        dialog.handle_char('b');
        dialog.handle_char('c');
        dialog.cursor_home();
        dialog.handle_delete();

        assert_eq!(dialog.token(), "bc");
    }

    #[test]
    fn test_git_token_dialog_error_clears_on_input() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");

        dialog.set_error("Validation failed");
        assert!(dialog.error.is_some());

        dialog.handle_char('x');
        assert!(dialog.error.is_none());
    }

    #[test]
    fn test_git_token_dialog_token_getter() {
        let mut dialog = GitTokenDialog::new();
        dialog.show("github", "GitHub", "", "");
        dialog.handle_char('t');
        dialog.handle_char('o');
        dialog.handle_char('k');
        assert_eq!(dialog.token(), "tok");

        dialog.hide();
        assert!(dialog.token().is_empty());
    }
}
