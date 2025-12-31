//! Kanban view for managing external provider collections and syncing work

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::services::SyncableCollection;

/// Information about a syncable collection for display
#[derive(Debug, Clone)]
pub struct KanbanCollectionInfo {
    /// Provider name (e.g., "jira", "linear")
    pub provider: String,
    /// Project/team key
    pub project_key: String,
    /// Collection name in Operator
    pub collection_name: String,
    /// User ID configured for sync (will be displayed when sync UI is expanded)
    #[allow(dead_code)]
    pub sync_user_id: String,
    /// Number of statuses configured
    pub status_count: usize,
}

impl From<&SyncableCollection> for KanbanCollectionInfo {
    fn from(collection: &SyncableCollection) -> Self {
        Self {
            provider: collection.provider.clone(),
            project_key: collection.project_key.clone(),
            collection_name: collection.collection_name.clone(),
            sync_user_id: collection.sync_user_id.clone(),
            status_count: collection.sync_statuses.len(),
        }
    }
}

/// Result of a kanban view action
#[derive(Debug, Clone)]
pub enum KanbanViewResult {
    /// User requested to sync a collection
    Sync {
        provider: String,
        project_key: String,
    },
    /// User dismissed the view
    Dismissed,
}

/// View for managing kanban provider collections
pub struct KanbanView {
    pub visible: bool,
    /// List of configured collections
    collections: Vec<KanbanCollectionInfo>,
    /// ratatui list selection state
    list_state: ListState,
    /// Whether a sync is in progress
    pub syncing: bool,
    /// Status message to display
    pub status_message: Option<String>,
}

impl Default for KanbanView {
    fn default() -> Self {
        Self::new()
    }
}

impl KanbanView {
    pub fn new() -> Self {
        Self {
            visible: false,
            collections: Vec::new(),
            list_state: ListState::default(),
            syncing: false,
            status_message: None,
        }
    }

    /// Show the view with available collections
    pub fn show(&mut self, collections: Vec<SyncableCollection>) {
        self.visible = true;
        self.syncing = false;
        self.status_message = None;

        self.collections = collections.iter().map(KanbanCollectionInfo::from).collect();

        // Sort by provider, then project
        self.collections.sort_by(|a, b| {
            a.provider
                .cmp(&b.provider)
                .then_with(|| a.project_key.cmp(&b.project_key))
        });

        // Select first item if available
        if !self.collections.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Hide the view
    pub fn hide(&mut self) {
        self.visible = false;
        self.collections.clear();
        self.list_state.select(None);
        self.status_message = None;
    }

    /// Set status message
    pub fn set_status(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
    }

    /// Clear status message (used when sync completes)
    #[allow(dead_code)]
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    fn selected_collection(&self) -> Option<&KanbanCollectionInfo> {
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

    /// Handle key input, returns Some(result) if an action was triggered
    pub fn handle_key(&mut self, key: KeyCode) -> Option<KanbanViewResult> {
        // Don't handle keys while syncing
        if self.syncing {
            return None;
        }

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Sync the selected collection
                self.selected_collection()
                    .map(|collection| KanbanViewResult::Sync {
                        provider: collection.provider.clone(),
                        project_key: collection.project_key.clone(),
                    })
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.hide();
                Some(KanbanViewResult::Dismissed)
            }
            _ => None,
        }
    }

