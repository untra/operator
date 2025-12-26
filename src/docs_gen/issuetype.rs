//! Documentation generator for the issuetype schema.

use super::markdown::{bold, bullet_list, heading, inline_code, table};
use super::{format_header, DocGenerator};
use anyhow::Result;
use serde_json::Value;

/// Schema JSON embedded at compile time
const ISSUETYPE_SCHEMA: &str = include_str!("../templates/issuetype_schema.json");

/// Generates documentation from issuetype_schema.json
pub struct IssuetypeSchemaDocGenerator;

impl DocGenerator for IssuetypeSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "issuetype-schema"
    }

    fn source(&self) -> &'static str {
        "src/templates/issuetype_schema.json"
    }

    fn output_path(&self) -> &'static str {
        "schemas/issuetype.md"
    }

    fn generate(&self) -> Result<String> {
        let schema: Value = serde_json::from_str(ISSUETYPE_SCHEMA)?;
        let mut output = format_header("Issue Type Schema", self.source());

        // Title and description
        output.push_str(&heading(1, "Issue Type Schema"));

        if let Some(desc) = schema.get("description").and_then(|d| d.as_str()) {
            output.push_str(&format!("{}\n\n", desc));
        }

        // Schema metadata
        output.push_str(&heading(2, "Schema Information"));
        let schema_items = vec![
            format!(
                "{}: {}",
                bold("$schema"),
                inline_code(
                    schema
                        .get("$schema")
                        .and_then(|s| s.as_str())
                        .unwrap_or("N/A")
                )
            ),
            format!(
                "{}: {}",
                bold("$id"),
                inline_code(schema.get("$id").and_then(|s| s.as_str()).unwrap_or("N/A"))
            ),
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
        output.push_str(&heading(2, "Definitions"));
        output.push_str(&self.generate_definitions_section(&schema));

        Ok(output)
    }
}

impl IssuetypeSchemaDocGenerator {
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
                .filter(|(k, _)| *k != "$schema") // Skip $schema
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

            // Detailed property descriptions
            for (name, prop) in properties.iter() {
                if name == "$schema" {
                    continue;
                }

                output.push_str(&heading(3, name));

                let mut details = vec![];

                if let Some(desc) = prop.get("description").and_then(|d| d.as_str()) {
                    details.push(format!("{}: {}", bold("Description"), desc));
                }

                details.push(format!("{}: {}", bold("Type"), self.get_type_string(prop)));

                if let Some(pattern) = prop.get("pattern").and_then(|p| p.as_str()) {
                    details.push(format!("{}: {}", bold("Pattern"), inline_code(pattern)));
                }

                if let Some(default) = prop.get("default") {
                    details.push(format!(
                        "{}: {}",
                        bold("Default"),
                        inline_code(&default.to_string())
                    ));
                }

                if let Some(enum_values) = prop.get("enum").and_then(|e| e.as_array()) {
                    let values: Vec<String> = enum_values
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(inline_code)
                        .collect();
                    details.push(format!("{}: {}", bold("Allowed Values"), values.join(", ")));
                }

                if let Some(examples) = prop.get("examples").and_then(|e| e.as_array()) {
                    let ex: Vec<String> = examples
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(inline_code)
                        .collect();
                    if !ex.is_empty() {
                        details.push(format!("{}: {}", bold("Examples"), ex.join(", ")));
                    }
                }

                output.push_str(&bullet_list(&details));
            }
        }

        output
    }

    fn generate_definitions_section(&self, schema: &Value) -> String {
        let mut output = String::new();

        if let Some(definitions) = schema.get("definitions").and_then(|d| d.as_object()) {
            for (name, def) in definitions {
                output.push_str(&heading(3, &format!("Definition: {}", name)));

                if let Some(desc) = def.get("description").and_then(|d| d.as_str()) {
                    output.push_str(&format!("{}\n\n", desc));
                }

                // Show the definition structure
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

                // Show enum values for simple definitions
                if let Some(enum_values) = def.get("enum").and_then(|e| e.as_array()) {
                    let values: Vec<String> = enum_values
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("- {}", inline_code(s)))
                        .collect();
                    output.push_str(&format!(
                        "{} Values:\n{}\n\n",
                        bold("Allowed"),
                        values.join("\n")
                    ));
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
        let generator = IssuetypeSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("issuetype_schema.json"));

        // Should have the main heading
        assert!(result.contains("# Issue Type Schema"));

        // Should have required fields section
        assert!(result.contains("## Required Fields"));
        assert!(result.contains("`key`"));
        assert!(result.contains("`name`"));

        // Should have properties section
        assert!(result.contains("## Properties"));

        // Should have definitions section
        assert!(result.contains("## Definitions"));
        assert!(result.contains("### Definition: field"));
        assert!(result.contains("### Definition: step"));
    }

    #[test]
    fn test_schema_parses_successfully() {
        let schema: Value = serde_json::from_str(ISSUETYPE_SCHEMA).unwrap();
        assert!(schema.get("properties").is_some());
        assert!(schema.get("definitions").is_some());
    }
}
