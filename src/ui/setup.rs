//! Startup setup screen when .tickets/ directory is not found

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Available issuetype collections
pub const DEFAULT_COLLECTION: &[&str] = &["SPIKE", "INV", "FEAT", "FIX", "TASK"];

/// Collection source options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSourceOption {
    Simple,
    KanbanDefault,
    ImportJira,
    ImportNotion,
    CustomSelection,
}

impl CollectionSourceOption {
    pub fn all() -> &'static [CollectionSourceOption] {
        &[
            CollectionSourceOption::Simple,
            CollectionSourceOption::KanbanDefault,
            CollectionSourceOption::ImportJira,
            CollectionSourceOption::ImportNotion,
            CollectionSourceOption::CustomSelection,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Simple",
            CollectionSourceOption::KanbanDefault => "Kanban Default",
            CollectionSourceOption::ImportJira => "Import from Jira",
            CollectionSourceOption::ImportNotion => "Import from Notion",
            CollectionSourceOption::CustomSelection => "Custom Selection",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Just TASK - minimal setup for general work",
            CollectionSourceOption::KanbanDefault => "5 issue types: SPIKE, INV, FEAT, FIX, TASK",
            CollectionSourceOption::ImportJira => "(Coming soon)",
            CollectionSourceOption::ImportNotion => "(Coming soon)",
            CollectionSourceOption::CustomSelection => "Choose individual issue types",
        }
    }

    pub fn is_unimplemented(&self) -> bool {
        matches!(
            self,
            CollectionSourceOption::ImportJira | CollectionSourceOption::ImportNotion
        )
    }
}

/// Result of setup screen actions
#[derive(Debug, Clone)]
pub enum SetupResult {
    /// Continue to next step
    Continue,
    /// Cancel/quit setup
    Cancel,
    /// Exit with unimplemented message
    ExitUnimplemented(String),
    /// Setup complete, initialize
    Initialize,
}

/// Setup screen shown when .tickets/ directory doesn't exist
pub struct SetupScreen {
    /// Whether the screen is visible
    pub visible: bool,
    /// Current step in the setup process
    pub step: SetupStep,
    /// Current selection for confirmation: true = Initialize, false = Cancel
    pub confirm_selected: bool,
    /// Path where tickets directory will be created
    tickets_path: String,
    /// Discovered projects (directories with CLAUDE.md files)
    discovered_projects: Vec<String>,
    /// Selected issuetype collection
    pub selected_collection: Vec<String>,
    /// List state for collection source selection
    source_state: ListState,
    /// List state for custom collection selection
    collection_state: ListState,
    /// Whether we came from custom selection (for back navigation)
    from_custom: bool,
}

/// Steps in the setup process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStep {
    /// Welcome splash screen with discovered projects
    Welcome,
    /// Select template collection source
    CollectionSource,
    /// Custom issuetype selection (optional)
    CustomCollection,
    /// Confirm initialization
    Confirm,
}

impl SetupScreen {
    /// Create a new setup screen
    pub fn new(tickets_path: String, discovered_projects: Vec<String>) -> Self {
        let mut source_state = ListState::default();
        source_state.select(Some(0));

        let mut collection_state = ListState::default();
        collection_state.select(Some(0));

        Self {
            visible: true,
            step: SetupStep::Welcome,
            confirm_selected: true, // Default to Initialize
            tickets_path,
            discovered_projects,
            selected_collection: DEFAULT_COLLECTION.iter().map(|s| s.to_string()).collect(),
            source_state,
            collection_state,
            from_custom: false,
        }
    }

    /// Get the selected issuetype collection
    pub fn collection(&self) -> Vec<String> {
        self.selected_collection.clone()
    }

    /// Get the currently selected source option
    fn selected_source(&self) -> Option<CollectionSourceOption> {
        self.source_state
            .selected()
            .map(|i| CollectionSourceOption::all()[i])
    }

