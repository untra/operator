#![allow(dead_code)]
#![allow(unused_imports)]

//! Dialog for creating new tickets with form-based field input

use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::queue::creator::{
    get_user_fields, parse_and_sort_schema, render_template, split_required_optional,
};
use crate::templates::schema::{FieldSchema, TemplateSchema};
use crate::templates::{glyph_for_key, TemplateType};
use crate::ui::form_field::{FormField, TicketForm};

/// Which step of the create dialog we're on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateDialogStep {
    /// Selecting template type
    Template,
    /// Selecting project
    Project,
    /// Filling required fields
    RequiredFields,
    /// Filling optional fields
    OptionalFields,
    /// Previewing the rendered template
    Preview,
}

/// Result of confirming the dialog
#[derive(Debug, Clone)]
pub struct CreateDialogResult {
    pub template_type: TemplateType,
    pub project: Option<String>,
    pub values: HashMap<String, String>,
}

/// Dialog for creating a ticket with form-based field input
pub struct CreateDialog {
    /// Whether the dialog is visible
    pub visible: bool,
    /// Current step in the dialog
    pub step: CreateDialogStep,
    /// List selection state for templates
    pub template_state: ListState,
    /// List selection state for projects
    pub project_state: ListState,
    /// Available template types
    pub templates: Vec<TemplateType>,
    /// Available projects (discovered from CLAUDE.md files)
    pub projects: Vec<String>,
    /// Selected template type (set after template selection)
    selected_template: Option<TemplateType>,
    /// Selected project
    selected_project: Option<String>,
    /// Parsed template schema
    schema: Option<TemplateSchema>,
    /// Required fields form
    required_form: Option<TicketForm>,
    /// Optional fields form
    optional_form: Option<TicketForm>,
    /// Auto-generated field values (id, created, status, branch)
    auto_values: HashMap<String, String>,
    /// Preview content
    preview_content: String,
    /// Preview scroll position
    preview_scroll: u16,
}

impl CreateDialog {
    /// Create a new create dialog
    pub fn new() -> Self {
        let templates = TemplateType::all().to_vec();
        let mut template_state = ListState::default();
        template_state.select(Some(0));

        Self {
            visible: false,
            step: CreateDialogStep::Template,
            template_state,
            project_state: ListState::default(),
            templates,
            projects: Vec::new(),
            selected_template: None,
            selected_project: None,
            schema: None,
            required_form: None,
            optional_form: None,
            auto_values: HashMap::new(),
            preview_content: String::new(),
            preview_scroll: 0,
        }
    }

    /// Update the list of available projects
    pub fn set_projects(&mut self, projects: Vec<String>) {
        self.projects = projects;
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.visible = true;
        self.step = CreateDialogStep::Template;
        self.template_state.select(Some(0));
        self.selected_template = None;
        self.selected_project = None;
        self.schema = None;
        self.required_form = None;
        self.optional_form = None;
        self.auto_values.clear();
        self.preview_content.clear();
        self.preview_scroll = 0;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.step = CreateDialogStep::Template;
        self.selected_template = None;
        self.selected_project = None;
        self.schema = None;
        self.required_form = None;
        self.optional_form = None;
        self.auto_values.clear();
        self.preview_content.clear();
    }

    /// Go back to previous step or hide if on first step
    pub fn go_back(&mut self) {
        match self.step {
            CreateDialogStep::Template => {
                self.hide();
            }
            CreateDialogStep::Project => {
                self.step = CreateDialogStep::Template;
                self.selected_template = None;
            }
            CreateDialogStep::RequiredFields => {
                self.step = CreateDialogStep::Project;
            }
            CreateDialogStep::OptionalFields => {
                self.step = CreateDialogStep::RequiredFields;
            }
            CreateDialogStep::Preview => {
                self.step = CreateDialogStep::OptionalFields;
            }
        }
    }

    /// Get the project list with optional "none" entry for SPIKE/INV
    fn project_list(&self) -> Vec<String> {
        let allows_none = self
            .selected_template
            .map(|t| t.project_optional())
            .unwrap_or(false);

        let mut list = Vec::new();
        if allows_none {
            list.push("(none)".to_string());
        }
        list.extend(self.projects.iter().cloned());
        list
    }

