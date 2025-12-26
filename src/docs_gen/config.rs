//! Documentation generator for configuration reference.

use super::markdown::{code_block, heading, inline_code, table};
use super::{format_header, DocGenerator};
use crate::config::Config;
use anyhow::Result;
use schemars::schema_for;
use serde_json::Value;

/// Configuration section metadata for documentation ordering
struct ConfigSection {
    /// TOML section name (e.g., "agents")
    name: &'static str,
    /// Schema definition name (e.g., "AgentsConfig")
    schema_name: &'static str,
    /// Human-readable description
    description: &'static str,
}

/// All configuration sections in documentation order
const CONFIG_SECTIONS: &[ConfigSection] = &[
    ConfigSection {
        name: "agents",
        schema_name: "AgentsConfig",
        description: "Agent lifecycle, parallelism, and health monitoring",
    },
    ConfigSection {
        name: "notifications",
        schema_name: "NotificationsConfig",
        description: "macOS notification preferences",
    },
    ConfigSection {
        name: "queue",
        schema_name: "QueueConfig",
        description: "Queue processing and ticket assignment",
    },
    ConfigSection {
        name: "paths",
        schema_name: "PathsConfig",
        description: "Directory paths for tickets, projects, and state",
    },
    ConfigSection {
        name: "ui",
        schema_name: "UiConfig",
        description: "Terminal UI appearance and behavior",
    },
    ConfigSection {
        name: "launch",
        schema_name: "LaunchConfig",
        description: "Agent launch behavior and confirmations",
    },
    ConfigSection {
        name: "templates",
        schema_name: "TemplatesConfig",
        description: "Issue type collections and presets",
    },
    ConfigSection {
        name: "api",
        schema_name: "ApiConfig",
        description: "External API integration settings",
    },
    ConfigSection {
        name: "logging",
        schema_name: "LoggingConfig",
        description: "Log level and output configuration",
    },
    ConfigSection {
        name: "tmux",
        schema_name: "TmuxConfig",
        description: "Tmux integration settings",
    },
    ConfigSection {
        name: "backstage",
        schema_name: "BackstageConfig",
        description: "Backstage server integration",
    },
    ConfigSection {
        name: "llm_tools",
        schema_name: "LlmToolsConfig",
        description: "LLM CLI tool detection and providers",
    },
];

/// Generates configuration documentation from schemars-derived JSON Schema
pub struct ConfigDocGenerator;

impl DocGenerator for ConfigDocGenerator {
    fn name(&self) -> &'static str {
        "config"
    }

    fn source(&self) -> &'static str {
        "src/config.rs"
    }

    fn output_path(&self) -> &'static str {
        "configuration/index.md"
    }

    fn generate(&self) -> Result<String> {
        // Generate schema at runtime from Config struct
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_value(&schema)?;

        let mut output = format_header("Configuration", self.source());

        // Introduction
        output.push_str(&heading(1, "Configuration"));
        output.push_str("Operator configuration is stored in `.tickets/operator/config.toml`.\n\n");

        // Quick reference table of all sections
        output.push_str(&heading(2, "Configuration Sections"));
        output.push_str(&self.generate_sections_overview());

        // Get definitions from schema
        let definitions = schema_json
            .get("$defs")
            .or_else(|| schema_json.get("definitions"))
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        // Detailed section for each config struct
        for section in CONFIG_SECTIONS {
            output.push_str(&self.generate_section_docs(section, &definitions));
        }

        // Example config.toml
        output.push_str(&heading(2, "Example Configuration"));
        output.push_str(&self.generate_example_toml());

        // Config file locations
        output.push_str(&heading(2, "Configuration Files"));
        output.push_str(&self.generate_config_locations());

        Ok(output)
    }
}

impl ConfigDocGenerator {
    /// Generate overview table of all configuration sections
    fn generate_sections_overview(&self) -> String {
        let headers = &["Section", "Description"];
        let rows: Vec<Vec<String>> = CONFIG_SECTIONS
            .iter()
            .map(|s| vec![format!("`[{}]`", s.name), s.description.to_string()])
            .collect();

        table(headers, &rows)
    }