    /// Toggle selection (Space key)
    pub fn toggle_selection(&mut self) {
        match self.step {
            SetupStep::CustomCollection => {
                // Toggle the currently highlighted collection item
                if let Some(i) = self.collection_state.selected() {
                    let types = DEFAULT_COLLECTION;
                    if i < types.len() {
                        let type_str = types[i].to_string();
                        if self.selected_collection.contains(&type_str) {
                            self.selected_collection.retain(|t| t != &type_str);
                        } else {
                            self.selected_collection.push(type_str);
                        }
                    }
                }
            }
            SetupStep::Confirm => {
                self.confirm_selected = !self.confirm_selected;
            }
            _ => {}
        }
    }

    /// Move to next item in list
    pub fn select_next(&mut self) {
        match self.step {
            SetupStep::CollectionSource => {
                let len = CollectionSourceOption::all().len();
                let i = self.source_state.selected().map_or(0, |i| (i + 1) % len);
                self.source_state.select(Some(i));
            }
            SetupStep::CustomCollection => {
                let len = DEFAULT_COLLECTION.len();
                let i = self
                    .collection_state
                    .selected()
                    .map_or(0, |i| (i + 1) % len);
                self.collection_state.select(Some(i));
            }
            _ => {}
        }
    }

    /// Move to previous item in list
    pub fn select_prev(&mut self) {
        match self.step {
            SetupStep::CollectionSource => {
                let len = CollectionSourceOption::all().len();
                let i =
                    self.source_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.source_state.select(Some(i));
            }
            SetupStep::CustomCollection => {
                let len = DEFAULT_COLLECTION.len();
                let i = self.collection_state.selected().map_or(0, |i| {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                });
                self.collection_state.select(Some(i));
            }
            _ => {}
        }
    }

    /// Proceed to next step or confirm (Enter key)
    pub fn confirm(&mut self) -> SetupResult {
        match self.step {
            SetupStep::Welcome => {
                self.step = SetupStep::CollectionSource;
                SetupResult::Continue
            }
            SetupStep::CollectionSource => {
                if let Some(source) = self.selected_source() {
                    match source {
                        CollectionSourceOption::Simple => {
                            // Just TASK for minimal setup
                            self.selected_collection = vec!["TASK".to_string()];
                            self.from_custom = false;
                            self.step = SetupStep::Confirm;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::KanbanDefault => {
                            // Auto-select all 5 types and skip to Confirm
                            self.selected_collection =
                                DEFAULT_COLLECTION.iter().map(|s| s.to_string()).collect();
                            self.from_custom = false;
                            self.step = SetupStep::Confirm;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::ImportJira => SetupResult::ExitUnimplemented(
                            "Jira import is not yet implemented".to_string(),
                        ),
                        CollectionSourceOption::ImportNotion => SetupResult::ExitUnimplemented(
                            "Notion import is not yet implemented".to_string(),
                        ),
                        CollectionSourceOption::CustomSelection => {
                            self.from_custom = true;
                            self.step = SetupStep::CustomCollection;
                            SetupResult::Continue
                        }
                    }
                } else {
                    SetupResult::Continue
                }
            }
            SetupStep::CustomCollection => {
                if !self.selected_collection.is_empty() {
                    self.step = SetupStep::Confirm;
                }
                SetupResult::Continue
            }
            SetupStep::Confirm => {
                if self.confirm_selected {
                    SetupResult::Initialize
                } else {
                    SetupResult::Cancel
                }
            }
        }
    }

    /// Go back to previous step (Esc key)
    pub fn go_back(&mut self) -> SetupResult {
        match self.step {
            SetupStep::Welcome => SetupResult::Cancel,
            SetupStep::CollectionSource => {
                self.step = SetupStep::Welcome;
                SetupResult::Continue
            }
            SetupStep::CustomCollection => {
                self.step = SetupStep::CollectionSource;
                SetupResult::Continue
            }
            SetupStep::Confirm => {
                if self.from_custom {
                    self.step = SetupStep::CustomCollection;
                } else {
                    self.step = SetupStep::CollectionSource;
                }
                SetupResult::Continue
            }
        }
    }

    /// Render the setup screen
    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        match self.step {
            SetupStep::Welcome => self.render_welcome_step(frame),
            SetupStep::CollectionSource => self.render_collection_source_step(frame),
            SetupStep::CustomCollection => self.render_custom_collection_step(frame),
            SetupStep::Confirm => self.render_confirm_step(frame),
        }
    }

    fn render_welcome_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
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
                Span::raw(" Workspace Setup "),
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
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Description
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Path info
                Constraint::Length(2), // Spacer
                Constraint::Min(6),    // Discovered projects
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Operator!",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![
            Line::from("A TUI for orchestrating Claude Code agents."),
            Line::from(""),
            Line::from("The ticket queue directory was not found."),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[4]);

