#![allow(dead_code)]

//! Reusable form field widgets for TUI forms

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use crate::templates::schema::{FieldSchema, FieldType};

/// A form field widget that can handle different input types
pub enum FormField {
    /// Single-line text input
    TextInput {
        value: String,
        cursor_pos: usize,
        placeholder: String,
        max_length: Option<usize>,
    },
    /// Multi-line text input using tui-textarea
    TextArea {
        textarea: Box<TextArea<'static>>,
        placeholder: String,
    },
    /// Enum selection from predefined options
    EnumSelect {
        options: Vec<String>,
        selected: usize,
        list_state: ListState,
    },
    /// Boolean toggle
    Toggle {
        value: bool,
        true_label: String,
        false_label: String,
    },
    /// Date input (YYYY-MM-DD format)
    DateInput { value: String, cursor_pos: usize },
}

impl FormField {
    /// Create a form field from a schema definition
    pub fn from_schema(schema: &FieldSchema) -> Self {
        match schema.field_type {
            FieldType::String => {
                let default_value = schema.default.clone().unwrap_or_default();
                FormField::TextInput {
                    cursor_pos: default_value.len(),
                    value: default_value,
                    placeholder: schema.placeholder.clone().unwrap_or_default(),
                    max_length: schema.max_length,
                }
            }
            FieldType::Text => {
                let mut textarea = TextArea::default();
                if let Some(ref default) = schema.default {
                    textarea.insert_str(default);
                }
                FormField::TextArea {
                    textarea: Box::new(textarea),
                    placeholder: schema.placeholder.clone().unwrap_or_default(),
                }
            }
            FieldType::Enum => {
                let options = schema.options.clone();
                let default_idx = schema
                    .default
                    .as_ref()
                    .and_then(|d| options.iter().position(|o| o == d))
                    .unwrap_or(0);
                let mut list_state = ListState::default();
                list_state.select(Some(default_idx));
                FormField::EnumSelect {
                    options,
                    selected: default_idx,
                    list_state,
                }
            }
            FieldType::Bool => {
                let value = schema
                    .default
                    .as_ref()
                    .map(|d| d == "true" || d == "yes")
                    .unwrap_or(false);
                FormField::Toggle {
                    value,
                    true_label: "Yes".to_string(),
                    false_label: "No".to_string(),
                }
            }
            FieldType::Date => {
                let default_value = schema.default.clone().unwrap_or_default();
                FormField::DateInput {
                    cursor_pos: default_value.len(),
                    value: default_value,
                }
            }
        }
    }

    /// Get the current value as a string
    pub fn value(&self) -> String {
        match self {
            FormField::TextInput { value, .. } => value.clone(),
            FormField::TextArea { textarea, .. } => textarea.lines().join("\n"),
            FormField::EnumSelect {
                options, selected, ..
            } => options.get(*selected).cloned().unwrap_or_default(),
            FormField::Toggle { value, .. } => value.to_string(),
            FormField::DateInput { value, .. } => value.clone(),
        }
    }

    /// Set the value from a string
    pub fn set_value(&mut self, new_value: &str) {
        match self {
            FormField::TextInput {
                value, cursor_pos, ..
            } => {
                *value = new_value.to_string();
                *cursor_pos = value.len();
            }
            FormField::TextArea { textarea, .. } => {
                textarea.select_all();
                textarea.cut();
                textarea.insert_str(new_value);
            }
            FormField::EnumSelect {
                options,
                selected,
                list_state,
            } => {
                if let Some(idx) = options.iter().position(|o| o == new_value) {
                    *selected = idx;
                    list_state.select(Some(idx));
                }
            }
            FormField::Toggle { value, .. } => {
                *value = new_value == "true" || new_value == "yes";
            }
            FormField::DateInput {
                value, cursor_pos, ..
            } => {
                *value = new_value.to_string();
                *cursor_pos = value.len();
            }
        }
    }