    /// Generate detailed documentation for a configuration section
    fn generate_section_docs(&self, section: &ConfigSection, definitions: &Value) -> String {
        let mut output = String::new();

        output.push_str(&heading(2, &format!("`[{}]`", section.name)));
        output.push_str(&format!("{}\n\n", section.description));

        // Get the schema definition for this section
        if let Some(def) = definitions.get(section.schema_name) {
            if let Some(properties) = def.get("properties").and_then(|p| p.as_object()) {
                let required: Vec<&str> = def
                    .get("required")
                    .and_then(|r| r.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();

                let headers = &["Field", "Type", "Default", "Description"];
                let rows: Vec<Vec<String>> = properties
                    .iter()
                    .map(|(name, prop)| {
                        let type_str = Self::get_type_string(prop);
                        let default = self.get_default_value(name, section.name);
                        let desc = prop
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .replace('\n', " ");
                        let required_marker = if required.contains(&name.as_str()) {
                            " *"
                        } else {
                            ""
                        };

                        vec![
                            format!("`{}`{}", name, required_marker),
                            type_str,
                            default,
                            desc,
                        ]
                    })
                    .collect();

                if !rows.is_empty() {
                    output.push_str(&table(headers, &rows));
                }
            }
        }

        output
    }

    /// Get type string from schema property
    fn get_type_string(prop: &Value) -> String {
        // Check for $ref first
        if let Some(ref_path) = prop.get("$ref").and_then(|r| r.as_str()) {
            let ref_name = ref_path.split('/').next_back().unwrap_or("object");
            return format!("→ {}", inline_code(ref_name));
        }

        // Check for allOf (common with schemars for nested types)
        if let Some(all_of) = prop.get("allOf").and_then(|a| a.as_array()) {
            for item in all_of {
                if let Some(ref_path) = item.get("$ref").and_then(|r| r.as_str()) {
                    let ref_name = ref_path.split('/').next_back().unwrap_or("object");
                    return format!("→ {}", inline_code(ref_name));
                }
            }
        }

        // Check for oneOf (enums with null)
        if let Some(one_of) = prop.get("oneOf").and_then(|o| o.as_array()) {
            let types: Vec<String> = one_of
                .iter()
                .filter_map(|item| {
                    if let Some(ref_path) = item.get("$ref").and_then(|r| r.as_str()) {
                        let ref_name = ref_path.split('/').next_back().unwrap_or("object");
                        Some(inline_code(ref_name))
                    } else {
                        item.get("type").and_then(|t| t.as_str()).map(inline_code)
                    }
                })
                .collect();
            if !types.is_empty() {
                return types.join(" \\| ");
            }
        }

        // Handle type field
        if let Some(type_val) = prop.get("type") {
            match type_val {
                Value::String(s) => {
                    let base = inline_code(s);
                    // Check for array items
                    if s == "array" {
                        if let Some(items) = prop.get("items") {
                            let item_type = Self::get_type_string(items);
                            return format!("{}[{}]", inline_code("array"), item_type);
                        }
                    }
                    base
                }
                Value::Array(arr) => {
                    let types: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(inline_code)
                        .collect();
                    types.join(" \\| ")
                }
                _ => inline_code("unknown"),
            }
        } else if prop.get("enum").is_some() {
            inline_code("enum")
        } else if prop.get("properties").is_some() {
            inline_code("object")
        } else {
            inline_code("any")
        }
    }

    /// Get default value for a field from Config::default()
    fn get_default_value(&self, field: &str, section: &str) -> String {
        let config = Config::default();

        let value = match section {
            "agents" => match field {
                "max_parallel" => Some(config.agents.max_parallel.to_string()),
                "cores_reserved" => Some(config.agents.cores_reserved.to_string()),
                "health_check_interval" => Some(config.agents.health_check_interval.to_string()),
                "generation_timeout_secs" => {
                    Some(config.agents.generation_timeout_secs.to_string())
                }
                "sync_interval" => Some(config.agents.sync_interval.to_string()),
                "step_timeout" => Some(config.agents.step_timeout.to_string()),
                "silence_threshold" => Some(config.agents.silence_threshold.to_string()),
                _ => None,
            },
            "notifications" => match field {
                "enabled" => Some(config.notifications.enabled.to_string()),
                "on_agent_start" => Some(config.notifications.on_agent_start.to_string()),
                "on_agent_complete" => Some(config.notifications.on_agent_complete.to_string()),
                "on_agent_needs_input" => {
                    Some(config.notifications.on_agent_needs_input.to_string())
                }
                "on_pr_created" => Some(config.notifications.on_pr_created.to_string()),
                "on_investigation_created" => {
                    Some(config.notifications.on_investigation_created.to_string())
                }
                "sound" => Some(config.notifications.sound.to_string()),
                _ => None,
            },
            "queue" => match field {
                "auto_assign" => Some(config.queue.auto_assign.to_string()),
                "priority_order" => Some(format!("{:?}", config.queue.priority_order)),
                "poll_interval_ms" => Some(config.queue.poll_interval_ms.to_string()),
                _ => None,
            },
            "paths" => match field {
                "tickets" => Some(config.paths.tickets.clone()),
                "projects" => Some(config.paths.projects.clone()),
                "state" => Some(config.paths.state.clone()),
                _ => None,
            },
            "ui" => match field {
                "refresh_rate_ms" => Some(config.ui.refresh_rate_ms.to_string()),
                "completed_history_hours" => Some(config.ui.completed_history_hours.to_string()),
                "summary_max_length" => Some(config.ui.summary_max_length.to_string()),
                _ => None,
            },
            "launch" => match field {
                "confirm_autonomous" => Some(config.launch.confirm_autonomous.to_string()),
                "confirm_paired" => Some(config.launch.confirm_paired.to_string()),
                "launch_delay_ms" => Some(config.launch.launch_delay_ms.to_string()),
                _ => None,
            },
            "api" => match field {
                "pr_check_interval_secs" => Some(config.api.pr_check_interval_secs.to_string()),
                "rate_limit_check_interval_secs" => {
                    Some(config.api.rate_limit_check_interval_secs.to_string())
                }
                "rate_limit_warning_threshold" => {
                    Some(config.api.rate_limit_warning_threshold.to_string())
                }
                _ => None,
            },
            "logging" => match field {
                "level" => Some(config.logging.level.clone()),
                "to_file" => Some(config.logging.to_file.to_string()),
                _ => None,
            },
            "backstage" => match field {
                "enabled" => Some(config.backstage.enabled.to_string()),
                "port" => Some(config.backstage.port.to_string()),
                "auto_start" => Some(config.backstage.auto_start.to_string()),
                "subpath" => Some(config.backstage.subpath.clone()),
                "branding_subpath" => Some(config.backstage.branding_subpath.clone()),
                _ => None,
            },
            _ => None,
        };

        value.unwrap_or_else(|| "-".to_string())
    }

    /// Generate example config.toml from defaults
    fn generate_example_toml(&self) -> String {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config)
            .unwrap_or_else(|_| "# Error generating example".to_string());
        code_block(&toml_str, Some("toml"))
    }

