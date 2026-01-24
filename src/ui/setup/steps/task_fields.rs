//! Task field configuration step rendering

use crate::ui::dialogs::centered_rect;
use crate::ui::setup::types::TASK_OPTIONAL_FIELDS;
use crate::ui::setup::SetupScreen;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

impl SetupScreen {
    pub(crate) fn render_task_field_config_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 60, frame.area());
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
                Span::raw(" Setup - Configure TASK Fields "),
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
                Constraint::Length(3), // Explanation
                Constraint::Length(2), // Instructions
                Constraint::Min(6),    // Field list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Configure TASK Fields",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Explanation
        let explanation = Paragraph::new(vec![
            Line::from("TASK is the foundational issuetype. Configure which optional"),
            Line::from("fields to include. These choices will propagate to other types."),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(explanation, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, chunks[2]);

        // Field list
        let items: Vec<ListItem> = TASK_OPTIONAL_FIELDS
            .iter()
            .map(|(name, description)| {
                let is_selected = self.task_optional_fields.contains(&name.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            checkbox,
                            Style::default().fg(if is_selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            *name,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(if is_selected {
                                    Color::White
                                } else {
                                    Color::Gray
                                }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(*description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[3], &mut self.field_state);

        // Footer
        let selected_count = self.task_optional_fields.len();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(
                    "{}/{} fields enabled",
                    selected_count,
                    TASK_OPTIONAL_FIELDS.len()
                ),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }
}
