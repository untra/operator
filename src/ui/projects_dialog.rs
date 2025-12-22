//! Dialog for project maintenance actions

use std::path::PathBuf;

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::agents::AgentTicketResult;

/// Which step of the projects dialog we're on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectsDialogStep {
    /// Selecting a project
    SelectProject,
    /// Selecting an action
    SelectAction,
    /// Showing result
    Result,
}

/// Available project actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAction {
    AddOperatorAgents,
}

impl ProjectAction {
    /// Returns all available actions
    pub fn all() -> &'static [ProjectAction] {
        &[ProjectAction::AddOperatorAgents]
    }

    /// Returns the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            ProjectAction::AddOperatorAgents => "Add Operator agents",
        }
    }

    /// Returns a description of this action
    pub fn description(&self) -> &'static str {
        match self {
            ProjectAction::AddOperatorAgents => "Create TASK tickets for missing agent files",
        }
    }
}

/// Result of confirming the dialog
#[derive(Debug, Clone)]
pub struct ProjectsDialogResult {
    pub project: String,
    pub project_path: PathBuf,
    pub action: ProjectAction,
}

/// State for agent ticket creation
#[derive(Debug, Clone)]
pub enum TicketCreationState {
    /// Not started
    Idle,
    /// Successfully created tickets
    Success(AgentTicketResult),
    /// Failed to create tickets
    Error { message: String },
}

/// Dialog for project maintenance actions
pub struct ProjectsDialog {
    /// Whether the dialog is visible
    pub visible: bool,
    /// Current step in the dialog
    pub step: ProjectsDialogStep,
    /// List selection state for projects
    pub project_state: ListState,
    /// List selection state for actions
    pub action_state: ListState,
    /// Available projects
    pub projects: Vec<String>,
    /// Projects base path
    projects_path: PathBuf,
    /// Selected project name
    selected_project: Option<String>,
    /// Selected action
    selected_action: Option<ProjectAction>,
    /// Ticket creation state
    creation_state: TicketCreationState,
}

impl ProjectsDialog {
    /// Create a new projects dialog
    pub fn new() -> Self {
        Self {
            visible: false,
            step: ProjectsDialogStep::SelectProject,
            project_state: ListState::default(),
            action_state: ListState::default(),
            projects: Vec::new(),
            projects_path: PathBuf::from("."),
            selected_project: None,
            selected_action: None,
            creation_state: TicketCreationState::Idle,
        }
    }

    /// Update the list of available projects
    pub fn set_projects(&mut self, projects: Vec<String>) {
        self.projects = projects;
    }

    /// Set the projects base path
    pub fn set_projects_path(&mut self, path: PathBuf) {
        self.projects_path = path;
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.visible = true;
        self.step = ProjectsDialogStep::SelectProject;
        self.project_state.select(if self.projects.is_empty() {
            None
        } else {
            Some(0)
        });
        self.action_state.select(Some(0));
        self.selected_project = None;
        self.selected_action = None;
        self.creation_state = TicketCreationState::Idle;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.step = ProjectsDialogStep::SelectProject;
        self.selected_project = None;
        self.selected_action = None;
        self.creation_state = TicketCreationState::Idle;
    }

    /// Go back to previous step or hide if on first step
    fn go_back(&mut self) {
        match self.step {
            ProjectsDialogStep::SelectProject => {
                self.hide();
            }
            ProjectsDialogStep::SelectAction => {
                self.step = ProjectsDialogStep::SelectProject;
                self.selected_project = None;
            }
            ProjectsDialogStep::Result => {
                self.hide();
            }
        }
    }

    /// Set the result of ticket creation and transition to result step
    pub fn set_creation_result(&mut self, result: Result<AgentTicketResult, String>) {
        match result {
            Ok(ticket_result) => {
                self.creation_state = TicketCreationState::Success(ticket_result);
            }
            Err(message) => {
                self.creation_state = TicketCreationState::Error { message };
            }
        }
        self.step = ProjectsDialogStep::Result;
    }

    /// Handle key event, returns result when action should be executed
    pub fn handle_key(&mut self, key: KeyCode) -> Option<ProjectsDialogResult> {
        match self.step {
            ProjectsDialogStep::SelectProject => self.handle_project_key(key),
            ProjectsDialogStep::SelectAction => self.handle_action_key(key),
            ProjectsDialogStep::Result => self.handle_result_key(key),
        }
    }

