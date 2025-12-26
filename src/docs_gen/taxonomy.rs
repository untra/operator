//! Documentation generator for the 24-Kind project taxonomy.

use super::markdown::{bold, bullet_list, heading, inline_code, table};
use super::{format_header, DocGenerator};
use crate::backstage::taxonomy::{KindTier, Taxonomy};
use anyhow::Result;

/// Generates taxonomy documentation from taxonomy.toml
pub struct TaxonomyDocGenerator;

impl DocGenerator for TaxonomyDocGenerator {
    fn name(&self) -> &'static str {
        "taxonomy"
    }

    fn source(&self) -> &'static str {
        "src/backstage/taxonomy.toml"
    }

    fn output_path(&self) -> &'static str {
        "backstage/taxonomy.md"
    }

    fn generate(&self) -> Result<String> {
        let taxonomy = Taxonomy::load();
        let mut output = format_header("Project Taxonomy", self.source());

        // Introduction
        output.push_str(&heading(1, "Project Taxonomy"));
        output.push_str(&format!(
            "This document defines the **{} project Kinds** organized into **{} tiers**.\n\n",
            taxonomy.kinds.len(),
            taxonomy.tiers.len()
        ));
        output.push_str(
            "Each Kind represents a category of project that can be cataloged in Backstage. ",
        );
        output.push_str("The taxonomy is used by the `ASSESS` issue type to classify projects and generate `catalog-info.yaml` files.\n\n");

        // Version info
        output.push_str(&heading(2, "Version"));
        output.push_str(&format!(
            "- {}: {}\n",
            bold("Version"),
            inline_code(&taxonomy.meta.version)
        ));
        output.push_str(&format!(
            "- {}: {}\n\n",
            bold("Description"),
            &taxonomy.meta.description
        ));

        // Quick reference table
        output.push_str(&heading(2, "Quick Reference"));
        output.push_str("All 24 Kinds at a glance:\n\n");
        output.push_str(&self.generate_summary_table(taxonomy));

        // Tier sections
        for tier_enum in KindTier::all() {
            output.push_str(&self.generate_tier_section(taxonomy, *tier_enum));
        }

        // File pattern reference
        output.push_str(&self.generate_pattern_reference(taxonomy));

        // Backstage type mapping
        output.push_str(&self.generate_backstage_mapping(taxonomy));

        Ok(output)
    }
}

impl TaxonomyDocGenerator {
    fn generate_summary_table(&self, taxonomy: &Taxonomy) -> String {
        let headers = &["ID", "Key", "Name", "Tier", "Backstage Type"];
        let rows: Vec<Vec<String>> = taxonomy
            .kinds
            .iter()
            .map(|k| {
                vec![
                    k.id.to_string(),
                    inline_code(&k.key),
                    k.name.clone(),
                    k.tier.clone(),
                    inline_code(&k.backstage_type),
                ]
            })
            .collect();

        table(headers, &rows)
    }

    fn generate_tier_section(&self, taxonomy: &Taxonomy, tier: KindTier) -> String {
        let tier_def = taxonomy.tier_def(tier);
        let kinds = taxonomy.kinds_by_tier(tier);

        let mut output = String::new();

        if let Some(t) = tier_def {
            let heading_text = if let Some((start, end)) = t.range {
                format!("Tier: {} (Kinds {}-{})", t.name, start, end)
            } else {
                format!("Tier: {}", t.name)
            };
            output.push_str(&heading(2, &heading_text));
            output.push_str(&format!("{}\n\n", t.description));
        } else {
            output.push_str(&heading(2, &format!("Tier: {}", tier)));
        }

        // Table for this tier
        let headers = &["ID", "Key", "Name", "Stakeholder", "Output"];
        let rows: Vec<Vec<String>> = kinds
            .iter()
            .map(|k| {
                vec![
                    k.id.to_string(),
                    inline_code(&k.key),
                    k.name.clone(),
                    k.stakeholder.clone(),
                    k.output.clone(),
                ]
            })
            .collect();

        output.push_str(&table(headers, &rows));

        // Detailed descriptions for each kind in this tier
        for kind in kinds {
            output.push_str(&heading(3, &format!("{} - {}", kind.id, kind.name)));
            output.push_str(&format!("{}\n\n", kind.description));

            let details = vec![
                format!("{}: {}", bold("Key"), inline_code(&kind.key)),
                format!("{}: {}", bold("Stakeholder"), &kind.stakeholder),
                format!("{}: {}", bold("Primary Output"), &kind.output),
                format!(
                    "{}: {}",
                    bold("Backstage Type"),
                    inline_code(&kind.backstage_type)
                ),
            ];
            output.push_str(&bullet_list(&details));

            // File patterns
            output.push_str(&format!("{} File Patterns:\n", bold("Detection")));
            let patterns: Vec<String> = kind.file_patterns.iter().map(|p| inline_code(p)).collect();
            output.push_str(&bullet_list(&patterns));
        }

        output
    }

