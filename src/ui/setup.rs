//! Startup setup screen when .tickets/ directory is not found

use std::collections::HashMap;

use crate::config::CollectionPreset;
use crate::projects::TOOL_MARKERS;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Simplified tool info for display on the welcome screen
#[derive(Debug, Clone)]
pub struct DetectedToolInfo {
    pub name: String,
    pub version: String,
    pub model_count: usize,
}

/// Available issuetype collections (all known types)
pub const ALL_ISSUE_TYPES: &[&str] = &["TASK", "FEAT", "FIX", "SPIKE", "INV"];

/// Optional fields that can be configured for TASK (and propagated to other types)
/// Note: 'summary' remains required, 'id' is auto-generated
pub const TASK_OPTIONAL_FIELDS: &[(&str, &str)] = &[
    ("priority", "Priority level (P0-critical to P3-low)"),
    ("context", "Background context for the task"),
];

/// Collection source options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSourceOption {
    Simple,
    DevKanban,
    DevopsKanban,
    ImportJira,
    ImportNotion,
    CustomSelection,
}

impl CollectionSourceOption {
    pub fn all() -> &'static [CollectionSourceOption] {
        &[
            CollectionSourceOption::Simple,
            CollectionSourceOption::DevKanban,
            CollectionSourceOption::DevopsKanban,
            CollectionSourceOption::ImportJira,
            CollectionSourceOption::ImportNotion,
            CollectionSourceOption::CustomSelection,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Simple",
            CollectionSourceOption::DevKanban => "Dev Kanban",
            CollectionSourceOption::DevopsKanban => "DevOps Kanban",
            CollectionSourceOption::ImportJira => "Import from Jira",
            CollectionSourceOption::ImportNotion => "Import from Notion",
            CollectionSourceOption::CustomSelection => "Custom Selection",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Just TASK - minimal setup for general work",
            CollectionSourceOption::DevKanban => "3 issue types: TASK, FEAT, FIX",
            CollectionSourceOption::DevopsKanban => "5 issue types: TASK, SPIKE, INV, FEAT, FIX",
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
    /// Detected LLM tools (from LlmToolsConfig)
    detected_tools: Vec<DetectedToolInfo>,
    /// Projects grouped by tool
    projects_by_tool: HashMap<String, Vec<String>>,
    /// Selected collection preset
    pub selected_preset: CollectionPreset,
    /// Custom issuetype collection (only used when preset is Custom)
    pub custom_collection: Vec<String>,
    /// List state for collection source selection
    source_state: ListState,
    /// List state for custom collection selection
    collection_state: ListState,
    /// Whether we came from custom selection (for back navigation)
    from_custom: bool,
    /// Selected optional fields to include in TASK (and other types)
    pub task_optional_fields: Vec<String>,
    /// List state for field configuration selection
    field_state: ListState,
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
    /// Configure TASK optional fields
    TaskFieldConfig,
    /// Tmux onboarding/help
    TmuxOnboarding,
    /// Confirm initialization
    Confirm,
}

impl SetupScreen {
    /// Create a new setup screen
    pub fn new(
        tickets_path: String,
        detected_tools: Vec<DetectedToolInfo>,
        projects_by_tool: HashMap<String, Vec<String>>,
    ) -> Self {
        let mut source_state = ListState::default();
        source_state.select(Some(0));

        let mut collection_state = ListState::default();
        collection_state.select(Some(0));

        let mut field_state = ListState::default();
        field_state.select(Some(0));

        Self {
            visible: true,
            step: SetupStep::Welcome,
            confirm_selected: true, // Default to Initialize
            tickets_path,
            detected_tools,
            projects_by_tool,
            selected_preset: CollectionPreset::DevopsKanban,
            custom_collection: ALL_ISSUE_TYPES.iter().map(|s| s.to_string()).collect(),
            source_state,
            collection_state,
            from_custom: false,
            // Default: all optional fields enabled
            task_optional_fields: TASK_OPTIONAL_FIELDS
                .iter()
                .map(|(name, _)| name.to_string())
                .collect(),
            field_state,
        }
    }

    /// Get the selected collection preset
    pub fn preset(&self) -> CollectionPreset {
        self.selected_preset
    }

    /// Get the effective issuetype collection based on preset
    pub fn collection(&self) -> Vec<String> {
        match self.selected_preset {
            CollectionPreset::Custom => self.custom_collection.clone(),
            _ => self.selected_preset.issue_types(),
        }
    }

