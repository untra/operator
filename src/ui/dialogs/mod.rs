mod confirm;
mod help;
mod rejection;
mod session_recovery;
mod sync_confirm;

pub use confirm::{
    ConfirmDialog, ConfirmDialogFocus, ConfirmSelection, SelectedOption, SessionPlacementPreview,
};
pub use help::HelpDialog;
pub use rejection::{RejectionDialog, RejectionResult};
pub use session_recovery::{SessionRecoveryDialog, SessionRecoverySelection};
pub use sync_confirm::{SyncConfirmDialog, SyncConfirmResult, SyncableCollectionDisplay};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
};

/// Helper to create a centered rect
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
    use ratatui::layout::Rect;

    #[test]
    fn test_centered_rect_full_percentage() {
        let area = Rect::new(0, 0, 100, 50);
        let result = centered_rect(100, 100, area);
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn test_centered_rect_50_percent() {
        let area = Rect::new(0, 0, 100, 100);
        let result = centered_rect(50, 50, area);
        assert!(result.x > 0);
        assert!(result.y > 0);
        assert!(result.width > 0);
        assert!(result.height > 0);
        assert!(result.width <= 60);
        assert!(result.height <= 60);
    }

    #[test]
    fn test_centered_rect_zero_size_area() {
        let area = Rect::new(0, 0, 0, 0);
        let result = centered_rect(50, 50, area);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_centered_rect_within_parent() {
        let area = Rect::new(10, 20, 80, 60);
        let result = centered_rect(70, 80, area);
        assert!(result.x >= area.x);
        assert!(result.y >= area.y);
        assert!(result.right() <= area.right());
        assert!(result.bottom() <= area.bottom());
    }
}