    /// Check if the field value is valid (non-empty for required fields)
    pub fn is_valid(&self, required: bool) -> bool {
        if !required {
            return true;
        }
        match self {
            FormField::TextInput { value, .. } => !value.trim().is_empty(),
            FormField::TextArea { textarea, .. } => {
                !textarea.lines().iter().all(|l| l.trim().is_empty())
            }
            FormField::EnumSelect { options, .. } => !options.is_empty(),
            FormField::Toggle { .. } => true,
            FormField::DateInput { value, .. } => !value.trim().is_empty(),
        }
    }

    /// Handle a key event, returns true if the key was consumed
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match self {
            FormField::TextInput {
                value,
                cursor_pos,
                max_length,
                ..
            } => match key {
                KeyCode::Char(c) => {
                    if max_length.map(|m| value.len() < m).unwrap_or(true) {
                        value.insert(*cursor_pos, c);
                        *cursor_pos += 1;
                    }
                    true
                }
                KeyCode::Backspace => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                        value.remove(*cursor_pos);
                    }
                    true
                }
                KeyCode::Delete => {
                    if *cursor_pos < value.len() {
                        value.remove(*cursor_pos);
                    }
                    true
                }
                KeyCode::Left => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                    true
                }
                KeyCode::Right => {
                    if *cursor_pos < value.len() {
                        *cursor_pos += 1;
                    }
                    true
                }
                KeyCode::Home => {
                    *cursor_pos = 0;
                    true
                }
                KeyCode::End => {
                    *cursor_pos = value.len();
                    true
                }
                _ => false,
            },
            FormField::TextArea { textarea, .. } => {
                // TextArea handles its own key events
                textarea.input(crossterm::event::KeyEvent::new(
                    key,
                    crossterm::event::KeyModifiers::NONE,
                ));
                true
            }
            FormField::EnumSelect {
                options,
                selected,
                list_state,
            } => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if *selected > 0 {
                        *selected -= 1;
                        list_state.select(Some(*selected));
                    }
                    true
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *selected < options.len().saturating_sub(1) {
                        *selected += 1;
                        list_state.select(Some(*selected));
                    }
                    true
                }
                _ => false,
            },
            FormField::Toggle { value, .. } => match key {
                KeyCode::Char(' ') | KeyCode::Enter => {
                    *value = !*value;
                    true
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    *value = false;
                    true
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    *value = true;
                    true
                }
                _ => false,
            },
            FormField::DateInput {
                value, cursor_pos, ..
            } => match key {
                KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                    if value.len() < 10 {
                        value.insert(*cursor_pos, c);
                        *cursor_pos += 1;
                    }
                    true
                }
                KeyCode::Backspace => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                        value.remove(*cursor_pos);
                    }
                    true
                }
                KeyCode::Left => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                    true
                }
                KeyCode::Right => {
                    if *cursor_pos < value.len() {
                        *cursor_pos += 1;
                    }
                    true
                }
                _ => false,
            },
        }
    }

    /// Get the height needed to render this field
    pub fn render_height(&self) -> u16 {
        match self {
            FormField::TextInput { .. } => 1,
            FormField::TextArea { .. } => 5, // Multi-line gets more space
            FormField::EnumSelect { options, .. } => (options.len() as u16).min(5),
            FormField::Toggle { .. } => 1,
            FormField::DateInput { .. } => 1,
        }
    }

    /// Render the field
    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused { Color::Cyan } else { Color::Gray };

        match self {
            FormField::TextInput {
                value,
                cursor_pos,
                placeholder,
                max_length,
            } => {
                let display_text = if value.is_empty() && !focused {
                    Span::styled(placeholder.as_str(), Style::default().fg(Color::DarkGray))
                } else {
                    Span::raw(value.as_str())
                };

                let mut text = value.clone();
                if focused {
                    // Show cursor position
                    if *cursor_pos < text.len() {
                        text.insert(*cursor_pos, '|');
                    } else {
                        text.push('|');
                    }
                }

                let suffix = max_length
                    .map(|m| format!(" ({}/{})", value.len(), m))
                    .unwrap_or_default();

                let content = if value.is_empty() && !focused {
                    Line::from(display_text)
                } else {
                    Line::from(vec![
                        Span::raw(text),
                        Span::styled(suffix, Style::default().fg(Color::DarkGray)),
                    ])
                };

                let para = Paragraph::new(content).style(Style::default().fg(if focused {
                    Color::White
                } else {
                    Color::Gray
                }));
                frame.render_widget(para, area);
            }
            FormField::TextArea {
                textarea,
                placeholder,
            } => {
                // Configure textarea styling
                textarea.set_cursor_line_style(Style::default());
                textarea.set_cursor_style(if focused {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                });
                textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color)),
                );

                if textarea.lines().iter().all(|l| l.is_empty()) && !focused {
                    textarea.set_placeholder_text(placeholder.clone());
                    textarea.set_placeholder_style(Style::default().fg(Color::DarkGray));
                }

                frame.render_widget(&**textarea, area);
            }
            FormField::EnumSelect {
                options,
                selected,
                list_state,
            } => {
                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, opt)| {
                        let style = if i == *selected {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        };
                        ListItem::new(Span::styled(opt, style))
                    })
                    .collect();

                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::REVERSED)
                            .fg(Color::Cyan),
                    )
                    .highlight_symbol("> ");

                frame.render_stateful_widget(list, area, list_state);
            }
            FormField::Toggle {
                value,
                true_label,
                false_label,
            } => {
                let yes_style = if *value {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let no_style = if !*value {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let line = Line::from(vec![
                    Span::styled(format!("[{}]", true_label), yes_style),
                    Span::raw(" / "),
                    Span::styled(format!("[{}]", false_label), no_style),
                ]);

                let para = Paragraph::new(line);
                frame.render_widget(para, area);
            }
            FormField::DateInput {
                value, cursor_pos, ..
            } => {
                let mut text = value.clone();
                if focused {
                    if *cursor_pos < text.len() {
                        text.insert(*cursor_pos, '|');
                    } else {
                        text.push('|');
                    }
                }

                let placeholder_text = "YYYY-MM-DD";
                let display = if value.is_empty() && !focused {
                    Line::from(Span::styled(
                        placeholder_text,
                        Style::default().fg(Color::DarkGray),
                    ))
                } else {
                    Line::from(text)
                };

                let para = Paragraph::new(display).style(Style::default().fg(if focused {
                    Color::White
                } else {
                    Color::Gray
                }));
                frame.render_widget(para, area);
            }
        }
    }
}

