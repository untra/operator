use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::licensing::{LicenseResponse, LicenseStatus};
use crate::ui::dialogs::centered_rect;
use crate::ui::setup::SetupScreen;

impl SetupScreen {
    pub(crate) fn render_license_step(&self, frame: &mut Frame) {
        let area = centered_rect(72, 76, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Operator Premium ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new("Premium adds remote execution").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[0],
        );

        frame.render_widget(
            Paragraph::new(
                "Multiple local agents and local containers are free. \
                 SSH hosts and Coder workspaces require a licence.",
            )
            .style(Style::default().fg(Color::Gray)),
            chunks[1],
        );

        frame.render_widget(
            Paragraph::new(terms_lines(self.license.as_ref())),
            chunks[2],
        );

        self.license_input.render(
            frame,
            chunks[3],
            "paste a licence key, or leave blank",
            true,
            self.license_error.is_some(),
        );

        if let Some(error) = &self.license_error {
            frame.render_widget(
                Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red)),
                chunks[4],
            );
        }

        frame.render_widget(
            Paragraph::new("Enter Continue (installs a pasted key)   Esc Back")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            chunks[5],
        );
    }
}

/// Status colour follows the semantic role, not the literal word: a missing
/// licence is the free tier working as intended, not a fault.
fn status_style(status: LicenseStatus) -> (&'static str, Color) {
    match status {
        LicenseStatus::Valid => ("Premium", Color::Green),
        LicenseStatus::Missing => ("Free", Color::Gray),
        LicenseStatus::Expired => ("Expired", Color::Yellow),
        LicenseStatus::NotYetValid => ("Not yet valid", Color::Yellow),
        LicenseStatus::Invalid => ("Invalid", Color::Red),
    }
}

fn field(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<18}"), Style::default().fg(Color::DarkGray)),
        Span::raw(value),
    ])
}

fn timestamp(seconds: i64) -> String {
    chrono::DateTime::from_timestamp(seconds, 0).map_or_else(
        || "-".to_string(),
        |t| t.format("%Y-%m-%d %H:%M UTC").to_string(),
    )
}

fn terms_lines(license: Option<&LicenseResponse>) -> Vec<Line<'static>> {
    let Some(license) = license else {
        return vec![Line::from(Span::styled(
            "Reading licence…",
            Style::default().fg(Color::DarkGray),
        ))];
    };
    let (label, color) = status_style(license.status);
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<18}", "Status"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                label,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]),
        field("Configuration", license.profile_id.to_string()),
    ];
    if let Some(terms) = &license.terms {
        lines.push(field("Licensed to", terms.sub.clone()));
        lines.push(field("Licence ID", terms.jti.clone()));
        lines.push(field("Tier", terms.tier.clone()));
        lines.push(field("Valid from", timestamp(terms.nbf)));
        lines.push(field("Expires", timestamp(terms.exp)));
    }
    if let Some(url) = &license.purchase_url {
        lines.push(field("Premium", url.clone()));
    }
    lines
}
