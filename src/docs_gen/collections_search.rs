//! Collection catalog manifest generator.
//!
//! Emits `docs/collections/search.json`: one machine-readable record per
//! published collection, flattening what the catalog page displays (identity,
//! provenance, dates, issue-type roster, loop shape) into a single document.
//!
//! Two consumers:
//!
//! * The `/workflows/` docs page renders its cards and table from these same
//!   values at generation time, and ships each row a `data-search` haystack
//!   derived from `search_text` so filtering needs no fetch.
//! * Anything else that wants to enumerate the catalog — an operator instance,
//!   a third-party tool — can read this instead of walking every manifest.
//!
//! Issue-type details are read from the published `<KEY>.json` files via
//! `TemplateSchema`, so the step counts here are the real ones.

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::DocGenerator;
use crate::collections::manifest::{CollectionManifest, CollectionTier};
use crate::collections::EMBEDDED_COLLECTIONS;
use crate::templates::schema::TemplateSchema;

/// Schema version of `search.json`, independent of the manifest version.
const SEARCH_SCHEMA_VERSION: u32 = 1;

/// The catalog document served at `/collections/search.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionCatalog {
    pub schema_version: u32,
    pub collections: Vec<CatalogEntry>,
}

/// One collection, as the catalog page and its search index see it.
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    /// `official` or `community`.
    pub tier: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    /// Icon path relative to `/collections/`.
    pub icon_path: Option<String>,
    /// Docs page for this collection.
    pub docs_path: String,
    /// Manifest path relative to `/collections/`.
    pub manifest_path: String,
    pub issue_type_count: usize,
    pub issue_types: Vec<CatalogIssueType>,
    /// Shape of the agentic loop this collection encodes.
    pub loop_kind: Option<String>,
    pub review_gates: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub memory_surfaces: Vec<String>,
    /// Lowercased haystack the catalog page filters on.
    pub search_text: String,
}

/// One issue type within a collection.
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogIssueType {
    pub key: String,
    pub name: String,
    pub mode: String,
    pub glyph: String,
    pub step_count: usize,
    /// Schema path relative to the collection directory.
    pub schema_path: String,
}

/// Generates the collection catalog manifest.
pub struct CollectionsSearchGenerator;

fn tier_slug(tier: CollectionTier) -> &'static str {
    match tier {
        CollectionTier::Official => "official",
        CollectionTier::Community => "community",
    }
}

/// Everything a reader might plausibly type, lowercased and deduplicated.
///
/// Deliberately generous: a catalog is only useful if searching for the loop
/// shape, the author, or an issue-type key all land on the same card.
fn search_text(entry: &CatalogEntry) -> String {
    let mut terms: BTreeSet<String> = BTreeSet::new();
    let mut push = |value: &str| {
        for word in value.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
            if !cleaned.is_empty() {
                terms.insert(cleaned.to_lowercase());
            }
        }
    };

    push(&entry.id);
    push(&entry.name);
    push(&entry.description);
    push(entry.tier.as_str());
    for value in [
        &entry.author,
        &entry.publisher,
        &entry.license,
        &entry.loop_kind,
    ]
    .into_iter()
    .flatten()
    {
        push(value);
    }
    for tag in &entry.tags {
        push(tag);
    }
    for gate in &entry.review_gates {
        push(gate);
    }
    for it in &entry.issue_types {
        push(&it.key);
        push(&it.name);
        push(&it.mode);
    }

    terms.into_iter().collect::<Vec<_>>().join(" ")
}

