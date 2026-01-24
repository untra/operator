//! Confirm step rendering

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
    pub(crate) fn render_confirm_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Confirm Setup ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Description
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Path info
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Selected collection
                Constraint::Min(4),    // What will be created
                Constraint::Length(3), // Buttons
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Ready to Initialize",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![Line::from(
            "Would you like to initialize the ticket queue with these settings?",
        )])
        .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[4]);

        // Selected collection
        let effective_collection = self.collection();
        let collection_text = vec![
            Line::from(Span::styled(
                format!(
                    "Selected issue types ({}):",
                    self.selected_preset.display_name()
                ),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    effective_collection.join(", "),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];
        frame.render_widget(Paragraph::new(collection_text), chunks[6]);

        // What will be created
        let will_create = vec![
            Line::from(Span::styled(
                "This will create:",
                Style::default().fg(Color::Gray),
            )),
            Line::from("  .tickets/queue/  .tickets/in-progress/  .tickets/completed/"),
            Line::from(Span::styled(
                "  .tickets/templates/ (with selected issue type templates)",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(will_create), chunks[7]);

        // Buttons
        let init_style = if self.confirm_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let cancel_style = if !self.confirm_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let buttons = Line::from(vec![
            Span::raw("     "),
            Span::styled(" [I]nitialize ", init_style),
            Span::raw("     "),
            Span::styled(" [C]ancel ", cancel_style),
        ]);

        let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[8]);
    }
}