    fn generate_pattern_reference(&self, taxonomy: &Taxonomy) -> String {
        let mut output = heading(2, "File Pattern Detection");
        output.push_str("The taxonomy uses file pattern matching to suggest project Kinds. ");
        output.push_str("When analyzing a project, patterns are matched against file paths, ");
        output.push_str("and the Kind with the most matches is suggested.\n\n");

        output.push_str(&heading(3, "Pattern Syntax"));
        output.push_str("Patterns use glob syntax:\n\n");
        let syntax_items = vec![
            format!("{} - Match any characters except `/`", inline_code("*")),
            format!("{} - Match any characters including `/`", inline_code("**")),
            format!("{} - Match any single character", inline_code("?")),
            format!("{} - Match any character in brackets", inline_code("[abc]")),
        ];
        output.push_str(&bullet_list(&syntax_items));

        output.push_str(&heading(3, "All Patterns by Kind"));
        for kind in &taxonomy.kinds {
            output.push_str(&format!(
                "{} ({}):\n",
                bold(&kind.name),
                inline_code(&kind.key)
            ));
            let patterns: Vec<String> = kind.file_patterns.iter().map(|p| inline_code(p)).collect();
            output.push_str(&bullet_list(&patterns));
        }

        output
    }

    fn generate_backstage_mapping(&self, taxonomy: &Taxonomy) -> String {
        let mut output = heading(2, "Backstage Type Mapping");
        output.push_str("Each Kind maps to a Backstage catalog type:\n\n");

        let headers = &["Backstage Type", "Kinds"];
        let mut type_map: std::collections::HashMap<&str, Vec<&str>> =
            std::collections::HashMap::new();

        for kind in &taxonomy.kinds {
            type_map
                .entry(&kind.backstage_type)
                .or_default()
                .push(&kind.key);
        }

        let mut rows: Vec<Vec<String>> = type_map
            .into_iter()
            .map(|(btype, kinds)| {
                vec![
                    inline_code(btype),
                    kinds
                        .into_iter()
                        .map(inline_code)
                        .collect::<Vec<_>>()
                        .join(", "),
                ]
            })
            .collect();

        // Sort by backstage type
        rows.sort_by(|a, b| a[0].cmp(&b[0]));

        output.push_str(&table(headers, &rows));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_valid_markdown() {
        let generator = TaxonomyDocGenerator;
        let result = generator.generate().unwrap();

        // Should have the auto-generated header
        assert!(result.contains("AUTO-GENERATED FROM"));
        assert!(result.contains("taxonomy.toml"));

        // Should have the main heading
        assert!(result.contains("# Project Taxonomy"));

        // Should mention project Kinds (flexible - any count)
        assert!(result.contains("project Kinds"));

        // Should have tier headings for each tier
        let taxonomy = Taxonomy::load();
        for tier in &taxonomy.tiers {
            assert!(
                result.contains(&format!("## Tier: {}", tier.name)),
                "Missing tier heading for {}",
                tier.name
            );
        }

        // Should have the quick reference table
        assert!(result.contains("## Quick Reference"));
        assert!(result.contains("| ID | Key | Name |"));
    }

    #[test]
    fn test_summary_table_has_all_kinds() {
        let generator = TaxonomyDocGenerator;
        let taxonomy = Taxonomy::load();
        let table = generator.generate_summary_table(taxonomy);

        // Should have header + separator + data rows
        // Count non-empty lines (excludes trailing blank line)
        let line_count = table.lines().filter(|l| !l.is_empty()).count();
        // At least: 1 header + 1 separator + number of kinds
        let expected_min = 2 + taxonomy.kinds.len();
        assert_eq!(
            line_count,
            expected_min,
            "Table should have header + separator + {} data rows",
            taxonomy.kinds.len()
        );
    }
}
