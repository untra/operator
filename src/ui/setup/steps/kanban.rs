//! Kanban integration step rendering

use crate::api::providers::kanban::{KanbanProviderType, ProviderStatus};
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
    pub(crate) fn render_kanban_info_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Kanban Integration ")
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
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Supported providers header
                Constraint::Length(6), // Supported providers list (4 providers)
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Detected header
                Constraint::Min(4),    // Detected providers list
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Action rows
                Constraint::Length(2), // Footer/help
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "Kanban",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Integration Setup"),
        ])])
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new("Operator can sync issues from external kanban providers.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[1]);

        // Supported providers header
        let supported_header = Paragraph::new(Line::from(vec![Span::styled(
            "Supported Providers:",
            Style::default().fg(Color::Yellow),
        )]));
        frame.render_widget(supported_header, chunks[3]);

        // Supported providers list
        let supported = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Jira Cloud", Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(
                    "OPERATOR_JIRA_API_KEY",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Linear", Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(
                    "OPERATOR_LINEAR_API_KEY",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("GitHub Projects", Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(
                    "OPERATOR_GITHUB_TOKEN",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("OpenSpec", Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(
                    "local files, experimental",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]),
        ]);
        frame.render_widget(supported, chunks[4]);

        // Detected header
        let detected_header = Paragraph::new(Line::from(vec![Span::styled(
            "Detected Providers:",
            Style::default().fg(Color::Yellow),
        )]));
        frame.render_widget(detected_header, chunks[6]);

        // Detected providers list
        let mut detected_lines = Vec::new();
        if self.detected_kanban_providers.is_empty() {
            detected_lines.push(Line::from(vec![Span::styled(
                "  No providers detected from environment variables",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            for provider in &self.detected_kanban_providers {
                let (icon, icon_color) = match &provider.status {
                    ProviderStatus::Untested => ("?", Color::Yellow),
                    ProviderStatus::Testing => ("~", Color::Yellow),
                    ProviderStatus::Valid => ("✓", Color::Green),
                    ProviderStatus::Failed { .. } => ("✗", Color::Red),
                };

                let provider_name = match provider.provider_type {
                    KanbanProviderType::Jira => "Jira",
                    KanbanProviderType::Linear => "Linear",
                    KanbanProviderType::Github => "GitHub",
                    KanbanProviderType::Openspec => "OpenSpec",
                };

                let status_text = match &provider.status {
                    ProviderStatus::Untested => "not tested".to_string(),
                    ProviderStatus::Testing => "testing...".to_string(),
                    ProviderStatus::Valid => "valid".to_string(),
                    ProviderStatus::Failed { error } => {
                        format!("failed: {}", error.chars().take(30).collect::<String>())
                    }
                };

                detected_lines.push(Line::from(vec![
                    Span::raw("  ["),
                    Span::styled(icon, Style::default().fg(icon_color)),
                    Span::raw("] "),
                    Span::styled(provider_name, Style::default().fg(Color::White)),
                    Span::raw(" - "),
                    Span::styled(&provider.domain, Style::default().fg(Color::Cyan)),
                    Span::raw(" ("),
                    Span::styled(status_text, Style::default().fg(icon_color)),
                    Span::raw(")"),
                ]));
            }
        }
        let detected_list = Paragraph::new(detected_lines);
        frame.render_widget(detected_list, chunks[7]);

        // Actions. Connecting hands off to the shared onboarding dialog, so
        // credentials are collected the same way here and from the dashboard.
        let selected = self.kanban_choice_state.selected().unwrap_or(0);
        let action_rows = Paragraph::new(vec![
            choice_line("Connect a kanban provider", selected == 0),
            choice_line("Skip for now", selected == 1),
        ]);
        frame.render_widget(action_rows, chunks[9]);

        let footer = Line::from(vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Confirm  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back"),
        ]);
        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, chunks[10]);
    }
}

/// A selectable action row on the kanban info step.
fn choice_line(label: &str, selected: bool) -> Line<'_> {
    let (marker, color) = if selected {
        ("> ", Color::Cyan)
    } else {
        ("  ", Color::Gray)
    };
    Line::from(vec![
        Span::raw("  "),
        Span::styled(marker, Style::default().fg(color)),
        Span::styled(label.to_string(), Style::default().fg(color)),
    ])
}
