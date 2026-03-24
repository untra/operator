use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::state::{AgentState, OrphanSession};
use crate::ui::panels::format_display_id;

pub struct InProgressPanel {
    pub agents: Vec<AgentState>,
    pub orphan_sessions: Vec<OrphanSession>,
    pub state: ListState,
    pub title: String,
}

impl InProgressPanel {
    pub fn new(title: String) -> Self {
        Self {
            agents: Vec::new(),
            orphan_sessions: Vec::new(),
            state: ListState::default(),
            title,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, max_agents: usize) {
        let has_awaiting = self.agents.iter().any(|a| a.status == "awaiting_input");

        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else if has_awaiting {
            // Strobe effect: 6-second cycle with pulse for first 500ms
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            let cycle_position = now % 6000;

            if cycle_position < 500 {
                // Pulse ON - bright orange
                Style::default().fg(Color::Rgb(255, 165, 0))
            } else if cycle_position < 1000 {
                // Fade out from orange to gray
                let progress = (cycle_position - 500) as f32 / 500.0;
                let r = (255.0 - progress * 127.0) as u8; // 255 -> 128
                let g = (165.0 - progress * 83.0) as u8; // 165 -> 82
                let b = (progress * 82.0) as u8; // 0 -> 82
                Style::default().fg(Color::Rgb(r, g, b))
            } else {
                Style::default().fg(Color::Gray)
            }
        } else {
            Style::default().fg(Color::Gray)
        };

        let mut items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|a| {
                // Check review state first for awaiting_input agents
                let (status_icon, status_color) = if a.status == "awaiting_input" {
                    match a.review_state.as_deref() {
                        Some("pending_plan") => ("\u{1f4cb}", Color::Yellow), // 📋 Plan review
                        Some("pending_visual") => ("\u{1f441}", Color::Magenta), // 👁 Visual review
                        Some("pending_pr_creation") => ("\u{1f504}", Color::Blue), // 🔄 Creating PR
                        Some("pending_pr_merge") => ("\u{1f517}", Color::Cyan), // 🔗 Awaiting merge
                        _ => ("⏸", Color::Yellow),                            // Standard awaiting
                    }
                } else {
                    match a.status.as_str() {
                        "running" => ("▶", Color::Green),
                        "completing" => ("✓", Color::Cyan),
                        _ => ("?", Color::Gray),
                    }
                };

                // Tool indicator (A=Anthropic/Claude, G=Gemini, O=OpenAI/Codex)
                let tool_indicator = match a.llm_tool.as_deref() {
                    Some("claude") => ("A", Color::Rgb(193, 95, 60)),
                    Some("gemini") => ("G", Color::Rgb(111, 66, 193)),
                    Some("codex") => ("O", Color::Green),
                    _ => (" ", Color::Reset),
                };

                // Check launch mode for docker and yolo
                let is_docker = a.launch_mode.as_ref().is_some_and(|m| m.contains("docker"));
                let is_yolo = a.launch_mode.as_ref().is_some_and(|m| m.contains("yolo"));

                // YOLO indicator with rainbow animation (6-second cycle: R -> G -> B)
                let yolo_indicator = if is_yolo {
                    let phase = (chrono::Utc::now().timestamp() / 2) % 3;
                    let color = match phase {
                        0 => Color::Red,
                        1 => Color::Green,
                        _ => Color::Blue,
                    };
                    ("Y", color)
                } else {
                    (" ", Color::Reset)
                };

                // Docker indicator (D on gray background)
                let docker_indicator = if is_docker {
                    ("D", Color::White)
                } else {
                    (" ", Color::Reset)
                };
                let docker_bg = if is_docker {
                    Color::DarkGray
                } else {
                    Color::Reset
                };

                // Wrapper badge: C=cmux, T=tmux, Z=zellij, V=vscode
                let wrapper_badge = match a.session_wrapper.as_deref() {
                    Some("cmux") => "C",
                    Some("tmux") => "T",
                    Some("zellij") => "Z",
                    Some("vscode") => "V",
                    _ => " ",
                };

                // Get the current step display text
                let step_display = a
                    .current_step
                    .as_ref()
                    .map(|s| format!("[{s}]"))
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
                    format!("{elapsed}s")
                };

                // Build the first line with tool indicators
                let mut line1_spans = vec![Span::styled(
                    tool_indicator.0,
                    Style::default().fg(tool_indicator.1),
                )];

                // Add YOLO indicator (with or without docker background)
                if is_yolo {
                    line1_spans.push(Span::styled(
                        yolo_indicator.0,
                        Style::default().fg(yolo_indicator.1).bg(docker_bg),
                    ));
                } else if is_docker {
                    // Docker without YOLO - show D
                    line1_spans.push(Span::styled(
                        docker_indicator.0,
                        Style::default().fg(docker_indicator.1).bg(docker_bg),
                    ));
                }

                // Wrapper badge
                line1_spans.push(Span::styled(
                    wrapper_badge,
                    Style::default().fg(Color::DarkGray),
                ));

                line1_spans.extend(vec![
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::raw(" "),
                    Span::styled(&a.project, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(step_display, Style::default().fg(Color::Cyan)),
                ]);

                // Build line 2: ticket ID, elapsed, and cmux refs if applicable
                let mut line2_spans = vec![
                    Span::raw("  "),
                    Span::styled(
                        format_display_id(&a.ticket_id),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::raw(" "),
                    Span::styled(elapsed_display, Style::default().fg(Color::DarkGray)),
                ];

                // Add cmux workspace/window refs (abbreviated to first 6 chars)
                if a.session_wrapper.as_deref() == Some("cmux") {
                    if let Some(ref ws_ref) = a.session_context_ref {
                        let abbrev = &ws_ref[..ws_ref.len().min(6)];
                        line2_spans.push(Span::styled(
                            format!(" ws:{abbrev}"),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    if let Some(ref win_ref) = a.session_window_ref {
                        let abbrev = &win_ref[..win_ref.len().min(6)];
                        line2_spans.push(Span::styled(
                            format!(" win:{abbrev}"),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }

                let mut lines = vec![Line::from(line1_spans), Line::from(line2_spans)];

                // Add review hint line for agents awaiting review
                if a.status == "awaiting_input" {
                    let hint = match a.review_state.as_deref() {
                        Some("pending_plan") => Some("[a]pprove [r]eject plan"),
                        Some("pending_visual") => Some("[a]pprove [r]eject visual"),
                        Some("pending_pr_creation") => Some("Creating PR..."),
                        Some("pending_pr_merge") => {
                            if a.pr_url.is_some() {
                                None
                            } else {
                                Some("Waiting for PR merge")
                            }
                        }
                        _ => None,
                    };

                    if let Some(hint_text) = hint {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(
                                hint_text,
                                Style::default()
                                    .fg(Color::DarkGray)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                        ]));
                    }
                }

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_agent(id: &str, status: &str) -> AgentState {
        AgentState {
            id: id.to_string(),
            ticket_id: format!("FEAT-{id}"),
            ticket_type: "FEAT".to_string(),
            project: "test-project".to_string(),
            status: status.to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now(),
            last_message: None,
            paired: false,
            session_name: None,
            session_wrapper: None,
            session_window_ref: None,
            session_context_ref: None,
            session_pane_ref: None,
            content_hash: None,
            current_step: None,
            step_started_at: None,
            last_content_change: None,
            pr_url: None,
            pr_number: None,
            github_repo: None,
            pr_status: None,
            completed_steps: Vec::new(),
            llm_tool: None,
            llm_model: None,
            launch_mode: None,
            review_state: None,
            dev_server_pid: None,
            worktree_path: None,
        }
    }

    fn make_orphan(name: &str, attached: bool) -> OrphanSession {
        OrphanSession {
            session_name: name.to_string(),
            created: None,
            attached,
        }
    }

    #[test]
    fn test_new_creates_empty_panel() {
        let panel = InProgressPanel::new("In Progress".to_string());
        assert!(panel.agents.is_empty());
        assert!(panel.orphan_sessions.is_empty());
        assert_eq!(panel.title, "In Progress");
        assert_eq!(panel.state.selected(), None);
    }

    #[test]
    fn test_total_items_agents_only() {
        let mut panel = InProgressPanel::new("In Progress".to_string());
        panel.agents = vec![
            make_agent("1", "running"),
            make_agent("2", "running"),
            make_agent("3", "awaiting_input"),
        ];
        assert_eq!(panel.total_items(), 3);
    }

    #[test]
    fn test_total_items_with_orphans() {
        let mut panel = InProgressPanel::new("In Progress".to_string());
        panel.agents = vec![
            make_agent("1", "running"),
            make_agent("2", "running"),
            make_agent("3", "awaiting_input"),
        ];
        panel.orphan_sessions = vec![make_orphan("op-abc", false), make_orphan("op-def", true)];
        // 3 agents + 1 separator + 2 orphans = 6
        assert_eq!(panel.total_items(), 6);
    }

    #[test]
    fn test_selected_orphan_returns_none_for_agent_selection() {
        let mut panel = InProgressPanel::new("In Progress".to_string());
        panel.agents = vec![make_agent("1", "running"), make_agent("2", "running")];
        panel.orphan_sessions = vec![make_orphan("op-abc", false)];
        panel.state.select(Some(0)); // selecting first agent
        assert!(panel.selected_orphan().is_none());

        panel.state.select(Some(1)); // selecting second agent
        assert!(panel.selected_orphan().is_none());
    }

    #[test]
    fn test_selected_orphan_returns_orphan_past_separator() {
        let mut panel = InProgressPanel::new("In Progress".to_string());
        panel.agents = vec![make_agent("1", "running"), make_agent("2", "running")];
        panel.orphan_sessions = vec![make_orphan("op-abc", false), make_orphan("op-def", true)];

        // Index 2 = separator (agents.len() == 2), should return None
        panel.state.select(Some(2));
        assert!(panel.selected_orphan().is_none());

        // Index 3 = first orphan (2 agents + 1 separator = index 3)
        panel.state.select(Some(3));
        let orphan = panel.selected_orphan();
        assert!(orphan.is_some());
        assert_eq!(orphan.unwrap().session_name, "op-abc");

        // Index 4 = second orphan
        panel.state.select(Some(4));
        let orphan = panel.selected_orphan();
        assert!(orphan.is_some());
        assert_eq!(orphan.unwrap().session_name, "op-def");
        assert!(orphan.unwrap().attached);
    }
}
