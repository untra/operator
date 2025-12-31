//! Documentation generator for the state JSON schema.
//!
//! Generates human-readable markdown from docs/schemas/state.json.

use super::markdown::{bold, bullet_list, heading, inline_code, table};
use super::{format_header, DocGenerator};
use anyhow::Result;
use serde_json::Value;

/// Schema JSON embedded at compile time
const STATE_SCHEMA: &str = include_str!("../../docs/schemas/state.json");

/// Generates documentation from state.json schema
pub struct StateSchemaDocGenerator;

impl DocGenerator for StateSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "state-schema"
    }

    fn source(&self) -> &'static str {
        "docs/schemas/state.json"
    }

    fn output_path(&self) -> &'static str {
        "schemas/state.md"
    }

    fn generate(&self) -> Result<String> {
        let schema: Value = serde_json::from_str(STATE_SCHEMA)?;
        let mut output = format_header("Application State Schema", self.source());

        // Title and description
        output.push_str(&heading(1, "Application State Schema"));
        output.push_str(
            "JSON Schema for the Operator runtime state file (`state.json`).\n\n\
            This file tracks the current state of agents, completed tickets, and system status.\n\n",
        );

        // Schema metadata
        output.push_str(&heading(2, "Schema Information"));
        let schema_items = vec![
            format!(
                "{}: {}",
                bold("$schema"),
                inline_code("https://json-schema.org/draft/2020-12/schema")
            ),
            format!("{}: {}", bold("title"), inline_code("State")),
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

impl StateSchemaDocGenerator {
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
                    let desc = self.get_property_description(name, prop);

                    vec![inline_code(name), prop_type, is_required.to_string(), desc]
                })
                .collect();

            output.push_str(&table(headers, &rows));
        }

        output
    }

    fn get_property_description(&self, name: &str, prop: &Value) -> String {
        // Provide descriptions for top-level properties
        if let Some(desc) = prop.get("description").and_then(|d| d.as_str()) {
            return desc.replace('\n', " ");
        }

        // Fallback descriptions for known fields
        match name {
            "paused" => "Whether agent processing is paused".to_string(),
            "agents" => "Currently active agents".to_string(),
            "completed" => "Recently completed tickets".to_string(),
            _ => String::new(),
        }
    }

    fn generate_definitions_section(&self, schema: &Value) -> String {
        let mut output = String::new();

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
        let generator = StateSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("state.json"));

        // Should have the main heading
        assert!(result.contains("# Application State Schema"));

        // Should have required fields section
        assert!(result.contains("## Required Fields"));
        assert!(result.contains("`paused`"));
        assert!(result.contains("`agents`"));
        assert!(result.contains("`completed`"));

        // Should have properties section
        assert!(result.contains("## Properties"));

        // Should have definitions section
        assert!(result.contains("## Type Definitions"));
        assert!(result.contains("### AgentState"));
        assert!(result.contains("### CompletedTicket"));
    }

    #[test]
    fn test_schema_parses_successfully() {
        let schema: Value = serde_json::from_str(STATE_SCHEMA).unwrap();
        assert!(schema.get("properties").is_some());
        assert!(schema.get("$defs").is_some());
    }
}