    /// Handle key event
    pub fn handle_key(&mut self, key: KeyCode) -> Option<CreateDialogResult> {
        match self.step {
            CreateDialogStep::Template => self.handle_template_key(key),
            CreateDialogStep::Project => self.handle_project_key(key),
            CreateDialogStep::RequiredFields => self.handle_required_fields_key(key),
            CreateDialogStep::OptionalFields => self.handle_optional_fields_key(key),
            CreateDialogStep::Preview => self.handle_preview_key(key),
        }
    }

    fn handle_template_key(&mut self, key: KeyCode) -> Option<CreateDialogResult> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = match self.template_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => self.templates.len().saturating_sub(1),
                };
                self.template_state.select(Some(i));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = match self.template_state.selected() {
                    Some(i) if i < self.templates.len() - 1 => i + 1,
                    _ => 0,
                };
                self.template_state.select(Some(i));
            }
            KeyCode::Enter => {
                if let Some(i) = self.template_state.selected() {
                    let template_type = self.templates[i];
                    self.selected_template = Some(template_type);

                    // Always go to project selection step
                    self.step = CreateDialogStep::Project;
                    self.project_state.select(Some(0));
                }
            }
            KeyCode::Esc => {
                self.go_back();
            }
            _ => {}
        }
        None
    }

    fn handle_project_key(&mut self, key: KeyCode) -> Option<CreateDialogResult> {
        let list = self.project_list();
        let requires_project = !self
            .selected_template
            .map(|t| t.project_optional())
            .unwrap_or(true);

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if !list.is_empty() {
                    let i = match self.project_state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => list.len().saturating_sub(1),
                    };
                    self.project_state.select(Some(i));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !list.is_empty() {
                    let i = match self.project_state.selected() {
                        Some(i) if i < list.len() - 1 => i + 1,
                        _ => 0,
                    };
                    self.project_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                // Block proceeding if projects are required but none found
                if list.is_empty() && requires_project {
                    return None;
                }

                if list.is_empty() {
                    self.selected_project = None;
                } else if let Some(i) = self.project_state.selected() {
                    let selected = &list[i];
                    self.selected_project = if selected == "(none)" {
                        None
                    } else {
                        Some(selected.clone())
                    };
                }
                self.initialize_forms();
                self.step = CreateDialogStep::RequiredFields;
            }
            KeyCode::Esc => {
                self.go_back();
            }
            _ => {}
        }
        None
    }

    fn handle_required_fields_key(&mut self, key: KeyCode) -> Option<CreateDialogResult> {
        if let Some(ref mut form) = self.required_form {
            match key {
                KeyCode::Tab => {
                    form.next_field();
                }
                KeyCode::BackTab => {
                    form.prev_field();
                }
                KeyCode::Enter => {
                    // If on last field and form is valid, proceed
                    if form.is_last_field() && form.is_valid() {
                        self.step = CreateDialogStep::OptionalFields;
                    } else if form.is_valid() {
                        form.next_field();
                    }
                }
                KeyCode::Esc => {
                    self.go_back();
                }
                _ => {
                    // Pass key to focused field
                    if let Some(field) = form.focused_field_mut() {
                        field.handle_key(key);
                    }
                }
            }
        }
        None
    }

    fn handle_optional_fields_key(&mut self, key: KeyCode) -> Option<CreateDialogResult> {
        if let Some(ref mut form) = self.optional_form {
            match key {
                KeyCode::Tab => {
                    form.next_field();
                }
                KeyCode::BackTab => {
                    form.prev_field();
                }
                KeyCode::Enter => {
                    // Proceed to preview
                    self.generate_preview();
                    self.step = CreateDialogStep::Preview;
                }
                KeyCode::Esc => {
                    self.go_back();
                }
                _ => {
                    // Pass key to focused field
                    if let Some(field) = form.focused_field_mut() {
                        field.handle_key(key);
                    }
                }
            }
        } else {
            // No optional fields, go straight to preview
            match key {
                KeyCode::Enter => {
                    self.generate_preview();
                    self.step = CreateDialogStep::Preview;
                }
                KeyCode::Esc => {
                    self.go_back();
                }
                _ => {}
            }
        }
        None
    }

    fn handle_preview_key(&mut self, key: KeyCode) -> Option<CreateDialogResult> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.preview_scroll = self.preview_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.preview_scroll = self.preview_scroll.saturating_add(1);
            }
            KeyCode::Enter => {
                // Finalize and return result
                return self.finalize();
            }
            KeyCode::Esc => {
                self.go_back();
            }
            _ => {}
        }
        None
    }

    /// Initialize forms after template and project are selected
    fn initialize_forms(&mut self) {
        let template_type = match self.selected_template {
            Some(t) => t,
            None => return,
        };

        // Parse schema
        let schema_json = template_type.schema();
        let schema = match parse_and_sort_schema(schema_json) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Generate auto values
        self.auto_values = self.generate_auto_values(template_type);

        // Get user-editable fields and split into required/optional
        let user_fields = get_user_fields(&schema);
        let (required, optional) = split_required_optional(user_fields);

        // Create forms
        if !required.is_empty() {
            let schemas: Vec<FieldSchema> = required.iter().map(|f| (*f).clone()).collect();
            let mut form = TicketForm::new(schemas);

            // Pre-fill project if selected
            if let Some(ref project) = self.selected_project {
                if let Some(field) = form.fields.get_mut("project") {
                    field.set_value(project);
                }
            }

            self.required_form = Some(form);
        }

        if !optional.is_empty() {
            let schemas: Vec<FieldSchema> = optional.iter().map(|f| (*f).clone()).collect();
            let mut opt_form = TicketForm::new(schemas);

            // Pre-fill project if selected (project is often in optional fields)
            if let Some(ref project) = self.selected_project {
                if let Some(field) = opt_form.fields.get_mut("project") {
                    field.set_value(project);
                }
            }

            self.optional_form = Some(opt_form);
        }

        self.schema = Some(schema);
    }

    /// Generate auto values (id, created, created_date, created_datetime, status, branch)
    fn generate_auto_values(&self, template_type: TemplateType) -> HashMap<String, String> {
        use chrono::Utc;

        let now = Utc::now();
        let date = now.format("%Y-%m-%d").to_string();
        let datetime = now.format("%Y-%m-%d %H:%M").to_string();
        let id = format!("{:04}", now.timestamp() % 10000);
        let type_str = template_type.as_str();
        let branch_prefix = type_str.to_lowercase();

        let mut values = HashMap::new();
        values.insert("id".to_string(), format!("{}-{}", type_str, id));
        values.insert("created".to_string(), date.clone());
        values.insert("created_date".to_string(), date);
        values.insert("created_datetime".to_string(), datetime);
        values.insert("status".to_string(), "queued".to_string());

        if let Some(ref project) = self.selected_project {
            values.insert("project".to_string(), project.clone());
        } else {
            values.insert("project".to_string(), String::new());
        }

        values.insert(
            "branch".to_string(),
            format!("{}/{}-{}-short-description", branch_prefix, type_str, id),
        );

        values
    }

    /// Generate preview content
    fn generate_preview(&mut self) {
        let template_type = match self.selected_template {
            Some(t) => t,
            None => return,
        };

        // Collect all values
        let mut values = self.auto_values.clone();

        if let Some(ref form) = self.required_form {
            values.extend(form.values());
        }

        if let Some(ref form) = self.optional_form {
            values.extend(form.values());
        }

        // Render template
        let template = template_type.template_content();
        match render_template(template, &values) {
            Ok(content) => self.preview_content = content,
            Err(_) => self.preview_content = "Error rendering template".to_string(),
        }
    }

    /// Finalize and return the result
    fn finalize(&mut self) -> Option<CreateDialogResult> {
        let template_type = self.selected_template?;

        // Collect all values
        let mut values = self.auto_values.clone();

        if let Some(ref form) = self.required_form {
            values.extend(form.values());
        }

        if let Some(ref form) = self.optional_form {
            values.extend(form.values());
        }

        self.hide();

        Some(CreateDialogResult {
            template_type,
            project: self.selected_project.clone(),
            values,
        })
    }

    /// Get the currently selected template type (for display purposes)
    pub fn current_template(&self) -> Option<TemplateType> {
        self.selected_template
            .or_else(|| self.template_state.selected().map(|i| self.templates[i]))
    }

    /// Render the dialog
    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        match self.step {
            CreateDialogStep::Template => self.render_template_step(frame),
            CreateDialogStep::Project => self.render_project_step(frame),
            CreateDialogStep::RequiredFields => self.render_required_fields_step(frame),
            CreateDialogStep::OptionalFields => self.render_optional_fields_step(frame),
            CreateDialogStep::Preview => self.render_preview_step(frame),
        }
    }

    fn render_template_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(50, 50, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Create Ticket ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Min(8),    // Template list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            "Select template type:",
            Style::default().fg(Color::Gray),
        )]));
        frame.render_widget(instructions, chunks[0]);

        // Template list
        let items: Vec<ListItem> = self
            .templates
            .iter()
            .map(|t| {
                let icon = format!("{} ", glyph_for_key(t.as_str()));

                let mode = if t.is_paired() { " (paired)" } else { "" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(icon),
                        Span::styled(
                            t.display_name(),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(mode, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(t.description(), Style::default().fg(Color::Gray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.template_state);

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

    fn render_project_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(50, 60, frame.area());
        frame.render_widget(Clear, area);

        let title = match self.selected_template {
            Some(t) => format!(" Select Project ({}) ", t.display_name()),
            None => " Select Project ".to_string(),
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
                Constraint::Min(8),    // Project list
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let optional_note = if self
            .selected_template
            .map(|t| t.project_optional())
            .unwrap_or(false)
        {
            " (optional)"
        } else {
            ""
        };
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            format!("Select project{}:", optional_note),
            Style::default().fg(Color::Gray),
        )]));
        frame.render_widget(instructions, chunks[0]);

        // Project list
        let project_list = self.project_list();
        let items: Vec<ListItem> = project_list
            .iter()
            .map(|p| {
                let (icon, style) = if p == "(none)" {
                    ("  ", Style::default().fg(Color::DarkGray))
                } else {
                    ("  ", Style::default())
                };
                ListItem::new(Line::from(vec![Span::raw(icon), Span::styled(p, style)]))
            })
            .collect();

        let requires_project = !self
            .selected_template
            .map(|t| t.project_optional())
            .unwrap_or(true);

        let items_empty = items.is_empty();

        if items_empty {
            if requires_project {
                // Clear warning message for required project
                let msg = Paragraph::new(vec![
                    Line::from(Span::styled(
                        "No projects found",
                        Style::default().fg(Color::Yellow),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Add a CLAUDE.md file to project directories",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(Span::styled(
                        "to register them with operator.",
                        Style::default().fg(Color::Gray),
                    )),
                ]);
                frame.render_widget(msg, chunks[1]);
            } else {
                // For optional projects, show simpler message
                let empty_msg = Paragraph::new(Line::from(vec![Span::styled(
                    "No projects discovered (press Enter to skip)",
                    Style::default().fg(Color::DarkGray),
                )]));
                frame.render_widget(empty_msg, chunks[1]);
            }
        } else {
            let list = List::new(items)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.project_state);
        }

        // Footer - different text when projects are required but empty
        let footer_text = if items_empty && requires_project {
            vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]
        } else {
            vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" select  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]
        };
        let footer = Paragraph::new(Line::from(footer_text)).alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn render_required_fields_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 70, frame.area());
        frame.render_widget(Clear, area);

        let title = match self.selected_template {
            Some(t) => format!(" Required Fields ({}) ", t.display_name()),
            None => " Required Fields ".to_string(),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(ref mut form) = self.required_form {
            render_form(frame, inner, form);
        } else {
            let msg = Paragraph::new("No required fields")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, inner);
        }
    }

    fn render_optional_fields_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 70, frame.area());
        frame.render_widget(Clear, area);

        let title = match self.selected_template {
            Some(t) => format!(" Optional Fields ({}) ", t.display_name()),
            None => " Optional Fields ".to_string(),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(ref mut form) = self.optional_form {
            render_form(frame, inner, form);
        } else {
            // No optional fields - show message and footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(4), Constraint::Length(2)])
                .split(inner);

            let msg = Paragraph::new("No optional fields - press Enter to continue")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, chunks[0]);

            let footer = Paragraph::new(Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" preview  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back"),
            ]))
            .alignment(Alignment::Center);
            frame.render_widget(footer, chunks[1]);
        }
    }

    fn render_preview_step(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let title = match self.selected_template {
            Some(t) => format!(" Preview ({}) ", t.display_name()),
            None => " Preview ".to_string(),
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
                Constraint::Min(10),   // Preview content
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            "Review the ticket content. Press Enter to open in editor.",
            Style::default().fg(Color::Gray),
        )]));
        frame.render_widget(instructions, chunks[0]);

        // Preview content with scroll
        let lines: Vec<Line> = self
            .preview_content
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();

        let total_lines = lines.len() as u16;
        let visible_height = chunks[1].height;

        // Clamp scroll position
        if total_lines > visible_height {
            self.preview_scroll = self.preview_scroll.min(total_lines - visible_height);
        } else {
            self.preview_scroll = 0;
        }

        let preview = Paragraph::new(lines)
            .scroll((self.preview_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(preview, chunks[1]);

        // Scrollbar
        if total_lines > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state =
                ScrollbarState::new(total_lines as usize).position(self.preview_scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                chunks[1].inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" open in editor  "),
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(" scroll  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }
}

impl Default for CreateDialog {
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

/// Render a form with its fields (free function to avoid borrow issues)
#[allow(dead_code)]
fn render_form(frame: &mut Frame, area: Rect, form: &mut TicketForm) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(4),    // Form fields
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Calculate field heights and create constraints
    let mut field_constraints = Vec::new();
    for name in &form.field_order {
        if let (Some(schema), Some(field)) = (form.schemas.get(name), form.fields.get(name)) {
            // Label + field height
            let label_height = 1;
            let field_height = field.render_height();
            field_constraints.push(Constraint::Length(label_height + field_height + 1)); // +1 for spacing
            let _ = schema; // Silence unused warning
        }
    }

    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(field_constraints)
        .split(chunks[0]);

    // Render each field
    for (idx, name) in form.field_order.iter().enumerate() {
        if let (Some(schema), Some(field)) = (form.schemas.get(name), form.fields.get_mut(name)) {
            if idx < field_areas.len() {
                let field_area = field_areas[idx];
                let is_focused = idx == form.focused_index;

                // Label with required indicator
                let required_marker = if schema.required { "*" } else { "" };
                let label = format!("{}{}", schema.name, required_marker);
                let label_style = if is_focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let label_para = Paragraph::new(Line::from(vec![
                    Span::styled(label, label_style),
                    Span::raw(" "),
                    Span::styled(&schema.description, Style::default().fg(Color::DarkGray)),
                ]));

                let label_area = Rect {
                    height: 1,
                    ..field_area
                };
                frame.render_widget(label_para, label_area);

                // Field input
                let input_area = Rect {
                    y: field_area.y + 1,
                    height: field.render_height(),
                    ..field_area
                };
                field.render(frame, input_area, is_focused);
            }
        }
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" next  "),
        Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" prev  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" continue  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dialog_new_initializes_correctly() {
        let dialog = CreateDialog::new();

        assert!(!dialog.visible);
        assert_eq!(dialog.step, CreateDialogStep::Template);
        assert!(dialog.selected_template.is_none());
        assert!(dialog.selected_project.is_none());
        assert!(!dialog.templates.is_empty());
        assert!(dialog.template_state.selected().is_some());
    }

    #[test]
    fn test_create_dialog_default_equals_new() {
        let d1 = CreateDialog::new();
        let d2 = CreateDialog::default();

        assert_eq!(d1.visible, d2.visible);
        assert_eq!(d1.step, d2.step);
    }

    #[test]
    fn test_create_dialog_set_projects() {
        let mut dialog = CreateDialog::new();
        dialog.set_projects(vec!["p1".to_string(), "p2".to_string()]);

        assert_eq!(dialog.projects.len(), 2);
    }

    #[test]
    fn test_create_dialog_show_resets_state() {
        let mut dialog = CreateDialog::new();
        dialog.step = CreateDialogStep::Preview;
        dialog.selected_template = Some(TemplateType::Feature);

        dialog.show();

        assert!(dialog.visible);
        assert_eq!(dialog.step, CreateDialogStep::Template);
        assert!(dialog.selected_template.is_none());
        assert!(dialog.selected_project.is_none());
    }

    #[test]
    fn test_create_dialog_hide_clears_state() {
        let mut dialog = CreateDialog::new();
        dialog.visible = true;
        dialog.step = CreateDialogStep::Project;
        dialog.selected_template = Some(TemplateType::Fix);

        dialog.hide();

        assert!(!dialog.visible);
        assert_eq!(dialog.step, CreateDialogStep::Template);
        assert!(dialog.selected_template.is_none());
    }

    #[test]
    fn test_create_dialog_go_back_from_template_hides() {
        let mut dialog = CreateDialog::new();
        dialog.show();

        dialog.go_back();

        assert!(!dialog.visible);
    }

    #[test]
    fn test_create_dialog_go_back_from_project_to_template() {
        let mut dialog = CreateDialog::new();
        dialog.show();
        dialog.step = CreateDialogStep::Project;
        dialog.selected_template = Some(TemplateType::Feature);

        dialog.go_back();

        assert_eq!(dialog.step, CreateDialogStep::Template);
        assert!(dialog.selected_template.is_none());
    }

    #[test]
    fn test_create_dialog_go_back_from_required_fields_to_project() {
        let mut dialog = CreateDialog::new();
        dialog.step = CreateDialogStep::RequiredFields;

        dialog.go_back();

        assert_eq!(dialog.step, CreateDialogStep::Project);
    }

    #[test]
    fn test_create_dialog_go_back_from_optional_fields_to_required() {
        let mut dialog = CreateDialog::new();
        dialog.step = CreateDialogStep::OptionalFields;

        dialog.go_back();

        assert_eq!(dialog.step, CreateDialogStep::RequiredFields);
    }

    #[test]
    fn test_create_dialog_go_back_from_preview_to_optional() {
        let mut dialog = CreateDialog::new();
        dialog.step = CreateDialogStep::Preview;

        dialog.go_back();

        assert_eq!(dialog.step, CreateDialogStep::OptionalFields);
    }

    #[test]
    fn test_create_dialog_project_list_without_optional_template() {
        let mut dialog = CreateDialog::new();
        dialog.set_projects(vec!["p1".to_string(), "p2".to_string()]);
        dialog.selected_template = Some(TemplateType::Feature); // Feature requires project

        let list = dialog.project_list();

        assert_eq!(list.len(), 2);
        assert!(!list.contains(&"(none)".to_string()));
    }

    #[test]
    fn test_create_dialog_project_list_with_optional_template() {
        let mut dialog = CreateDialog::new();
        dialog.set_projects(vec!["p1".to_string(), "p2".to_string()]);
        dialog.selected_template = Some(TemplateType::Spike); // Spike has optional project

        let list = dialog.project_list();

        assert_eq!(list.len(), 3);
        assert_eq!(list[0], "(none)");
    }

    #[test]
    fn test_create_dialog_template_navigation() {
        let mut dialog = CreateDialog::new();
        dialog.show();

        // Initially at 0
        assert_eq!(dialog.template_state.selected(), Some(0));

        // Navigate down
        dialog.handle_key(KeyCode::Down);
        assert_eq!(dialog.template_state.selected(), Some(1));

        // Navigate up
        dialog.handle_key(KeyCode::Up);
        assert_eq!(dialog.template_state.selected(), Some(0));

        // Wrap around at top
        dialog.handle_key(KeyCode::Up);
        assert_eq!(
            dialog.template_state.selected(),
            Some(dialog.templates.len() - 1)
        );
    }

    #[test]
    fn test_create_dialog_template_selection_proceeds_to_project() {
        let mut dialog = CreateDialog::new();
        dialog.show();

        dialog.handle_key(KeyCode::Enter);

        assert!(dialog.selected_template.is_some());
        assert_eq!(dialog.step, CreateDialogStep::Project);
    }

    #[test]
    fn test_create_dialog_esc_in_template_hides() {
        let mut dialog = CreateDialog::new();
        dialog.show();

        dialog.handle_key(KeyCode::Esc);

        assert!(!dialog.visible);
    }

    #[test]
    fn test_create_dialog_generate_auto_values_creates_expected_keys() {
        let dialog = CreateDialog::new();

        let values = dialog.generate_auto_values(TemplateType::Feature);

        assert!(values.contains_key("id"));
        assert!(values.contains_key("created"));
        assert!(values.contains_key("created_date"));
        assert!(values.contains_key("created_datetime"));
        assert!(values.contains_key("status"));
        assert!(values.contains_key("branch"));
        assert!(values.contains_key("project"));

        assert_eq!(values.get("status"), Some(&"queued".to_string()));
        assert!(values.get("id").unwrap().starts_with("FEAT-"));
        assert!(values.get("branch").unwrap().starts_with("feat/FEAT-"));
    }

    #[test]
    fn test_create_dialog_generate_auto_values_with_project() {
        let mut dialog = CreateDialog::new();
        dialog.selected_project = Some("my-project".to_string());

        let values = dialog.generate_auto_values(TemplateType::Fix);

        assert_eq!(values.get("project"), Some(&"my-project".to_string()));
        assert!(values.get("id").unwrap().starts_with("FIX-"));
    }

    #[test]
    fn test_create_dialog_current_template() {
        let mut dialog = CreateDialog::new();
        dialog.show();

        // No explicit selection, returns from template_state
        let template = dialog.current_template();
        assert!(template.is_some());

        // After selection, returns selected
        dialog.selected_template = Some(TemplateType::Spike);
        assert_eq!(dialog.current_template(), Some(TemplateType::Spike));
    }
}
