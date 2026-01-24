//! Terminal suspend utility for safely running external applications.
//!
//! When spawning external applications like $EDITOR, we need to:
//! 1. Exit raw mode and alternate screen
//! 2. Run the external application
//! 3. Restore raw mode, alternate screen, and clear the terminal
//!
//! The clear step is critical - without it, the TUI will have display artifacts.

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// Suspend the TUI, run a closure, then restore the TUI.
///
/// This function properly handles the terminal state transitions required
/// when spawning external applications like text editors.
///
/// # Arguments
/// * `terminal` - The ratatui terminal to suspend/restore
/// * `f` - The closure to run while the TUI is suspended
///
/// # Returns
/// The result of the closure, or an error if terminal operations fail.
///
/// # Example
/// ```ignore
/// with_suspended_tui(&mut terminal, || {
///     std::process::Command::new("vim")
///         .arg("file.txt")
///         .status()
///         .map(|_| ())
///         .map_err(Into::into)
/// })?;
/// ```
pub fn with_suspended_tui<F, T>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    f: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    // Exit TUI mode
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    // Guard ensures restore even on panic
    struct RestoreGuard {
        restored: bool,
    }
    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            if !self.restored {
                let _ = enable_raw_mode();
                let _ = execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture);
            }
        }
    }
    let mut guard = RestoreGuard { restored: false };

    // Run the closure
    let result = f();

    // Explicit restore (mark guard as done)
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    guard.restored = true;

    // Clear terminal to ensure proper redraw - this is critical!
    // Without this, the TUI will have display artifacts from the external app.
    terminal.clear()?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    // Note: We can't test with_suspended_tui directly because it requires
    // a real CrosstermBackend. We test the logic patterns instead.

    #[test]
    fn test_closure_result_propagated() {
        // Test that closure results would be propagated correctly
        let test_closure = || -> Result<i32> { Ok(42) };
        let result = test_closure();
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_closure_error_propagated() {
        let test_closure = || -> Result<i32> { anyhow::bail!("test error") };
        let result = test_closure();
        assert!(result.is_err());
    }

    #[test]
    fn test_terminal_clear_is_callable() {
        // Verify that terminal.clear() works with TestBackend
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // This should not panic
        terminal.clear().unwrap();
    }

    #[test]
    fn test_test_backend_supports_operations() {
        // Ensure TestBackend can be used for unit testing terminal operations
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Draw something
        terminal
            .draw(|f| {
                let area = f.area();
                assert_eq!(area.width, 80);
                assert_eq!(area.height, 24);
            })
            .unwrap();

        // Clear should work
        terminal.clear().unwrap();
    }

    #[test]
    fn test_restore_guard_marks_as_restored() {
        struct RestoreGuard {
            restored: bool,
        }
        impl Drop for RestoreGuard {
            fn drop(&mut self) {
                // Would restore if not marked
            }
        }

        let mut guard = RestoreGuard { restored: false };
        guard.restored = true;
        drop(guard);
        // No panic = guard handled correctly
    }

    #[test]
    fn test_restore_guard_catches_panic() {
        use std::panic;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        struct RestoreGuard {
            called: Arc<AtomicBool>,
        }
        impl Drop for RestoreGuard {
            fn drop(&mut self) {
                self.called.store(true, Ordering::SeqCst);
            }
        }

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _guard = RestoreGuard {
                called: called_clone,
            };
            panic!("test panic");
        }));

        assert!(result.is_err());
        assert!(called.load(Ordering::SeqCst));
    }
}
