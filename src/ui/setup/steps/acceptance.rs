//! Acceptance criteria step rendering

use crate::ui::dialogs::centered_rect;
use crate::ui::setup::SetupScreen;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

impl SetupScreen {
    pub(crate) fn render_acceptance_criteria_step(&self, frame: &mut Frame) {
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
                Span::raw(" Setup - Acceptance Criteria "),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Description
                Constraint::Min(8),    // Acceptance criteria content
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Review Acceptance Criteria",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![
            Line::from("These criteria will be used to validate completed work."),
            Line::from("Other template files (Definition of Done, Definition of Ready) will be written from defaults."),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[1]);

        // Acceptance criteria content (read-only preview)
        let content_block = Block::default()
            .title(" Acceptance Criteria Template ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let content = Paragraph::new(self.acceptance_criteria_text.as_str())
            .block(content_block)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(content, chunks[2]);

        // Footer with key hints
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" accept  |  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }
}
