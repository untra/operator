use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::queue::Ticket;
use crate::state::{AgentState, CompletedTicket, OrphanSession};
use crate::templates::{color_for_key, glyph_for_key};

/// Format the ticket ID for display.
/// The ticket_id field already contains the full ID (e.g., "FEAT-1234"),
/// so we should NOT prepend the ticket_type again.
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
                let glyph_color = color_for_key(&t.ticket_type)
                    .map(|c| match c {
                        "blue" => Color::Blue,
                        "cyan" => Color::Cyan,
                        "green" => Color::Green,
                        "yellow" => Color::Yellow,
                        "magenta" => Color::Magenta,
                        "red" => Color::Red,
                        _ => priority_color,
                    })
                    .unwrap_or(priority_color);

                // Trim summary to fit
                let summary = if t.summary.len() > max_summary_len {
                    format!("{}...", &t.summary[..max_summary_len.saturating_sub(3)])
                } else {
                    t.summary.clone()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", glyph), Style::default().fg(glyph_color)),
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

pub struct AgentsPanel {
    pub agents: Vec<AgentState>,
    pub orphan_sessions: Vec<OrphanSession>,
    pub state: ListState,
    pub title: String,
}

impl AgentsPanel {
    pub fn new(title: String) -> Self {
        Self {
            agents: Vec::new(),
            orphan_sessions: Vec::new(),
            state: ListState::default(),
            title,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, max_agents: usize) {
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let mut items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|a| {
                let status_icon = match a.status.as_str() {
                    "running" => "▶",
                    "awaiting_input" => "⏸",
                    "completing" => "✓",
                    _ => "?",
                };

                let status_color = match a.status.as_str() {
                    "running" => Color::Green,
                    "awaiting_input" => Color::Yellow,
                    "completing" => Color::Cyan,
                    _ => Color::Gray,
                };

                // Get the current step display text
                let step_display = a
                    .current_step
                    .as_ref()
                    .map(|s| format!("[{}]", s))
                    .unwrap_or_default();

                // Calculate elapsed time
                let elapsed = chrono::Utc::now()
                    .signed_duration_since(a.started_at)
                    .num_seconds();
                let elapsed_display = if elapsed >= 3600 {
                    format!("{}h{}m", elapsed / 3600, (elapsed % 3600) / 60)
                } else if elapsed >= 60 {
                    format!("{}m", elapsed / 60)
                } else {
                    format!("{}s", elapsed)
                };

                let lines = vec![
                    Line::from(vec![
                        Span::styled(status_icon, Style::default().fg(status_color)),
                        Span::raw(" "),
                        Span::styled(&a.project, Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                        Span::styled(step_display, Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format_display_id(&a.ticket_id),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::raw(" "),
                        Span::styled(elapsed_display, Style::default().fg(Color::DarkGray)),
                    ]),
                ];

                ListItem::new(lines)
            })
            .collect();

        // Add orphan sessions below a fold separator if any exist
        if !self.orphan_sessions.is_empty() {
            // Add separator line
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "── Orphan Sessions ──",
                Style::default().fg(Color::DarkGray),
            )])));

            // Add each orphan session
            for orphan in &self.orphan_sessions {
                let mut spans = vec![
                    Span::styled("⚠ ", Style::default().fg(Color::Red)),
                    Span::styled(
                        &orphan.session_name,
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ];

                if orphan.attached {
                    spans.push(Span::styled(
                        " [attached]",
                        Style::default().fg(Color::Yellow),
                    ));
                }

                items.push(ListItem::new(Line::from(spans)));
            }
        }

        let title = format!("{} ({}/{})", self.title, self.agents.len(), max_agents);
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    /// Get the total number of items (agents + separator + orphans) for selection
    pub fn total_items(&self) -> usize {
        let orphan_items = if self.orphan_sessions.is_empty() {
            0
        } else {
            1 + self.orphan_sessions.len() // separator + orphans
        };
        self.agents.len() + orphan_items
    }

    /// Get the selected orphan session, if any
    pub fn selected_orphan(&self) -> Option<&OrphanSession> {
        if let Some(selected) = self.state.selected() {
            if selected > self.agents.len() && !self.orphan_sessions.is_empty() {
                // selected - agents.len() - 1 (for separator) = orphan index
                let orphan_idx = selected - self.agents.len() - 1;
                return self.orphan_sessions.get(orphan_idx);
            }
        }
        None
    }
}

