#![allow(unused_imports)]

pub mod collection_dialog;
pub mod create_dialog;
pub mod dashboard;
pub mod dialogs;
pub mod form_field;
pub mod kanban_view;
pub mod keybindings;
pub mod paginated_list;
mod panels;
pub mod projects_dialog;
pub mod session_preview;
pub mod setup;
pub mod terminal_suspend;

pub use collection_dialog::{CollectionInfo, CollectionSwitchDialog, CollectionSwitchResult};
pub use dashboard::Dashboard;
pub use dialogs::{
    ConfirmDialog, ConfirmDialogFocus, ConfirmSelection, RejectionDialog, RejectionResult,
    SelectedOption, SessionRecoveryDialog, SessionRecoverySelection, SyncConfirmDialog,
    SyncConfirmResult,
};
pub use kanban_view::{KanbanView, KanbanViewResult};
pub use paginated_list::{render_paginated_list, PaginatedList};
pub use projects_dialog::ProjectsDialog;
pub use session_preview::SessionPreview;
pub use terminal_suspend::with_suspended_tui;
