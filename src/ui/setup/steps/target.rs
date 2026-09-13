use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::config::{DEFAULT_CODER_TOKEN_ENV, DEFAULT_CODER_URL_ENV};
use crate::ui::dialogs::centered_rect;
use crate::ui::setup::{
    CoderSetupField, SetupScreen, CODER_TARGET_OPTION_INDEX, LOCAL_TARGET_OPTION_INDEX,
};

impl SetupScreen {
    pub(crate) fn render_execution_target_step(&self, frame: &mut Frame) {
        let area = centered_rect(72, 76, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Execution Target ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(2),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new("Where agent commands run").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[0],
        );

        let selected = self
            .execution_target_state
            .selected()
            .unwrap_or(LOCAL_TARGET_OPTION_INDEX);
        frame.render_widget(
            Paragraph::new(vec![
                target_line(
                    "Local",
                    selected == LOCAL_TARGET_OPTION_INDEX,
                    "run beside Operator",
                ),
                target_line(
                    "Coder",
                    selected == CODER_TARGET_OPTION_INDEX,
                    "one workspace per ticket over SSH",
                ),
            ]),
            chunks[1],
        );

        if selected == CODER_TARGET_OPTION_INDEX {
            render_field(
                frame,
                chunks[2],
                "Target name",
                &self.coder_target_name,
                self.coder_field == CoderSetupField::TargetName,
            );
            render_field(
                frame,
                chunks[3],
                "Coder template",
                &self.coder_template,
                self.coder_field == CoderSetupField::Template,
            );
            let credentials = format!(
                "{DEFAULT_CODER_URL_ENV}: {}   {DEFAULT_CODER_TOKEN_ENV}: {}",
                env_status(DEFAULT_CODER_URL_ENV),
                env_status(DEFAULT_CODER_TOKEN_ENV)
            );
            frame.render_widget(
                Paragraph::new(credentials).style(Style::default().fg(Color::DarkGray)),
                chunks[4],
            );
        }

        if let Some(error) = &self.execution_target_error {
            frame.render_widget(
                Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red)),
                chunks[5],
            );
        }

        frame.render_widget(
            Paragraph::new("↑/↓ Select   Tab Switch field   Enter Continue   Esc Back")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            chunks[6],
        );
    }
}

fn target_line(name: &str, selected: bool, description: &str) -> Line<'static> {
    let marker = if selected { "(o)" } else { "( )" };
    let color = if selected { Color::Cyan } else { Color::Gray };
    Line::from(vec![
        Span::styled(format!("{marker} {name}"), Style::default().fg(color)),
        Span::raw(format!("  {description}")),
    ])
}

fn render_field(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    label: &str,
    value: &str,
    focused: bool,
) {
    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    frame.render_widget(
        Paragraph::new(value.to_string()).block(
            Block::default()
                .title(format!(" {label} "))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border)),
        ),
        area,
    );
}

fn env_status(name: &str) -> &'static str {
    if std::env::var_os(name).is_some() {
        "set"
    } else {
        "missing"
    }
}
