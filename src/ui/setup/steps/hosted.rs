//! Hosted collection picker step rendering

use crate::collections::fetch::CollectionOrigin;
use crate::ui::dialogs::centered_rect;
use crate::ui::setup::SetupScreen;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

impl SetupScreen {
    pub(crate) fn render_hosted_collection_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Hosted Collections ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(2), // Instructions
                Constraint::Min(6),    // List
                Constraint::Length(4), // Details of highlighted
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Choose a Collection",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Loading / empty states.
        if !self.hosted_loaded {
            let msg = Paragraph::new("Fetching collections…")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(msg, chunks[2]);
            return;
        }
        if self.hosted_resolved.is_empty() {
            let msg = Paragraph::new("No collections available.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, chunks[2]);
            return;
        }

        let instructions =
            Paragraph::new(vec![Line::from("Use arrows to navigate, Enter to select")])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Collection list: name + version + verification status.
        let items: Vec<ListItem> = self
            .hosted_resolved
            .iter()
            .map(|r| {
                let (badge, badge_style) = match r.origin {
                    CollectionOrigin::Hosted => ("✓ verified", Style::default().fg(Color::Green)),
                    CollectionOrigin::Embedded => {
                        ("ⓘ built-in", Style::default().fg(Color::Yellow))
                    }
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            r.manifest.name.clone(),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("  "),
                        Span::styled(
                            format!("v{}", r.manifest.version),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::raw("  "),
                        Span::styled(badge, badge_style),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(
                            r.manifest.description.clone(),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");
        frame.render_stateful_widget(list, chunks[2], &mut self.hosted_state);

        // Details for the highlighted collection: issue types + workflow hints.
        if let Some(r) = self.highlighted_hosted() {
            let mut lines = vec![Line::from(vec![
                Span::styled("Types: ", Style::default().fg(Color::Cyan)),
                Span::raw(r.manifest.type_keys().join(", ")),
            ])];
            if let Some(hints) = &r.manifest.workflow_hints {
                let mut hint_spans = vec![Span::styled("Loop: ", Style::default().fg(Color::Cyan))];
                hint_spans.push(Span::raw(
                    hints.loop_kind.clone().unwrap_or_else(|| "—".to_string()),
                ));
                if !hints.review_gates.is_empty() {
                    hint_spans.push(Span::styled("  Gates: ", Style::default().fg(Color::Cyan)));
                    hint_spans.push(Span::raw(hints.review_gates.join(", ")));
                }
                lines.push(Line::from(hint_spans));
            }
            if let Some(note) = &r.note {
                lines.push(Line::from(vec![Span::styled(
                    format!("⚠ {note}"),
                    Style::default().fg(Color::Yellow),
                )]));
            }
            let details = Paragraph::new(lines).style(Style::default().fg(Color::Gray));
            frame.render_widget(details, chunks[3]);
        }

        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }
}
