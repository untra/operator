use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::centered_rect;
use crate::config::SessionWrapperType;
use crate::ui::keybindings::{shortcuts_by_category_for_context, ShortcutContext};

pub struct HelpDialog {
    pub visible: bool,
    pub wrapper_type: SessionWrapperType,
}

impl HelpDialog {
    pub fn new(wrapper_type: SessionWrapperType) -> Self {
        Self {
            visible: false,
            wrapper_type,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let mut help_text = vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )),
            Line::from(""),
        ];

        // Add global shortcuts grouped by category
        for (category, shortcuts) in shortcuts_by_category_for_context(ShortcutContext::Global) {
            // Add category header (skip for first category to keep it compact)
            if category != crate::ui::keybindings::ShortcutCategory::General {
                help_text.push(Line::from(""));
            }

            for shortcut in shortcuts {
                help_text.push(Line::from(vec![
                    Span::styled(
                        shortcut.key_display_padded(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
        }

        // Add Launch Dialog section
        help_text.push(Line::from(""));
        help_text.push(Line::from(Span::styled(
            "In Launch Dialog:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )));

        for (_, shortcuts) in shortcuts_by_category_for_context(ShortcutContext::LaunchDialog) {
            for shortcut in shortcuts {
                help_text.push(Line::from(vec![
                    Span::styled(
                        shortcut.key_display_padded(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
        }

        // Add Session Preview section
        help_text.push(Line::from(""));
        help_text.push(Line::from(Span::styled(
            "In Session Preview:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )));

        for (_, shortcuts) in shortcuts_by_category_for_context(ShortcutContext::Preview) {
            for shortcut in shortcuts {
                help_text.push(Line::from(vec![
                    Span::styled(
                        shortcut.key_display_padded(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
        }

        // Zellij-specific reference keys
        if self.wrapper_type == SessionWrapperType::Zellij {
            help_text.push(Line::from(""));
            help_text.push(Line::from(Span::styled(
                "Zellij Keys (handled by Zellij, not Operator)",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            let zellij_keys: &[(&str, &str)] = &[
                ("Ctrl+t ", "Tab mode (switch/create/close tabs)"),
                ("Ctrl+p ", "Pane mode (split/move/resize panes)"),
                ("Ctrl+o w", "Session manager"),
                ("Ctrl+o f", "Toggle floating pane"),
                ("Alt+n  ", "New pane"),
                ("Alt+←/→", "Switch tabs"),
            ];
            for (key, desc) in zellij_keys {
                help_text.push(Line::from(vec![
                    Span::styled(format!("{key:<7}"), Style::default().fg(Color::Yellow)),
                    Span::raw(*desc),
                ]));
            }
        }

        // Footer
        help_text.push(Line::from(""));
        help_text.push(Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::Gray),
        )));

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(help, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_dialog_toggle() {
        let mut dialog = HelpDialog::new(SessionWrapperType::default());
        assert!(!dialog.visible);

        dialog.toggle();
        assert!(dialog.visible);

        dialog.toggle();
        assert!(!dialog.visible);
    }

    #[test]
    fn test_help_dialog_new_starts_hidden() {
        let dialog = HelpDialog::new(SessionWrapperType::default());
        assert!(!dialog.visible);
    }
}
