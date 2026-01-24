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
                Constraint::Length(4), // Supported providers list
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Detected header
                Constraint::Min(6),    // Detected providers list
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
            for (i, provider) in self.detected_kanban_providers.iter().enumerate() {
                let is_valid = self.valid_kanban_providers.contains(&i);
                let (icon, icon_color) = match &provider.status {
                    ProviderStatus::Untested => ("?", Color::Yellow),
                    ProviderStatus::Testing => ("~", Color::Yellow),
                    ProviderStatus::Valid => ("✓", Color::Green),
                    ProviderStatus::Failed { .. } => ("✗", Color::Red),
                };

                let provider_name = match provider.provider_type {
                    KanbanProviderType::Jira => "Jira",
                    KanbanProviderType::Linear => "Linear",
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
                    Span::styled(
                        provider_name,
                        Style::default().fg(if is_valid {
                            Color::White
                        } else {
                            Color::DarkGray
                        }),
                    ),
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

        // Footer
        let footer = if self.valid_kanban_providers.is_empty() {
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(" Continue  "),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Back"),
            ])
        } else {
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(" Configure providers  "),
                Span::styled("[S]", Style::default().fg(Color::Yellow)),
                Span::raw(" Skip  "),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Back"),
            ])
        };
        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, chunks[8]);
    }

    pub(crate) fn render_kanban_provider_setup_step(
        &mut self,
        frame: &mut Frame,
        provider_index: usize,
    ) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        // Get the provider being configured
        let provider_idx = self
            .valid_kanban_providers
            .get(provider_index)
            .copied()
            .unwrap_or(0);
        let provider = self.detected_kanban_providers.get(provider_idx);

        let title = if let Some(p) = provider {
            let provider_name = match p.provider_type {
                KanbanProviderType::Jira => "Jira",
                KanbanProviderType::Linear => "Linear",
            };
            format!(" Setup: {} - {} ", provider_name, p.domain)
        } else {
            " Kanban Provider Setup ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Length(1), // Spacer
                Constraint::Min(10),   // Project list
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Preview info
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let instructions =
            Paragraph::new("Select a project to sync:").style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[0]);

        // Project list
        if self.kanban_projects.is_empty() {
            let loading = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Loading projects...",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "(Projects will be fetched when you enter this step)",
                    Style::default().fg(Color::DarkGray),
                )]),
            ])
            .alignment(Alignment::Center);
            frame.render_widget(loading, chunks[2]);
        } else {
            crate::ui::paginated_list::render_paginated_list(
                frame,
                chunks[2],
                &mut self.kanban_projects,
                "Projects",
                |project, _selected| {
                    ratatui::widgets::ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:8}", project.key),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" - "),
                        Span::styled(project.name.clone(), Style::default().fg(Color::White)),
                    ]))
                },
            );
        }

        // Preview info
        let preview = if !self.kanban_issue_types.is_empty() {
            Line::from(vec![
                Span::styled("Issue Types: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.kanban_issue_types.join(", "),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  |  "),
                Span::styled("Members: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.kanban_member_count.to_string(),
                    Style::default().fg(Color::White),
                ),
            ])
        } else {
            Line::from(vec![Span::styled(
                "Select a project to see details",
                Style::default().fg(Color::DarkGray),
            )])
        };
        let preview_para = Paragraph::new(preview);
        frame.render_widget(preview_para, chunks[4]);

        // Footer
        let footer = Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("[n/p]", Style::default().fg(Color::Yellow)),
            Span::raw(" Page  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Skip provider"),
        ]);
        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, chunks[5]);
    }
}
