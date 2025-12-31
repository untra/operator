//! Git operations module - worktree management and CLI wrapper.
//!
//! This module provides git worktree management following vibe-kanban patterns:
//! - Isolated worktrees per ticket for parallel development
//! - Global locking to prevent race conditions
//! - Comprehensive cleanup on completion

#![allow(dead_code)] // Types are for future integration with step runner
#![allow(unused_imports)]

mod cli;
mod worktree;

pub use cli::GitCli;
pub use worktree::{WorktreeInfo, WorktreeManager};