    fn handle_project_key(&mut self, key: KeyCode) -> Option<ProjectsDialogResult> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.projects.is_empty() {
                    let i = match self.project_state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => self.projects.len().saturating_sub(1),
                    };
                    self.project_state.select(Some(i));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.projects.is_empty() {
                    let i = match self.project_state.selected() {
                        Some(i) if i < self.projects.len() - 1 => i + 1,
                        _ => 0,
                    };
                    self.project_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.project_state.selected() {
                    self.selected_project = Some(self.projects[i].clone());
                    self.step = ProjectsDialogStep::SelectAction;
                    self.action_state.select(Some(0));
                }
            }
            KeyCode::Esc => {
                self.go_back();
            }
            _ => {}
        }
        None
    }

    fn handle_action_key(&mut self, key: KeyCode) -> Option<ProjectsDialogResult> {
        let actions = ProjectAction::all();
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if !actions.is_empty() {
                    let i = match self.action_state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => actions.len().saturating_sub(1),
                    };
                    self.action_state.select(Some(i));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !actions.is_empty() {
                    let i = match self.action_state.selected() {
                        Some(i) if i < actions.len() - 1 => i + 1,
                        _ => 0,
                    };
                    self.action_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.action_state.selected() {
                    self.selected_action = Some(actions[i]);

                    // Return result so app can execute the action
                    let project = self.selected_project.clone()?;
                    let project_path = self.projects_path.join(&project);
                    return Some(ProjectsDialogResult {
                        project,
                        project_path,
                        action: actions[i],
                    });
                }
            }
            KeyCode::Esc => {
                self.go_back();
            }
            _ => {}
        }
        None
    }

    fn handle_result_key(&mut self, key: KeyCode) -> Option<ProjectsDialogResult> {
        match key {
            KeyCode::Enter | KeyCode::Esc => {
                self.hide();
            }
            _ => {}
        }
        None
    }

    /// Render the dialog
    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        match self.step {
            ProjectsDialogStep::SelectProject => self.render_project_step(frame),
            ProjectsDialogStep::SelectAction => self.render_action_step(frame),
            ProjectsDialogStep::Result => self.render_result_step(frame),
        }
    }

    fn render_project_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(50, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Projects ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Project list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            "Select a project:",
            Style::default().fg(Color::Gray),
        )]));
        frame.render_widget(instructions, chunks[0]);

        // Project list
        if self.projects.is_empty() {
            let empty_msg = Paragraph::new(Line::from(vec![Span::styled(
                "No projects discovered (no CLAUDE.md files found)",
                Style::default().fg(Color::DarkGray),
            )]));
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .projects
                .iter()
                .map(|p| ListItem::new(Line::from(vec![Span::raw("  "), Span::raw(p)])))
                .collect();

            let list = List::new(items)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.project_state);
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn render_action_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(50, 50, frame.area());
        frame.render_widget(Clear, area);

        let title = match &self.selected_project {
            Some(p) => format!(" {} - Actions ", p),
            None => " Actions ".to_string(),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Min(6),    // Action list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            "Select an action:",
            Style::default().fg(Color::Gray),
        )]));
        frame.render_widget(instructions, chunks[0]);

        // Action list
        let actions = ProjectAction::all();
        let items: Vec<ListItem> = actions
            .iter()
            .map(|a| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(a.label(), Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(a.description(), Style::default().fg(Color::Gray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.action_state);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" execute  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn render_result_step(&self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let (title, border_color) = match &self.creation_state {
            TicketCreationState::Success(result) if result.errors.is_empty() => {
                if result.created.is_empty() && result.skipped.is_empty() {
                    (" No Changes ", Color::Gray)
                } else if result.created.is_empty() {
                    (" All Agents Exist ", Color::Gray)
                } else {
                    (" Tasks Created ", Color::Green)
                }
            }
            TicketCreationState::Success(_) => (" Partial Success ", Color::Yellow),
            TicketCreationState::Error { .. } => (" Error ", Color::Red),
            TicketCreationState::Idle => (" Result ", Color::Gray),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(6),    // Content
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Content based on state
        match &self.creation_state {
            TicketCreationState::Success(result) => {
                let mut lines = vec![];

                // Show created tickets
                if !result.created.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        format!("Created {} TASK tickets:", result.created.len()),
                        Style::default().fg(Color::Green),
                    )]));

                    for ticket_id in &result.created {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(
                                format!("> {}", ticket_id),
                                Style::default().fg(Color::Cyan),
                            ),
                        ]));
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![Span::styled(
                        "Launch these tasks to generate agent files.",
                        Style::default().fg(Color::Gray),
                    )]));
                }

                // Show skipped agents
                if !result.skipped.is_empty() {
                    if !result.created.is_empty() {
                        lines.push(Line::from("")); // Spacer
                    }
                    lines.push(Line::from(vec![Span::styled(
                        format!("Skipped {} (already exist):", result.skipped.len()),
                        Style::default().fg(Color::DarkGray),
                    )]));

                    for key in &result.skipped {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(
                                format!("- {}-operator.md", key.to_lowercase()),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]));
                    }
                }

                // Show errors if any
                if !result.errors.is_empty() {
                    lines.push(Line::from("")); // Spacer
                    lines.push(Line::from(vec![Span::styled(
                        format!("Failed {}:", result.errors.len()),
                        Style::default().fg(Color::Yellow),
                    )]));

                    for (key, error) in &result.errors {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("x {}: ", key), Style::default().fg(Color::Red)),
                            Span::styled(
                                truncate_str(error, 40),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]));
                    }
                }

                // Empty state
                if result.created.is_empty()
                    && result.skipped.is_empty()
                    && result.errors.is_empty()
                {
                    lines.push(Line::from(vec![Span::styled(
                        "No agent templates found with agent_prompt defined.",
                        Style::default().fg(Color::DarkGray),
                    )]));
                }

                let content = Paragraph::new(lines);
                frame.render_widget(content, chunks[0]);
            }
            TicketCreationState::Error { message } => {
                let content = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "Failed to create tickets:",
                        Style::default().fg(Color::Red),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        message.as_str(),
                        Style::default().fg(Color::White),
                    )]),
                ])
                .wrap(Wrap { trim: true });
                frame.render_widget(content, chunks[0]);
            }
            TicketCreationState::Idle => {}
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" or "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" to close"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[1]);
    }
}

impl Default for ProjectsDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a centered rect
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

/// Truncate a string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
