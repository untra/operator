mod confirm;
mod help;
mod rejection;
mod session_recovery;
mod sync_confirm;

pub use confirm::{ConfirmDialog, ConfirmDialogFocus, ConfirmSelection, SelectedOption};
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
