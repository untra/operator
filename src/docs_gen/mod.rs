#![allow(dead_code)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::double_ended_iterator_last)]

//! Documentation generation from code and schema sources.
//!
//! This module provides auto-documentation generation for:
//! - Project taxonomy (24 Kinds from taxonomy.toml)
//! - Issue type schemas (from issuetype_schema.json)
//! - Ticket metadata schema (from ticket_metadata.schema.json)
//!
//! Generated docs include a header warning and are written to `docs/`.

pub mod issuetype;
pub mod markdown;
pub mod metadata;
pub mod taxonomy;

use anyhow::Result;
use std::path::Path;

/// Header added to all auto-generated documentation files
pub const AUTO_GEN_HEADER: &str = r#"---
title: "{title}"
layout: doc
---

<!-- AUTO-GENERATED FROM {source} - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

"#;

/// Trait for documentation generators
pub trait DocGenerator {
    /// Name of this generator (for logging)
    fn name(&self) -> &'static str;

    /// Source file(s) this generator reads from
    fn source(&self) -> &'static str;

    /// Output path relative to docs/ directory
    fn output_path(&self) -> &'static str;

    /// Generate the documentation content
    fn generate(&self) -> Result<String>;

    /// Write the generated documentation to disk
    fn write(&self, docs_dir: &Path) -> Result<()> {
        let content = self.generate()?;
        let output_path = docs_dir.join(self.output_path());

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&output_path, content)?;
        tracing::info!(
            generator = self.name(),
            output = %output_path.display(),
            "Generated documentation"
        );
        Ok(())
    }
}

/// Generate all documentation
pub fn generate_all(docs_dir: &Path) -> Result<()> {
    let generators: Vec<Box<dyn DocGenerator>> = vec![
        Box::new(taxonomy::TaxonomyDocGenerator),
        Box::new(issuetype::IssuetypeSchemaDocGenerator),
        Box::new(metadata::MetadataSchemaDocGenerator),
    ];

    for generator in generators {
        generator.write(docs_dir)?;
    }

    Ok(())
}

/// Format the auto-generated header with title and source
pub fn format_header(title: &str, source: &str) -> String {
    AUTO_GEN_HEADER
        .replace("{title}", title)
        .replace("{source}", source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_header() {
        let header = format_header("Test Title", "test.toml");
        assert!(header.contains("title: \"Test Title\""));
        assert!(header.contains("AUTO-GENERATED FROM test.toml"));
        assert!(header.contains("DO NOT EDIT MANUALLY"));
    }
}
