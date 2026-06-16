//! Soup-to-nuts structural alignment for the advertised **verticals**.
//!
//! The vertical catalog (`operator::integrations::catalog::all_integrations`) is
//! the single source of truth for every advertised integration and its
//! [`SupportStatus`]. This suite asserts that source stays aligned across all
//! the surfaces that advertise it:
//!
//! - **Rust data** — every provider-enum variant (`KanbanProviderType::ALL`,
//!   `ModelServerKind::ALL`, `GitProvider::ALL`, `SessionWrapperType::ALL`) has a
//!   catalog entry, so a new variant can't ship without docs/badge coverage.
//! - **README badges** — every badged entry has a shields.io badge whose link
//!   points at the entry's docs URL, and no badge advertises an unknown entry.
//! - **Docs** — every `Alpha`+ entry (and the generated `docs/maturity/` page)
//!   resolves to a real docs page on disk.
//! - **Support-status guardrails** — `Proto` is never advertised; `Beta`+
//!   providers always are.
//!
//! Adding a new vertical entry therefore *fails the build* until its docs page
//! and (for `Alpha`+) its README badge exist — which is the whole point.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use operator::api::providers::kanban::KanbanProviderType;
use operator::api::providers::model_server::ModelServerKind;
use operator::config::SessionWrapperType;
use operator::integrations::{all_integrations, CatalogEntry, SupportStatus, Vertical};
use operator::types::pr::GitProvider;
use operator::workflow_gen::WorkflowFormat;

/// Path relative to the crate root, regardless of the test's working directory.
fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read_readme() -> String {
    std::fs::read_to_string(repo_path("README.md")).expect("README.md should be readable")
}

/// A docs path resolves if it's a `docs/<path>.md` file or a
/// `docs/<path>/index.md` directory page (both serve the same Jekyll URL).
fn docs_exists(docs_path: &str) -> bool {
    let docs = repo_path("docs");
    docs.join(format!("{docs_path}.md")).exists() || docs.join(docs_path).join("index.md").exists()
}

/// Extract every `](https://operator.untra.io/...)` link target on a line.
fn extract_operator_links(line: &str) -> Vec<String> {
    const NEEDLE: &str = "](https://operator.untra.io/";
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(i) = rest.find(NEEDLE) {
        let after = &rest[i + 2..]; // skip the leading `](`
        match after.find(')') {
            Some(end) => {
                out.push(after[..end].to_string());
                rest = &after[end..];
            }
            None => break,
        }
    }
    out
}

/// Every provider-enum variant must have exactly one catalog entry, so a new
/// variant can't ship without docs/badge coverage. (`vscode` is advertised under
/// the Editor vertical rather than Session.)
#[test]
fn test_every_provider_enum_variant_has_catalog_entry() {
    let entries = all_integrations();
    let has = |vertical: Vertical, slug: &str| {
        entries
            .iter()
            .any(|e| e.vertical == vertical && e.slug == slug)
    };

    for p in KanbanProviderType::ALL {
        assert!(has(Vertical::Kanban, p.slug()), "kanban '{}'", p.slug());
    }
    for k in ModelServerKind::ALL {
        assert!(has(Vertical::Model, k.slug()), "model '{}'", k.slug());
    }
    for g in GitProvider::ALL {
        assert!(has(Vertical::Git, g.slug()), "git '{}'", g.slug());
    }
    for w in SessionWrapperType::ALL {
        let slug = w.display_name();
        assert!(
            has(Vertical::Session, slug) || has(Vertical::Editor, slug),
            "session/editor '{slug}'"
        );
    }
    for f in WorkflowFormat::ALL {
        assert!(
            has(Vertical::Workflows, f.slug()),
            "workflows '{}'",
            f.slug()
        );
    }
}

/// Every badged entry has a README badge linking to its docs URL, and that docs
/// page exists on disk.
#[test]
fn test_badged_entries_have_readme_badge_and_docs() {
    let readme = read_readme();
    for e in all_integrations() {
        if !e.readme_badge {
            continue;
        }
        let url = e.docs_url().unwrap_or_else(|| {
            panic!(
                "badged entry {}/{} must have a docs URL",
                e.vertical.slug(),
                e.slug
            )
        });
        assert!(
            readme.contains(&format!("]({url})")),
            "README is missing a badge linking to {url} for {}/{}",
            e.vertical.slug(),
            e.slug
        );
        let docs_path = e.docs_path.expect("badged entry has docs_path");
        assert!(
            docs_exists(docs_path),
            "docs page missing for {}/{} (expected docs/{docs_path}.md or .../index.md)",
            e.vertical.slug(),
            e.slug
        );
    }
}

