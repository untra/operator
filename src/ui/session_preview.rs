#![allow(unused_variables)]

//! Session preview dialog for viewing agent tmux session content.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::state::AgentState;

/// Dialog for previewing agent session content
pub struct SessionPreview {
    /// Whether the preview is visible
    pub visible: bool,
    /// The agent being previewed
    pub agent: Option<AgentState>,
    /// The captured session content
    pub content: String,
    /// Current scroll offset
    pub scroll: u16,
    /// Total lines in content
    pub total_lines: u16,
    /// Error message if content couldn't be captured
    pub error: Option<String>,
}

impl SessionPreview {
    pub fn new() -> Self {
        Self {
            visible: false,
            agent: None,
            content: String::new(),
            scroll: 0,
            total_lines: 0,
            error: None,
        }
    }

    /// Show the preview for an agent
    pub fn show(&mut self, agent: &AgentState, content: Result<String, String>) {
        self.agent = Some(agent.clone());
        self.visible = true;
        self.scroll = 0;

        match content {
            Ok(c) => {
                self.total_lines = c.lines().count() as u16;
                self.content = c;
                self.error = None;
            }
            Err(e) => {
                self.content = String::new();
                self.total_lines = 0;
                self.error = Some(e);
            }
        }
    }

    /// Hide the preview
    pub fn hide(&mut self) {
        self.visible = false;
        self.agent = None;
        self.content.clear();
        self.error = None;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, viewport_height: u16) {
        let max_scroll = self.total_lines.saturating_sub(viewport_height);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    /// Page up
    pub fn page_up(&mut self, viewport_height: u16) {
        self.scroll = self.scroll.saturating_sub(viewport_height);
    }

    /// Page down
    pub fn page_down(&mut self, viewport_height: u16) {
        let max_scroll = self.total_lines.saturating_sub(viewport_height);
        self.scroll = (self.scroll + viewport_height).min(max_scroll);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, viewport_height: u16) {
        self.scroll = self.total_lines.saturating_sub(viewport_height);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = frame.area();

        // Create a centered popup that takes up 80% of the screen
        let popup_area = centered_rect(80, 80, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        // Get agent info for title
        let title = match &self.agent {
            Some(a) => format!(
                " Session Preview: {} ({}-{}) ",
                a.project, a.ticket_type, a.ticket_id
            ),
            None => " Session Preview ".to_string(),
        };

        // Layout: header, content, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header info
                Constraint::Min(5),    // Content
                Constraint::Length(2), // Footer/help
            ])
            .split(popup_area);

        // Outer block
        let outer_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = outer_block.inner(popup_area);
        frame.render_widget(outer_block, popup_area);

        // Re-split the inner area
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status line
                Constraint::Min(5),    // Content
                Constraint::Length(1), // Help line
            ])
            .split(inner_area);

        // Status line
        let status_text = if let Some(a) = &self.agent {
            if let Some(session) = &a.session_name {
                format!(
                    "Session: {} | Status: {} | Scroll: {}/{}",
                    session,
                    a.status,
                    self.scroll + 1,
                    self.total_lines.max(1)
                )
            } else {
                format!("Status: {} | No session attached", a.status)
            }
        } else {
            "No agent selected".to_string()
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left);
        frame.render_widget(status, inner_chunks[0]);

        // Content area
        if let Some(ref err) = self.error {
            let error_text = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Failed to capture session content:",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(err.as_str(), Style::default().fg(Color::Red))),
            ])
            .block(Block::default().borders(Borders::TOP))
            .alignment(Alignment::Center);
            frame.render_widget(error_text, inner_chunks[1]);
        } else if self.content.is_empty() {
            let empty_text = Paragraph::new("(Session content is empty)")
                .style(
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, inner_chunks[1]);
        } else {
            // Parse content into lines, applying scroll offset
            let lines: Vec<Line> = self
                .content
                .lines()
                .skip(self.scroll as usize)
                .take(inner_chunks[1].height as usize)
                .map(|line| Line::from(line.to_string()))
                .collect();

            let content = Paragraph::new(lines)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: false });
            frame.render_widget(content, inner_chunks[1]);

            // Scrollbar
            if self.total_lines > inner_chunks[1].height {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));

                let mut scrollbar_state =
                    ScrollbarState::new(self.total_lines as usize).position(self.scroll as usize);

                frame.render_stateful_widget(scrollbar, inner_chunks[1], &mut scrollbar_state);
            }
        }

        // Help line
        let help_text = "↑/↓ or j/k: Scroll | PgUp/PgDn: Page | g/G: Top/Bottom | Esc/q: Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, inner_chunks[2]);
    }
}

impl Default for SessionPreview {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
