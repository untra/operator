//! JSON Schema generator for the issuetype template types.
//!
//! Generates `docs/schemas/issuetype_template.json` from the Rust `TemplateSchema` struct
//! via schemars, making Rust the single source of truth for the issuetype file format.

use super::DocGenerator;
use crate::templates::schema::TemplateSchema;
use anyhow::Result;
use schemars::schema_for;

/// Generates JSON Schema from the `TemplateSchema` Rust type
pub struct IssuetypeJsonSchemaDocGenerator;

impl DocGenerator for IssuetypeJsonSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "issuetype-json-schema"
    }

    fn source(&self) -> &'static str {
        "src/templates/schema.rs (TemplateSchema)"
    }

    fn output_path(&self) -> &'static str {
        "../src/schemas/issuetype_schema.json"
    }

    fn generate(&self) -> Result<String> {
        let schema = schema_for!(TemplateSchema);
        let mut schema_value = serde_json::to_value(&schema)?;

        // Add metadata to match the hand-written schema conventions
        if let Some(obj) = schema_value.as_object_mut() {
            obj.insert(
                "$schema".to_string(),
                serde_json::Value::String("http://json-schema.org/draft-07/schema#".to_string()),
            );
            obj.insert(
                "$id".to_string(),
                serde_json::Value::String(
                    "https://gbqr.us/operator/issuetype-template.schema.json".to_string(),
                ),
            );
            obj.insert(
                "$comment".to_string(),
                serde_json::Value::String(
                    "AUTO-GENERATED FROM src/templates/schema.rs - DO NOT EDIT. Regenerate with: cargo run -- docs --only issuetype-json-schema".to_string(),
                ),
            );
        }

        let json = serde_json::to_string_pretty(&schema_value)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_json_schema() {
        let generator = IssuetypeJsonSchemaDocGenerator;
        let result = generator.generate().unwrap();

        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have schema metadata
        assert_eq!(
            schema.get("$schema").and_then(|s| s.as_str()),
            Some("http://json-schema.org/draft-07/schema#")
        );
        assert!(schema.get("$id").is_some());

        // Should have properties for TemplateSchema fields
        let properties = schema.get("properties").expect("should have properties");
        assert!(properties.get("key").is_some(), "missing 'key' property");
        assert!(properties.get("name").is_some(), "missing 'name' property");
        assert!(
            properties.get("description").is_some(),
            "missing 'description' property"
        );
        assert!(properties.get("mode").is_some(), "missing 'mode' property");
        assert!(
            properties.get("glyph").is_some(),
            "missing 'glyph' property"
        );
        assert!(
            properties.get("fields").is_some(),
            "missing 'fields' property"
        );
        assert!(
            properties.get("steps").is_some(),
            "missing 'steps' property"
        );
    }

    #[test]
    fn test_required_fields() {
        let generator = IssuetypeJsonSchemaDocGenerator;
        let result = generator.generate().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        // key, name, description, mode, glyph, fields, steps should be required
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .expect("should have required array");

        let required_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(required_strs.contains(&"key"), "key should be required");
        assert!(required_strs.contains(&"name"), "name should be required");
        assert!(required_strs.contains(&"mode"), "mode should be required");
        assert!(required_strs.contains(&"glyph"), "glyph should be required");
        assert!(
            required_strs.contains(&"fields"),
            "fields should be required"
        );
        assert!(required_strs.contains(&"steps"), "steps should be required");
    }

    #[test]
    fn test_schema_has_definitions() {
        let generator = IssuetypeJsonSchemaDocGenerator;
        let result = generator.generate().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have $defs or definitions for sub-types
        assert!(
            schema.get("$defs").is_some() || schema.get("definitions").is_some(),
            "Schema should have definitions for sub-types like StepSchema, FieldSchema"
        );
    }
}
