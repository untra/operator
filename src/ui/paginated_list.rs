//! Reusable paginated list widget for displaying long lists with navigation
//!
//! This module provides a paginated list widget for ratatui that can display
//! items across multiple pages with navigation support. Used in the kanban
//! setup flow for project selection.

#![allow(dead_code)] // Methods are infrastructure for kanban setup flow

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
    Frame,
};

/// A paginated list that can display items across multiple pages
pub struct PaginatedList<T> {
    /// All items in the list
    items: Vec<T>,
    /// Current page (0-indexed)
    page: usize,
    /// Number of items per page
    page_size: usize,
    /// Currently selected index within the current page
    selected: usize,
    /// List state for rendering
    list_state: ListState,
}

impl<T> Default for PaginatedList<T> {
    fn default() -> Self {
        Self::new(10)
    }
}

impl<T> PaginatedList<T> {
    /// Create a new paginated list with the given page size
    pub fn new(page_size: usize) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            items: Vec::new(),
            page: 0,
            page_size,
            selected: 0,
            list_state,
        }
    }

    /// Set the items in the list
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.page = 0;
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    /// Get the total number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the total number of pages
    pub fn total_pages(&self) -> usize {
        if self.items.is_empty() {
            1
        } else {
            self.items.len().div_ceil(self.page_size)
        }
    }

    /// Get the current page number (1-indexed for display)
    pub fn current_page(&self) -> usize {
        self.page + 1
    }

    /// Get the items on the current page
    pub fn current_page_items(&self) -> &[T] {
        let start = self.page * self.page_size;
        let end = (start + self.page_size).min(self.items.len());
        if start >= self.items.len() {
            &[]
        } else {
            &self.items[start..end]
        }
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&T> {
        let global_index = self.page * self.page_size + self.selected;
        self.items.get(global_index)
    }

    /// Get the global index of the selected item
    pub fn selected_index(&self) -> usize {
        self.page * self.page_size + self.selected
    }

    /// Move selection to the next item
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let page_items = self.current_page_items().len();
        if page_items == 0 {
            return;
        }

        if self.selected + 1 < page_items {
            self.selected += 1;
        } else if self.page + 1 < self.total_pages() {
            // Move to next page
            self.page += 1;
            self.selected = 0;
        } else {
            // Wrap to first page
            self.page = 0;
            self.selected = 0;
        }
        self.list_state.select(Some(self.selected));
    }

    /// Move selection to the previous item
    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }

        if self.selected > 0 {
            self.selected -= 1;
        } else if self.page > 0 {
            // Move to previous page
            self.page -= 1;
            let page_items = self.current_page_items().len();
            self.selected = page_items.saturating_sub(1);
        } else {
            // Wrap to last page
            self.page = self.total_pages().saturating_sub(1);
            let page_items = self.current_page_items().len();
            self.selected = page_items.saturating_sub(1);
        }
        self.list_state.select(Some(self.selected));
    }

    /// Move to the next page
    pub fn next_page(&mut self) {
        if self.page + 1 < self.total_pages() {
            self.page += 1;
            self.selected = 0;
            self.list_state.select(Some(0));
        }
    }

    /// Move to the previous page
    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
            self.selected = 0;
            self.list_state.select(Some(0));
        }
    }

    /// Get the list state for rendering
    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    /// Create a footer line showing page info
    pub fn footer_line(&self) -> Line<'static> {
        if self.total_pages() <= 1 {
            Line::from(vec![Span::styled(
                format!("{} items", self.items.len()),
                Style::default().fg(Color::DarkGray),
            )])
        } else {
            Line::from(vec![
                Span::styled(
                    format!("Page {}/{}", self.current_page(), self.total_pages()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled("[n]", Style::default().fg(Color::Yellow)),
                Span::styled(" next  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[p]", Style::default().fg(Color::Yellow)),
                Span::styled(" prev", Style::default().fg(Color::DarkGray)),
            ])
        }
    }
}

/// Render a paginated list with a custom item renderer
pub fn render_paginated_list<T, F>(
    frame: &mut Frame,
    area: Rect,
    list: &mut PaginatedList<T>,
    title: &str,
    item_renderer: F,
) where
    F: Fn(&T, bool) -> ListItem<'static>,
{
    let items: Vec<ListItem> = list
        .current_page_items()
        .iter()
        .enumerate()
        .map(|(i, item)| item_renderer(item, i == list.selected))
        .collect();

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list_widget = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list_widget, area, list.list_state_mut());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_list_empty() {
        let list: PaginatedList<String> = PaginatedList::new(5);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.total_pages(), 1);
        assert!(list.selected_item().is_none());
    }

    #[test]
    fn test_paginated_list_single_page() {
        let mut list: PaginatedList<String> = PaginatedList::new(5);
        list.set_items(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        assert_eq!(list.len(), 3);
        assert_eq!(list.total_pages(), 1);
        assert_eq!(list.current_page(), 1);
        assert_eq!(list.current_page_items().len(), 3);
    }

    #[test]
    fn test_paginated_list_multiple_pages() {
        let mut list: PaginatedList<i32> = PaginatedList::new(3);
        list.set_items(vec![1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(list.total_pages(), 3);
        assert_eq!(list.current_page_items(), &[1, 2, 3]);

        list.next_page();
        assert_eq!(list.current_page(), 2);
        assert_eq!(list.current_page_items(), &[4, 5, 6]);

        list.next_page();
        assert_eq!(list.current_page(), 3);
        assert_eq!(list.current_page_items(), &[7]);
    }

    #[test]
    fn test_paginated_list_selection() {
        let mut list: PaginatedList<&str> = PaginatedList::new(3);
        list.set_items(vec!["a", "b", "c", "d", "e"]);

        assert_eq!(list.selected_item(), Some(&"a"));
        assert_eq!(list.selected_index(), 0);

        list.select_next();
        assert_eq!(list.selected_item(), Some(&"b"));
        assert_eq!(list.selected_index(), 1);

        list.select_next();
        list.select_next();
        // Should move to next page
        assert_eq!(list.current_page(), 2);
        assert_eq!(list.selected_item(), Some(&"d"));
    }

    #[test]
    fn test_paginated_list_wrap_around() {
        let mut list: PaginatedList<i32> = PaginatedList::new(3);
        list.set_items(vec![1, 2, 3, 4, 5]);

        // Go to last item
        list.page = 1;
        list.selected = 1; // item "5"
        list.list_state.select(Some(1));

        // Next should wrap to first
        list.select_next();
        assert_eq!(list.current_page(), 1);
        assert_eq!(list.selected_item(), Some(&1));
    }

    #[test]
    fn test_paginated_list_prev() {
        let mut list: PaginatedList<i32> = PaginatedList::new(3);
        list.set_items(vec![1, 2, 3, 4, 5]);

        // Start at first item
        list.select_prev();
        // Should wrap to last
        assert_eq!(list.current_page(), 2);
        assert_eq!(list.selected_item(), Some(&5));
    }
}
