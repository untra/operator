//! Documentation generator for the Jira API JSON schema.
//!
//! Generates human-readable markdown from docs/schemas/jira-api.json.

use super::markdown::{bold, heading, inline_code, table};
use super::{format_header, DocGenerator};
use anyhow::Result;
use serde_json::Value;
use std::fs;

/// Generates documentation from jira-api.json schema
pub struct JiraApiDocGenerator;

impl DocGenerator for JiraApiDocGenerator {
    fn name(&self) -> &'static str {
        "jira-api"
    }

    fn source(&self) -> &'static str {
        "docs/schemas/jira-api.json"
    }

    fn output_path(&self) -> &'static str {
        "getting-started/kanban/jira-api.md"
    }

    fn generate(&self) -> Result<String> {
        // Read the schema file (can't use include_str! since it's generated at runtime)
        let schema_path = "docs/schemas/jira-api.json";
        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = serde_json::from_str(&schema_content)?;

        let mut output = format_header("Jira API Reference", self.source());

        // Title and description
        output.push_str(&heading(1, "Jira API Reference"));
        output.push_str("Auto-generated documentation of Jira Cloud REST API response types used by Operator.\n\n");

        // Overview
        output.push_str(&heading(2, "Overview"));
        output
            .push_str("Operator integrates with the following Jira Cloud REST API endpoints:\n\n");
        output.push_str("| Endpoint | Description |\n");
        output.push_str("|----------|-------------|\n");
        output.push_str("| `GET /rest/api/3/user/assignable/search` | List assignable users |\n");
        output.push_str("| `GET /rest/api/3/project/{key}/statuses` | List project statuses |\n");
        output.push_str("| `GET /rest/api/3/search` | Search issues with JQL |\n\n");

        // Main response type
        output.push_str(&heading(2, "JiraSearchResponse"));
        output.push_str("Response from the JQL search endpoint.\n\n");
        output.push_str(&self.generate_properties_section(&schema));

        // Definitions
        output.push_str(&heading(2, "Type Definitions"));
        output.push_str(&self.generate_definitions_section(&schema));

        Ok(output)
    }
}

impl JiraApiDocGenerator {
    fn generate_properties_section(&self, schema: &Value) -> String {
        let mut output = String::new();

        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            let headers = &["Property", "Type", "Description"];

            let rows: Vec<Vec<String>> = properties
                .iter()
                .filter(|(k, _)| *k != "$schema" && *k != "$comment")
                .map(|(name, prop)| {
                    let prop_type = Self::get_type_string(prop);
                    let desc = prop
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .replace('\n', " ");

                    vec![inline_code(name), prop_type, desc]
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
            // Sort by name for consistent output
            let mut sorted_defs: Vec<_> = defs.iter().collect();
            sorted_defs.sort_by(|a, b| a.0.cmp(b.0));

            for (name, def) in sorted_defs {
                output.push_str(&heading(3, name));

                if let Some(desc) = def.get("description").and_then(|d| d.as_str()) {
                    output.push_str(&format!("{}\n\n", desc));
                }

                // Show properties if object type
                if let Some(properties) = def.get("properties").and_then(|p| p.as_object()) {
                    let headers = &["Property", "Type", "Description"];

                    let rows: Vec<Vec<String>> = properties
                        .iter()
                        .map(|(prop_name, prop)| {
                            let prop_type = Self::get_type_string(prop);
                            let desc = prop
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .replace('\n', " ");

                            vec![inline_code(prop_name), prop_type, desc]
                        })
                        .collect();

                    output.push_str(&table(headers, &rows));
                }

                // Handle array items
                if let Some(items) = def.get("items") {
                    output.push_str(&format!("{} ", bold("Array of:")));
                    let item_type = Self::get_type_string(items);
                    output.push_str(&format!("{}\n\n", item_type));
                }
            }
        }

        output
    }

    fn get_type_string(prop: &Value) -> String {
        if let Some(type_val) = prop.get("type") {
            match type_val {
                Value::String(s) => {
                    // Handle array types with items
                    if s == "array" {
                        if let Some(items) = prop.get("items") {
                            let item_type = Self::get_type_string(items);
                            format!("{}[]", item_type)
                        } else {
                            inline_code("array")
                        }
                    } else {
                        inline_code(s)
                    }
                }
                Value::Array(arr) => {
                    let types: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .filter(|s| *s != "null")
                        .map(inline_code)
                        .collect();
                    if types.len() == 1 {
                        format!("{} (optional)", types[0])
                    } else {
                        types.join(" | ")
                    }
                }
                _ => "unknown".to_string(),
            }
        } else if let Some(ref_path) = prop.get("$ref").and_then(|r| r.as_str()) {
            let ref_name = ref_path.split('/').next_back().unwrap_or("ref");
            inline_code(ref_name)
        } else if prop.get("oneOf").is_some() || prop.get("anyOf").is_some() {
            // Handle nullable types
            let variants = prop
                .get("oneOf")
                .or_else(|| prop.get("anyOf"))
                .and_then(|v| v.as_array());

            if let Some(vars) = variants {
                let types: Vec<String> = vars
                    .iter()
                    .filter_map(|v| {
                        if v.get("type").and_then(|t| t.as_str()) == Some("null") {
                            None
                        } else {
                            Some(Self::get_type_string(v))
                        }
                    })
                    .collect();
                if types.len() == 1 {
                    format!("{} (optional)", types[0])
                } else {
                    types.join(" | ")
                }
            } else {
                "variant".to_string()
            }
        } else {
            inline_code("object")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_api_generator_name() {
        let generator = JiraApiDocGenerator;
        assert_eq!(generator.name(), "jira-api");
        assert_eq!(generator.source(), "docs/schemas/jira-api.json");
        assert_eq!(
            generator.output_path(),
            "getting-started/kanban/jira-api.md"
        );
    }
}
