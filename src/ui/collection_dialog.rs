//! Collection switch dialog for changing the active issue type collection

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::issuetypes::{BuiltinPreset, IssueTypeCollection, IssueTypeRegistry};

/// Information about a collection for display
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub description: String,
    pub type_count: usize,
    pub is_builtin: bool,
    /// Provider name if collection was synced from external source
    pub sync_source: Option<String>,
}

impl CollectionInfo {
    fn from_collection(collection: &IssueTypeCollection) -> Self {
        Self {
            name: collection.name.clone(),
            description: collection.description.clone(),
            type_count: collection.types.len(),
            is_builtin: BuiltinPreset::from_name(&collection.name).is_some(),
            sync_source: collection.sync_source.as_ref().map(|s| s.provider.clone()),
        }
    }
}

/// Result when confirming a collection switch
#[derive(Debug, Clone)]
pub struct CollectionSwitchResult {
    /// The collection to switch to
    pub collection_name: String,
    /// Project scope: None = global, Some(project) = per-project preference
    pub project_scope: Option<String>,
}

/// Dialog for switching issue type collections
pub struct CollectionSwitchDialog {
    pub visible: bool,
    /// List of available collections
    collections: Vec<CollectionInfo>,
    /// ratatui list selection state
    list_state: ListState,
    /// Currently active collection name (for highlighting)
    active_collection: String,
    /// Optional project context (for per-project prefs)
    project_context: Option<String>,
}

