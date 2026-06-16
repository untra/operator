//! Feature-maturity documentation generator.
//!
//! Emits `docs/maturity/index.md` from the vertical catalog
//! ([`crate::integrations::catalog::all_integrations`]) — a human-facing
//! companion to the machine-checked `tests/vertical_parity.rs`. Because it is
//! derived from the same source of truth as the REST `/api/v1/integrations`
//! endpoint and the README badges, the page can never drift from reality.

use anyhow::Result;

use super::{format_header, DocGenerator};
use crate::integrations::{all_integrations, SupportStatus, Vertical};

/// Generator for the feature-maturity page.
pub struct MaturityDocGenerator;

/// A shields.io badge for a support status, e.g.
/// `![Beta](https://img.shields.io/badge/Beta-E8A33D)`.
fn status_badge(status: SupportStatus) -> String {
    format!(
        "![{label}](https://img.shields.io/badge/{label}-{color})",
        label = status.label(),
        color = status.badge_color(),
    )
}

impl DocGenerator for MaturityDocGenerator {
    fn name(&self) -> &'static str {
        "maturity"
    }

    fn source(&self) -> &'static str {
        "src/integrations/catalog.rs"
    }

    fn output_path(&self) -> &'static str {
        "maturity/index.md"
    }

    fn generate(&self) -> Result<String> {
        let mut content = format_header("Feature Maturity", self.source());

        content.push_str(
            "# Feature Maturity\n\n\
             Operator integrates with many providers and tools across several **verticals**. \
             Each integration carries an official **support status** so you know what to expect \
             before you depend on it. This page is generated from the same source of truth that \
             drives the README badges and the `/api/v1/integrations` API, so it always reflects \
             the current state.\n\n\
             ## Support levels\n\n",
        );

        // Legend — one colored badge + blurb per level, most→least mature.
        for status in [
            SupportStatus::Ga,
            SupportStatus::Beta,
            SupportStatus::Alpha,
            SupportStatus::Proto,
        ] {
            content.push_str(&format!(
                "- {badge} — {blurb}\n",
                badge = status_badge(status),
                blurb = status.blurb(),
            ));
        }

        // One table per vertical, in README order.
        let entries = all_integrations();
        for vertical in Vertical::ALL {
            let rows: Vec<_> = entries.iter().filter(|e| e.vertical == vertical).collect();
            if rows.is_empty() {
                continue;
            }
            content.push_str(&format!("\n## {}\n\n", vertical.label()));
            content.push_str("| Integration | Status | Docs |\n|---|---|---|\n");
            for e in rows {
                let docs = match e.docs_url() {
                    Some(url) => format!("[{}]({})", e.label, url),
                    None => "—".to_string(),
                };
                content.push_str(&format!(
                    "| {label} | {badge} | {docs} |\n",
                    label = e.label,
                    badge = status_badge(e.status),
                ));
            }
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maturity_generator_metadata() {
        let gen = MaturityDocGenerator;
        assert_eq!(gen.name(), "maturity");
        assert_eq!(gen.output_path(), "maturity/index.md");
        assert!(gen.source().contains("catalog.rs"));
    }

    #[test]
    fn test_maturity_content_has_legend_and_tables() {
        let content = MaturityDocGenerator.generate().unwrap();
        assert!(content.contains("# Feature Maturity"));
        assert!(content.contains("## Support levels"));
        // Legend badges for all four levels.
        for status in SupportStatus::ALL {
            assert!(
                content.contains(&format!("badge/{}-", status.label())),
                "legend should contain a {} badge",
                status.label()
            );
        }
        // Per-vertical tables.
        assert!(content.contains("## Kanban Provider"));
        assert!(content.contains("## Model Provider"));
        // A known row with a docs link.
        assert!(content.contains("[Jira](https://operator.untra.io/getting-started/kanban/jira/)"));
        // AUTO-GENERATED header present.
        assert!(content.contains("AUTO-GENERATED FROM"));
    }
}