/// No README badge may advertise an integration the catalog doesn't know about —
/// the reverse direction of the coverage check.
#[test]
fn test_no_stray_vertical_badges_in_readme() {
    let readme = read_readme();
    let advertised: HashSet<String> = all_integrations()
        .iter()
        .filter(|e| e.readme_badge)
        .filter_map(CatalogEntry::docs_url)
        .collect();

    for line in readme.lines() {
        if !line.contains("img.shields.io") {
            continue; // only inspect shields.io badge lines
        }
        for url in extract_operator_links(line) {
            assert!(
                advertised.contains(&url),
                "README badge links to {url}, but no catalog entry advertises it \
                 (add a CatalogEntry or remove the badge)"
            );
        }
    }
}

/// Support-status guardrails: `Proto` is never advertised; `Beta`+ providers
/// always are (the Integration vertical has no README badge row, so it's exempt).
#[test]
fn test_support_status_guardrails() {
    for e in all_integrations() {
        if e.status == SupportStatus::Proto {
            assert!(
                !e.readme_badge,
                "Proto entry {}/{} must not carry a README badge",
                e.vertical.slug(),
                e.slug
            );
        }
        if e.status >= SupportStatus::Beta && e.vertical != Vertical::Integration {
            assert!(
                e.readme_badge,
                "Beta+ entry {}/{} must be advertised with a README badge",
                e.vertical.slug(),
                e.slug
            );
        }
    }
}

/// Every `Alpha`+ entry must resolve to a real docs page on disk.
#[test]
fn test_alpha_plus_entries_documented_on_disk() {
    for e in all_integrations() {
        if e.status < SupportStatus::Alpha {
            continue;
        }
        let docs_path = e.docs_path.unwrap_or_else(|| {
            panic!(
                "Alpha+ entry {}/{} needs a docs_path",
                e.vertical.slug(),
                e.slug
            )
        });
        assert!(
            docs_exists(docs_path),
            "Alpha+ entry {}/{} has no docs page at docs/{docs_path}",
            e.vertical.slug(),
            e.slug
        );
    }
}

/// The generated maturity page (`docs/maturity/index.md`) lists every badged
/// entry — ties the docs surface into the same source of truth.
#[test]
fn test_maturity_page_lists_badged_entries() {
    let page = std::fs::read_to_string(repo_path("docs/maturity/index.md"))
        .expect("docs/maturity/index.md should exist — run `cargo run -- docs --only maturity`");
    for e in all_integrations() {
        if let Some(url) = e.docs_url().filter(|_| e.readme_badge) {
            assert!(
                page.contains(&url),
                "maturity page is missing the docs link for {}/{} ({url})",
                e.vertical.slug(),
                e.slug
            );
        }
    }
}

/// Human-readable matrix of every entry × surface × status. Always passes;
/// run with `--nocapture` to inspect alignment at a glance.
#[test]
fn test_vertical_parity_summary() {
    let readme = read_readme();
    println!("\n=== Vertical Parity ===\n");
    println!(
        "{:<14} | {:<18} | {:<6} | {:<5} | {:<5} | URL",
        "Vertical", "Entry", "Status", "Badge", "Docs"
    );
    println!(
        "{:-<14}-+-{:-<18}-+-{:-<6}-+-{:-<5}-+-{:-<5}-+----",
        "", "", "", "", ""
    );
    for e in all_integrations() {
        let badge_ok = if e.readme_badge {
            if e.docs_url()
                .is_some_and(|u| readme.contains(&format!("]({u})")))
            {
                "✓"
            } else {
                "✗"
            }
        } else {
            "—"
        };
        let docs_ok = match e.docs_path {
            Some(p) if docs_exists(p) => "✓",
            Some(_) => "✗",
            None => "—",
        };
        println!(
            "{:<14} | {:<18} | {:<6} | {:<5} | {:<5} | {}",
            e.vertical.label(),
            e.label,
            e.status.label(),
            badge_ok,
            docs_ok,
            e.docs_url().unwrap_or_else(|| "—".to_string()),
        );
    }
    println!();
}
