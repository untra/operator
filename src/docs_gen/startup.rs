//! Documentation generator for startup/setup wizard steps.

use super::markdown::{heading, table};
use super::{format_header, DocGenerator};
use crate::startup::SETUP_STEPS;
use anyhow::Result;

/// Generates setup wizard documentation from the startup step registry
pub struct StartupDocGenerator;

impl DocGenerator for StartupDocGenerator {
    fn name(&self) -> &'static str {
        "startup"
    }

    fn source(&self) -> &'static str {
        "src/startup/mod.rs"
    }

    fn output_path(&self) -> &'static str {
        "startup/index.md"
    }

    fn generate(&self) -> Result<String> {
        let mut output = format_header("Setup Wizard", self.source());

        // Introduction
        output.push_str(&heading(1, "Setup Wizard"));
        output.push_str("When Operator starts and no `.tickets/` directory exists, ");
        output.push_str("the setup wizard guides you through first-time initialization. ");
        output.push_str("This reference documents each step of the wizard.\n\n");

        // Quick reference table
        output.push_str(&heading(2, "Steps Overview"));
        output.push_str(&self.generate_overview_table());

        // Detailed sections for each step
        output.push_str(&heading(2, "Step Details"));
        output.push('\n');

        for (i, step) in SETUP_STEPS.iter().enumerate() {
            output.push_str(&heading(3, &format!("{}. {}", i + 1, step.name)));
            output.push_str(&format!("*{}*\n\n", step.description));
            output.push_str(step.help_text);
            output.push_str("\n\n");
            output.push_str(&format!("**Navigation**: {}\n\n", step.navigation));
        }

        // Keyboard shortcuts summary
        output.push_str(&heading(2, "Keyboard Shortcuts"));
        output.push_str("Common keys used throughout the setup wizard:\n\n");
        let shortcut_headers = &["Key", "Action"];
        let shortcut_rows = vec![
            vec!["`Enter`".to_string(), "Confirm/Continue".to_string()],
            vec!["`Esc`".to_string(), "Go back/Cancel".to_string()],
            vec![
                "`↑`/`↓` or `j`/`k`".to_string(),
                "Navigate list items".to_string(),
            ],
            vec!["`Space`".to_string(), "Toggle selection".to_string()],
            vec!["`Tab`".to_string(), "Switch between options".to_string()],
        ];
        output.push_str(&table(shortcut_headers, &shortcut_rows));

        Ok(output)
    }
}

impl StartupDocGenerator {
    fn generate_overview_table(&self) -> String {
        let headers = &["Step", "Name", "Description"];
        let rows: Vec<Vec<String>> = SETUP_STEPS
            .iter()
            .enumerate()
            .map(|(i, step)| {
                vec![
                    format!("{}", i + 1),
                    step.name.to_string(),
                    step.description.to_string(),
                ]
            })
            .collect();

        table(headers, &rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = StartupDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("startup/mod.rs"));

        // Should have the main heading
        assert!(result.contains("# Setup Wizard"));

        // Should have overview section
        assert!(result.contains("## Steps Overview"));

        // Should have details section
        assert!(result.contains("## Step Details"));

        // Should have keyboard shortcuts
        assert!(result.contains("## Keyboard Shortcuts"));
    }

    #[test]
    fn test_all_steps_documented() {
        let generator = StartupDocGenerator;
        let result = generator.generate().unwrap();

        // Check that all step names appear in the documentation
        for step in SETUP_STEPS {
            assert!(
                result.contains(step.name),
                "Step '{}' should be documented",
                step.name
            );
        }
    }

    #[test]
    fn test_overview_table_has_all_steps() {
        let generator = StartupDocGenerator;
        let overview = generator.generate_overview_table();

        // Count table rows (lines starting with |)
        let row_count = overview.lines().filter(|l| l.starts_with('|')).count();
        // Should have header + separator + all steps
        assert_eq!(row_count, SETUP_STEPS.len() + 2);
    }

    #[test]
    fn test_step_numbers_are_sequential() {
        let generator = StartupDocGenerator;
        let result = generator.generate().unwrap();

        // Check that numbered headings exist
        for i in 1..=SETUP_STEPS.len() {
            assert!(
                result.contains(&format!("### {}.", i)),
                "Step {} should be numbered",
                i
            );
        }
    }
}
