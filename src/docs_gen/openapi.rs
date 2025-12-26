//! OpenAPI specification documentation generator.
//!
//! Generates OpenAPI 3.0 specification from utoipa annotations.

use anyhow::Result;

use super::DocGenerator;
use crate::rest::ApiDoc;

/// Generates OpenAPI specification from REST API annotations
pub struct OpenApiDocGenerator;

impl DocGenerator for OpenApiDocGenerator {
    fn name(&self) -> &'static str {
        "OpenAPI"
    }

    fn source(&self) -> &'static str {
        "src/rest/ (utoipa annotations)"
    }

    fn output_path(&self) -> &'static str {
        "schemas/openapi.json"
    }

    fn generate(&self) -> Result<String> {
        let spec =
            ApiDoc::json().map_err(|e| anyhow::anyhow!("Failed to generate OpenAPI: {}", e))?;
        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_metadata() {
        let gen = OpenApiDocGenerator;
        assert_eq!(gen.name(), "OpenAPI");
        assert_eq!(gen.output_path(), "schemas/openapi.json");
    }

    #[test]
    fn test_generate_spec() {
        let gen = OpenApiDocGenerator;
        let content = gen.generate().expect("Failed to generate");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("Generated content is not valid JSON");

        // Verify it has OpenAPI structure
        assert!(parsed.get("openapi").is_some());
        assert!(parsed.get("info").is_some());
        assert!(parsed.get("paths").is_some());
    }
}
