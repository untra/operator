//! Workflow catalog page generator.
//!
//! Emits the browsable face of the hosted collection bundle:
//!
//! ```text
//! docs/workflows/
//! ├── index.md          the catalog: vocabulary, cards, table, contributor CTA
//! └── <id>/index.md     one collection: metadata + the split-view explorer
//! ```
//!
//! Cards and table rows are rendered **here**, at generation time, each carrying
//! a `data-search` haystack. `<operator-collection-search>` then filters the DOM
//! that already exists, so the catalog renders in full with `JavaScript`
//! disabled
//! and the search never depends on a fetch.
//!
//! The graph is the one exception: `<operator-workflow-explorer>` loads the
//! collection's `<KEY>.json` at runtime and draws it with the same component the
//! SPA uses. Nothing per-workflow is pre-rendered.

use anyhow::Result;
use std::path::Path;

use super::{format_header, DocGenerator};
use crate::docs_gen::collections_search::{build_catalog, CatalogEntry};

/// The vocabulary preamble. "Workflow" is overloaded across the ecosystem, so
/// the catalog opens by saying exactly what each term means here.
const VOCABULARY: &str = r"
An **Operator workflow** is a process defined once in JSON: an ordered graph of
typed steps, review gates, and retry edges that an LLM agent can follow. It is
the native format — Operator runs it directly, and every
[export format](/getting-started/workflows/) (Claude, AGNT) is derived from it.

Three terms, three different things:

| Term | What it is |
|------|-----------|
| **Operator workflow** | The step graph itself. Lives in an issue type's `steps`. |
| **Issue type** | One kind of work — `FEAT`, `PRD`, `ELVSTAGE`. Carries identity, input fields, and exactly one Operator workflow. |
| **Collection** | A named, versioned bundle of issue types: a complete, shareable way of working. This page lists them. |

Collections are deliberately separate from your **kanban issue types**. Jira,
Linear, and GitHub Projects types describe how *your* team labels work; a
collection describes how the *agents* do it. Map one onto the other once, and
the workflow travels between projects, teams, and providers unchanged.
";

/// The contributor call to action, mirroring `collections/README.md`.
const CONTRIBUTING: &str = r#"
## Contribute a collection

There is no single best way to run agents — the right loop depends on the work.
That is exactly why these are shareable: a workflow that works for you is worth
publishing, and one that does not fit is worth forking.