    /// Generate documentation about config file locations
    fn generate_config_locations(&self) -> String {
        let mut output = String::new();

        output.push_str(
            "Configuration is loaded in this order (later sources override earlier):\n\n",
        );
        output.push_str("1. **Built-in defaults** - Embedded in the binary\n");
        output.push_str("2. **Project config** - `.tickets/operator/config.toml`\n");
        output.push_str("3. **User config** - `~/.config/operator/config.toml`\n");
        output.push_str("4. **CLI flag** - `--config <path>`\n");
        output
            .push_str("5. **Environment variables** - `OPERATOR_*` prefix with `__` separator\n\n");

        output.push_str("### Environment Variable Override\n\n");
        output
            .push_str("Any configuration option can be overridden via environment variables.\n\n");
        output.push_str("**Format**: `OPERATOR_<SECTION>__<FIELD>`\n\n");
        output.push_str("**Examples**:\n");
        output.push_str("- `OPERATOR_AGENTS__MAX_PARALLEL=2`\n");
        output.push_str("- `OPERATOR_LOGGING__LEVEL=debug`\n");
        output.push_str("- `OPERATOR_BACKSTAGE__PORT=8080`\n\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = ConfigDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("config.rs"));

        // Should have the main heading
        assert!(result.contains("# Configuration"));

        // Should have sections overview
        assert!(result.contains("## Configuration Sections"));

        // Should have individual section docs
        assert!(result.contains("## `[agents]`"));
        assert!(result.contains("## `[notifications]`"));
        assert!(result.contains("## `[queue]`"));
        assert!(result.contains("## `[paths]`"));
        assert!(result.contains("## `[backstage]`"));

        // Should have example config
        assert!(result.contains("## Example Configuration"));
        assert!(result.contains("```toml"));

        // Should have config file locations
        assert!(result.contains("## Configuration Files"));
        assert!(result.contains("OPERATOR_"));
    }

    #[test]
    fn test_all_sections_documented() {
        let generator = ConfigDocGenerator;
        let result = generator.generate().unwrap();

        for section in CONFIG_SECTIONS {
            assert!(
                result.contains(&format!("## `[{}]`", section.name)),
                "Section {} not documented",
                section.name
            );
        }
    }

    #[test]
    fn test_example_toml_is_valid() {
        let generator = ConfigDocGenerator;
        let example = generator.generate_example_toml();

        // Should be a code block
        assert!(example.starts_with("```toml"));
        assert!(example.ends_with("```\n\n"));

        // Extract TOML content and verify it parses
        let toml_content = example
            .strip_prefix("```toml\n")
            .unwrap()
            .strip_suffix("\n```\n\n")
            .unwrap();

        let parsed: Result<Config, _> = toml::from_str(toml_content);
        assert!(parsed.is_ok(), "Example TOML should be valid");
    }

    #[test]
    fn test_sections_overview_has_all_sections() {
        let generator = ConfigDocGenerator;
        let overview = generator.generate_sections_overview();

        for section in CONFIG_SECTIONS {
            assert!(
                overview.contains(&format!("`[{}]`", section.name)),
                "Overview missing section {}",
                section.name
            );
        }
    }

    #[test]
    fn test_schema_generates_successfully() {
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_value(&schema).unwrap();

        // Should have $defs or definitions
        assert!(
            schema_json.get("$defs").is_some() || schema_json.get("definitions").is_some(),
            "Schema should have definitions"
        );
    }
}