        // Discovered projects
        let projects_text = if self.discovered_projects.is_empty() {
            vec![
                Line::from(Span::styled(
                    "Discovered projects:",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(Span::styled(
                    "  (none - no directories with CLAUDE.md found)",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            let mut lines = vec![Line::from(Span::styled(
                "Discovered projects:",
                Style::default().fg(Color::Yellow),
            ))];
            let project_list = self.discovered_projects.join(", ");
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(project_list, Style::default().fg(Color::Green)),
            ]));
            lines
        };
        frame.render_widget(Paragraph::new(projects_text), chunks[6]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[7]);
    }

    fn render_collection_source_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Select Template Collection ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Options list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Choose Template Source",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions =
            Paragraph::new(vec![Line::from("Use arrows to navigate, Enter to select")])
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Options list
        let items: Vec<ListItem> = CollectionSourceOption::all()
            .iter()
            .map(|opt| {
                let style = if opt.is_unimplemented() {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        opt.label(),
                        style.add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(opt.description(), Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[2], &mut self.source_state);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    fn render_custom_collection_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.area());
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
                Span::raw(" Setup - Issue Types "),
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
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Collection list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Select Issue Types",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[1]);

        // Collection list
        let items: Vec<ListItem> = DEFAULT_COLLECTION
            .iter()
            .map(|t| {
                let is_selected = self.selected_collection.contains(&t.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                let description = match *t {
                    "SPIKE" => "Research or exploration (paired mode)",
                    "INV" => "Incident investigation (paired mode)",
                    "FEAT" => "New feature or enhancement",
                    "FIX" => "Bug fix, follow-up work, tech debt",
                    "TASK" => "Neutral task that outputs a plan",
                    _ => "",
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            checkbox,
                            Style::default().fg(if is_selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            *t,
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
                        Span::styled(description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[2], &mut self.collection_state);

        // Footer
        let selected_count = self.selected_collection.len();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} selected", selected_count),
                Style::default().fg(if selected_count > 0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    fn render_confirm_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Confirm Setup ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Description
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Path info
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Selected collection
                Constraint::Min(4),    // What will be created
                Constraint::Length(3), // Buttons
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Ready to Initialize",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Description
        let desc = Paragraph::new(vec![Line::from(
            "Would you like to initialize the ticket queue with these settings?",
        )])
        .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[4]);

        // Selected collection
        let collection_text = vec![
            Line::from(Span::styled(
                "Selected issue types:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    self.selected_collection.join(", "),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];
        frame.render_widget(Paragraph::new(collection_text), chunks[6]);

        // What will be created
        let will_create = vec![
            Line::from(Span::styled(
                "This will create:",
                Style::default().fg(Color::Gray),
            )),
            Line::from("  .tickets/queue/  .tickets/in-progress/  .tickets/completed/"),
            Line::from(Span::styled(
                "  .tickets/templates/ (with selected issue type templates)",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(will_create), chunks[7]);

        // Buttons
        let init_style = if self.confirm_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let cancel_style = if !self.confirm_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let buttons = Line::from(vec![
            Span::raw("     "),
            Span::styled(" [I]nitialize ", init_style),
            Span::raw("     "),
            Span::styled(" [C]ancel ", cancel_style),
        ]);

        let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[8]);
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
