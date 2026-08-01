#![allow(dead_code)]
#![allow(clippy::redundant_closure)]

//! Documentation generation from code and schema sources.
//!
//! This module provides auto-documentation generation for:
//! - Project taxonomy (24 Kinds from taxonomy.toml)
//! - Issue type schemas (from `issuetype_schema.json`)
//! - Ticket metadata schema (from `ticket_metadata.schema.json`)
//! - Keyboard shortcuts (from keybindings registry)
//! - CLI reference (from clap definitions and env vars registry)
//! - Configuration reference (from config.rs via schemars)
//!
//! Generated docs include a header warning and are written to `docs/`.

pub mod cli;
pub mod collections_manifest;
pub mod collections_pages;
pub mod collections_search;
pub mod config;
pub mod config_schema;
pub mod integrations;
pub mod issuetype;
pub mod issuetype_json_schema;
pub mod jira_api;
pub mod llm_tools;
pub mod llms;
pub mod markdown;
pub mod metadata;
pub mod openapi;
pub mod operator_output_schema;
pub mod project_analysis_schema;
pub mod schema_index;
pub mod shortcuts;
pub mod startup;
pub mod state_schema;
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

/// Every generator `operator docs` runs, paired with its `--only` key.
///
/// **Single source of truth.** [`generate_all`], the `docs` CLI command, its
/// `--only` filter, and the CLI help text all read this one list, so a new
/// generator cannot be registered in one place and silently missed by another.
///
/// A few keys differ from the generator's `name()` for historical reasons; the
/// keys are the documented CLI contract and are what belongs here.
///
/// `LlmToolsDocGenerator` is deliberately **not** listed: `docs/llm-tools/index.md`
/// is hand-written, and running the generator would overwrite it. It stays
/// reachable by key for a deliberate regeneration.
pub fn all_generators() -> Vec<(&'static str, Box<dyn DocGenerator>)> {
    vec![
        ("taxonomy", Box::new(taxonomy::TaxonomyDocGenerator)),
        (
            "issuetype",
            Box::new(issuetype::IssuetypeSchemaDocGenerator),
        ),
        ("metadata", Box::new(metadata::MetadataSchemaDocGenerator)),
        ("shortcuts", Box::new(shortcuts::ShortcutsDocGenerator)),
        ("cli", Box::new(cli::CliDocGenerator)),
        ("config", Box::new(config::ConfigDocGenerator)),
        ("openapi", Box::new(openapi::OpenApiDocGenerator)),
        ("startup", Box::new(startup::StartupDocGenerator)),
        (
            "config-schema",
            Box::new(config_schema::ConfigSchemaDocGenerator),
        ),
        (
            "state-schema",
            Box::new(state_schema::StateSchemaDocGenerator),
        ),
        (
            "schema-index",
            Box::new(schema_index::SchemaIndexDocGenerator),
        ),
        ("jira-api", Box::new(jira_api::JiraApiDocGenerator)),
        (
            "operator-output-schema",
            Box::new(operator_output_schema::OperatorOutputSchemaDocGenerator),
        ),
        (
            "issuetype-json-schema",
            Box::new(issuetype_json_schema::IssuetypeJsonSchemaDocGenerator),
        ),
        (
            "project-analysis-schema",
            Box::new(project_analysis_schema::ProjectAnalysisSchemaDocGenerator),
        ),
        ("llms", Box::new(llms::LlmsTxtDocGenerator)),
        (
            "collections-manifest",
            Box::new(collections_manifest::CollectionsManifestGenerator),
        ),
        (
            "collections-search",
            Box::new(collections_search::CollectionsSearchGenerator),
        ),
        (
            "collections-pages",
            Box::new(collections_pages::CollectionsPagesGenerator),
        ),
        ("maturity", Box::new(integrations::MaturityDocGenerator)),
    ]
}

/// Generators reachable by `--only` but excluded from a full run, because they
/// would overwrite a page that is currently maintained by hand.
fn opt_in_generators() -> Vec<(&'static str, Box<dyn DocGenerator>)> {
    vec![("llm-tools", Box::new(llm_tools::LlmToolsDocGenerator))]
}

/// Resolve a `--only` key to its generator.
pub fn generator_for_key(key: &str) -> Option<Box<dyn DocGenerator>> {
    all_generators()
        .into_iter()
        .chain(opt_in_generators())
        .find(|(k, _)| *k == key)
        .map(|(_, generator)| generator)
}

/// Every valid `--only` key, for CLI help.
pub fn generator_keys() -> Vec<&'static str> {
    all_generators()
        .into_iter()
        .chain(opt_in_generators())
        .map(|(key, _)| key)
        .collect()
}

/// Generate all documentation
pub fn generate_all(docs_dir: &Path) -> Result<()> {
    for (_, generator) in all_generators() {
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