    /// Render the kanban view
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate dialog size (60% width, 70% height, centered)
        let dialog_width = (area.width as f32 * 0.6) as u16;
        let dialog_height = (area.height as f32 * 0.7) as u16;
        let dialog_x = (area.width - dialog_width) / 2;
        let dialog_y = (area.height - dialog_height) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Main block
        let title = if self.syncing {
            " Kanban Providers [Syncing...] "
        } else {
            " Kanban Providers "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout: list + footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // List
                Constraint::Length(1), // Status
                Constraint::Length(2), // Footer
            ])
            .split(inner_area);

        // Render collection list
        self.render_list(frame, chunks[0]);

        // Render status message
        if let Some(ref message) = self.status_message {
            let status = Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(status, chunks[1]);
        }

        // Render footer
        self.render_footer(frame, chunks[2]);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        if self.collections.is_empty() {
            let empty_msg = Paragraph::new("No kanban providers configured")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_msg, area);
            return;
        }

        let items: Vec<ListItem> = self
            .collections
            .iter()
            .enumerate()
            .map(|(i, collection)| {
                let is_selected = self.list_state.selected() == Some(i);

                // Provider badge
                let provider_badge = match collection.provider.as_str() {
                    "jira" => {
                        Span::styled(" JIRA ", Style::default().bg(Color::Blue).fg(Color::White))
                    }
                    "linear" => Span::styled(
                        " LINEAR ",
                        Style::default().bg(Color::Magenta).fg(Color::White),
                    ),
                    _ => Span::styled(
                        format!(" {} ", collection.provider.to_uppercase()),
                        Style::default().bg(Color::Gray).fg(Color::White),
                    ),
                };

                // Project key
                let project = Span::styled(
                    format!(" {} ", collection.project_key),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                );

                // Collection name
                let collection_name = Span::styled(
                    format!(" → {} ", collection.collection_name),
                    Style::default().fg(Color::DarkGray),
                );

                // Status count
                let status_info = if collection.status_count > 0 {
                    Span::styled(
                        format!("({} statuses)", collection.status_count),
                        Style::default().fg(Color::DarkGray),
                    )
                } else {
                    Span::styled("(default)", Style::default().fg(Color::DarkGray))
                };

                let line = Line::from(vec![provider_badge, project, collection_name, status_info]);

                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let shortcuts = if self.syncing {
            vec![Span::styled(
                " Syncing... Please wait ",
                Style::default().fg(Color::Yellow),
            )]
        } else {
            vec![
                Span::styled("[S]", Style::default().fg(Color::Cyan)),
                Span::raw("ync  "),
                Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
                Span::raw("Navigate  "),
                Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
                Span::raw("Close"),
            ]
        };

        let footer = Paragraph::new(Line::from(shortcuts))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));

        frame.render_widget(footer, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanban_view_new() {
        let view = KanbanView::new();
        assert!(!view.visible);
        assert!(view.collections.is_empty());
    }

    #[test]
    fn test_kanban_view_show_hide() {
        let mut view = KanbanView::new();

        let collections = vec![SyncableCollection {
            provider: "jira".to_string(),
            project_key: "PROJ".to_string(),
            collection_name: "jira-proj".to_string(),
            sync_user_id: "user123".to_string(),
            sync_statuses: vec!["To Do".to_string()],
        }];

        view.show(collections);
        assert!(view.visible);
        assert_eq!(view.collections.len(), 1);

        view.hide();
        assert!(!view.visible);
        assert!(view.collections.is_empty());
    }

    #[test]
    fn test_kanban_view_navigation() {
        let mut view = KanbanView::new();

        let collections = vec![
            SyncableCollection {
                provider: "jira".to_string(),
                project_key: "PROJ1".to_string(),
                collection_name: "jira-proj1".to_string(),
                sync_user_id: "user1".to_string(),
                sync_statuses: vec![],
            },
            SyncableCollection {
                provider: "linear".to_string(),
                project_key: "ENG".to_string(),
                collection_name: "linear-eng".to_string(),
                sync_user_id: "user2".to_string(),
                sync_statuses: vec![],
            },
        ];

        view.show(collections);

        // Initially at 0
        assert_eq!(view.selected_index(), 0);

        // Navigate down
        view.handle_key(KeyCode::Down);
        assert_eq!(view.selected_index(), 1);

        // Wrap around
        view.handle_key(KeyCode::Down);
        assert_eq!(view.selected_index(), 0);

        // Navigate up
        view.handle_key(KeyCode::Up);
        assert_eq!(view.selected_index(), 1);
    }

    #[test]
    fn test_kanban_view_sync_action() {
        let mut view = KanbanView::new();

        let collections = vec![SyncableCollection {
            provider: "jira".to_string(),
            project_key: "PROJ".to_string(),
            collection_name: "jira-proj".to_string(),
            sync_user_id: "user123".to_string(),
            sync_statuses: vec![],
        }];

        view.show(collections);

        let result = view.handle_key(KeyCode::Char('s'));
        assert!(matches!(
            result,
            Some(KanbanViewResult::Sync { provider, project_key })
            if provider == "jira" && project_key == "PROJ"
        ));
    }

    #[test]
    fn test_kanban_view_escape() {
        let mut view = KanbanView::new();
        view.show(vec![]);

        let result = view.handle_key(KeyCode::Esc);
        assert!(matches!(result, Some(KanbanViewResult::Dismissed)));
        assert!(!view.visible);
    }
}
