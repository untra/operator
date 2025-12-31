//! Documentation generator for the schema index page.
//!
//! Generates a landing page listing all available schemas.

use super::markdown::{heading, table};
use super::{format_header, DocGenerator};
use anyhow::Result;

/// Generates the schema index/overview page
pub struct SchemaIndexDocGenerator;

impl DocGenerator for SchemaIndexDocGenerator {
    fn name(&self) -> &'static str {
        "schema-index"
    }

    fn source(&self) -> &'static str {
        "docs/schemas/"
    }

    fn output_path(&self) -> &'static str {
        "schemas/index.md"
    }

    fn generate(&self) -> Result<String> {
        let mut output = format_header("Schema Reference", self.source());

        output.push_str(&heading(1, "Schema Reference"));
        output.push_str(
            "This section documents all JSON schemas and type definitions used by Operator.\n\n",
        );

        // Documentation pages
        output.push_str(&heading(2, "Documentation"));
        output.push_str("Human-readable documentation for each schema:\n\n");

        let headers = &["Schema", "Description"];
        let rows = vec![
            vec![
                "[Configuration](config/)".to_string(),
                "Structure of `config.toml` - agents, paths, UI, notifications, and integrations"
                    .to_string(),
            ],
            vec![
                "[Application State](state/)".to_string(),
                "Runtime state file (`state.json`) - active agents, completed tickets, system status".to_string(),
            ],
            vec![
                "[Issue Type](issuetype/)".to_string(),
                "Issue type template format - fields, steps, permissions, and workflows".to_string(),
            ],
            vec![
                "[Ticket Metadata](metadata/)".to_string(),
                "Ticket YAML frontmatter - status, priority, sessions, and LLM task tracking"
                    .to_string(),
            ],
            vec![
                "[REST API](api/)".to_string(),
                "Interactive OpenAPI documentation with Swagger UI".to_string(),
            ],
        ];
        output.push_str(&table(headers, &rows));

        // Raw JSON schemas
        output.push_str(&heading(2, "Raw JSON Schemas"));
        output
            .push_str("Machine-readable JSON Schema files for validation and code generation:\n\n");

        let json_headers = &["File", "Format", "Description"];
        let json_rows = vec![
            vec![
                "[config.json](config.json)".to_string(),
                "JSON Schema".to_string(),
                "Configuration file schema (generated via schemars)".to_string(),
            ],
            vec![
                "[state.json](state.json)".to_string(),
                "JSON Schema".to_string(),
                "Runtime state file schema (generated via schemars)".to_string(),
            ],
            vec![
                "[openapi.json](openapi.json)".to_string(),
                "OpenAPI 3.0".to_string(),
                "REST API specification (generated via utoipa)".to_string(),
            ],
        ];
        output.push_str(&table(json_headers, &json_rows));

        // TypeScript types
        output.push_str(&heading(2, "TypeScript Types"));
        output.push_str(
            "TypeScript type definitions are available for frontend integration:\n\n\
            - [TypeScript API Documentation](/typescript/) - Generated via TypeDoc\n\
            - Source: `shared/types.ts` (generated via ts-rs)\n\n",
        );

        // Regeneration instructions
        output.push_str(&heading(2, "Regenerating Schemas"));
        output.push_str(
            "Schemas are auto-generated from source code. To regenerate:\n\n\
            ```bash\n\
            # Generate JSON schemas and TypeScript types\n\
            cargo run --bin generate_types\n\n\
            # Generate documentation pages\n\
            cargo run -- docs\n\n\
            # Generate TypeScript API docs\n\
            npm run docs:typescript\n\
            ```\n",
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = SchemaIndexDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));

        // Should have the main heading
        assert!(result.contains("# Schema Reference"));

        // Should list all schema pages
        assert!(result.contains("[Configuration](config/)"));
        assert!(result.contains("[Application State](state/)"));
        assert!(result.contains("[Issue Type](issuetype/)"));
        assert!(result.contains("[Ticket Metadata](metadata/)"));
        assert!(result.contains("[REST API](api/)"));

        // Should list raw JSON files
        assert!(result.contains("config.json"));
        assert!(result.contains("state.json"));
        assert!(result.contains("openapi.json"));

        // Should have regeneration instructions
        assert!(result.contains("cargo run --bin generate_types"));
    }
}
