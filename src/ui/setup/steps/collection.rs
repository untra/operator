//! Collection source and custom collection step rendering

use crate::ui::dialogs::centered_rect;
use crate::ui::setup::types::{CollectionSourceOption, ALL_ISSUE_TYPES};
use crate::ui::setup::SetupScreen;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

impl SetupScreen {
    pub(crate) fn render_collection_source_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Select Template Collection ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Options list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Choose Template Source",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions =
            Paragraph::new(vec![Line::from("Use arrows to navigate, Enter to select")])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Options list
        let items: Vec<ListItem> = CollectionSourceOption::all()
            .iter()
            .map(|opt| {
                let style = if opt.is_unimplemented() {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        opt.label(),
                        style.add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(opt.description(), Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[2], &mut self.source_state);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    pub(crate) fn render_custom_collection_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
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
                Span::raw(" Setup - Issue Types "),
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
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Collection list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Select Issue Types",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Collection list
        let items: Vec<ListItem> = ALL_ISSUE_TYPES
            .iter()
            .map(|t| {
                let is_selected = self.custom_collection.contains(&t.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                let description = match *t {
                    "TASK" => "Focused task that executes one specific thing",
                    "FEAT" => "New feature or enhancement",
                    "FIX" => "Bug fix, follow-up work, tech debt",
                    "SPIKE" => "Research or exploration (paired mode)",
                    "INV" => "Incident investigation (paired mode)",
                    _ => "",
                };
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
                            *t,
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
                        Span::styled(description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[2], &mut self.collection_state);

        // Footer
        let selected_count = self.custom_collection.len();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} selected", selected_count),
                Style::default().fg(if selected_count > 0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }
}
