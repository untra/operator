#![allow(dead_code)]

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::centered_rect;
use crate::config::LlmProvider;
use crate::queue::Ticket;

/// Selection state for the confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmSelection {
    Yes = 0,
    View = 1,
    No = 2,
}

impl ConfirmSelection {
    fn next(self) -> Self {
        match self {
            Self::Yes => Self::View,
            Self::View => Self::No,
            Self::No => Self::Yes,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Yes => Self::No,
            Self::View => Self::Yes,
            Self::No => Self::View,
        }
    }
}

/// Focus area in the confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmDialogFocus {
    /// Focused on the Yes/View/No buttons (default)
    Buttons,
    /// Focused on the options section (provider, project)
    Options,
}

/// Which option is currently selected in the options section
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedOption {
    Provider = 0,
    Project = 1,
}

impl SelectedOption {
    fn next(self) -> Self {
        match self {
            Self::Provider => Self::Project,
            Self::Project => Self::Provider,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Provider => Self::Project,
            Self::Project => Self::Provider,
        }
    }
}

pub struct ConfirmDialog {
    pub visible: bool,
    pub ticket: Option<Ticket>,
    pub selection: ConfirmSelection,

    // Focus and navigation state
    /// Current focus area (buttons or options)
    pub focus: ConfirmDialogFocus,
    /// Currently selected option when in Options focus
    pub selected_option: SelectedOption,

    // Launch options (only shown if config enables them)
    /// Available LLM providers (tool + model combinations)
    pub provider_options: Vec<LlmProvider>,
    /// Currently selected provider index
    pub selected_provider: usize,
    /// Whether docker mode option is available
    pub docker_enabled: bool,
    /// Whether docker mode is selected
    pub docker_selected: bool,
    /// Whether YOLO mode option is available
    pub yolo_enabled: bool,
    /// Whether YOLO mode is selected
    pub yolo_selected: bool,

    // Project override options
    /// Available projects for override (includes "global" option)
    pub project_options: Vec<String>,
    /// Currently selected project index (0 = ticket's original project)
    pub selected_project: usize,
    /// The original project from the ticket (for display reference)
    pub original_project: String,
}

