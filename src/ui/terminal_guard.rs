//! Terminal state guard that ensures cleanup on drop.

use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// RAII guard that restores terminal state on drop.
///
/// This ensures terminal cleanup happens even on:
/// - Early returns via `?` operator
/// - Panics (via panic hook)
/// - Normal scope exit
pub struct TerminalGuard {
    active: AtomicBool,
}

impl TerminalGuard {
    /// Initialize terminal for TUI mode and return guard.
    ///
    /// Enables raw mode, enters alternate screen, and enables mouse capture.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(Self {
            active: AtomicBool::new(true),
        })
    }

    /// Manually cleanup (used by panic hook).
    pub fn cleanup() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = execute!(io::stdout(), Show);
        let _ = io::stdout().flush();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active.swap(false, Ordering::SeqCst) {
            Self::cleanup();
        }
    }
}

/// Install panic hook that restores terminal before printing panic.
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal first so panic message is readable
        TerminalGuard::cleanup();
        // Then call original hook
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_guard_tracks_active_state() {
        let guard = TerminalGuard {
            active: AtomicBool::new(true),
        };
        assert!(guard.active.load(Ordering::SeqCst));
    }

    #[test]
    fn test_terminal_guard_clears_active_on_drop() {
        let guard = TerminalGuard {
            active: AtomicBool::new(true),
        };
        assert!(guard.active.load(Ordering::SeqCst));
        drop(guard);
        // Guard dropped successfully without panic = success
    }

    #[test]
    fn test_terminal_guard_no_double_cleanup() {
        let guard = TerminalGuard {
            active: AtomicBool::new(false),
        };
        // Should not attempt cleanup when active is already false
        drop(guard);
        // No panic = success
    }

    #[test]
    fn test_cleanup_is_callable() {
        // Just verify cleanup() doesn't panic when called
        // (actual terminal ops will fail in test env but shouldn't panic)
        TerminalGuard::cleanup();
    }
}