impl Default for CollectionSwitchDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectionSwitchDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            collections: Vec::new(),
            list_state: ListState::default(),
            active_collection: String::new(),
            project_context: None,
        }
    }

    /// Show the dialog with available collections
    pub fn show(
        &mut self,
        registry: &IssueTypeRegistry,
        current_active: &str,
        project_context: Option<&str>,
    ) {
        self.visible = true;
        self.active_collection = current_active.to_string();
        self.project_context = project_context.map(|s| s.to_string());

        // Build collection info list
        self.collections = registry
            .all_collections()
            .map(CollectionInfo::from_collection)
            .collect();

        // Sort: builtins first, then by name
        self.collections
            .sort_by(|a, b| match (a.is_builtin, b.is_builtin) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            });

        // Select the currently active collection
        let selected = self
            .collections
            .iter()
            .position(|c| c.name == current_active)
            .unwrap_or(0);
        self.list_state.select(Some(selected));
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.collections.clear();
        self.list_state.select(None);
    }

    fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    fn selected_collection(&self) -> Option<&CollectionInfo> {
        self.collections.get(self.selected_index())
    }

    fn select_next(&mut self) {
        if self.collections.is_empty() {
            return;
        }
        let current = self.selected_index();
        let next = (current + 1) % self.collections.len();
        self.list_state.select(Some(next));
    }

    fn select_prev(&mut self) {
        if self.collections.is_empty() {
            return;
        }
        let current = self.selected_index();
        let prev = if current == 0 {
            self.collections.len() - 1
        } else {
            current - 1
        };
        self.list_state.select(Some(prev));
    }

    /// Handle key input, returns Some(result) if a collection was selected
    pub fn handle_key(&mut self, key: KeyCode) -> Option<CollectionSwitchResult> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Enter => self.confirm_switch(false),
            KeyCode::Char('g') => self.confirm_switch(true), // Set as global default
            KeyCode::Esc | KeyCode::Char('q') => {
                self.hide();
                None
            }
            _ => None,
        }
    }

    /// Confirm the switch, optionally forcing global scope
    fn confirm_switch(&mut self, force_global: bool) -> Option<CollectionSwitchResult> {
        let collection = self.selected_collection()?;

        let result = CollectionSwitchResult {
            collection_name: collection.name.clone(),
            project_scope: if force_global {
                None
            } else {
                self.project_context.clone()
            },
        };

        self.hide();
        Some(result)
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        // Center the dialog
        let area = centered_rect(55, 60, frame.area());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Switch Collection ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Layout: header, list, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Context info
                Constraint::Min(8),    // Collection list
                Constraint::Length(3), // Footer with shortcuts
            ])
            .margin(1)
            .split(inner);

        // Render context info
        let context = if let Some(ref proj) = self.project_context {
            format!("Project: {} | Active: {}", proj, self.active_collection)
        } else {
            format!("Active: {}", self.active_collection)
        };
        frame.render_widget(
            Paragraph::new(context).style(Style::default().fg(Color::Gray)),
            chunks[0],
        );

        // Render collection list
        let items: Vec<ListItem> = self
            .collections
            .iter()
            .map(|c| {
                let marker = if c.name == self.active_collection {
                    " * "
                } else {
                    "   "
                };

                let sync_badge = c
                    .sync_source
                    .as_ref()
                    .map(|s| format!(" [{}]", s))
                    .unwrap_or_default();

                let builtin_badge = if c.is_builtin { "" } else { " (custom)" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(marker),
                        Span::styled(&c.name, Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(sync_badge, Style::default().fg(Color::Cyan)),
                        Span::styled(builtin_badge, Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!(" ({} types)", c.type_count),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(&c.description, Style::default().fg(Color::Gray)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);

        // Footer with shortcuts
        let footer_text = if self.project_context.is_some() {
            vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" for project  "),
                Span::styled("g", Style::default().fg(Color::Yellow)),
                Span::raw(" global  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" cancel"),
            ]
        } else {
            vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" switch  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" cancel"),
            ]
        };

        let footer = Paragraph::new(Line::from(footer_text)).alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }
}

/// Helper function to center a rect within another rect
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

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
    fn test_collection_dialog_new() {
        let dialog = CollectionSwitchDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.collections.is_empty());
    }

    #[test]
    fn test_collection_info_from_collection() {
        use crate::issuetypes::IssueTypeCollection;

        let collection =
            IssueTypeCollection::new("test", "Test collection").with_types(["FEAT", "FIX"]);

        let info = CollectionInfo::from_collection(&collection);
        assert_eq!(info.name, "test");
        assert_eq!(info.description, "Test collection");
        assert_eq!(info.type_count, 2);
        assert!(!info.is_builtin);
        assert!(info.sync_source.is_none());
    }

    #[test]
    fn test_collection_info_builtin_detection() {
        use crate::issuetypes::IssueTypeCollection;

        let builtin = IssueTypeCollection::new("dev_kanban", "Dev Kanban");
        let info = CollectionInfo::from_collection(&builtin);
        assert!(info.is_builtin);

        let custom = IssueTypeCollection::new("my_workflow", "Custom");
        let info = CollectionInfo::from_collection(&custom);
        assert!(!info.is_builtin);
    }

    #[test]
    fn test_dialog_navigation() {
        let mut dialog = CollectionSwitchDialog::new();
        dialog.collections = vec![
            CollectionInfo {
                name: "a".to_string(),
                description: "".to_string(),
                type_count: 1,
                is_builtin: true,
                sync_source: None,
            },
            CollectionInfo {
                name: "b".to_string(),
                description: "".to_string(),
                type_count: 2,
                is_builtin: false,
                sync_source: None,
            },
        ];
        dialog.list_state.select(Some(0));
        dialog.visible = true;

        // Navigate down
        dialog.select_next();
        assert_eq!(dialog.selected_index(), 1);

        // Navigate down wraps
        dialog.select_next();
        assert_eq!(dialog.selected_index(), 0);

        // Navigate up wraps
        dialog.select_prev();
        assert_eq!(dialog.selected_index(), 1);
    }

    #[test]
    fn test_confirm_switch_global() {
        let mut dialog = CollectionSwitchDialog::new();
        dialog.collections = vec![CollectionInfo {
            name: "test".to_string(),
            description: "".to_string(),
            type_count: 1,
            is_builtin: false,
            sync_source: None,
        }];
        dialog.list_state.select(Some(0));
        dialog.visible = true;
        dialog.project_context = Some("myproject".to_string());

        // Confirm with global flag
        let result = dialog.confirm_switch(true);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.collection_name, "test");
        assert!(result.project_scope.is_none()); // Global, not per-project
    }

    #[test]
    fn test_confirm_switch_project() {
        let mut dialog = CollectionSwitchDialog::new();
        dialog.collections = vec![CollectionInfo {
            name: "test".to_string(),
            description: "".to_string(),
            type_count: 1,
            is_builtin: false,
            sync_source: None,
        }];
        dialog.list_state.select(Some(0));
        dialog.visible = true;
        dialog.project_context = Some("myproject".to_string());

        // Confirm without global flag
        let result = dialog.confirm_switch(false);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.collection_name, "test");
        assert_eq!(result.project_scope, Some("myproject".to_string()));
    }
}