impl ConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            ticket: None,
            selection: ConfirmSelection::Yes,
            focus: ConfirmDialogFocus::Buttons,
            selected_option: SelectedOption::Provider,
            provider_options: Vec::new(),
            selected_provider: 0,
            docker_enabled: false,
            docker_selected: false,
            yolo_enabled: false,
            yolo_selected: false,
            project_options: Vec::new(),
            selected_project: 0,
            original_project: String::new(),
        }
    }

    /// Configure the dialog with available options from config
    pub fn configure(
        &mut self,
        providers: Vec<LlmProvider>,
        projects: Vec<String>,
        docker_enabled: bool,
        yolo_enabled: bool,
    ) {
        self.provider_options = providers;
        self.project_options = projects;
        self.docker_enabled = docker_enabled;
        self.yolo_enabled = yolo_enabled;
    }

    pub fn show(&mut self, ticket: Ticket) {
        // Store original project from ticket
        self.original_project = ticket.project.clone();

        // Reset project selection to match ticket's project if it exists in options
        self.selected_project = self
            .project_options
            .iter()
            .position(|p| p == &ticket.project)
            .unwrap_or(0);

        self.ticket = Some(ticket);
        self.visible = true;
        self.selection = ConfirmSelection::Yes; // Default to Yes
        self.focus = ConfirmDialogFocus::Buttons; // Default to buttons
        self.selected_option = SelectedOption::Provider;
        // Reset mode selections but keep provider selection
        self.docker_selected = false;
        self.yolo_selected = false;
    }

    /// Cycle to the next provider
    pub fn cycle_provider(&mut self) {
        if !self.provider_options.is_empty() {
            self.selected_provider = (self.selected_provider + 1) % self.provider_options.len();
        }
    }

    /// Toggle docker mode
    pub fn toggle_docker(&mut self) {
        if self.docker_enabled {
            self.docker_selected = !self.docker_selected;
        }
    }

    /// Toggle YOLO mode
    pub fn toggle_yolo(&mut self) {
        if self.yolo_enabled {
            self.yolo_selected = !self.yolo_selected;
        }
    }

    /// Get the selected provider (if any)
    pub fn selected_provider(&self) -> Option<&LlmProvider> {
        self.provider_options.get(self.selected_provider)
    }

    /// Check if any options are available
    pub fn has_options(&self) -> bool {
        self.provider_options.len() > 1
            || self.project_options.len() > 1
            || self.docker_enabled
            || self.yolo_enabled
    }

    /// Cycle to the next project
    pub fn cycle_project(&mut self) {
        if !self.project_options.is_empty() {
            self.selected_project = (self.selected_project + 1) % self.project_options.len();
        }
    }

    /// Cycle to the previous project
    pub fn cycle_project_prev(&mut self) {
        if !self.project_options.is_empty() {
            if self.selected_project == 0 {
                self.selected_project = self.project_options.len() - 1;
            } else {
                self.selected_project -= 1;
            }
        }
    }

    /// Get the selected project name
    pub fn selected_project_name(&self) -> Option<&String> {
        self.project_options.get(self.selected_project)
    }

    /// Check if the project has been overridden from the original
    pub fn is_project_overridden(&self) -> bool {
        self.selected_project_name()
            .is_some_and(|p| p != &self.original_project)
    }

    /// Move focus to options section
    pub fn focus_options(&mut self) {
        if self.has_options() {
            self.focus = ConfirmDialogFocus::Options;
        }
    }

    /// Move focus to buttons section
    pub fn focus_buttons(&mut self) {
        self.focus = ConfirmDialogFocus::Buttons;
    }

    /// Check if focused on options
    pub fn is_options_focused(&self) -> bool {
        self.focus == ConfirmDialogFocus::Options
    }

    /// Navigate to next option (within options section)
    pub fn next_option(&mut self) {
        self.selected_option = self.selected_option.next();
    }

    /// Navigate to previous option (within options section)
    pub fn prev_option(&mut self) {
        self.selected_option = self.selected_option.prev();
    }

    /// Cycle the currently selected option's value
    pub fn cycle_current_option(&mut self) {
        match self.selected_option {
            SelectedOption::Provider => self.cycle_provider(),
            SelectedOption::Project => self.cycle_project(),
        }
    }

    /// Cycle the currently selected option's value in reverse
    pub fn cycle_current_option_prev(&mut self) {
        match self.selected_option {
            SelectedOption::Provider => {
                // Cycle provider in reverse
                if !self.provider_options.is_empty() {
                    if self.selected_provider == 0 {
                        self.selected_provider = self.provider_options.len() - 1;
                    } else {
                        self.selected_provider -= 1;
                    }
                }
            }
            SelectedOption::Project => self.cycle_project_prev(),
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.ticket = None;
    }

    pub fn select_next(&mut self) {
        self.selection = self.selection.next();
    }

    pub fn select_prev(&mut self) {
        self.selection = self.selection.prev();
    }

    /// Get the ticket filepath for viewing/editing
    pub fn ticket_filepath(&self) -> Option<String> {
        self.ticket.as_ref().map(|t| t.filepath.clone())
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let Some(ticket) = &self.ticket else {
            return;
        };

        // Check if priority should be shown (only if the schema has a priority field)
        let show_priority = ticket
            .template_schema()
            .map(|schema| schema.fields.iter().any(|f| f.name == "priority"))
            .unwrap_or(false);

        // Calculate dialog height based on options
        let has_options = self.has_options();
        let base_height = if has_options { 60 } else { 45 };
        let dialog_height = if show_priority {
            base_height
        } else {
            base_height - 2
        };

        // Center the dialog
        let area = centered_rect(60, dialog_height, frame.area());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Launch Agent? ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate options and priority heights
        let options_height = if has_options { 6 } else { 0 };
        let priority_height = if show_priority { 2 } else { 0 };

        // Dialog content layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),               // Type/ID
                Constraint::Min(3),                  // Summary
                Constraint::Length(2),               // Project
                Constraint::Length(priority_height), // Priority (conditional)
                Constraint::Length(options_height),  // Options (if any)
                Constraint::Length(3),               // Buttons
            ])
            .margin(1)
            .split(inner);

        // Type and ID
        let type_line = Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &ticket.ticket_type,
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(&ticket.id, Style::default().add_modifier(Modifier::BOLD)),
        ]);
        frame.render_widget(Paragraph::new(type_line), chunks[0]);

        // Summary (now second, after Type/ID)
        let summary = Paragraph::new(ticket.summary.clone())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        frame.render_widget(summary, chunks[1]);

        // Project
        let project_line = Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &ticket.project,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(project_line), chunks[2]);

        // Priority (only if schema has priority field)
        if show_priority {
            let priority_color = match ticket.priority.as_str() {
                "P0-critical" => Color::Red,
                "P1-high" => Color::Yellow,
                "P2-medium" => Color::White,
                _ => Color::Gray,
            };
            let priority_line = Line::from(vec![
                Span::styled("Priority: ", Style::default().fg(Color::Gray)),
                Span::styled(&ticket.priority, Style::default().fg(priority_color)),
            ]);
            frame.render_widget(Paragraph::new(priority_line), chunks[3]);
        }

        // Render options section if any are available
        if has_options {
            self.render_options(frame, chunks[4]);
        }

        // Dim buttons when options are focused
        let buttons_focused = self.focus == ConfirmDialogFocus::Buttons;

        // Buttons - Yes, View, No
        let yes_style = if self.selection == ConfirmSelection::Yes && buttons_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if buttons_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let view_style = if self.selection == ConfirmSelection::View && buttons_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else if buttons_focused {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let no_style = if self.selection == ConfirmSelection::No && buttons_focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else if buttons_focused {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let buttons = Line::from(vec![
            Span::raw("   "),
            Span::styled(" [Y]es ", yes_style),
            Span::raw("   "),
            Span::styled(" [V]iew ", view_style),
            Span::raw("   "),
            Span::styled(" [N]o ", no_style),
        ]);

        // Add hint for navigating to options
        let hint = if has_options && buttons_focused {
            Line::from(vec![Span::styled(
                "↑ to edit launch options",
                Style::default().fg(Color::DarkGray),
            )])
        } else if has_options {
            Line::from(vec![Span::styled(
                "↓ to confirm, ←/→ to change value",
                Style::default().fg(Color::DarkGray),
            )])
        } else {
            Line::from("")
        };

        let buttons_para = Paragraph::new(vec![buttons, hint]).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[5]);
    }

    /// Render the options section (provider, project, docker, yolo toggles)
    fn render_options(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        let options_focused = self.focus == ConfirmDialogFocus::Options;

        // Separator line with focus indicator
        let separator_style = if options_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![Span::styled(
            "── Launch Options ──",
            separator_style,
        )]));

        // Provider option (if multiple providers)
        if self.provider_options.len() > 1 {
            let is_selected = options_focused && self.selected_option == SelectedOption::Provider;
            let provider_display = self
                .selected_provider()
                .map(|p| {
                    p.display_name
                        .clone()
                        .unwrap_or_else(|| format!("{} {}", p.tool, p.model))
                })
                .unwrap_or_else(|| "Default".to_string());

            let prefix = if is_selected { "▶ " } else { "  " };
            let value_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled("Provider: ", Style::default().fg(Color::Gray)),
                Span::styled(provider_display, value_style),
            ]));
        }

        // Project option (if multiple projects)
        if self.project_options.len() > 1 {
            let is_selected = options_focused && self.selected_option == SelectedOption::Project;
            let project_display = self
                .selected_project_name()
                .cloned()
                .unwrap_or_else(|| self.original_project.clone());

            let prefix = if is_selected { "▶ " } else { "  " };

            // Show if project is overridden from original
            let (value_style, suffix) = if self.is_project_overridden() {
                (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    " (override)",
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    "",
                )
            } else {
                (Style::default().fg(Color::White), "")
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled("Project:  ", Style::default().fg(Color::Gray)),
                Span::styled(project_display, value_style),
                Span::styled(suffix, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // Docker option
        if self.docker_enabled {
            let (indicator, color) = if self.docker_selected {
                ("●", Color::Green)
            } else {
                ("○", Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("[D] ", Style::default().fg(Color::Yellow)),
                Span::styled("Docker: ", Style::default().fg(Color::Gray)),
                Span::styled(indicator, Style::default().fg(color)),
                Span::styled(
                    if self.docker_selected { " On" } else { " Off" },
                    Style::default().fg(color),
                ),
            ]));
        }

        // YOLO option (use 'A' for Auto-accept to avoid conflict with 'Y' for Yes)
        if self.yolo_enabled {
            let (indicator, color) = if self.yolo_selected {
                ("●", Color::Red)
            } else {
                ("○", Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("[A] ", Style::default().fg(Color::Yellow)),
                Span::styled("Auto:   ", Style::default().fg(Color::Gray)),
                Span::styled(indicator, Style::default().fg(color)),
                Span::styled(
                    if self.yolo_selected {
                        " On (YOLO mode)"
                    } else {
                        " Off"
                    },
                    Style::default().fg(color),
                ),
            ]));
        }

        let options = Paragraph::new(lines);
        frame.render_widget(options, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_ticket(project: &str) -> Ticket {
        Ticket {
            filename: format!("20241225-1200-TASK-{}-test.md", project),
            filepath: format!("/tmp/tickets/queue/20241225-1200-TASK-{}-test.md", project),
            timestamp: "20241225-1200".to_string(),
            ticket_type: "TASK".to_string(),
            project: project.to_string(),
            id: "TASK-1234".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "queued".to_string(),
            step: String::new(),
            content: "Test content".to_string(),
            sessions: HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    // ConfirmSelection tests
    #[test]
    fn test_confirm_selection_next_cycles_correctly() {
        assert_eq!(ConfirmSelection::Yes.next(), ConfirmSelection::View);
        assert_eq!(ConfirmSelection::View.next(), ConfirmSelection::No);
        assert_eq!(ConfirmSelection::No.next(), ConfirmSelection::Yes);
    }

    #[test]
    fn test_confirm_selection_prev_cycles_correctly() {
        assert_eq!(ConfirmSelection::Yes.prev(), ConfirmSelection::No);
        assert_eq!(ConfirmSelection::View.prev(), ConfirmSelection::Yes);
        assert_eq!(ConfirmSelection::No.prev(), ConfirmSelection::View);
    }

    // SelectedOption tests
    #[test]
    fn test_selected_option_next_cycles_correctly() {
        assert_eq!(SelectedOption::Provider.next(), SelectedOption::Project);
        assert_eq!(SelectedOption::Project.next(), SelectedOption::Provider);
    }

    #[test]
    fn test_selected_option_prev_cycles_correctly() {
        assert_eq!(SelectedOption::Provider.prev(), SelectedOption::Project);
        assert_eq!(SelectedOption::Project.prev(), SelectedOption::Provider);
    }

    // ConfirmDialog tests
    #[test]
    fn test_confirm_dialog_new_initializes_correctly() {
        let dialog = ConfirmDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.ticket.is_none());
        assert_eq!(dialog.selection, ConfirmSelection::Yes);
        assert_eq!(dialog.focus, ConfirmDialogFocus::Buttons);
        assert!(dialog.provider_options.is_empty());
        assert!(!dialog.docker_selected);
        assert!(!dialog.yolo_selected);
    }

    #[test]
    fn test_confirm_dialog_configure_sets_options() {
        let mut dialog = ConfirmDialog::new();
        let providers = vec![
            LlmProvider {
                tool: "claude".to_string(),
                model: "opus".to_string(),
                display_name: Some("Claude Opus".to_string()),
                ..Default::default()
            },
            LlmProvider {
                tool: "claude".to_string(),
                model: "sonnet".to_string(),
                display_name: None,
                ..Default::default()
            },
        ];
        let projects = vec!["project-a".to_string(), "project-b".to_string()];

        dialog.configure(providers.clone(), projects.clone(), true, false);

        assert_eq!(dialog.provider_options.len(), 2);
        assert_eq!(dialog.project_options.len(), 2);
        assert!(dialog.docker_enabled);
        assert!(!dialog.yolo_enabled);
    }

    #[test]
    fn test_confirm_dialog_show_sets_ticket_and_resets_state() {
        let mut dialog = ConfirmDialog::new();
        dialog.configure(
            vec![],
            vec!["project-a".to_string(), "project-b".to_string()],
            false,
            false,
        );
        dialog.selection = ConfirmSelection::No;
        dialog.docker_selected = true;

        let ticket = make_test_ticket("project-b");
        dialog.show(ticket);

        assert!(dialog.visible);
        assert!(dialog.ticket.is_some());
        assert_eq!(dialog.selection, ConfirmSelection::Yes);
        assert_eq!(dialog.selected_project, 1); // project-b is at index 1
        assert!(!dialog.docker_selected); // Reset
    }

    #[test]
    fn test_confirm_dialog_cycle_provider() {
        let mut dialog = ConfirmDialog::new();
        dialog.provider_options = vec![
            LlmProvider {
                tool: "a".to_string(),
                model: "1".to_string(),
                display_name: None,
                ..Default::default()
            },
            LlmProvider {
                tool: "b".to_string(),
                model: "2".to_string(),
                display_name: None,
                ..Default::default()
            },
            LlmProvider {
                tool: "c".to_string(),
                model: "3".to_string(),
                display_name: None,
                ..Default::default()
            },
        ];

        assert_eq!(dialog.selected_provider, 0);
        dialog.cycle_provider();
        assert_eq!(dialog.selected_provider, 1);
        dialog.cycle_provider();
        assert_eq!(dialog.selected_provider, 2);
        dialog.cycle_provider();
        assert_eq!(dialog.selected_provider, 0); // Wraps
    }

    #[test]
    fn test_confirm_dialog_toggle_docker_respects_enabled() {
        let mut dialog = ConfirmDialog::new();
        dialog.docker_enabled = false;

        dialog.toggle_docker();
        assert!(!dialog.docker_selected); // No-op when disabled

        dialog.docker_enabled = true;
        dialog.toggle_docker();
        assert!(dialog.docker_selected);
        dialog.toggle_docker();
        assert!(!dialog.docker_selected);
    }

    #[test]
    fn test_confirm_dialog_toggle_yolo_respects_enabled() {
        let mut dialog = ConfirmDialog::new();
        dialog.yolo_enabled = false;

        dialog.toggle_yolo();
        assert!(!dialog.yolo_selected); // No-op when disabled

        dialog.yolo_enabled = true;
        dialog.toggle_yolo();
        assert!(dialog.yolo_selected);
    }

    #[test]
    fn test_confirm_dialog_cycle_project() {
        let mut dialog = ConfirmDialog::new();
        dialog.project_options = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        dialog.cycle_project();
        assert_eq!(dialog.selected_project, 1);
        dialog.cycle_project();
        assert_eq!(dialog.selected_project, 2);
        dialog.cycle_project();
        assert_eq!(dialog.selected_project, 0); // Wraps
    }

    #[test]
    fn test_confirm_dialog_cycle_project_prev() {
        let mut dialog = ConfirmDialog::new();
        dialog.project_options = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        dialog.cycle_project_prev();
        assert_eq!(dialog.selected_project, 2); // Wraps to end
        dialog.cycle_project_prev();
        assert_eq!(dialog.selected_project, 1);
    }

    #[test]
    fn test_confirm_dialog_is_project_overridden() {
        let mut dialog = ConfirmDialog::new();
        dialog.project_options = vec!["a".to_string(), "b".to_string()];
        dialog.original_project = "a".to_string();
        dialog.selected_project = 0;

        assert!(!dialog.is_project_overridden());

        dialog.selected_project = 1;
        assert!(dialog.is_project_overridden());
    }

    #[test]
    fn test_confirm_dialog_has_options() {
        let mut dialog = ConfirmDialog::new();
        assert!(!dialog.has_options());

        dialog.docker_enabled = true;
        assert!(dialog.has_options());

        dialog.docker_enabled = false;
        dialog.yolo_enabled = true;
        assert!(dialog.has_options());

        dialog.yolo_enabled = false;
        dialog.provider_options = vec![
            LlmProvider {
                tool: "a".to_string(),
                model: "1".to_string(),
                display_name: None,
                ..Default::default()
            },
            LlmProvider {
                tool: "b".to_string(),
                model: "2".to_string(),
                display_name: None,
                ..Default::default()
            },
        ];
        assert!(dialog.has_options());
    }

    #[test]
    fn test_confirm_dialog_focus_management() {
        let mut dialog = ConfirmDialog::new();
        dialog.docker_enabled = true; // Enable options

        assert!(!dialog.is_options_focused());
        dialog.focus_options();
        assert!(dialog.is_options_focused());
        dialog.focus_buttons();
        assert!(!dialog.is_options_focused());
    }

    #[test]
    fn test_confirm_dialog_option_navigation() {
        let mut dialog = ConfirmDialog::new();
        assert_eq!(dialog.selected_option, SelectedOption::Provider);

        dialog.next_option();
        assert_eq!(dialog.selected_option, SelectedOption::Project);

        dialog.prev_option();
        assert_eq!(dialog.selected_option, SelectedOption::Provider);
    }

    #[test]
    fn test_confirm_dialog_select_next_prev() {
        let mut dialog = ConfirmDialog::new();
        assert_eq!(dialog.selection, ConfirmSelection::Yes);

        dialog.select_next();
        assert_eq!(dialog.selection, ConfirmSelection::View);

        dialog.select_prev();
        assert_eq!(dialog.selection, ConfirmSelection::Yes);
    }

    #[test]
    fn test_confirm_dialog_hide_clears_state() {
        let mut dialog = ConfirmDialog::new();
        dialog.visible = true;
        dialog.ticket = Some(make_test_ticket("test"));

        dialog.hide();

        assert!(!dialog.visible);
        assert!(dialog.ticket.is_none());
    }
}
