#![allow(dead_code)]

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::centered_rect;

/// Display information for a syncable collection
#[derive(Debug, Clone)]
pub struct SyncableCollectionDisplay {
    pub provider: String,
    pub project_key: String,
    pub collection_name: String,
    pub status_count: usize,
}

impl From<&crate::services::SyncableCollection> for SyncableCollectionDisplay {
    fn from(collection: &crate::services::SyncableCollection) -> Self {
        Self {
            provider: collection.provider.clone(),
            project_key: collection.project_key.clone(),
            collection_name: collection.collection_name.clone(),
            status_count: collection.sync_statuses.len(),
        }
    }
}

/// Result from the sync confirmation dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncConfirmResult {
    /// User confirmed, sync should proceed
    Confirmed,
    /// User cancelled
    Cancelled,
}

/// Dialog for confirming kanban sync of all collections
pub struct SyncConfirmDialog {
    pub visible: bool,
    collections: Vec<SyncableCollectionDisplay>,
    /// Whether sync is in progress
    pub syncing: bool,
    /// Current sync progress (0-indexed)
    current_sync_index: usize,
    /// Total number of collections to sync
    total_collections: usize,
    /// Status message to display
    status_message: Option<String>,
}

impl Default for SyncConfirmDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            collections: Vec::new(),
            syncing: false,
            current_sync_index: 0,
            total_collections: 0,
            status_message: None,
        }
    }

    /// Show the dialog with collections to sync
    pub fn show(&mut self, collections: Vec<crate::services::SyncableCollection>) {
        self.collections = collections
            .iter()
            .map(SyncableCollectionDisplay::from)
            .collect();
        self.total_collections = self.collections.len();
        self.syncing = false;
        self.current_sync_index = 0;
        self.status_message = None;
        self.visible = true;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.collections.clear();
        self.syncing = false;
        self.status_message = None;
    }

    /// Set syncing state with progress
    pub fn set_syncing(&mut self, index: usize, total: usize) {
        self.syncing = true;
        self.current_sync_index = index;
        self.total_collections = total;
        self.status_message = Some(format!("Syncing collection {}/{}...", index + 1, total));
    }

    /// Set completion message
    pub fn set_complete(&mut self, message: &str) {
        self.syncing = false;
        self.status_message = Some(message.to_string());
    }

    /// Handle key input, returns Some(result) if an action was triggered
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Option<SyncConfirmResult> {
        use crossterm::event::KeyCode;

        // Don't handle keys while syncing
        if self.syncing {
            return None;
        }

        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Some(SyncConfirmResult::Confirmed)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.hide();
                Some(SyncConfirmResult::Cancelled)
            }
            _ => None,
        }
    }

    /// Check if dialog has any collections
    pub fn has_collections(&self) -> bool {
        !self.collections.is_empty()
    }

    /// Render the dialog
    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        // Calculate dialog size based on number of collections
        let collection_count = self.collections.len();
        let dialog_height = (40 + collection_count * 3).min(70) as u16;

        let area = centered_rect(55, dialog_height, frame.area());
        frame.render_widget(Clear, area);

        let title = if self.syncing {
            " Kanban Sync [Syncing...] "
        } else {
            " Kanban Sync "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header text
                Constraint::Min(3),    // Collections list
                Constraint::Length(2), // Status message
                Constraint::Length(3), // Footer/buttons
            ])
            .margin(1)
            .split(inner);

        // Header
        let header = Paragraph::new(Line::from(Span::styled(
            "The following collections will be synced:",
            Style::default().fg(Color::White),
        )));
        frame.render_widget(header, chunks[0]);

        // Collections list
        self.render_collections(frame, chunks[1]);

        // Status message
        if let Some(ref message) = self.status_message {
            let status = Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(status, chunks[2]);
        }

        // Footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_collections(&self, frame: &mut Frame, area: Rect) {
        if self.collections.is_empty() {
            let empty_msg = Paragraph::new("No kanban providers configured")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_msg, area);
            return;
        }

        let mut lines = Vec::new();
        for (i, collection) in self.collections.iter().enumerate() {
            // Show syncing indicator for current collection
            let prefix = if self.syncing && i == self.current_sync_index {
                "▶ "
            } else if self.syncing && i < self.current_sync_index {
                "✓ "
            } else {
                "  "
            };

            // Provider badge
            let provider_badge = match collection.provider.as_str() {
                "jira" => Span::styled(" JIRA ", Style::default().bg(Color::Blue).fg(Color::White)),
                "linear" => Span::styled(
                    " LINEAR ",
                    Style::default().bg(Color::Magenta).fg(Color::White),
                ),
                _ => Span::styled(
                    format!(" {} ", collection.provider.to_uppercase()),
                    Style::default().bg(Color::Gray).fg(Color::White),
                ),
            };

            // Status count suffix
            let status_suffix = if collection.status_count > 0 {
                format!(" ({} statuses)", collection.status_count)
            } else {
                " (default)".to_string()
            };

            lines.push(Line::from(vec![
                Span::raw(prefix),
                provider_badge,
                Span::styled(
                    format!(" {} ", collection.project_key),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("→ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &collection.collection_name,
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(status_suffix, Style::default().fg(Color::DarkGray)),
            ]));
        }

        let list = Paragraph::new(lines);
        frame.render_widget(list, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = if self.syncing {
            Line::from(vec![Span::styled(
                " Syncing... Please wait ",
                Style::default().fg(Color::Yellow),
            )])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    " [Y]es, Sync All ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(" [N]o, Cancel ", Style::default().fg(Color::Red)),
            ])
        };

        let footer_para = Paragraph::new(footer).alignment(Alignment::Center);
        frame.render_widget(footer_para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_confirm_dialog_new() {
        let dialog = SyncConfirmDialog::new();
        assert!(!dialog.visible);
        assert!(!dialog.syncing);
        assert!(!dialog.has_collections());
    }

    #[test]
    fn test_sync_confirm_dialog_show_hide() {
        let mut dialog = SyncConfirmDialog::new();

        let collections = vec![crate::services::SyncableCollection {
            provider: "jira".to_string(),
            project_key: "PROJ".to_string(),
            collection_name: "jira-proj".to_string(),
            sync_user_id: "user123".to_string(),
            sync_statuses: vec!["To Do".to_string()],
        }];

        dialog.show(collections);
        assert!(dialog.visible);
        assert!(dialog.has_collections());
        assert!(!dialog.syncing);

        dialog.hide();
        assert!(!dialog.visible);
        assert!(!dialog.has_collections());
    }

    #[test]
    fn test_sync_confirm_dialog_key_handling() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.show(vec![]);

        // Y key should confirm
        let result = dialog.handle_key(crossterm::event::KeyCode::Char('y'));
        assert_eq!(result, Some(SyncConfirmResult::Confirmed));

        // Reset dialog
        dialog.visible = true;

        // N key should cancel
        let result = dialog.handle_key(crossterm::event::KeyCode::Char('n'));
        assert_eq!(result, Some(SyncConfirmResult::Cancelled));
        assert!(!dialog.visible); // Should be hidden
    }

    #[test]
    fn test_sync_confirm_dialog_syncing_blocks_keys() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.show(vec![]);
        dialog.syncing = true;

        // Keys should be blocked while syncing
        let result = dialog.handle_key(crossterm::event::KeyCode::Char('y'));
        assert!(result.is_none());
    }

    #[test]
    fn test_sync_confirm_dialog_set_syncing() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.show(vec![]);

        dialog.set_syncing(1, 3);
        assert!(dialog.syncing);
    }

    #[test]
    fn test_sync_confirm_dialog_set_complete() {
        let mut dialog = SyncConfirmDialog::new();
        dialog.syncing = true;

        dialog.set_complete("Sync complete: 5 tickets created");
        assert!(!dialog.syncing);
    }
}
