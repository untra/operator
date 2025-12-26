//! Documentation generator for CLI reference and environment variables.

use super::markdown::{heading, table};
use super::{format_header, DocGenerator};
use crate::env_vars::{env_vars_by_category, ENV_VARS};
use crate::Cli;
use anyhow::Result;
use clap::CommandFactory;

/// Generates CLI reference documentation from clap definitions and env var registry
pub struct CliDocGenerator;

impl DocGenerator for CliDocGenerator {
    fn name(&self) -> &'static str {
        "cli"
    }

    fn source(&self) -> &'static str {
        "src/main.rs, src/env_vars.rs"
    }

    fn output_path(&self) -> &'static str {
        "cli/index.md"
    }

    fn generate(&self) -> Result<String> {
        let cmd = Cli::command();

        let mut output = format_header("CLI Reference", self.source());

        // Introduction
        output.push_str(&heading(1, "CLI Reference"));
        output.push_str(
            "Operator provides both a TUI dashboard and CLI commands for queue management.\n\n",
        );

        // Global options
        output.push_str(&heading(2, "Global Options"));
        output.push_str(&self.generate_global_options(&cmd));

        // Commands
        output.push_str(&heading(2, "Commands"));
        output.push_str(
            "When run without a command, Operator launches the interactive TUI dashboard.\n\n",
        );

        for subcmd in cmd.get_subcommands() {
            if subcmd.get_name() == "help" {
                continue; // Skip auto-generated help command
            }
            output.push_str(&self.generate_subcommand_docs(subcmd));
        }

        // Environment Variables
        output.push_str(&heading(2, "Environment Variables"));
        output.push_str(&self.generate_env_vars_docs());

        Ok(output)
    }
}

impl CliDocGenerator {
    fn generate_global_options(&self, cmd: &clap::Command) -> String {
        let mut rows: Vec<Vec<String>> = Vec::new();

        for arg in cmd.get_arguments() {
            if arg.is_positional() || arg.get_id() == "help" || arg.get_id() == "version" {
                continue;
            }

            let short = arg
                .get_short()
                .map(|s| format!("-{}", s))
                .unwrap_or_default();
            let long = arg
                .get_long()
                .map(|l| format!("--{}", l))
                .unwrap_or_default();

            let flag = if short.is_empty() {
                long
            } else if long.is_empty() {
                short
            } else {
                format!("{}, {}", short, long)
            };

            let description = arg
                .get_help()
                .map(|h| h.to_string())
                .unwrap_or_else(|| String::from("No description"));

            rows.push(vec![format!("`{}`", flag), description]);
        }

        if rows.is_empty() {
            return String::from("No global options.\n\n");
        }

        let headers = &["Option", "Description"];
        table(headers, &rows)
    }

    fn generate_subcommand_docs(&self, cmd: &clap::Command) -> String {
        let mut output = String::new();

        // Command heading
        output.push_str(&heading(3, &format!("`{}`", cmd.get_name())));

        // Description
        if let Some(about) = cmd.get_about() {
            output.push_str(&format!("{}\n\n", about));
        }

        // Arguments and options
        let mut rows: Vec<Vec<String>> = Vec::new();

        // Positional arguments first
        for arg in cmd.get_arguments() {
            if !arg.is_positional() || arg.get_id() == "help" {
                continue;
            }

            let name = format!("`<{}>`", arg.get_id().as_str().to_uppercase());
            let description = arg
                .get_help()
                .map(|h| h.to_string())
                .unwrap_or_else(|| String::from("No description"));

            rows.push(vec![name, description]);
        }

        // Options
        for arg in cmd.get_arguments() {
            if arg.is_positional() || arg.get_id() == "help" {
                continue;
            }

            let short = arg
                .get_short()
                .map(|s| format!("-{}", s))
                .unwrap_or_default();
            let long = arg
                .get_long()
                .map(|l| format!("--{}", l))
                .unwrap_or_default();

            let flag = if short.is_empty() {
                long
            } else if long.is_empty() {
                short
            } else {
                format!("{}, {}", short, long)
            };

            let mut description = arg
                .get_help()
                .map(|h| h.to_string())
                .unwrap_or_else(|| String::from("No description"));

            // Add default value if present
            if let Some(default) = arg.get_default_values().first() {
                description.push_str(&format!(" (default: {})", default.to_string_lossy()));
            }

            rows.push(vec![format!("`{}`", flag), description]);
        }

        if !rows.is_empty() {
            let headers = &["Argument/Option", "Description"];
            output.push_str(&table(headers, &rows));
        } else {
            output.push_str("No additional arguments.\n\n");
        }

        output
    }

    fn generate_env_vars_docs(&self) -> String {
        let mut output = String::new();

        output.push_str("All configuration can be overridden via environment variables using the ");
        output
            .push_str("`OPERATOR_` prefix with `__` as the separator for nested config paths.\n\n");

        // Summary table
        output.push_str(&heading(3, "Quick Reference"));
        let headers = &["Variable", "Description", "Default"];
        let rows: Vec<Vec<String>> = ENV_VARS
            .iter()
            .map(|v| {
                vec![
                    format!("`{}`", v.name),
                    v.description.to_string(),
                    v.default.unwrap_or("-").to_string(),
                ]
            })
            .collect();
        output.push_str(&table(headers, &rows));

        // Detailed sections by category
        for (category, vars) in env_vars_by_category() {
            output.push_str(&heading(3, category.display_name()));

            let headers = &["Variable", "Description", "Default"];
            let rows: Vec<Vec<String>> = vars
                .iter()
                .map(|v| {
                    vec![
                        format!("`{}`", v.name),
                        v.description.to_string(),
                        v.default.unwrap_or("-").to_string(),
                    ]
                })
                .collect();
            output.push_str(&table(headers, &rows));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = CliDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("main.rs"));

        // Should have the main heading
        assert!(result.contains("# CLI Reference"));

        // Should have global options section
        assert!(result.contains("## Global Options"));

        // Should have commands section
        assert!(result.contains("## Commands"));

        // Should have environment variables section
        assert!(result.contains("## Environment Variables"));
    }

    #[test]
    fn test_all_subcommands_documented() {
        let generator = CliDocGenerator;
        let result = generator.generate().unwrap();

        // Check that known subcommands appear
        assert!(result.contains("### `queue`"));
        assert!(result.contains("### `launch`"));
        assert!(result.contains("### `agents`"));
        assert!(result.contains("### `pause`"));
        assert!(result.contains("### `resume`"));
        assert!(result.contains("### `stalled`"));
        assert!(result.contains("### `alert`"));
        assert!(result.contains("### `create`"));
        assert!(result.contains("### `docs`"));
    }

    #[test]
    fn test_env_vars_documented() {
        let generator = CliDocGenerator;
        let result = generator.generate().unwrap();

        // Check that environment variables are documented
        assert!(result.contains("OPERATOR_"));
        assert!(result.contains("### Quick Reference"));
        assert!(result.contains("### Authentication"));
        assert!(result.contains("### Agents"));
    }

    #[test]
    fn test_global_options_documented() {
        let generator = CliDocGenerator;
        let result = generator.generate().unwrap();

        // Check that global options are documented
        assert!(result.contains("--config"));
        assert!(result.contains("--debug"));
    }
}