/// Build the catalog record for one collection, resolving each issue type's
/// details from its schema JSON.
fn entry_for<F>(manifest: &CollectionManifest, mut read_schema: F) -> Result<CatalogEntry>
where
    F: FnMut(&str) -> Result<String>,
{
    let mut issue_types = Vec::new();
    for it in &manifest.issue_types {
        let json = read_schema(&it.schema_path)?;
        let schema: TemplateSchema = serde_json::from_str(&json)
            .with_context(|| format!("parsing {} in collection {}", it.schema_path, manifest.id))?;
        issue_types.push(CatalogIssueType {
            key: schema.key.clone(),
            name: schema.name.clone(),
            mode: format!("{:?}", schema.mode).to_lowercase(),
            glyph: schema.glyph.clone(),
            step_count: schema.steps.len(),
            schema_path: it.schema_path.clone(),
        });
    }

    let hints = manifest.workflow_hints.as_ref();
    let mut entry = CatalogEntry {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        tier: tier_slug(manifest.tier).to_string(),
        author: manifest.author.clone(),
        publisher: manifest.publisher.clone(),
        url: manifest.url.clone(),
        license: manifest.license.clone(),
        tags: manifest.tags.clone(),
        created: manifest.created.clone(),
        updated: manifest.updated.clone(),
        icon_path: manifest
            .icon_path
            .as_ref()
            .map(|p| format!("{}/{p}", manifest.id)),
        docs_path: format!("/workflows/{}/", manifest.id),
        manifest_path: format!("{}/collection.json", manifest.id),
        issue_type_count: issue_types.len(),
        issue_types,
        loop_kind: hints.and_then(|h| h.loop_kind.clone()),
        review_gates: hints.map(|h| h.review_gates.clone()).unwrap_or_default(),
        stop_conditions: hints.map(|h| h.stop_conditions.clone()).unwrap_or_default(),
        memory_surfaces: hints.map(|h| h.memory_surfaces.clone()).unwrap_or_default(),
        search_text: String::new(),
    };
    entry.search_text = search_text(&entry);
    Ok(entry)
}

/// The raw SVG markup for a collection's icon.
///
/// Resolved from the compiled-in bytes for embedded collections and from disk
/// for community ones. Callers inline this rather than linking the published
/// file: an `<img>` loads the SVG as a separate document, where `currentColor`
/// cannot resolve, so a linked icon would render black in dark mode. Inlined,
/// it tints from the surrounding text color for free.
pub fn icon_svg_for(id: &str) -> Option<String> {
    if let Some(embedded) = crate::collections::get_embedded_collection(id) {
        let manifest = embedded.manifest_parsed().ok()?;
        manifest.icon_path.as_ref()?;
        return Some(embedded.icon_svg.to_string());
    }

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("collections/community")
        .join(id);
    let manifest =
        CollectionManifest::from_json(&std::fs::read_to_string(dir.join("collection.json")).ok()?)
            .ok()?;
    std::fs::read_to_string(dir.join(manifest.icon_path?)).ok()
}

/// Build the full catalog: embedded collections in `EMBEDDED_COLLECTIONS`
/// order, then community collections sorted by id — matching `index.json`.
pub fn build_catalog() -> Result<CollectionCatalog> {
    let mut collections = Vec::new();

    for embedded in EMBEDDED_COLLECTIONS {
        let manifest = embedded
            .manifest_parsed()
            .with_context(|| format!("parsing embedded manifest for {}", embedded.name))?;
        collections.push(entry_for(&manifest, |path| {
            let key = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
            embedded
                .issuetypes
                .iter()
                .find(|it| it.key == key)
                .map(|it| it.schema_json.to_string())
                .ok_or_else(|| anyhow::anyhow!("no embedded schema for {path}"))
        })?);
    }

    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("collections/community");
    if let Ok(entries) = std::fs::read_dir(&base) {
        let mut dirs: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("collection.json").is_file())
            .collect();
        dirs.sort();

        for dir in dirs {
            let manifest = CollectionManifest::from_json(&std::fs::read_to_string(
                dir.join("collection.json"),
            )?)
            .with_context(|| format!("parsing manifest in {}", dir.display()))?;
            collections.push(entry_for(&manifest, |path| {
                std::fs::read_to_string(dir.join(path))
                    .with_context(|| format!("reading {path} in {}", dir.display()))
            })?);
        }
    }

    Ok(CollectionCatalog {
        schema_version: SEARCH_SCHEMA_VERSION,
        collections,
    })
}