/// A complete form with multiple fields
pub struct TicketForm {
    /// Field names in order
    pub field_order: Vec<String>,
    /// Field schemas by name
    pub schemas: std::collections::HashMap<String, FieldSchema>,
    /// Field widgets by name
    pub fields: std::collections::HashMap<String, FormField>,
    /// Currently focused field index
    pub focused_index: usize,
}

impl TicketForm {
    /// Create a new form from field schemas
    pub fn new(schemas: Vec<FieldSchema>) -> Self {
        let mut field_order = Vec::new();
        let mut schema_map = std::collections::HashMap::new();
        let mut fields = std::collections::HashMap::new();

        for schema in schemas {
            let name = schema.name.clone();
            let field = FormField::from_schema(&schema);
            field_order.push(name.clone());
            schema_map.insert(name.clone(), schema);
            fields.insert(name, field);
        }

        Self {
            field_order,
            schemas: schema_map,
            fields,
            focused_index: 0,
        }
    }

    /// Get the currently focused field name
    pub fn focused_field_name(&self) -> Option<&str> {
        self.field_order.get(self.focused_index).map(|s| s.as_str())
    }

    /// Get a mutable reference to the focused field
    pub fn focused_field_mut(&mut self) -> Option<&mut FormField> {
        let name = self.field_order.get(self.focused_index)?;
        self.fields.get_mut(name)
    }

