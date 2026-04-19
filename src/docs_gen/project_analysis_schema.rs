//! JSON Schema generator for the `ProjectAnalysis` type.
//!
//! Generates `docs/schemas/project_analysis.json` from the Rust `ProjectAnalysis` struct
//! via schemars, making Rust the single source of truth for structured output.

use super::DocGenerator;
use crate::backstage::analyzer::ProjectAnalysis;
use anyhow::Result;
use schemars::schema_for;

/// Generates JSON Schema from the `ProjectAnalysis` Rust type
pub struct ProjectAnalysisSchemaDocGenerator;

impl DocGenerator for ProjectAnalysisSchemaDocGenerator {
    fn name(&self) -> &'static str {
        "project-analysis-schema"
    }

    fn source(&self) -> &'static str {
        "src/backstage/analyzer.rs (ProjectAnalysis)"
    }

    fn output_path(&self) -> &'static str {
        "schemas/project_analysis.json"
    }

    fn generate(&self) -> Result<String> {
        let schema = schema_for!(ProjectAnalysis);
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
                    "https://gbqr.us/operator/project-analysis.schema.json".to_string(),
                ),
            );
            obj.insert(
                "$comment".to_string(),
                serde_json::Value::String(
                    "AUTO-GENERATED FROM src/backstage/analyzer.rs - DO NOT EDIT. Regenerate with: cargo run -- docs --only project-analysis-schema".to_string(),
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
        let generator = ProjectAnalysisSchemaDocGenerator;
        let result = generator.generate().unwrap();

        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have schema metadata
        assert_eq!(
            schema.get("$schema").and_then(|s| s.as_str()),
            Some("http://json-schema.org/draft-07/schema#")
        );
        assert!(schema.get("$id").is_some());

        // Should have properties for ProjectAnalysis fields
        let properties = schema.get("properties").expect("should have properties");
        assert!(properties.get("project_name").is_some());
        assert!(properties.get("project_path").is_some());
        assert!(properties.get("languages").is_some());
        assert!(properties.get("frameworks").is_some());
        assert!(properties.get("databases").is_some());
        assert!(properties.get("docker").is_some());
        assert!(properties.get("testing").is_some());
        assert!(properties.get("commands").is_some());
    }

    #[test]
    fn test_required_fields() {
        let generator = ProjectAnalysisSchemaDocGenerator;
        let result = generator.generate().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .expect("should have required array");

        let required_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(required_strs.contains(&"project_name"));
        assert!(required_strs.contains(&"kind_assessment"));
        assert!(required_strs.contains(&"languages"));
        assert!(required_strs.contains(&"frameworks"));
    }

    #[test]
    fn test_schema_has_definitions() {
        let generator = ProjectAnalysisSchemaDocGenerator;
        let result = generator.generate().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(
            schema.get("$defs").is_some() || schema.get("definitions").is_some(),
            "Schema should have definitions for sub-types"
        );
    }
}