pub struct AwaitingPanel {
    pub agents: Vec<AgentState>,
    pub state: ListState,
    pub title: String,
}

impl AwaitingPanel {
    pub fn new(title: String) -> Self {
        Self {
            agents: Vec::new(),
            state: ListState::default(),
            title,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|a| {
                // Get the current step display text
                let step_display = a
                    .current_step
                    .as_ref()
                    .map(|s| format!("[{}]", s))
                    .unwrap_or_default();

                let lines = vec![
                    Line::from(vec![
                        Span::styled("⏸ ", Style::default().fg(Color::Yellow)),
                        Span::styled(&a.project, Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                        Span::styled(step_display, Style::default().fg(Color::Cyan)),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}]", format_display_id(&a.ticket_id)),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            a.last_message.as_deref().unwrap_or("Awaiting input..."),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]),
                ];

                ListItem::new(lines)
            })
            .collect();

        let title = format!("{} ({})", self.title, self.agents.len());
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

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
}

impl StatusBar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let status = if self.paused {
            Span::styled("⏸ PAUSED", Style::default().fg(Color::Yellow))
        } else {
            Span::styled("▶ RUNNING", Style::default().fg(Color::Green))
        };

        let agents = Span::styled(
            format!("  {}/{} agents", self.agent_count, self.max_agents),
            Style::default().fg(Color::Gray),
        );

        let help = Span::styled(
            "  [Q]ueue [L]aunch [C]reate Pro[J]ects [P]ause [R]esume [A]gents [S]ync [V]iew [?]Help [q]uit",
            Style::default().fg(Color::DarkGray),
        );

        let content = Line::from(vec![status, agents, help]);

        let bar = Paragraph::new(content).block(Block::default().borders(Borders::TOP));

        frame.render_widget(bar, area);
    }
}

pub struct HeaderBar<'a> {
    pub version: &'static str,
    pub rate_limit: Option<&'a crate::api::RateLimitInfo>,
}

impl HeaderBar<'_> {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut spans = vec![
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
        ];

        // Add rate limit meter if available
        if let Some(info) = self.rate_limit {
            spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));

            if info.is_rate_limited {
                // Rate limited - show warning
                let msg = if let Some(secs) = info.retry_after_secs {
                    format!("RATE LIMITED ({}s)", secs)
                } else {
                    "RATE LIMITED".to_string()
                };
                spans.push(Span::styled(msg, Style::default().fg(Color::Red)));
            } else if let Some(pct) = info.best_remaining_pct() {
                // Show progress bar and percentage
                let bar = info.progress_bar(10);
                let color = if pct < 0.2 {
                    Color::Yellow // Warning: below 20%
                } else {
                    Color::Green
                };
                spans.push(Span::styled(bar, Style::default().fg(color)));
                spans.push(Span::styled(
                    format!(" {:.0}%", pct * 100.0),
                    Style::default().fg(color),
                ));
                spans.push(Span::styled(
                    format!(" {}", info.provider),
                    Style::default().fg(Color::DarkGray),
                ));
            } else {
                spans.push(Span::styled(
                    format!("{}: synced", info.provider),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            spans.push(Span::styled(
                "  [S]ync",
                Style::default().fg(Color::DarkGray),
            ));
        } else {
            // No rate limit info - show hint
            spans.push(Span::styled(
                "  │  [S]ync rate limits",
                Style::default().fg(Color::DarkGray),
            ));
        }

        let content = Line::from(spans);
        let bar = Paragraph::new(content).block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(bar, area);
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
            "Display ID should not have duplicated prefix, got: {}",
            display_id
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
                "format_display_id({}) should return {}, got {}",
                input, expected, result
            );
        }
    }
}
