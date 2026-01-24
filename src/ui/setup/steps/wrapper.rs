//! Session wrapper choice, tmux onboarding, and vscode setup step rendering

use crate::ui::dialogs::centered_rect;
use crate::ui::setup::types::{SessionWrapperOption, TmuxDetectionStatus, VSCodeDetectionStatus};
use crate::ui::setup::SetupScreen;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

impl SetupScreen {
    pub(crate) fn render_session_wrapper_choice_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 60, frame.area());
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
                Span::raw(" Setup - Session Wrapper "),
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
                Constraint::Length(3), // Description
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Options list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Session Wrapper Configuration",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let description = Paragraph::new(vec![
            Line::from("Operator runs agents in terminal sessions."),
            Line::from("Select your preferred wrapper:"),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(description, chunks[1]);

        // Instructions
        let instructions =
            Paragraph::new(vec![Line::from("Use arrows to navigate, Enter to select")])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, chunks[2]);

        // Options list
        let items: Vec<ListItem> = SessionWrapperOption::all()
            .iter()
            .map(|opt| {
                let is_selected = opt.to_wrapper_type() == self.selected_wrapper;
                let radio = if is_selected { "(o)" } else { "( )" };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            radio,
                            Style::default().fg(if is_selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            opt.label(),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(if is_selected {
                                    Color::White
                                } else {
                                    Color::Gray
                                }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(opt.description(), Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[3], &mut self.wrapper_state);

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

    pub(crate) fn render_tmux_onboarding_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 75, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Tmux Configuration ")
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
                Constraint::Length(3), // Status
                Constraint::Length(1), // Spacer
                Constraint::Min(12),   // Help text or install instructions
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Tmux Session Configuration",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Status indicator
        let status_line = match &self.tmux_status {
            TmuxDetectionStatus::NotChecked => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[?] ", Style::default().fg(Color::Yellow)),
                Span::styled("Not checked", Style::default().fg(Color::Yellow)),
            ]),
            TmuxDetectionStatus::Available { version } => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[+] ", Style::default().fg(Color::Green)),
                Span::styled(
                    format!("Available (v{})", version),
                    Style::default().fg(Color::Green),
                ),
            ]),
            TmuxDetectionStatus::NotInstalled => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[x] ", Style::default().fg(Color::Red)),
                Span::styled("Not installed", Style::default().fg(Color::Red)),
            ]),
            TmuxDetectionStatus::VersionTooOld { current, required } => Line::from(vec![
                Span::styled("Tmux status: ", Style::default().fg(Color::Gray)),
                Span::styled("[x] ", Style::default().fg(Color::Red)),
                Span::styled(
                    format!("Version too old (v{}, need {}+)", current, required),
                    Style::default().fg(Color::Red),
                ),
            ]),
        };
        let status = Paragraph::new(vec![status_line]).alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);

        // Help text or install instructions
        let help_text = if matches!(self.tmux_status, TmuxDetectionStatus::Available { .. }) {
            vec![
                Line::from(Span::styled(
                    "Essential Commands:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Detach from session:  ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "Ctrl+a",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " (quick, no prefix needed!)",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Fallback detach:      ", Style::default().fg(Color::Gray)),
                    Span::styled("Ctrl+b", Style::default().fg(Color::Cyan)),
                    Span::styled(" then ", Style::default().fg(Color::Gray)),
                    Span::styled("d", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  List sessions:        ", Style::default().fg(Color::Gray)),
                    Span::styled("tmux ls", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Attach to session:    ", Style::default().fg(Color::Gray)),
                    Span::styled("tmux attach -t ", Style::default().fg(Color::Green)),
                    Span::styled("<name>", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Operator session names start with 'op-'",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    "Install tmux:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  macOS:         ", Style::default().fg(Color::Gray)),
                    Span::styled("brew install tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Ubuntu/Debian: ", Style::default().fg(Color::Gray)),
                    Span::styled("sudo apt install tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Fedora/RHEL:   ", Style::default().fg(Color::Gray)),
                    Span::styled("sudo dnf install tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Arch:          ", Style::default().fg(Color::Gray)),
                    Span::styled("sudo pacman -S tmux", Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "After installing, press [R] to re-check",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        };
        frame.render_widget(Paragraph::new(help_text), chunks[4]);

        // Footer - different depending on status
        let footer = if matches!(self.tmux_status, TmuxDetectionStatus::Available { .. }) {
            Paragraph::new(Line::from(vec![
                Span::styled("[R]", Style::default().fg(Color::Yellow)),
                Span::raw(" re-check  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" continue  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]))
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled("[R]", Style::default().fg(Color::Green)),
                Span::raw(" re-check tmux  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]))
        };
        frame.render_widget(footer.alignment(Alignment::Center), chunks[5]);
    }

    pub(crate) fn render_vscode_setup_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" VS Code Extension Setup ")
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
                Constraint::Length(3), // Status
                Constraint::Length(1), // Spacer
                Constraint::Min(12),   // Instructions
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "VS Code Integration Setup",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Status indicator
        let status_line = match &self.vscode_status {
            VSCodeDetectionStatus::NotChecked => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[?] ", Style::default().fg(Color::Yellow)),
                Span::styled("Not checked", Style::default().fg(Color::Yellow)),
            ]),
            VSCodeDetectionStatus::Checking => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[~] ", Style::default().fg(Color::Yellow)),
                Span::styled("Checking...", Style::default().fg(Color::Yellow)),
            ]),
            VSCodeDetectionStatus::Connected { version } => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[+] ", Style::default().fg(Color::Green)),
                Span::styled(
                    format!("Connected (v{})", version),
                    Style::default().fg(Color::Green),
                ),
            ]),
            VSCodeDetectionStatus::NotReachable => Line::from(vec![
                Span::styled("Extension status: ", Style::default().fg(Color::Gray)),
                Span::styled("[x] ", Style::default().fg(Color::Red)),
                Span::styled("Not detected", Style::default().fg(Color::Red)),
            ]),
        };
        let status = Paragraph::new(vec![status_line]).alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);

        // Instructions
        let instructions = vec![
            Line::from(Span::styled(
                "To use VS Code integration:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  1. ", Style::default().fg(Color::Cyan)),
                Span::raw("Install the Operator extension from:"),
            ]),
            Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    "https://operator.untra.io/vscode",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  2. ", Style::default().fg(Color::Cyan)),
                Span::raw("Restart VS Code after installation"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  3. ", Style::default().fg(Color::Cyan)),
                Span::raw("The extension will start automatically on port 7009"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Note: VS Code extension support is coming soon!",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(instructions), chunks[4]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[T]", Style::default().fg(Color::Yellow)),
            Span::raw(" test connection  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[5]);
    }
}