    /// Get the configured optional fields for TASK (and propagation to other types)
    pub fn configured_task_fields(&self) -> Vec<String> {
        self.task_optional_fields.clone()
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
                    let types = ALL_ISSUE_TYPES;
                    if i < types.len() {
                        let type_str = types[i].to_string();
                        if self.custom_collection.contains(&type_str) {
                            self.custom_collection.retain(|t| t != &type_str);
                        } else {
                            self.custom_collection.push(type_str);
                        }
                    }
                }
            }
            SetupStep::TaskFieldConfig => {
                // Toggle the currently highlighted field
                if let Some(i) = self.field_state.selected() {
                    if i < TASK_OPTIONAL_FIELDS.len() {
                        let field_name = TASK_OPTIONAL_FIELDS[i].0.to_string();
                        if self.task_optional_fields.contains(&field_name) {
                            self.task_optional_fields.retain(|f| f != &field_name);
                        } else {
                            self.task_optional_fields.push(field_name);
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
                let len = ALL_ISSUE_TYPES.len();
                let i = self
                    .collection_state
                    .selected()
                    .map_or(0, |i| (i + 1) % len);
                self.collection_state.select(Some(i));
            }
            SetupStep::TaskFieldConfig => {
                let len = TASK_OPTIONAL_FIELDS.len();
                let i = self.field_state.selected().map_or(0, |i| (i + 1) % len);
                self.field_state.select(Some(i));
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
                let len = ALL_ISSUE_TYPES.len();
                let i = self.collection_state.selected().map_or(0, |i| {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                });
                self.collection_state.select(Some(i));
            }
            SetupStep::TaskFieldConfig => {
                let len = TASK_OPTIONAL_FIELDS.len();
                let i =
                    self.field_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.field_state.select(Some(i));
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
                            self.selected_preset = CollectionPreset::Simple;
                            self.from_custom = false;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::DevKanban => {
                            self.selected_preset = CollectionPreset::DevKanban;
                            self.from_custom = false;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::DevopsKanban => {
                            self.selected_preset = CollectionPreset::DevopsKanban;
                            self.from_custom = false;
                            self.step = SetupStep::TaskFieldConfig;
                            SetupResult::Continue
                        }
                        CollectionSourceOption::ImportJira => SetupResult::ExitUnimplemented(
                            "Jira import is not yet implemented".to_string(),
                        ),
                        CollectionSourceOption::ImportNotion => SetupResult::ExitUnimplemented(
                            "Notion import is not yet implemented".to_string(),
                        ),
                        CollectionSourceOption::CustomSelection => {
                            self.selected_preset = CollectionPreset::Custom;
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
                if !self.custom_collection.is_empty() {
                    self.step = SetupStep::TaskFieldConfig;
                }
                SetupResult::Continue
            }
            SetupStep::TaskFieldConfig => {
                self.step = SetupStep::TmuxOnboarding;
                SetupResult::Continue
            }
            SetupStep::TmuxOnboarding => {
                self.step = SetupStep::Confirm;
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
            SetupStep::TaskFieldConfig => {
                if self.from_custom {
                    self.step = SetupStep::CustomCollection;
                } else {
                    self.step = SetupStep::CollectionSource;
                }
                SetupResult::Continue
            }
            SetupStep::TmuxOnboarding => {
                self.step = SetupStep::TaskFieldConfig;
                SetupResult::Continue
            }
            SetupStep::Confirm => {
                self.step = SetupStep::TmuxOnboarding;
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
            SetupStep::TaskFieldConfig => self.render_task_field_config_step(frame),
            SetupStep::TmuxOnboarding => self.render_tmux_onboarding_step(frame),
            SetupStep::Confirm => self.render_confirm_step(frame),
        }
    }

    fn render_welcome_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
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
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Description
                Constraint::Length(1), // Spacer
                Constraint::Length(6), // Detected LLM Tools
                Constraint::Length(1), // Spacer
                Constraint::Min(6),    // Discovered projects by tool
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Path info
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
        let desc = Paragraph::new(vec![Line::from("A TUI for orchestrating LLM Code agents.")])
            .alignment(Alignment::Center);
        frame.render_widget(desc, chunks[2]);

        // Detected LLM Tools
        let mut tools_text = vec![Line::from(Span::styled(
            "Detected LLM Tools:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        // Show each known tool with detection status
        for (tool_name, _marker) in TOOL_MARKERS {
            let detected = self.detected_tools.iter().find(|t| t.name == *tool_name);

            let line = if let Some(tool) = detected {
                Line::from(vec![
                    Span::styled("  + ", Style::default().fg(Color::Green)),
                    Span::styled(
                        tool_name.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" (v{}) - {} models", tool.version, tool.model_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  - ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tool_name.to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(" - not installed", Style::default().fg(Color::DarkGray)),
                ])
            };
            tools_text.push(line);
        }
        frame.render_widget(Paragraph::new(tools_text), chunks[4]);

        // Discovered projects by tool
        let mut projects_text = vec![Line::from(Span::styled(
            "Discovered Projects:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        let mut has_any_projects = false;
        for (tool_name, _marker) in TOOL_MARKERS {
            if let Some(projects) = self.projects_by_tool.get(*tool_name) {
                if !projects.is_empty() {
                    has_any_projects = true;
                    let project_list = projects.join(", ");
                    projects_text.push(Line::from(vec![
                        Span::styled(
                            format!("  {}: ", tool_name),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(project_list, Style::default().fg(Color::Green)),
                    ]));
                }
            }
        }

        if !has_any_projects {
            projects_text.push(Line::from(Span::styled(
                "  (no projects with marker files found)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        frame.render_widget(Paragraph::new(projects_text), chunks[6]);

        // Path info
        let path_info = Paragraph::new(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&self.tickets_path, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(path_info, chunks[8]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[9]);
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
        let items: Vec<ListItem> = ALL_ISSUE_TYPES
            .iter()
            .map(|t| {
                let is_selected = self.custom_collection.contains(&t.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
                let description = match *t {
                    "TASK" => "Focused task that executes one specific thing",
                    "FEAT" => "New feature or enhancement",
                    "FIX" => "Bug fix, follow-up work, tech debt",
                    "SPIKE" => "Research or exploration (paired mode)",
                    "INV" => "Incident investigation (paired mode)",
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
        let selected_count = self.custom_collection.len();
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

    fn render_task_field_config_step(&mut self, frame: &mut Frame) {
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
                Span::raw(" Setup - Configure TASK Fields "),
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
                Constraint::Length(3), // Explanation
                Constraint::Length(2), // Instructions
                Constraint::Min(6),    // Field list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Configure TASK Fields",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Explanation
        let explanation = Paragraph::new(vec![
            Line::from("TASK is the foundational issuetype. Configure which optional"),
            Line::from("fields to include. These choices will propagate to other types."),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(explanation, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(
            "Use arrows to navigate, Space to toggle, Enter to continue",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, chunks[2]);

        // Field list
        let items: Vec<ListItem> = TASK_OPTIONAL_FIELDS
            .iter()
            .map(|(name, description)| {
                let is_selected = self.task_optional_fields.contains(&name.to_string());
                let checkbox = if is_selected { "[x]" } else { "[ ]" };
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
                            *name,
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
                        Span::styled(*description, Style::default().fg(Color::DarkGray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[3], &mut self.field_state);

        // Footer
        let selected_count = self.task_optional_fields.len();
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(
                    "{}/{} fields enabled",
                    selected_count,
                    TASK_OPTIONAL_FIELDS.len()
                ),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  |  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }

    fn render_tmux_onboarding_step(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
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
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Intro
                Constraint::Length(2), // Spacer
                Constraint::Min(12),   // Help text
                Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Tmux Session Help",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Intro
        let intro = Paragraph::new(vec![Line::from(
            "Operator launches Claude agents in tmux sessions.",
        )])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(intro, chunks[2]);

        // Help text
        let help_text = vec![
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
        ];
        frame.render_widget(Paragraph::new(help_text), chunks[4]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" continue  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[5]);
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
        let effective_collection = self.collection();
        let collection_text = vec![
            Line::from(Span::styled(
                format!(
                    "Selected issue types ({}):",
                    self.selected_preset.display_name()
                ),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    effective_collection.join(", "),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_tool_info_creation() {
        let info = DetectedToolInfo {
            name: "claude".to_string(),
            version: "2.0.76".to_string(),
            model_count: 3,
        };
        assert_eq!(info.name, "claude");
        assert_eq!(info.version, "2.0.76");
        assert_eq!(info.model_count, 3);
    }

    #[test]
    fn test_setup_screen_new_with_detected_tools() {
        let tools = vec![DetectedToolInfo {
            name: "claude".to_string(),
            version: "2.0.76".to_string(),
            model_count: 3,
        }];
        let mut projects = HashMap::new();
        projects.insert("claude".to_string(), vec!["project-a".to_string()]);

        let screen = SetupScreen::new(".tickets".to_string(), tools, projects);

        assert!(screen.visible);
        assert_eq!(screen.step, SetupStep::Welcome);
    }

    #[test]
    fn test_setup_screen_with_no_detected_tools() {
        let screen = SetupScreen::new(".tickets".to_string(), vec![], HashMap::new());
        assert!(screen.visible);
        assert_eq!(screen.step, SetupStep::Welcome);
    }

    #[test]
    fn test_setup_screen_with_multiple_tools() {
        let tools = vec![
            DetectedToolInfo {
                name: "claude".to_string(),
                version: "2.0.0".to_string(),
                model_count: 3,
            },
            DetectedToolInfo {
                name: "gemini".to_string(),
                version: "1.0.0".to_string(),
                model_count: 2,
            },
        ];
        let mut projects = HashMap::new();
        projects.insert(
            "claude".to_string(),
            vec!["api".to_string(), "web".to_string()],
        );
        projects.insert("gemini".to_string(), vec!["api".to_string()]);

        let screen = SetupScreen::new(".tickets".to_string(), tools, projects);
        assert!(screen.visible);
        assert_eq!(screen.step, SetupStep::Welcome);
    }
}