impl DocGenerator for CollectionsSearchGenerator {
    fn name(&self) -> &'static str {
        "collections-search"
    }

    fn source(&self) -> &'static str {
        "src/collections/*/collection.json + collections/community/*/collection.json"
    }

    fn output_path(&self) -> &'static str {
        "collections/search.json"
    }

    fn generate(&self) -> Result<String> {
        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&build_catalog()?)?
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_covers_every_published_collection() {
        let catalog = build_catalog().unwrap();
        let ids: Vec<&str> = catalog.collections.iter().map(|c| c.id.as_str()).collect();
        for name in crate::collections::embedded_collection_names() {
            assert!(ids.contains(&name), "catalog missing {name}");
        }
        assert!(
            ids.contains(&"example_chores"),
            "community collections must appear in the catalog"
        );
    }

    #[test]
    fn test_issue_type_counts_come_from_the_real_schemas() {
        let catalog = build_catalog().unwrap();
        let dev = catalog
            .collections
            .iter()
            .find(|c| c.id == "dev_kanban")
            .unwrap();
        assert_eq!(dev.issue_type_count, 3);
        assert_eq!(dev.issue_types.len(), 3);

        let feat = dev.issue_types.iter().find(|it| it.key == "FEAT").unwrap();
        assert_eq!(feat.name, "Feature");
        assert_eq!(feat.mode, "autonomous");
        // plan -> build -> code -> test -> deploy
        assert_eq!(feat.step_count, 5);
    }

    #[test]
    fn test_community_entries_carry_full_attribution() {
        let catalog = build_catalog().unwrap();
        let ralph = catalog
            .collections
            .iter()
            .find(|c| c.id == "ralph_loop")
            .unwrap();
        assert_eq!(ralph.tier, "community");
        assert_eq!(ralph.author.as_deref(), Some("snarktank"));
        assert!(ralph.url.is_some());
        assert_eq!(ralph.license.as_deref(), Some("MIT"));
        assert_eq!(ralph.icon_path.as_deref(), Some("ralph_loop/icon.svg"));
        assert_eq!(ralph.docs_path, "/workflows/ralph_loop/");
        assert!(ralph.created.is_some() && ralph.updated.is_some());
    }

    #[test]
    fn test_search_text_covers_the_terms_a_reader_would_type() {
        let catalog = build_catalog().unwrap();
        let ralph = catalog
            .collections
            .iter()
            .find(|c| c.id == "ralph_loop")
            .unwrap();

        for term in ["ralph", "snarktank", "community", "prd", "story"] {
            assert!(
                ralph.search_text.contains(term),
                "search_text for ralph_loop should match '{term}': {}",
                ralph.search_text
            );
        }
        // Normalized: lowercase, no duplicates.
        assert_eq!(ralph.search_text, ralph.search_text.to_lowercase());
        let terms: Vec<&str> = ralph.search_text.split(' ').collect();
        let unique: BTreeSet<&str> = terms.iter().copied().collect();
        assert_eq!(
            terms.len(),
            unique.len(),
            "search_text must be deduplicated"
        );
    }

    #[test]
    fn test_generation_is_deterministic() {
        let generator = CollectionsSearchGenerator;
        assert_eq!(generator.generate().unwrap(), generator.generate().unwrap());
    }

    #[test]
    fn test_catalog_order_matches_the_index() {
        let catalog = build_catalog().unwrap();
        let embedded_count = EMBEDDED_COLLECTIONS.len();
        let embedded_ids: Vec<&str> = catalog.collections[..embedded_count]
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(
            embedded_ids,
            crate::collections::embedded_collection_names()
        );
    }
}
