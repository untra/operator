//! Centralized keyboard shortcuts registry.
//!
//! This module provides a single source of truth for all keyboard shortcuts
//! used in the Operator TUI. It is consumed by:
//! - `HelpDialog` for displaying help text
//! - `ShortcutsDocGenerator` for generating documentation

use crossterm::event::KeyCode;

/// A keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct Shortcut {
    /// Primary key for this shortcut
    pub key: KeyCode,
    /// Alternative key (e.g., lowercase variant or arrow key)
    pub alt_key: Option<KeyCode>,
    /// Human-readable description of what this shortcut does
    pub description: &'static str,
    /// Category for grouping in help/docs
    pub category: ShortcutCategory,
    /// Context where this shortcut is active
    pub context: ShortcutContext,
}

/// Categories for organizing shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutCategory {
    General,
    Navigation,
    Actions,
    Dialogs,
}

/// Contexts where shortcuts are active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    /// Active in the main dashboard
    Global,
    /// Active in session preview mode
    Preview,
    /// Active in the launch confirmation dialog
    LaunchDialog,
}

impl ShortcutCategory {
    /// Display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            ShortcutCategory::General => "General",
            ShortcutCategory::Navigation => "Navigation",
            ShortcutCategory::Actions => "Actions",
            ShortcutCategory::Dialogs => "Dialogs",
        }
    }

    /// All categories in display order
    pub fn all() -> &'static [ShortcutCategory] {
        &[
            ShortcutCategory::General,
            ShortcutCategory::Navigation,
            ShortcutCategory::Actions,
            ShortcutCategory::Dialogs,
        ]
    }
}

impl ShortcutContext {
    /// Display name for this context
    pub fn display_name(&self) -> &'static str {
        match self {
            ShortcutContext::Global => "Dashboard",
            ShortcutContext::Preview => "Session Preview",
            ShortcutContext::LaunchDialog => "Launch Dialog",
        }
    }

    /// All contexts in display order
    pub fn all() -> &'static [ShortcutContext] {
        &[
            ShortcutContext::Global,
            ShortcutContext::Preview,
            ShortcutContext::LaunchDialog,
        ]
    }
}

impl Shortcut {
    /// Format key for display (e.g., "q", "Tab", "j/↓")
    pub fn key_display(&self) -> String {
        let primary = format_keycode(&self.key);
        match &self.alt_key {
            Some(alt) => format!("{}/{}", primary, format_keycode(alt)),
            None => primary,
        }
    }

    /// Format key for help dialog (left-padded to 7 chars)
    pub fn key_display_padded(&self) -> String {
        format!("{:<7}", self.key_display())
    }
}

/// Format a KeyCode for display
fn format_keycode(key: &KeyCode) -> String {
    match key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "Shift+Tab".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => format!("{:?}", key),
    }
}

/// Static registry of all keyboard shortcuts
pub static SHORTCUTS: &[Shortcut] = &[
    // === Global Context ===
    // General
    Shortcut {
        key: KeyCode::Char('q'),
        alt_key: None,
        description: "Quit Operator",
        category: ShortcutCategory::General,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('?'),
        alt_key: None,
        description: "Toggle help",
        category: ShortcutCategory::General,
        context: ShortcutContext::Global,
    },
    // Navigation
    Shortcut {
        key: KeyCode::Tab,
        alt_key: None,
        description: "Switch between panels",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('j'),
        alt_key: Some(KeyCode::Down),
        description: "Move down",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('k'),
        alt_key: Some(KeyCode::Up),
        description: "Move up",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('Q'),
        alt_key: None,
        description: "Focus Queue panel",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('A'),
        alt_key: Some(KeyCode::Char('a')),
        description: "Focus Agents panel",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Global,
    },
    // Actions
    Shortcut {
        key: KeyCode::Enter,
        alt_key: None,
        description: "Select / Confirm",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Esc,
        alt_key: None,
        description: "Cancel / Close",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('L'),
        alt_key: Some(KeyCode::Char('l')),
        description: "Launch selected ticket",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('P'),
        alt_key: Some(KeyCode::Char('p')),
        description: "Pause queue processing",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('R'),
        alt_key: Some(KeyCode::Char('r')),
        description: "Resume queue processing",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('S'),
        alt_key: None,
        description: "Manual sync (rate limits + sessions)",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('W'),
        alt_key: Some(KeyCode::Char('w')),
        description: "Toggle Backstage server",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('V'),
        alt_key: Some(KeyCode::Char('v')),
        description: "Show session preview",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Global,
    },
    // Dialogs
    Shortcut {
        key: KeyCode::Char('C'),
        alt_key: None,
        description: "Create new ticket",
        category: ShortcutCategory::Dialogs,
        context: ShortcutContext::Global,
    },
    Shortcut {
        key: KeyCode::Char('J'),
        alt_key: None,
        description: "Open Projects menu",
        category: ShortcutCategory::Dialogs,
        context: ShortcutContext::Global,
    },
    // === Preview Context ===
    Shortcut {
        key: KeyCode::Char('g'),
        alt_key: None,
        description: "Scroll to top",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Preview,
    },
    Shortcut {
        key: KeyCode::Char('G'),
        alt_key: None,
        description: "Scroll to bottom",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Preview,
    },
    Shortcut {
        key: KeyCode::PageUp,
        alt_key: None,
        description: "Page up",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Preview,
    },
    Shortcut {
        key: KeyCode::PageDown,
        alt_key: None,
        description: "Page down",
        category: ShortcutCategory::Navigation,
        context: ShortcutContext::Preview,
    },
    Shortcut {
        key: KeyCode::Esc,
        alt_key: Some(KeyCode::Char('q')),
        description: "Close preview",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::Preview,
    },
    // === Launch Dialog Context ===
    Shortcut {
        key: KeyCode::Char('Y'),
        alt_key: Some(KeyCode::Char('y')),
        description: "Launch agent",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::LaunchDialog,
    },
    Shortcut {
        key: KeyCode::Char('V'),
        alt_key: Some(KeyCode::Char('v')),
        description: "View ticket ($VISUAL or open)",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::LaunchDialog,
    },
    Shortcut {
        key: KeyCode::Char('E'),
        alt_key: Some(KeyCode::Char('e')),
        description: "Edit ticket ($EDITOR)",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::LaunchDialog,
    },
    Shortcut {
        key: KeyCode::Char('N'),
        alt_key: Some(KeyCode::Char('n')),
        description: "Cancel",
        category: ShortcutCategory::Actions,
        context: ShortcutContext::LaunchDialog,
    },
];