    /// Move to the next field
    pub fn next_field(&mut self) {
        if self.focused_index < self.field_order.len().saturating_sub(1) {
            self.focused_index += 1;
        }
    }

    /// Move to the previous field
    pub fn prev_field(&mut self) {
        if self.focused_index > 0 {
            self.focused_index -= 1;
        }
    }

    /// Check if all required fields are valid
    pub fn is_valid(&self) -> bool {
        for name in &self.field_order {
            if let (Some(schema), Some(field)) = (self.schemas.get(name), self.fields.get(name)) {
                if !field.is_valid(schema.required) {
                    return false;
                }
            }
        }
        true
    }

    /// Get all field values as a map
    pub fn values(&self) -> std::collections::HashMap<String, String> {
        self.fields
            .iter()
            .map(|(k, v)| (k.clone(), v.value()))
            .collect()
    }

    /// Check if we're on the last field
    pub fn is_last_field(&self) -> bool {
        self.focused_index >= self.field_order.len().saturating_sub(1)
    }

    /// Check if we're on the first field
    pub fn is_first_field(&self) -> bool {
        self.focused_index == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_string_schema(name: &str, required: bool) -> FieldSchema {
        FieldSchema {
            name: name.to_string(),
            description: "Test field".to_string(),
            field_type: FieldType::String,
            required,
            default: None,
            auto: None,
            options: vec![],
            placeholder: Some("Enter value".to_string()),
            max_length: None,
            display_order: None,
            user_editable: true,
        }
    }

    #[test]
    fn test_text_input_handles_chars() {
        let mut field = FormField::TextInput {
            value: String::new(),
            cursor_pos: 0,
            placeholder: "test".to_string(),
            max_length: None,
        };

        assert!(field.handle_key(KeyCode::Char('h')));
        assert!(field.handle_key(KeyCode::Char('i')));
        assert_eq!(field.value(), "hi");
    }

    #[test]
    fn test_text_input_respects_max_length() {
        let mut field = FormField::TextInput {
            value: String::new(),
            cursor_pos: 0,
            placeholder: "test".to_string(),
            max_length: Some(3),
        };

        field.handle_key(KeyCode::Char('a'));
        field.handle_key(KeyCode::Char('b'));
        field.handle_key(KeyCode::Char('c'));
        field.handle_key(KeyCode::Char('d')); // Should be ignored
        assert_eq!(field.value(), "abc");
    }

    #[test]
    fn test_enum_select_navigation() {
        let schema = FieldSchema {
            name: "priority".to_string(),
            description: "Priority".to_string(),
            field_type: FieldType::Enum,
            required: true,
            default: Some("P2-medium".to_string()),
            auto: None,
            options: vec![
                "P0-critical".to_string(),
                "P1-high".to_string(),
                "P2-medium".to_string(),
            ],
            placeholder: None,
            max_length: None,
            display_order: None,
            user_editable: true,
        };

        let mut field = FormField::from_schema(&schema);
        assert_eq!(field.value(), "P2-medium");

        field.handle_key(KeyCode::Up);
        assert_eq!(field.value(), "P1-high");

        field.handle_key(KeyCode::Down);
        assert_eq!(field.value(), "P2-medium");
    }

    #[test]
    fn test_form_validation() {
        let schemas = vec![
            make_string_schema("required_field", true),
            make_string_schema("optional_field", false),
        ];

        let mut form = TicketForm::new(schemas);

        // Form is invalid because required field is empty
        assert!(!form.is_valid());

        // Fill in the required field
        if let Some(field) = form.fields.get_mut("required_field") {
            field.set_value("some value");
        }

        // Now form should be valid
        assert!(form.is_valid());
    }
}
