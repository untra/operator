use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::ui::dialogs::centered_rect;
use crate::ui::setup::{SetupScreen, LOCAL_EXECUTION_OPTION_INDEX, REMOTE_EXECUTION_OPTION_INDEX};

impl SetupScreen {
    pub(crate) fn render_execution_mode_step(&self, frame: &mut Frame) {
        let area = centered_rect(72, 70, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Execution Mode ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(5),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new("Where will agents run?").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[0],
        );

        frame.render_widget(
            Paragraph::new("Both modes support multiple agents.")
                .style(Style::default().fg(Color::Gray)),
            chunks[1],
        );

        let selected = self
            .execution_mode_state
            .selected()
            .unwrap_or(LOCAL_EXECUTION_OPTION_INDEX);
        frame.render_widget(
            Paragraph::new(vec![
                mode_line(
                    "This machine",
                    selected == LOCAL_EXECUTION_OPTION_INDEX,
                    "agents and local containers run beside Operator",
                    false,
                ),
                mode_line(
                    "Remote targets",
                    selected == REMOTE_EXECUTION_OPTION_INDEX,
                    "SSH hosts and Coder workspaces report back here",
                    true,
                ),
            ]),
            chunks[2],
        );

        frame.render_widget(Paragraph::new(self.entitlement_lines()), chunks[3]);

        frame.render_widget(
            Paragraph::new("↑/↓ Select   Enter Continue   Esc Back")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            chunks[4],
        );
    }

    /// The paywall panel: shown inline whenever remote execution is unavailable.
    fn entitlement_lines(&self) -> Vec<Line<'static>> {
        if self.premium_entitled() {
            return vec![Line::from(Span::styled(
                "Premium licence active - remote targets available.",
                Style::default().fg(Color::Green),
            ))];
        }
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "Remote targets",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("Premium", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::styled(
                "Available with an Operator Premium licence for this configuration.",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "Press Esc to go back and add a licence.",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        if let Some(url) = self.license.as_ref().and_then(|l| l.purchase_url.as_ref()) {
            lines.push(Line::from(Span::styled(
                url.clone(),
                Style::default().fg(Color::DarkGray),
            )));
        }
        if let Some(error) = &self.license_error {
            lines.push(Line::from(Span::styled(
                error.clone(),
                Style::default().fg(Color::Red),
            )));
        }
        lines
    }
}

fn mode_line(name: &str, selected: bool, description: &str, premium: bool) -> Line<'static> {
    let marker = if selected { "(o)" } else { "( )" };
    let color = if selected { Color::Cyan } else { Color::Gray };
    let mut spans = vec![Span::styled(
        format!("{marker} {name}"),
        Style::default().fg(color),
    )];
    if premium {
        spans.push(Span::styled(
            " · Premium",
            Style::default().fg(Color::Yellow),
        ));
    }
    spans.push(Span::raw(format!("  {description}")));
    Line::from(spans)
}