/// Get all shortcuts for a given context
#[allow(dead_code)]
pub fn shortcuts_for_context(context: ShortcutContext) -> impl Iterator<Item = &'static Shortcut> {
    SHORTCUTS.iter().filter(move |s| s.context == context)
}

/// Get shortcuts grouped by category for a given context
pub fn shortcuts_by_category_for_context(
    context: ShortcutContext,
) -> Vec<(ShortcutCategory, Vec<&'static Shortcut>)> {
    let mut result = Vec::new();
    for category in ShortcutCategory::all() {
        let shortcuts: Vec<&Shortcut> = SHORTCUTS
            .iter()
            .filter(|s| s.context == context && s.category == *category)
            .collect();
        if !shortcuts.is_empty() {
            result.push((*category, shortcuts));
        }
    }
    result
}

/// Grouped shortcuts by category
pub type GroupedByCategory = Vec<(ShortcutCategory, Vec<&'static Shortcut>)>;

/// Get all shortcuts grouped by context, then by category
pub fn all_shortcuts_grouped() -> Vec<(ShortcutContext, GroupedByCategory)> {
    ShortcutContext::all()
        .iter()
        .map(|ctx| (*ctx, shortcuts_by_category_for_context(*ctx)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_shortcuts_have_descriptions() {
        for shortcut in SHORTCUTS {
            assert!(
                !shortcut.description.is_empty(),
                "Shortcut {:?} has empty description",
                shortcut.key
            );
        }
    }

    #[test]
    fn test_key_display_single_key() {
        let shortcut = Shortcut {
            key: KeyCode::Char('q'),
            alt_key: None,
            description: "Test",
            category: ShortcutCategory::General,
            context: ShortcutContext::Global,
        };
        assert_eq!(shortcut.key_display(), "q");
    }

    #[test]
    fn test_key_display_with_alt() {
        let shortcut = Shortcut {
            key: KeyCode::Char('j'),
            alt_key: Some(KeyCode::Down),
            description: "Test",
            category: ShortcutCategory::Navigation,
            context: ShortcutContext::Global,
        };
        assert_eq!(shortcut.key_display(), "j/↓");
    }

    #[test]
    fn test_key_display_special_keys() {
        assert_eq!(format_keycode(&KeyCode::Enter), "Enter");
        assert_eq!(format_keycode(&KeyCode::Esc), "Esc");
        assert_eq!(format_keycode(&KeyCode::Tab), "Tab");
        assert_eq!(format_keycode(&KeyCode::PageUp), "PgUp");
        assert_eq!(format_keycode(&KeyCode::PageDown), "PgDn");
    }

    #[test]
    fn test_shortcuts_for_context() {
        let global_shortcuts: Vec<_> = shortcuts_for_context(ShortcutContext::Global).collect();
        assert!(!global_shortcuts.is_empty());
        assert!(global_shortcuts
            .iter()
            .all(|s| s.context == ShortcutContext::Global));
    }

    #[test]
    fn test_shortcuts_by_category_for_context() {
        let grouped = shortcuts_by_category_for_context(ShortcutContext::Global);
        assert!(!grouped.is_empty());
        // Should have at least General, Navigation, Actions
        let categories: Vec<_> = grouped.iter().map(|(cat, _)| cat).collect();
        assert!(categories.contains(&&ShortcutCategory::General));
        assert!(categories.contains(&&ShortcutCategory::Navigation));
        assert!(categories.contains(&&ShortcutCategory::Actions));
    }

    #[test]
    fn test_category_display_names() {
        assert_eq!(ShortcutCategory::General.display_name(), "General");
        assert_eq!(ShortcutCategory::Navigation.display_name(), "Navigation");
        assert_eq!(ShortcutCategory::Actions.display_name(), "Actions");
        assert_eq!(ShortcutCategory::Dialogs.display_name(), "Dialogs");
    }

    #[test]
    fn test_context_display_names() {
        assert_eq!(ShortcutContext::Global.display_name(), "Dashboard");
        assert_eq!(ShortcutContext::Preview.display_name(), "Session Preview");
        assert_eq!(
            ShortcutContext::LaunchDialog.display_name(),
            "Launch Dialog"
        );
    }

    #[test]
    fn test_all_shortcuts_grouped() {
        let grouped = all_shortcuts_grouped();
        assert_eq!(grouped.len(), 3); // Global, Preview, LaunchDialog
    }
}
