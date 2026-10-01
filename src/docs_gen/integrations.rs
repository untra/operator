//! Feature-maturity documentation generator.
//!
//! Emits `docs/maturity/index.md` from the vertical catalog
//! ([`crate::integrations::catalog::all_integrations`]) - a human-facing
//! companion to the machine-checked `tests/vertical_parity.rs`. Because it is
//! derived from the same source of truth as the REST `/api/v1/integrations`
//! endpoint and the README badges, the page can never drift from reality.

use anyhow::Result;

use super::{format_header, DocGenerator};
use crate::integrations::support_catalog::{
    compatibility, llm_tool_supports, model_supports, Compatibility,
};
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
            "Operator integrates with many providers and tools across several **verticals**. \
             Each integration carries an official **support status** so you know what to expect \
             before you depend on it. This page is generated from the same source of truth that \
             drives the README badges and the `/api/v1/integrations` API, so it always reflects \
             the current state.\n\n\
             ## Support levels\n\n",
        );

        // Legend - one colored badge + blurb per level, most→least mature.
        for status in [
            SupportStatus::Ga,
            SupportStatus::Beta,
            SupportStatus::Alpha,
            SupportStatus::Proto,
        ] {
            content.push_str(&format!(
                "- {badge} - {blurb}\n",
                badge = status_badge(status),
                blurb = status.blurb(),
            ));
        }

        // One table per vertical, in README order.
        let entries = all_integrations();
        for vertical in Vertical::ALL {
            let rows: Vec<_> = entries
                .iter()
                .filter(|e| e.vertical == vertical && e.is_public())
                .collect();
            if rows.is_empty() {
                continue;
            }
            content.push_str(&format!("\n## {}\n\n", vertical.label()));
            content.push_str("| Integration | Status | Availability | Docs |\n|---|---|---|---|\n");
            for e in rows {
                let docs = match e.docs_url() {
                    Some(url) => format!("[{}]({})", e.label, url),
                    None => "-".to_string(),
                };
                content.push_str(&format!(
                    "| {label} | {badge} | {availability} | {docs} |\n",
                    label = e.label,
                    badge = status_badge(e.status),
                    availability = if e.premium { "Premium" } else { "Included" },
                ));
            }
        }

        content.push_str(&write_llm_capability_section());
        content.push_str(&write_model_capability_section());
        content.push_str(&compat_matrix_section());

        Ok(content)
    }
}

fn write_llm_capability_section() -> String {
    let mut out = String::from(
        "\n## LLM tool capabilities\n\n\
         Advertising status (GA/Beta/Alpha) is not the same as a live connection probe. \
         LLM tool **health** is `path-version`: the binary is on PATH. That is weaker than a \
         model provider's `/models` probe, which proves an API key is accepted.\n\n\
         | Tool | Health | Auth | Native protocol | Sessions | Headless | YOLO | Relay |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    let entries = all_integrations();
    for (slug, support) in llm_tool_supports() {
        let label = entries
            .iter()
            .find(|e| e.vertical == Vertical::LlmTool && e.slug == *slug)
            .map(|e| e.label)
            .unwrap_or(*slug);
        let protocols: Vec<&str> = support.native_protocols.iter().map(|p| p.slug()).collect();
        out.push_str(&format!(
            "| {label} | {health} | {auth} | {protocols} | {sessions} | {headless} | {yolo} | {relay} |\n",
            health = support.health.slug(),
            auth = support.auth.slug(),
            protocols = protocols.join(", "),
            sessions = yn(support.sessions),
            headless = yn(support.headless),
            yolo = yn(support.yolo),
            relay = support.relay.slug(),
        ));
    }
    out
}

fn write_model_capability_section() -> String {
    let mut out = String::from(
        "\n## Model provider capabilities\n\n\
         A model provider is **connected** when its model-list probe succeeds. \
         Gateways (Ollama, OpenRouter, OpenAI-compatible) speak the OpenAI protocol; \
         first-party Anthropic and Google do not.\n\n\
         | Provider | Protocol | Class | Probe | Key injectable | Implicit for |\n\
         |---|---|---|---|---|---|\n",
    );
    let entries = all_integrations();
    for (slug, support) in model_supports() {
        let Some(entry) = entries
            .iter()
            .find(|e| e.vertical == Vertical::Model && e.slug == *slug)
        else {
            continue;
        };
        if !entry.is_public() {
            continue;
        }
        out.push_str(&format!(
            "| {label} | {protocol} | {class} | {probe} | {key} | {implicit} |\n",
            label = entry.label,
            protocol = support.protocol.slug(),
            class = support.class.slug(),
            probe = yn(support.probe),
            key = support.key_injectable.slug(),
            implicit = support.implicit_for.unwrap_or("-"),
        ));
    }
    out
}

fn compat_matrix_section() -> String {
    let entries = all_integrations();
    let tools: Vec<_> = llm_tool_supports()
        .iter()
        .filter_map(|(slug, _)| {
            entries
                .iter()
                .find(|e| e.vertical == Vertical::LlmTool && e.slug == *slug && e.is_public())
                .map(|e| (e.slug, e.label))
        })
        .collect();
    let models: Vec<_> = model_supports()
        .iter()
        .filter_map(|(slug, _)| {
            entries
                .iter()
                .find(|e| e.vertical == Vertical::Model && e.slug == *slug && e.is_public())
                .map(|e| (e.slug, e.label))
        })
        .collect();

    let mut out = String::from(
        "\n## LLM tool × model provider\n\n\
         **Native** — the CLI speaks this provider's protocol. \
         **Bridge** — a protocol-preserving front (claude-code-router, litellm, …) is required. \
         **Incompatible** — this first-party API is the wrong protocol for the CLI.\n\n\
         Operator does not currently block incompatible delegators at launch; \
         this matrix is the catalog fact.\n\n| Tool |",
    );
    for (_, label) in &models {
        out.push_str(&format!(" {label} |"));
    }
    out.push('\n');
    out.push('|');
    out.push_str(&"---|".repeat(models.len() + 1));
    out.push('\n');
    for (tool_slug, tool_label) in &tools {
        out.push_str(&format!("| {tool_label} |"));
        for (model_slug, _) in &models {
            let cell = compatibility(tool_slug, model_slug)
                .map(Compatibility::label)
                .unwrap_or("-");
            out.push_str(&format!(" {cell} |"));
        }
        out.push('\n');
    }
    out
}

fn yn(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
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
        assert!(content.contains("title: \"Feature Maturity\""));
        assert!(!content.contains("\n# Feature Maturity"));
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
        assert!(content.contains("## Remote Targets"));
        assert!(content.contains("| Premium |"));
        assert!(!content.contains("| Cursor |"));
        // A known row with a docs link.
        assert!(content.contains("[Jira](https://operator.untra.io/getting-started/kanban/jira/)"));
        assert!(content.contains("## LLM tool capabilities"));
        assert!(content.contains("## Model provider capabilities"));
        assert!(content.contains("## LLM tool × model provider"));
        assert!(content.contains("path-version"));
        assert!(content.contains("| Claude |"));
        assert!(content.contains("Native"));
        assert!(content.contains("Bridge"));
        assert!(content.contains("Incompatible"));
        // AUTO-GENERATED header present.
        assert!(content.contains("AUTO-GENERATED FROM"));
    }
}
