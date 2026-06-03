use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::queue::Ticket;
use crate::rest::RestApiStatus;
use crate::state::CompletedTicket;
use crate::templates::{color_for_key, glyph_for_key};
use crate::ui::status_panel::RowHints;

/// Format the ticket ID for display.
/// The `ticket_id` field already contains the full ID (e.g., "FEAT-1234"),
/// so we should NOT prepend the `ticket_type` again.
pub fn format_display_id(ticket_id: &str) -> String {
    ticket_id.to_string()
}

pub struct QueuePanel {
    pub tickets: Vec<Ticket>,
    pub state: ListState,
    pub title: String,
}

impl QueuePanel {
    pub fn new(title: String) -> Self {
        Self {
            tickets: Vec::new(),
            state: ListState::default(),
            title,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Calculate max summary length based on area width
        let max_summary_len = (area.width as usize).saturating_sub(6); // glyph + space + padding

        let items: Vec<ListItem> = self
            .tickets
            .iter()
            .map(|t| {
                let glyph = glyph_for_key(&t.ticket_type);

                let priority_color = match t.priority.as_str() {
                    "P0-critical" => Color::Red,
                    "P1-high" => Color::Yellow,
                    "P2-medium" => Color::White,
                    _ => Color::Gray,
                };

                // Get glyph color from template, fall back to priority color
                let glyph_color =
                    color_for_key(&t.ticket_type).map_or(priority_color, |c| match c {
                        "blue" => Color::Blue,
                        "cyan" => Color::Cyan,
                        "green" => Color::Green,
                        "yellow" => Color::Yellow,
                        "magenta" => Color::Magenta,
                        "red" => Color::Red,
                        _ => priority_color,
                    });

                // Trim summary to fit
                let summary = if t.summary.len() > max_summary_len {
                    format!("{}...", &t.summary[..max_summary_len.saturating_sub(3)])
                } else {
                    t.summary.clone()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{glyph} "), Style::default().fg(glyph_color)),
                    Span::styled(summary, Style::default().fg(priority_color)),
                ]))
            })
            .collect();

        let title = format!("{} ({})", self.title, self.tickets.len());
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

pub struct CompletedPanel {
    pub tickets: Vec<CompletedTicket>,
    pub title: String,
}

impl CompletedPanel {
    pub fn new(title: String) -> Self {
        Self {
            tickets: Vec::new(),
            title,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<ListItem> = self
            .tickets
            .iter()
            .map(|t| {
                let time = t.completed_at.format("%H:%M").to_string();

                ListItem::new(Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format_display_id(&t.ticket_id),
                        Style::default().fg(Color::White),
                    ),
                    Span::raw(" "),
                    Span::styled(time, Style::default().fg(Color::Gray)),
                ]))
            })
            .collect();

        let title = format!("{} ({})", self.title, self.tickets.len());
        let list = List::new(items).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(list, area);
    }
}

pub struct StatusBar {
    pub paused: bool,
    pub agent_count: usize,
    pub max_agents: usize,
    pub rest_api_status: RestApiStatus,
    pub embed_ui_available: bool,
    pub exit_confirmation_mode: bool,
    pub update_available_version: Option<String>,
    pub status_message: Option<String>,
    pub row_hints: Option<RowHints>,
}

impl StatusBar {
    /// Build keyboard hints based on available terminal width
    fn build_hints(width: u16) -> Span<'static> {
        // Full: all hints (requires ~120 chars)
        let full = "[Q]ueue [L]aunch [C]reate Pro[J]ects [K]anban [P]ause [R]esume [A]gents [S]ync [V]iew [?]Help [q]uit";
        // Medium: abbreviated hints (requires ~100 chars)
        let medium = "[Q] [L]aunch [C]reate [K]anban [P]/[R] [S]ync [V]iew [?] [q]";
        // Short: essential hints only (requires ~80 chars)
        let short = "[L]aunch [C]reate [S]ync [V] [?] [q]";
        // Minimal: just help
        let minimal = "[?]Help";

        let hint_text = if width >= 120 {
            full
        } else if width >= 100 {
            medium
        } else if width >= 80 {
            short
        } else {
            minimal
        };