Official collections live in the [operator repository](https://github.com/untra/operator/tree/main/collections):

1. Create `collections/community/<id>/`, where `<id>` matches `^[a-z0-9_]{3,64}$`.
2. Add a `collection.json` conforming to [the collection schema](/collections/schema.json),
   with `tier: "community"` plus `author`, `url`, and `license`.
3. Add one `<KEY>.json` per issue type — see [the issue type schema](/schemas/issuetype/) —
   and an optional `<KEY>.md` ticket template.
4. Add an `icon.svg` following the
   [Simple Icons](https://github.com/simple-icons/simple-icons) shape: a 24×24
   viewBox, a single `<path>`, and no `fill` or `stroke` so it inherits the
   page's color.
5. Leave checksums out — they are computed at publish time.
6. Run the CI gate locally, then open a pull request:

```bash
cargo test --test community_collections
```

Submissions are reviewed for prompt quality and safety, not just schema
validity. A good collection describes a workflow shape worth sharing: what loop
it runs, what memory it keeps, what gates it enforces, and when it stops.
"#;

/// Generates `docs/workflows/index.md` and the per-collection pages.
pub struct CollectionsPagesGenerator;

/// Escape text destined for an HTML attribute or text node.
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `1 issue type` / `3 issue types`.
fn issue_type_count(count: usize) -> String {
    if count == 1 {
        "1 issue type".to_string()
    } else {
        format!("{count} issue types")
    }
}

/// The provenance badge shown on a card and in the table.
fn tier_badge(tier: &str) -> String {
    let class = if tier == "community" {
        "badge alpha"
    } else {
        "badge recommended"
    };
    format!(r#"<span class="{class}">{tier}</span>"#)
}

/// The collection's icon, inlined into the page.
///
/// Inlined rather than linked with an `<img>`: an `<img>` loads the SVG as a
/// separate document where `currentColor` cannot resolve, so a linked icon
/// would stay black and vanish against the dark theme. Inline, it tints from
/// the surrounding text color. Marked `aria-hidden` because the collection name
/// sits right beside it.
///
/// The markup is the same file published at `/collections/<id>/icon.svg`, and
/// `tests/collection_icons.rs` constrains it to a single `<path>` with no
/// scripting, so inlining introduces nothing the bundle does not already serve.
fn icon_svg(entry: &CatalogEntry) -> String {
    let Some(svg) = crate::docs_gen::collections_search::icon_svg_for(&entry.id) else {
        return String::new();
    };
    svg.trim().replacen(
        "<svg ",
        r#"<svg class="collection-icon" aria-hidden="true" focusable="false" "#,
        1,
    )
}

fn card(entry: &CatalogEntry) -> String {
    let types = entry
        .issue_types
        .iter()
        .map(|it| format!(r"<code>{}</code>", escape(&it.key)))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        r#"  <article class="collection-card" data-search="{search}">
    <a class="collection-card-link" href="{docs_path}">
      {icon}
      <h3 class="collection-card-title">{name}</h3>
    </a>
    <p class="collection-card-description">{description}</p>
    <p class="collection-card-types">{types}</p>
    <p class="collection-card-meta">
      {badge}
      <span>{count}</span>
      <span>{author}</span>
      <span>updated {updated}</span>
    </p>
  </article>
"#,
        search = escape(&entry.search_text),
        docs_path = escape(&entry.docs_path),
        icon = icon_svg(entry),
        name = escape(&entry.name),
        description = escape(&entry.description),
        types = types,
        badge = tier_badge(&entry.tier),
        count = issue_type_count(entry.issue_type_count),
        author = escape(entry.author.as_deref().unwrap_or("Operator!")),
        updated = escape(entry.updated.as_deref().unwrap_or("—")),
    )
}

fn table_row(entry: &CatalogEntry) -> String {
    format!(
        r#"    <tr data-search="{search}">
      <td>{icon}<a href="{docs_path}">{name}</a></td>
      <td>{description}</td>
      <td>{count}</td>
      <td>{loop_kind}</td>
      <td>{author}</td>
      <td>{badge}</td>
      <td>{created}</td>
      <td>{updated}</td>
    </tr>
"#,
        search = escape(&entry.search_text),
        icon = icon_svg(entry),
        docs_path = escape(&entry.docs_path),
        name = escape(&entry.name),
        description = escape(&entry.description),
        count = entry.issue_type_count,
        loop_kind = escape(entry.loop_kind.as_deref().unwrap_or("—")),
        author = escape(entry.author.as_deref().unwrap_or("Operator!")),
        badge = tier_badge(&entry.tier),
        created = escape(entry.created.as_deref().unwrap_or("—")),
        updated = escape(entry.updated.as_deref().unwrap_or("—")),
    )
}

/// The catalog page: vocabulary, the two statically-rendered views, the CTA.
fn hub_page(entries: &[CatalogEntry]) -> String {
    let mut out = format_header("Workflows", "src/collections/ + collections/community/");
    // Drives the sidebar's active state without matching on URL substrings,
    // which would also light up /getting-started/workflows/.
    out = out.replace("layout: doc\n", "layout: doc\nsection: workflows\n");

    out.push_str(VOCABULARY);
    out.push_str(
        "\nEvery collection below is installable from Operator directly — they are \
         published from this site as a [machine-readable index](/collections/index.json) \
         that operator instances read on startup.\n\n",
    );

    out.push_str(&format!(
        "<operator-collection-search for=\"collection-catalog\"></operator-collection-search>\n\n\
         <div id=\"collection-catalog\" data-view=\"cards\">\n\
         <div class=\"collection-grid\" data-view-target=\"cards\">\n{}</div>\n",
        entries.iter().map(card).collect::<String>()
    ));

    out.push_str(
        "<table class=\"collection-table\" data-view-target=\"table\">\n  <thead>\n    <tr>\
         <th>Collection</th><th>Description</th><th>Issue types</th><th>Loop</th>\
         <th>Author</th><th>Tier</th><th>Created</th><th>Updated</th></tr>\n  </thead>\n  <tbody>\n",
    );
    out.push_str(&entries.iter().map(table_row).collect::<String>());
    out.push_str("  </tbody>\n</table>\n</div>\n");

    out.push_str(CONTRIBUTING);
    out
}

/// A single collection's page: metadata, then the split-view explorer.
fn detail_page(entry: &CatalogEntry) -> String {
    let mut out = format_header(
        &entry.name,
        &format!("the {} collection manifest", entry.id),
    );
    out = out.replace("layout: doc\n", "layout: doc\nsection: workflows\n");

    out.push_str(&format!("{}\n\n", entry.description));

    out.push_str("| | |\n|---|---|\n");
    out.push_str(&format!("| **Tier** | {} |\n", entry.tier));
    if let Some(author) = &entry.author {
        let rendered = entry
            .url
            .as_ref()
            .map_or_else(|| author.clone(), |url| format!("[{author}]({url})"));
        out.push_str(&format!("| **Author** | {rendered} |\n"));
    }
    if let Some(license) = &entry.license {
        out.push_str(&format!("| **License** | {license} |\n"));
    }
    out.push_str(&format!("| **Version** | {} |\n", entry.version));
    if let Some(created) = &entry.created {
        out.push_str(&format!("| **Created** | {created} |\n"));
    }
    if let Some(updated) = &entry.updated {
        out.push_str(&format!("| **Updated** | {updated} |\n"));
    }
    if let Some(loop_kind) = &entry.loop_kind {
        out.push_str(&format!("| **Loop shape** | `{loop_kind}` |\n"));
    }
    if !entry.review_gates.is_empty() {
        out.push_str(&format!(
            "| **Review gates** | {} |\n",
            entry
                .review_gates
                .iter()
                .map(|g| format!("`{g}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !entry.stop_conditions.is_empty() {
        out.push_str(&format!(
            "| **Stops when** | {} |\n",
            entry.stop_conditions.join("; ")
        ));
    }
    out.push_str(&format!(
        "| **Manifest** | [`collection.json`](/collections/{}) |\n\n",
        entry.manifest_path
    ));

    out.push_str("## Issue types\n\n| Key | Name | Mode | Steps |\n|---|---|---|---|\n");
    for it in &entry.issue_types {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            it.key, it.name, it.mode, it.step_count
        ));
    }

    out.push_str(
        "\n## Workflows\n\nSelect an issue type to see the Operator workflow it defines. \
         This is the same graph the Operator app draws, rendered from the same \
         published JSON.\n\n",
    );
    out.push_str(&format!(
        "<div class=\"workflow-explorer\">\n  \
         <operator-workflow-explorer base=\"/collections/{}/\"></operator-workflow-explorer>\n\
         </div>\n\n",
        entry.id
    ));

    out.push_str(&format!(
        "## Install\n\nOperator reads the hosted catalog on startup, so this collection \
         appears in the setup picker. To pin it explicitly:\n\n\
         ```toml\n# config.toml\n[templates]\nactive_collection = \"{}\"\n```\n",
        entry.id
    ));

    out
}

impl DocGenerator for CollectionsPagesGenerator {
    fn name(&self) -> &'static str {
        "collections-pages"
    }

    fn source(&self) -> &'static str {
        "src/collections/*/collection.json + collections/community/*/collection.json"
    }

    fn output_path(&self) -> &'static str {
        "workflows/index.md"
    }

    fn generate(&self) -> Result<String> {
        Ok(hub_page(&build_catalog()?.collections))
    }

    fn write(&self, docs_dir: &Path) -> Result<()> {
        let catalog = build_catalog()?;
        let workflows_dir = docs_dir.join("workflows");
        std::fs::create_dir_all(&workflows_dir)?;

        std::fs::write(
            workflows_dir.join("index.md"),
            hub_page(&catalog.collections),
        )?;

        for entry in &catalog.collections {
            let dir = workflows_dir.join(&entry.id);
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("index.md"), detail_page(entry))?;
        }

        tracing::info!(
            generator = self.name(),
            output = %workflows_dir.display(),
            collections = catalog.collections.len(),
            "Generated workflow catalog pages"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Vec<CatalogEntry> {
        build_catalog().unwrap().collections
    }

    #[test]
    fn test_hub_page_renders_every_collection_in_both_views() {
        let entries = catalog();
        let page = hub_page(&entries);
        for entry in &entries {
            // Once as a card, once as a table row — both filterable.
            assert_eq!(
                page.matches(&format!("data-search=\"{}\"", escape(&entry.search_text)))
                    .count(),
                2,
                "{} should appear in both the card grid and the table",
                entry.id
            );
            assert!(page.contains(&entry.docs_path), "{} needs a link", entry.id);
        }
    }

    #[test]
    fn test_hub_page_works_without_javascript() {
        // The search element only filters DOM that is already present; if cards
        // were rendered client-side this assertion would fail.
        let page = hub_page(&catalog());
        assert!(page.contains("class=\"collection-grid\""));
        assert!(page.contains("class=\"collection-card\""));
        assert!(
            !page.contains("fetch("),
            "the catalog must not be fetched at runtime"
        );
    }

    #[test]
    fn test_hub_page_states_the_vocabulary() {
        let page = hub_page(&catalog());
        for term in ["Operator workflow", "Issue type", "Collection"] {
            assert!(page.contains(term), "vocabulary must define '{term}'");
        }
        // The distinction from kanban types is the point of the page.
        assert!(page.contains("kanban issue types"));
    }

    #[test]
    fn test_pages_are_tagged_for_the_sidebar() {
        let page = hub_page(&catalog());
        assert!(
            page.contains("section: workflows"),
            "front matter must carry the section flag the sidebar keys on"
        );
        let detail = detail_page(&catalog()[0]);
        assert!(detail.contains("section: workflows"));
    }

    #[test]
    fn test_detail_page_mounts_the_explorer_at_the_published_bundle() {
        let ralph = catalog()
            .into_iter()
            .find(|c| c.id == "ralph_loop")
            .unwrap();
        let page = detail_page(&ralph);
        assert!(page.contains(r#"<operator-workflow-explorer base="/collections/ralph_loop/">"#));
        assert!(page.contains("| `PRD` |"));
        assert!(page.contains("snarktank"));
        assert!(page.contains("active_collection = \"ralph_loop\""));
    }

    #[test]
    fn test_icons_are_inlined_so_they_tint_with_the_theme() {
        let page = hub_page(&catalog());
        // An <img> would load the SVG as its own document, where currentColor
        // cannot resolve — the icon would stay black and vanish in dark mode.
        assert!(
            !page.contains("<img class=\"collection-icon\""),
            "collection icons must be inlined, not linked"
        );
        assert!(page.contains("<svg class=\"collection-icon\""));
        assert!(page.contains("aria-hidden=\"true\""));
        // Still the published markup, viewBox and all.
        assert!(page.contains(r#"viewBox="0 0 24 24""#));
    }

    #[test]
    fn test_html_is_escaped() {
        let mut entry = catalog().into_iter().next().unwrap();
        entry.description = r#"a <script>alert("x")</script> & more"#.to_string();
        let page = card(&entry);
        assert!(!page.contains("<script>"));
        assert!(page.contains("&lt;script&gt;"));
        assert!(page.contains("&amp; more"));
    }

    #[test]
    fn test_generation_is_deterministic() {
        let generator = CollectionsPagesGenerator;
        assert_eq!(generator.generate().unwrap(), generator.generate().unwrap());
    }
}
