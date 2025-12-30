//! Documentation generator for the config JSON schema.
//!
//! Generates human-readable markdown from docs/schemas/config.json.

use super::markdown::{bold, bullet_list, heading, inline_code, table};
use super::{format_header, DocGenerator};
use anyhow::Result;
use serde_json::Value;

/// Schema JSON embedded at compile time
const CONFIG_SCHEMA: &str = include_str!("../../docs/schemas/config.json");

/// Generates documentation from config.json schema
pub struct ConfigSchemaDocGenerator;

impl DocGenerator for ConfigSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "config-schema"
    }

    fn source(&self) -> &'static str {
        "docs/schemas/config.json"
    }

    fn output_path(&self) -> &'static str {
        "schemas/config.md"
    }

    fn generate(&self) -> Result<String> {
        let schema: Value = serde_json::from_str(CONFIG_SCHEMA)?;
        let mut output = format_header("Configuration Schema", self.source());

        // Title and description
        output.push_str(&heading(1, "Configuration Schema"));
        output.push_str("JSON Schema for the Operator configuration file (`config.toml`).\n\n");

        // Schema metadata
        output.push_str(&heading(2, "Schema Information"));
        let schema_items = vec![
            format!(
                "{}: {}",
                bold("$schema"),
                inline_code("https://json-schema.org/draft/2020-12/schema")
            ),
            format!("{}: {}", bold("title"), inline_code("Config")),
        ];
        output.push_str(&bullet_list(&schema_items));

        // Required fields
        output.push_str(&heading(2, "Required Fields"));
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            let required_items: Vec<String> = required
                .iter()
                .filter_map(|r| r.as_str())
                .map(inline_code)
                .collect();
            output.push_str(&bullet_list(&required_items));
        }

        // Top-level properties
        output.push_str(&heading(2, "Properties"));
        output.push_str(&self.generate_properties_section(&schema));

        // Definitions
        output.push_str(&heading(2, "Type Definitions"));
        output.push_str(&self.generate_definitions_section(&schema));

        Ok(output)
    }
}

impl ConfigSchemaDocGenerator {
    fn generate_properties_section(&self, schema: &Value) -> String {
        let mut output = String::new();

        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            let headers = &["Property", "Type", "Required", "Description"];
            let required: Vec<&str> = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();

            let rows: Vec<Vec<String>> = properties
                .iter()
                .filter(|(k, _)| *k != "$schema")
                .map(|(name, prop)| {
                    let prop_type = self.get_type_string(prop);
                    let is_required = if required.contains(&name.as_str()) {
                        "Yes"
                    } else {
                        "No"
                    };
                    let desc = prop
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .replace('\n', " ");

                    vec![inline_code(name), prop_type, is_required.to_string(), desc]
                })
                .collect();

            output.push_str(&table(headers, &rows));
        }

        output
    }

    fn generate_definitions_section(&self, schema: &Value) -> String {
        let mut output = String::new();

        // Check both $defs (JSON Schema draft 2020-12) and definitions (draft-07)
        let definitions = schema
            .get("$defs")
            .or_else(|| schema.get("definitions"))
            .and_then(|d| d.as_object());

        if let Some(defs) = definitions {
            for (name, def) in defs {
                output.push_str(&heading(3, name));

                if let Some(desc) = def.get("description").and_then(|d| d.as_str()) {
                    output.push_str(&format!("{}\n\n", desc));
                }

                // Show properties if object type
                if let Some(properties) = def.get("properties").and_then(|p| p.as_object()) {
                    let headers = &["Property", "Type", "Required", "Description"];
                    let required: Vec<&str> = def
                        .get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                        .unwrap_or_default();

                    let rows: Vec<Vec<String>> = properties
                        .iter()
                        .map(|(prop_name, prop)| {
                            let prop_type = self.get_type_string(prop);
                            let is_required = if required.contains(&prop_name.as_str()) {
                                "Yes"
                            } else {
                                "No"
                            };
                            let desc = prop
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .replace('\n', " ");

                            vec![
                                inline_code(prop_name),
                                prop_type,
                                is_required.to_string(),
                                desc,
                            ]
                        })
                        .collect();

                    output.push_str(&table(headers, &rows));
                }

                // Handle oneOf (enum-like types)
                if let Some(one_of) = def.get("oneOf").and_then(|e| e.as_array()) {
                    output.push_str(&format!("{}\n\n", bold("Allowed Values:")));
                    for variant in one_of {
                        if let Some(const_val) = variant.get("const").and_then(|c| c.as_str()) {
                            let desc = variant
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("");
                            output.push_str(&format!("- {} - {}\n", inline_code(const_val), desc));
                        }
                    }
                    output.push('\n');
                }
            }
        }

        output
    }

    fn get_type_string(&self, prop: &Value) -> String {
        if let Some(type_val) = prop.get("type") {
            match type_val {
                Value::String(s) => inline_code(s),
                Value::Array(arr) => {
                    let types: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(inline_code)
                        .collect();
                    types.join(" \\| ")
                }
                _ => "unknown".to_string(),
            }
        } else if prop.get("$ref").is_some() {
            let ref_path = prop.get("$ref").and_then(|r| r.as_str()).unwrap_or("");
            let ref_name = ref_path.split('/').next_back().unwrap_or("ref");
            format!("→ {}", inline_code(ref_name))
        } else if prop.get("oneOf").is_some() {
            "enum".to_string()
        } else {
            "object".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = ConfigSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("config.json"));

        // Should have the main heading
        assert!(result.contains("# Configuration Schema"));

        // Should have required fields section
        assert!(result.contains("## Required Fields"));
        assert!(result.contains("`agents`"));
        assert!(result.contains("`paths`"));

        // Should have properties section
        assert!(result.contains("## Properties"));

        // Should have definitions section with key types
        assert!(result.contains("## Type Definitions"));
        assert!(result.contains("### AgentsConfig"));
        assert!(result.contains("### NotificationsConfig"));
    }

    #[test]
    fn test_schema_parses_successfully() {
        let schema: Value = serde_json::from_str(CONFIG_SCHEMA).unwrap();
        assert!(schema.get("properties").is_some());
        assert!(schema.get("$defs").is_some());
    }
}
