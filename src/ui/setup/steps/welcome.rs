//! Welcome step rendering

use crate::projects::TOOL_MARKERS;
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
    pub(crate) fn render_welcome_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
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
                Span::raw(" Workspace Setup "),
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
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Description
                Constraint::Length(1), // Spacer
                Constraint::Length(6), // Detected LLM Tools
                Constraint::Length(1), // Spacer
                Constraint::Min(6),    // Discovered projects by tool
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Path info
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Operator!",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![Line::from("A TUI for orchestrating LLM Code agents.")])
            .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Detected LLM Tools
        let mut tools_text = vec![Line::from(Span::styled(
            "Detected LLM Tools:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        // Show each known tool with detection status
        for (tool_name, _marker) in TOOL_MARKERS {
            let detected = self.detected_tools.iter().find(|t| t.name == *tool_name);

            let line = if let Some(tool) = detected {
                Line::from(vec![
                    Span::styled("  + ", Style::default().fg(Color::Green)),
                    Span::styled(
                        tool_name.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" (v{}) - {} models", tool.version, tool.model_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  - ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tool_name.to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(" - not installed", Style::default().fg(Color::DarkGray)),
                ])
            };
            tools_text.push(line);
        }
        frame.render_widget(Paragraph::new(tools_text), chunks[4]);

        // Discovered projects by tool
        let mut projects_text = vec![Line::from(Span::styled(
            "Discovered Projects:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        let mut has_any_projects = false;
        for (tool_name, _marker) in TOOL_MARKERS {
            if let Some(projects) = self.projects_by_tool.get(*tool_name) {
                if !projects.is_empty() {
                    has_any_projects = true;
                    let project_list = projects.join(", ");
                    projects_text.push(Line::from(vec![
                        Span::styled(
                            format!("  {}: ", tool_name),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(project_list, Style::default().fg(Color::Green)),
                    ]));
                }
            }
        }

        if !has_any_projects {
            projects_text.push(Line::from(Span::styled(
                "  (no projects with marker files found)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        frame.render_widget(Paragraph::new(projects_text), chunks[6]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[8]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[9]);
    }
}
