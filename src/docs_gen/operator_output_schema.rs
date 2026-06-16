//! JSON Schema generator for the `OperatorOutput` type.
//!
//! Generates `docs/schemas/operator_output.json` from the Rust `OperatorOutput` struct
//! via schemars, making Rust the single source of truth.

use super::DocGenerator;
use crate::rest::dto::OperatorOutput;
use anyhow::Result;
use schemars::schema_for;

/// Generates JSON Schema from the `OperatorOutput` Rust type
pub struct OperatorOutputSchemaDocGenerator;

impl DocGenerator for OperatorOutputSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "operator-output-schema"
    }

    fn source(&self) -> &'static str {
        "src/rest/dto.rs (OperatorOutput)"
    }

    fn output_path(&self) -> &'static str {
        "schemas/operator_output.json"
    }

    fn generate(&self) -> Result<String> {
        let schema = schema_for!(OperatorOutput);
        let mut schema_value = serde_json::to_value(&schema)?;

        // Add metadata to match the hand-written schema
        if let Some(obj) = schema_value.as_object_mut() {
            obj.insert(
                "$schema".to_string(),
                serde_json::Value::String("http://json-schema.org/draft-07/schema#".to_string()),
            );
            obj.insert(
                "$id".to_string(),
                serde_json::Value::String(
                    "https://operator.untra.io/schemas/operator_output.json".to_string(),
                ),
            );
            obj.insert(
                "$comment".to_string(),
                serde_json::Value::String(
                    "AUTO-GENERATED FROM src/rest/dto.rs - DO NOT EDIT. Regenerate with: cargo run -- docs --only operator-output-schema".to_string(),
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
        let generator = OperatorOutputSchemaDocGenerator;
        let result = generator.generate().unwrap();

        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have schema metadata
        assert_eq!(
            schema.get("$schema").and_then(|s| s.as_str()),
            Some("http://json-schema.org/draft-07/schema#")
        );
        assert!(schema.get("$id").is_some());

        // Should have properties for OperatorOutput fields
        let properties = schema.get("properties").expect("should have properties");
        assert!(properties.get("status").is_some());
        assert!(properties.get("exit_signal").is_some());
        assert!(properties.get("confidence").is_some());
        assert!(properties.get("summary").is_some());
        assert!(properties.get("blockers").is_some());
    }

    #[test]
    fn test_required_fields() {
        let generator = OperatorOutputSchemaDocGenerator;
        let result = generator.generate().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        // status and exit_signal should be required
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .expect("should have required array");

        let required_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(required_strs.contains(&"status"));
        assert!(required_strs.contains(&"exit_signal"));
    }
}
