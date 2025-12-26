//! Documentation generator for keyboard shortcuts.

use super::markdown::{heading, table};
use super::{format_header, DocGenerator};
use crate::ui::keybindings::{all_shortcuts_grouped, ShortcutContext, SHORTCUTS};
use anyhow::Result;

/// Generates keyboard shortcuts documentation from the keybindings registry
pub struct ShortcutsDocGenerator;

impl DocGenerator for ShortcutsDocGenerator {
    fn name(&self) -> &'static str {
        "shortcuts"
    }

    fn source(&self) -> &'static str {
        "src/ui/keybindings.rs"
    }

    fn output_path(&self) -> &'static str {
        "shortcuts/index.md"
    }

    fn generate(&self) -> Result<String> {
        let mut output = format_header("Keyboard Shortcuts", self.source());

        // Introduction
        output.push_str(&heading(1, "Keyboard Shortcuts"));
        output.push_str("Operator uses vim-style keybindings for navigation and actions. ");
        output.push_str("This reference documents all available keyboard shortcuts.\n\n");

        // Quick reference table
        output.push_str(&heading(2, "Quick Reference"));
        output.push_str(&self.generate_quick_reference());

        // Detailed sections by context
        for (context, categories) in all_shortcuts_grouped() {
            output.push_str(&heading(2, context.display_name()));
            output.push_str(self.generate_context_description(&context));
            output.push_str("\n\n");

            for (category, shortcuts) in categories {
                output.push_str(&heading(3, category.display_name()));

                let headers = &["Key", "Action"];
                let rows: Vec<Vec<String>> = shortcuts
                    .iter()
                    .map(|s| vec![format!("`{}`", s.key_display()), s.description.to_string()])
                    .collect();

                output.push_str(&table(headers, &rows));
            }
        }

        Ok(output)
    }
}

impl ShortcutsDocGenerator {
    fn generate_quick_reference(&self) -> String {
        let headers = &["Key", "Action", "Context"];
        let rows: Vec<Vec<String>> = SHORTCUTS
            .iter()
            .map(|s| {
                vec![
                    format!("`{}`", s.key_display()),
                    s.description.to_string(),
                    s.context.display_name().to_string(),
                ]
            })
            .collect();

        table(headers, &rows)
    }

    fn generate_context_description(&self, context: &ShortcutContext) -> &'static str {
        match context {
            ShortcutContext::Global => "These shortcuts are available in the main dashboard view.",
            ShortcutContext::Preview => {
                "These shortcuts are available when viewing a session preview."
            }
            ShortcutContext::LaunchDialog => {
                "These shortcuts are available in the ticket launch confirmation dialog."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = ShortcutsDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("keybindings.rs"));

        // Should have the main heading
        assert!(result.contains("# Keyboard Shortcuts"));

        // Should have quick reference section
        assert!(result.contains("## Quick Reference"));

        // Should have context sections
        assert!(result.contains("## Dashboard"));
        assert!(result.contains("## Session Preview"));
        assert!(result.contains("## Launch Dialog"));
    }

    #[test]
    fn test_all_shortcuts_documented() {
        let generator = ShortcutsDocGenerator;
        let result = generator.generate().unwrap();

        // Check that key shortcuts appear in the documentation
        assert!(result.contains("`q`"));
        assert!(result.contains("`Tab`"));
        assert!(result.contains("`j/↓`"));
        assert!(result.contains("`Enter`"));
        assert!(result.contains("`L/l`"));
    }

    #[test]
    fn test_categories_have_headings() {
        let generator = ShortcutsDocGenerator;
        let result = generator.generate().unwrap();

        // Should have category headings
        assert!(result.contains("### General"));
        assert!(result.contains("### Navigation"));
        assert!(result.contains("### Actions"));
    }

    #[test]
    fn test_quick_reference_has_all_shortcuts() {
        let generator = ShortcutsDocGenerator;
        let quick_ref = generator.generate_quick_reference();

        // Quick reference should have the expected number of rows
        // Count table rows (lines starting with |)
        let row_count = quick_ref.lines().filter(|l| l.starts_with('|')).count();
        // Should have header + separator + all shortcuts
        assert_eq!(row_count, SHORTCUTS.len() + 2);
    }
}
