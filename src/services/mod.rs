//! Background services for operator.
//!
//! This module contains long-running services that operate independently
//! of the main TUI loop.

#![allow(unused_imports)] // Re-exports for future integration

pub mod kanban_sync;
pub mod pr_monitor;

pub use kanban_sync::{KanbanSyncService, SyncResult, SyncableCollection};
pub use pr_monitor::{PrMonitorService, PrStatusEvent, TrackedPr};
