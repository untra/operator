//! Documentation generator for the ticket metadata schema.

use super::markdown::{bold, bullet_list, code_block, heading, inline_code, table};
use super::{format_header, DocGenerator};
use anyhow::Result;
use serde_json::Value;

/// Schema JSON embedded at compile time
const METADATA_SCHEMA: &str = include_str!("../schemas/ticket_metadata.schema.json");

/// Generates documentation from ticket_metadata.schema.json
pub struct MetadataSchemaDocGenerator;

impl DocGenerator for MetadataSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "metadata-schema"
    }

    fn source(&self) -> &'static str {
        "src/schemas/ticket_metadata.schema.json"
    }

    fn output_path(&self) -> &'static str {
        "schemas/metadata.md"
    }

    fn generate(&self) -> Result<String> {
        let schema: Value = serde_json::from_str(METADATA_SCHEMA)?;
        let mut output = format_header("Ticket Metadata Schema", self.source());

        // Title and description
        output.push_str(&heading(1, "Ticket Metadata Schema"));

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
            format!(
                "{}: {}",
                bold("Additional Properties"),
                if schema
                    .get("additionalProperties")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    "Allowed"
                } else {
                    "Not Allowed"
                }
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
        if schema.get("definitions").is_some() {
            output.push_str(&heading(2, "Definitions"));
            output.push_str(&self.generate_definitions_section(&schema));
        }

        // Examples
        if let Some(examples) = schema.get("examples").and_then(|e| e.as_array()) {
            output.push_str(&heading(2, "Examples"));
            output.push_str("Complete ticket metadata examples:\n\n");
            for (i, example) in examples.iter().enumerate() {
                output.push_str(&heading(3, &format!("Example {}", i + 1)));
                let pretty = serde_json::to_string_pretty(example).unwrap_or_default();
                output.push_str(&code_block(&pretty, Some("yaml")));
            }
        }

        Ok(output)
    }
}

impl MetadataSchemaDocGenerator {
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
                output.push_str(&heading(3, name));

                let mut details = vec![];

                if let Some(desc) = prop.get("description").and_then(|d| d.as_str()) {
                    details.push(format!("{}: {}", bold("Description"), desc));
                }

                details.push(format!("{}: {}", bold("Type"), self.get_type_string(prop)));

                if let Some(format) = prop.get("format").and_then(|f| f.as_str()) {
                    details.push(format!("{}: {}", bold("Format"), inline_code(format)));
                }

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
                        .filter_map(|v| {
                            if v.is_string() {
                                v.as_str().map(inline_code)
                            } else {
                                Some(inline_code(&v.to_string()))
                            }
                        })
                        .collect();
                    if !ex.is_empty() {
                        details.push(format!("{}: {}", bold("Examples"), ex.join(", ")));
                    }
                }

                output.push_str(&bullet_list(&details));

                // Handle nested object properties
                if let Some(nested_props) = prop.get("properties").and_then(|p| p.as_object()) {
                    output.push_str(&format!("{} Nested Properties:\n\n", bold("Object")));
                    let nested_headers = &["Property", "Type", "Description"];
                    let nested_rows: Vec<Vec<String>> = nested_props
                        .iter()
                        .map(|(nested_name, nested_prop)| {
                            vec![
                                inline_code(nested_name),
                                self.get_type_string(nested_prop),
                                nested_prop
                                    .get("description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            ]
                        })
                        .collect();
                    output.push_str(&table(nested_headers, &nested_rows));
                }

                // Handle additionalProperties for object types
                if let Some(additional) = prop.get("additionalProperties") {
                    if additional.is_object() {
                        let add_type = self.get_type_string(additional);
                        let add_desc = additional
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("Additional properties allowed");
                        output.push_str(&format!(
                            "{}: {} ({})\n\n",
                            bold("Additional Properties"),
                            add_type,
                            add_desc
                        ));
                    }
                }
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
                    let headers = &["Property", "Description"];
                    let rows: Vec<Vec<String>> = properties
                        .iter()
                        .map(|(prop_name, prop)| {
                            let desc = prop
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string();
                            vec![inline_code(prop_name), desc]
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
            let base_type = match type_val {
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
            };

            // Add format info if present
            if let Some(format) = prop.get("format").and_then(|f| f.as_str()) {
                format!("{} ({})", base_type, format)
            } else {
                base_type
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
        let generator = MetadataSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("ticket_metadata.schema.json"));

        // Should have the main heading
        assert!(result.contains("# Ticket Metadata Schema"));

        // Should have required fields section
        assert!(result.contains("## Required Fields"));
        assert!(result.contains("`id`"));
        assert!(result.contains("`status`"));

        // Should have properties section
        assert!(result.contains("## Properties"));

        // Should have specific properties documented
        assert!(result.contains("### id"));
        assert!(result.contains("### status"));
        assert!(result.contains("### priority"));
        assert!(result.contains("### sessions"));
        assert!(result.contains("### llm_task"));
    }

    #[test]
    fn test_schema_parses_successfully() {
        let schema: Value = serde_json::from_str(METADATA_SCHEMA).unwrap();
        assert!(schema.get("properties").is_some());
        assert!(schema.get("required").is_some());
    }

    #[test]
    fn test_status_enum_values_documented() {
        let generator = MetadataSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Status should have enum values documented
        assert!(result.contains("`queued`"));
        assert!(result.contains("`running`"));
        assert!(result.contains("`awaiting`"));
        assert!(result.contains("`completed`"));
    }

    #[test]
    fn test_priority_enum_values_documented() {
        let generator = MetadataSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Priority should have enum values documented
        assert!(result.contains("`P0-critical`"));
        assert!(result.contains("`P1-high`"));
        assert!(result.contains("`P2-medium`"));
        assert!(result.contains("`P3-low`"));
    }

    #[test]
    fn test_examples_section_present() {
        let generator = MetadataSchemaDocGenerator;
        let result = generator.generate().unwrap();

        // Should have examples section
        assert!(result.contains("## Examples"));
        assert!(result.contains("### Example 1"));
        assert!(result.contains("### Example 2"));
    }
}
