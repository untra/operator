//! Optional admin-password step.
//!
//! Local use needs no password — the TUI, the CLI, and `opr8r` authenticate
//! with the owner-only local token file. A browser cannot read that file, so
//! this step exists solely to unlock the web dashboard, which is why it is
//! skippable and why the copy says so.

use crate::ui::dialogs::centered_rect;
use crate::ui::setup::{PasswordField, SetupScreen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

impl SetupScreen {
    pub(crate) fn render_admin_password_step(&self, frame: &mut Frame) {
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
                Span::raw(" Setup - Web UI Password "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let has_error = self.password_error.is_some();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Heading
                Constraint::Length(4), // Explanation
                Constraint::Length(1), // Password label
                Constraint::Length(3), // Password field
                Constraint::Length(1), // Confirm label
                Constraint::Length(3), // Confirm field
                Constraint::Length(2), // Error
                Constraint::Min(0),    // Spacer
                Constraint::Length(2), // Instructions
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Set a password for the web dashboard",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))),
            chunks[0],
        );

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "This terminal and the CLI already authenticate automatically.",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    "A browser cannot, so set a password to use the web dashboard.",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    "Leave both fields blank to skip - you can set one later with",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "`operator auth bootstrap` or the /setup page.",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .wrap(Wrap { trim: false }),
            chunks[1],
        );

        let focused_password = self.password_field_focused == PasswordField::Password;

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Password",
                Style::default().fg(if focused_password {
                    Color::Cyan
                } else {
                    Color::Gray
                }),
            ))),
            chunks[2],
        );
        self.password.render(
            frame,
            chunks[3],
            "at least 12 characters",
            focused_password,
            has_error,
        );

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Confirm password",
                Style::default().fg(if focused_password {
                    Color::Gray
                } else {
                    Color::Cyan
                }),
            ))),
            chunks[4],
        );
        self.password_confirm.render(
            frame,
            chunks[5],
            "repeat the password",
            !focused_password,
            has_error,
        );

        if let Some(error) = &self.password_error {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    error.as_str(),
                    Style::default().fg(Color::Red),
                ))),
                chunks[6],
            );
        }

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" switch field  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" continue (blank = skip)  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]))
            .alignment(Alignment::Center),
            chunks[8],
        );
    }
}
