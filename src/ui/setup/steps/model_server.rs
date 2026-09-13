//! Model server step: declare which providers this workspace uses.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::api::providers::model_server::ModelServerKind;
use crate::integrations::catalog::CatalogEntry;
use crate::ui::dialogs::centered_rect;

use super::super::SetupScreen;

impl SetupScreen {
    pub(crate) fn render_model_server_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Model Providers ")
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
                Constraint::Min(10),   // Provider rows
                Constraint::Length(2), // Key hint
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Where inference happens",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        frame.render_widget(title, chunks[0]);

        let description = Paragraph::new(vec![
            Line::from("Declaring a provider records it in config.toml. Operator stores"),
            Line::from("only the name of the env var holding its key, never the key."),
        ])
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(description, chunks[1]);

        let selected = self.model_server_state.selected().unwrap_or(0);
        let mut rows = Vec::new();
        let mut last_class = None;
        let providers = SetupScreen::model_providers();
        for (i, (entry, kind)) in providers.iter().enumerate() {
            let class = kind.provider_class();
            if last_class != Some(class) {
                rows.push(Line::from(vec![Span::styled(
                    format!("  {}", class.display_name()),
                    Style::default().fg(Color::Yellow),
                )]));
                last_class = Some(class);
            }
            rows.push(self.provider_row(entry, *kind, i == selected));
        }
        frame.render_widget(Paragraph::new(rows), chunks[2]);

        let hint = match self
            .model_server_state
            .selected()
            .and_then(|i| providers.get(i).map(|(_, k)| k))
        {
            Some(kind) if !kind.connectable_from_defaults() => {
                Paragraph::new(Line::from(vec![Span::styled(
                    "Needs a base URL - add a [[model_servers]] entry to config.toml.",
                    Style::default().fg(Color::DarkGray),
                )]))
            }
            Some(kind) => match kind.default_api_key_env() {
                Some(env) => Paragraph::new(Line::from(vec![
                    Span::raw("Key env var: "),
                    Span::styled(env, Style::default().fg(Color::Cyan)),
                ])),
                None => Paragraph::new(Line::from(vec![Span::styled(
                    "No API key required.",
                    Style::default().fg(Color::DarkGray),
                )])),
            },
            None => Paragraph::new(""),
        };
        frame.render_widget(hint, chunks[3]);

        let footer = Line::from(vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("[Space]", Style::default().fg(Color::Yellow)),
            Span::raw(" Declare  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Continue  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back"),
        ]);
        frame.render_widget(
            Paragraph::new(footer).alignment(Alignment::Center),
            chunks[4],
        );
    }

    fn provider_row(
        &self,
        entry: &CatalogEntry,
        kind: ModelServerKind,
        highlighted: bool,
    ) -> Line<'static> {
        let slug = kind.slug();
        let declared = self.model_servers_declared.iter().any(|s| s == slug);
        let marker = if highlighted { "> " } else { "  " };
        let checkbox = if declared { "[x]" } else { "[ ]" };
        let name_color = if highlighted {
            Color::Cyan
        } else {
            Color::White
        };

        let (status, status_color) = if kind.connectable_from_defaults() {
            match self.model_server_probes.get(slug) {
                Some(s) if s.ends_with("models") => (s.clone(), Color::Green),
                Some(s) => (s.clone(), Color::DarkGray),
                None => ("checking...".to_string(), Color::DarkGray),
            }
        } else {
            ("needs base URL".to_string(), Color::DarkGray)
        };

        Line::from(vec![
            Span::raw("    "),
            Span::styled(marker, Style::default().fg(name_color)),
            Span::styled(checkbox, Style::default().fg(name_color)),
            Span::raw(" "),
            Span::styled(entry.label, Style::default().fg(name_color)),
            Span::raw("  "),
            Span::styled(status, Style::default().fg(status_color)),
        ])
    }
}
