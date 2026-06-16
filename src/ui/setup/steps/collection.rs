//! Collection source step rendering

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
                Constraint::Min(6),    // Options list
                Constraint::Length(2), // Notice (deferred import message)
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

        // Options list (curated collections, the hosted browser, then one import
        // option per configured kanban provider).
        let items: Vec<ListItem> = self
            .source_options
            .iter()
            .map(|opt| {
                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        opt.label(),
                        Style::default().add_modifier(Modifier::BOLD),
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

        // Deferred-import notice (shown when an import option is selected).
        if let Some(notice) = &self.import_notice {
            let notice_para = Paragraph::new(Line::from(vec![Span::styled(
                notice.clone(),
                Style::default().fg(Color::Yellow),
            )]))
            .alignment(Alignment::Center);
            frame.render_widget(notice_para, chunks[3]);
        }

        // Footer
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