        Span::styled(
            format!("  {hint_text}"),
            Style::default().fg(Color::DarkGray),
        )
    }

    /// Build context-sensitive hints from the selected status panel row.
    fn build_dynamic_hints(hints: &RowHints, width: u16) -> Span<'static> {
        let mut parts: Vec<String> = Vec::new();

        if let Some(verb) = hints.primary_verb {
            parts.push(format!("[Enter] {verb}"));
        }
        if let Some(title) = hints.special_title {
            parts.push(format!("[⇧Enter] {title}"));
        }
        if let Some(title) = hints.refresh_title {
            parts.push(format!("[^Enter] {title}"));
        }

        if parts.is_empty() {
            return Self::build_hints(width);
        }

        let full = parts.join("  ");
        let hint_text = if (full.len() + 2) <= width as usize {
            full
        } else if parts.len() > 2 {
            parts[..2].join("  ")
        } else if parts.len() > 1 {
            parts[0].clone()
        } else {
            parts.first().cloned().unwrap_or_default()
        };

        Span::styled(format!("  {hint_text}"), Style::default().fg(Color::Cyan))
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Exit confirmation mode - show only the exit message (highest priority)
        if self.exit_confirmation_mode {
            let content = Line::from(vec![Span::styled(
                "  Press Ctrl+C again to exit",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]);

            let bar = Paragraph::new(content).block(Block::default().borders(Borders::TOP));
            frame.render_widget(bar, area);
            return;
        }

        // Update notification - show update available message
        if let Some(ref new_version) = self.update_available_version {
            let current_version = env!("CARGO_PKG_VERSION");
            let message = format!(
                "  Update available: v{current_version} -> v{new_version} | Run: cargo install operator-tui"
            );

            let content = Line::from(vec![Span::styled(
                message,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]);

            let bar = Paragraph::new(content).block(Block::default().borders(Borders::TOP));
            frame.render_widget(bar, area);
            return;
        }

        // Normal mode - show regular status bar
        let status = if self.paused {
            Span::styled("⏸ PAUSED", Style::default().fg(Color::Yellow))
        } else {
            Span::styled("▶ RUNNING", Style::default().fg(Color::Green))
        };

        let agents = Span::styled(
            format!("  {}/{} agents", self.agent_count, self.max_agents),
            Style::default().fg(Color::Gray),
        );

        let web_ind = web_indicator(&self.rest_api_status, self.embed_ui_available);

        let help = match &self.row_hints {
            Some(hints) => Self::build_dynamic_hints(hints, area.width),
            None => Self::build_hints(area.width),
        };

        let mut spans = vec![status, agents, web_ind];

        // Show transient status message if present
        if let Some(ref msg) = self.status_message {
            spans.push(Span::styled(
                format!("  {msg}"),
                Style::default().fg(Color::Yellow),
            ));
        }

        spans.push(help);

        let content = Line::from(spans);

        let bar = Paragraph::new(content).block(Block::default().borders(Borders::TOP));

        frame.render_widget(bar, area);
    }
}

pub struct HeaderBar {
    pub version: &'static str,
    pub wrapper_name: &'static str,
}

impl HeaderBar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let spans = vec![
            Span::styled(
                " Operator!",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" v{}", self.version),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!(" \u{2502} {}", self.wrapper_name),
                Style::default().fg(Color::DarkGray),
            ),
        ];

        let content = Line::from(spans);
        let bar = Paragraph::new(content).block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(bar, area);
    }
}

