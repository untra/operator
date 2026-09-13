//! Git provider step: connect a provider so agents can branch, push and PR.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::api::cli_detection::onboarding_spec_for_slug;
use crate::integrations::catalog::CatalogEntry;
use crate::ui::dialogs::centered_rect;

use super::super::SetupScreen;

impl SetupScreen {
    pub(crate) fn render_git_provider_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Git Provider ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(3), // Description
                Constraint::Min(6),    // Provider rows
                Constraint::Length(3), // Token note
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Branches, pushes and pull requests",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        frame.render_widget(title, chunks[0]);

        let description = Paragraph::new(vec![
            Line::from("Operator adopts an existing CLI login where it finds one,"),
            Line::from("and otherwise asks for a personal access token."),
        ])
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(description, chunks[1]);

        let providers = SetupScreen::git_providers();
        let selected = self.git_provider_state.selected().unwrap_or(0);
        let mut rows: Vec<Line> = providers
            .iter()
            .enumerate()
            .map(|(i, entry)| self.git_provider_row(entry, i == selected))
            .collect();
        rows.push(Line::from(""));
        rows.push(row_line(
            "Continue without a git provider",
            selected == providers.len(),
            None,
        ));
        frame.render_widget(Paragraph::new(rows), chunks[2]);

        let note = match &self.git_export_hint {
            Some(export) => Paragraph::new(vec![
                Line::from("Token exported for this session only. To keep it:"),
                Line::from(Span::styled(
                    export.clone(),
                    Style::default().fg(Color::Cyan),
                )),
            ]),
            None => Paragraph::new(vec![
                Line::from("config.toml records only the env var name holding the token."),
                Line::from("Export it in your shell to keep it past this session."),
            ])
            .style(Style::default().fg(Color::DarkGray)),
        };
        frame.render_widget(note, chunks[3]);

        let footer = Line::from(vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Connect  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[4],
        );
    }

    fn git_provider_row(&self, entry: &CatalogEntry, highlighted: bool) -> Line<'static> {
        let status = self
            .git_provider_status
            .get(entry.slug)
            .cloned()
            .unwrap_or_else(|| match onboarding_spec_for_slug(entry.slug) {
                Some(spec) => format!("via {}, or a token", spec.command),
                None => "personal access token".to_string(),
            });
        row_line(entry.label, highlighted, Some(status))
    }
}

fn row_line(label: &str, highlighted: bool, status: Option<String>) -> Line<'static> {
    let (marker, color) = if highlighted {
        ("> ", Color::Cyan)
    } else {
        ("  ", Color::Gray)
    };
    let mut spans = vec![
        Span::raw("    "),
        Span::styled(marker, Style::default().fg(color)),
        Span::styled(label.to_string(), Style::default().fg(color)),
    ];
    if let Some(status) = status {
        let status_color = if status.starts_with("connected") {
            Color::Green
        } else {
            Color::DarkGray
        };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(status, Style::default().fg(status_color)));
    }
    Line::from(spans)
}