/// Build a status indicator span for the REST API / embedded web UI.
fn web_indicator(status: &RestApiStatus, embed_ui: bool) -> Span<'static> {
    let label = if embed_ui { "[W]eb" } else { "[A]PI" };
    match status {
        RestApiStatus::Running { port } => Span::styled(
            format!("  {label} ●:{port}"),
            Style::default().fg(Color::Green),
        ),
        RestApiStatus::Starting | RestApiStatus::Stopping => {
            Span::styled(format!("  {label} ●"), Style::default().fg(Color::Yellow))
        }
        RestApiStatus::Error(_) => {
            Span::styled(format!("  {label} ●"), Style::default().fg(Color::Red))
        }
        RestApiStatus::Stopped => {
            Span::styled(format!("  {label} ○"), Style::default().fg(Color::White))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_display_id_returns_ticket_id_as_is() {
        // The ticket_id already contains the full ID (e.g., "FEAT-1234")
        // so format_display_id should return it unchanged
        assert_eq!(format_display_id("FEAT-1234"), "FEAT-1234");
        assert_eq!(format_display_id("FIX-5678"), "FIX-5678");
        assert_eq!(format_display_id("SPIKE-9999"), "SPIKE-9999");
        assert_eq!(format_display_id("INV-0001"), "INV-0001");
    }

    #[test]
    fn test_format_display_id_does_not_duplicate_type() {
        // Verify that the display ID doesn't become "FEAT-FEAT-1234"
        let ticket_id = "FEAT-7598";
        let display_id = format_display_id(ticket_id);

        // Should NOT have the duplicated type prefix
        assert!(
            !display_id.starts_with("FEAT-FEAT"),
            "Display ID should not have duplicated prefix, got: {display_id}"
        );
        assert_eq!(display_id, "FEAT-7598");
    }

    #[test]
    fn test_format_display_id_handles_various_types() {
        // Test all ticket types
        let test_cases = vec![
            ("FEAT-1234", "FEAT-1234"),
            ("FIX-5678", "FIX-5678"),
            ("SPIKE-1111", "SPIKE-1111"),
            ("INV-2222", "INV-2222"),
            ("TASK-3333", "TASK-3333"),
        ];

        for (input, expected) in test_cases {
            let result = format_display_id(input);
            assert_eq!(
                result, expected,
                "format_display_id({input}) should return {expected}, got {result}"
            );
        }
    }

    #[test]
    fn test_build_hints_full_width() {
        let hints = StatusBar::build_hints(120);
        let content = hints.content;
        assert!(
            content.contains("[K]anban"),
            "Full width should include [K]anban"
        );
        assert!(
            content.contains("[Q]ueue"),
            "Full width should include [Q]ueue"
        );
        assert!(
            content.contains("[?]Help"),
            "Full width should include [?]Help"
        );
    }

    #[test]
    fn test_build_hints_medium_width() {
        let hints = StatusBar::build_hints(100);
        let content = hints.content;
        assert!(
            content.contains("[K]anban"),
            "Medium width should include [K]anban"
        );
        assert!(
            content.contains("[S]ync"),
            "Medium width should include [S]ync"
        );
    }

    #[test]
    fn test_build_hints_short_width() {
        let hints = StatusBar::build_hints(80);
        let content = hints.content;
        // Short width should NOT include [K]anban
        assert!(
            !content.contains("[K]anban"),
            "Short width should NOT include [K]anban"
        );
        assert!(
            content.contains("[S]ync"),
            "Short width should include [S]ync"
        );
        assert!(
            content.contains("[L]aunch"),
            "Short width should include [L]aunch"
        );
    }

    #[test]
    fn test_build_hints_minimal_width() {
        let hints = StatusBar::build_hints(60);
        let content = hints.content;
        assert!(
            content.contains("[?]Help"),
            "Minimal width should include [?]Help"
        );
        assert!(
            !content.contains("[S]ync"),
            "Minimal width should NOT include [S]ync"
        );
    }

    #[test]
    fn test_web_indicator_running() {
        let span = web_indicator(&RestApiStatus::Running { port: 7008 }, true);
        let text: &str = &span.content;
        assert!(text.contains("●:7008"), "should show port: {text}");
        assert_eq!(span.style.fg, Some(Color::Green));
    }

    #[test]
    fn test_web_indicator_starting() {
        let span = web_indicator(&RestApiStatus::Starting, true);
        assert_eq!(span.style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_web_indicator_stopping() {
        let span = web_indicator(&RestApiStatus::Stopping, true);
        assert_eq!(span.style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_web_indicator_error() {
        let span = web_indicator(&RestApiStatus::Error("bind failed".into()), true);
        assert_eq!(span.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_web_indicator_stopped() {
        let span = web_indicator(&RestApiStatus::Stopped, true);
        let text: &str = &span.content;
        assert!(text.contains('○'), "should show hollow circle: {text}");
        assert_eq!(span.style.fg, Some(Color::White));
    }

    #[test]
    fn test_web_indicator_label_with_embed_ui() {
        let span = web_indicator(&RestApiStatus::Stopped, true);
        let text: &str = &span.content;
        assert!(
            text.contains("[W]eb"),
            "embed_ui=true should show [W]eb: {text}"
        );
    }

    #[test]
    fn test_web_indicator_label_without_embed_ui() {
        let span = web_indicator(&RestApiStatus::Stopped, false);
        let text: &str = &span.content;
        assert!(
            text.contains("[A]PI"),
            "embed_ui=false should show [A]PI: {text}"
        );
    }

    #[test]
    fn test_build_dynamic_hints_all_actions() {
        let hints = RowHints {
            primary_verb: Some("Edit"),
            special_title: Some("Reset"),
            refresh_title: Some("Reload"),
        };
        let span = StatusBar::build_dynamic_hints(&hints, 120);
        let text: &str = &span.content;
        assert!(text.contains("[Enter] Edit"), "should show primary: {text}");
        assert!(
            text.contains("[⇧Enter] Reset"),
            "should show special: {text}"
        );
        assert!(
            text.contains("[^Enter] Reload"),
            "should show refresh: {text}"
        );
        assert_eq!(span.style.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_build_dynamic_hints_primary_only() {
        let hints = RowHints {
            primary_verb: Some("Open"),
            special_title: None,
            refresh_title: None,
        };
        let span = StatusBar::build_dynamic_hints(&hints, 120);
        let text: &str = &span.content;
        assert!(text.contains("[Enter] Open"), "should show primary: {text}");
        assert!(
            !text.contains("[⇧Enter]"),
            "should not show special: {text}"
        );
        assert!(
            !text.contains("[^Enter]"),
            "should not show refresh: {text}"
        );
    }

    #[test]
    fn test_build_dynamic_hints_no_actions_falls_back() {
        let hints = RowHints {
            primary_verb: None,
            special_title: None,
            refresh_title: None,
        };
        let span = StatusBar::build_dynamic_hints(&hints, 120);
        let text: &str = &span.content;
        assert!(
            text.contains("[Q]ueue") || text.contains("[?]Help"),
            "should fall back to static hints: {text}"
        );
        assert_eq!(span.style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_build_dynamic_hints_truncates_at_narrow_width() {
        let hints = RowHints {
            primary_verb: Some("Edit"),
            special_title: Some("Reset"),
            refresh_title: Some("Reload"),
        };
        let span = StatusBar::build_dynamic_hints(&hints, 40);
        let text: &str = &span.content;
        assert!(
            !text.contains("[^Enter] Reload"),
            "narrow width should drop refresh: {text}"
        );
    }
}
