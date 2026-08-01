//! Hosted collection manifest generator.
//!
//! Emits the static collection bundle served from the docs site:
//!
//! ```text
//! docs/collections/
//! ├── index.json                 (CollectionIndex of all collections)
//! └── <id>/
//!     ├── collection.json         (CollectionManifest with checksums)
//!     ├── icon.svg                (Simple Icons-shaped collection glyph)
//!     ├── <KEY>.json              (issuetype schema, byte-identical to embedded)
//!     └── <KEY>.md                (issuetype template)
//!
//! Two sources feed the bundle:
//!
//! * **Embedded** collections (`src/collections/`) are compiled into the binary
//!   and republished here byte-for-byte, so the hosted copy is guaranteed
//!   identical to the offline fallback.
//! * **Community** collections (`collections/community/`) are hosted-only. They
//!   are validated during generation, so a broken submission fails a PR's CI
//!   rather than a user's install.
//!
//! Per-issuetype SHA-256 checksums are computed and recorded in each manifest;
//! the runtime fetcher verifies them before trusting any fetched bytes. Icons
//! are deliberately excluded: they are presentational and never executed, and a
//! malformed one must not be able to fail an install.
//!
//! No workflow previews are emitted. The graph renders from `<KEY>.json` — the
//! native Operator workflow that is already published and already checksummed —
//! so there is nothing per-workflow to pre-generate.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::DocGenerator;
use crate::collections::fetch::{derive_manifest_checksum, sha256_hex};
use crate::collections::manifest::{
    CollectionIndex, CollectionIndexEntry, CollectionManifest, SCHEMA_VERSION,
};
use crate::collections::validate::validate_collection_dir;
use crate::collections::{EmbeddedCollection, EMBEDDED_COLLECTIONS};

/// Generates the hosted collection bundle under `docs/collections/`.
pub struct CollectionsManifestGenerator;

/// A fully-resolved hosted manifest plus the byte payloads it references.
struct HostedCollection {
    manifest: CollectionManifest,
    /// (relative path, bytes) for each issuetype schema/template file, plus the
    /// collection icon when the manifest declares one.
    files: Vec<(String, Vec<u8>)>,
}

/// Where community submissions live, relative to the repo root.
///
/// Resolved from the crate directory rather than the process CWD: this is a
/// repo-maintenance generator, and a missing directory is a no-op rather than
/// an error so the bundle still builds outside a checkout.
fn community_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("collections/community")
}

/// The docs-site path a collection's page is served from.
fn docs_path_for(id: &str) -> String {
    format!("/workflows/{id}/")
}

/// Fill in per-file checksums and the derived manifest checksum, given a way to
/// resolve each referenced file's bytes.
///
/// Shared by the embedded and community paths so both produce byte-identical
/// manifest structure and identical checksum derivation.
fn finalize<F>(manifest: &mut CollectionManifest, mut read: F) -> Result<Vec<(String, Vec<u8>)>>
where
    F: FnMut(&str) -> Result<Vec<u8>>,
{
    let mut files = Vec::new();

    for entry in &mut manifest.issue_types {
        let schema_bytes = read(&entry.schema_path)?;
        entry.schema_checksum = sha256_hex(&schema_bytes);
        files.push((entry.schema_path.clone(), schema_bytes));

        if let Some(template_path) = entry.template_path.clone() {
            let md_bytes = read(&template_path)?;
            entry.template_checksum = Some(sha256_hex(&md_bytes));
            files.push((template_path, md_bytes));
        }
    }

    // Icon last, and outside the checksum derivation below.
    if let Some(icon_path) = manifest.icon_path.clone() {
        let icon_bytes = read(&icon_path)?;
        files.push((icon_path, icon_bytes));
    }

    manifest.checksum = Some(derive_manifest_checksum(&manifest.issue_types));
    Ok(files)
}

/// Build a hosted manifest for an embedded collection, resolving files from the
/// bytes compiled into the binary.
fn build_embedded(embedded: &EmbeddedCollection) -> Result<HostedCollection> {
    let mut manifest = embedded
        .manifest_parsed()
        .map_err(|e| anyhow!("parsing embedded manifest for {}: {e}", embedded.name))?;
    let icon_path = manifest.icon_path.clone();

    let files = finalize(&mut manifest, |path| {
        if Some(path) == icon_path.as_deref() {
            return Ok(embedded.icon_svg.as_bytes().to_vec());
        }
        let key = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
        let it = embedded
            .issuetypes
            .iter()
            .find(|it| it.key == key)
            .ok_or_else(|| {
                anyhow!(
                    "collection {} manifest references {path} but no embedded file exists",
                    embedded.name
                )
            })?;
        // Manifest paths are exact filenames, so an exact extension match is
        // what we want: `TASK.MD` is a different reference, not the template.
        let is_template = std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext == "md");
        Ok(if is_template {
            it.template_md.as_bytes().to_vec()
        } else {
            it.schema_json.as_bytes().to_vec()
        })
    })?;

    Ok(HostedCollection { manifest, files })
}

/// Build a hosted manifest for a community collection, resolving files from
/// disk. The directory is validated first, so an invalid submission fails
/// generation (and therefore CI) rather than shipping.
fn build_community(dir: &Path) -> Result<HostedCollection> {
    let mut manifest = validate_collection_dir(dir)
        .with_context(|| format!("invalid community collection at {}", dir.display()))?;

    let files = finalize(&mut manifest, |path| {
        std::fs::read(dir.join(path))
            .with_context(|| format!("reading {path} in {}", dir.display()))
    })?;

    Ok(HostedCollection { manifest, files })
}

/// Every community collection directory, sorted by id so generation is
/// deterministic. A missing `collections/community/` yields an empty list.
fn community_collections() -> Result<Vec<HostedCollection>> {
    let base = community_dir();
    let Ok(entries) = std::fs::read_dir(&base) else {
        return Ok(Vec::new());
    };

    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("collection.json").is_file())
        .collect();
    dirs.sort();

    dirs.iter().map(|dir| build_community(dir)).collect()
}

/// Serialize a hosted manifest to its canonical on-disk form (pretty JSON,
/// trailing newline).
fn manifest_json(manifest: &CollectionManifest) -> Result<String> {
    Ok(format!("{}\n", manifest.to_json()?))
}

/// Every collection to publish: embedded first (in `EMBEDDED_COLLECTIONS`
/// order), then community (sorted by id).
fn all_hosted() -> Result<Vec<HostedCollection>> {
    let mut hosted: Vec<HostedCollection> = EMBEDDED_COLLECTIONS
        .iter()
        .map(build_embedded)
        .collect::<Result<_>>()?;
    hosted.extend(community_collections()?);
    Ok(hosted)
}

/// Build the top-level index over every published collection.
fn build_index() -> Result<CollectionIndex> {
    let mut collections = Vec::new();
    for hosted in all_hosted()? {
        let json = manifest_json(&hosted.manifest)?;
        collections.push(CollectionIndexEntry {
            id: hosted.manifest.id.clone(),
            name: hosted.manifest.name.clone(),
            description: hosted.manifest.description.clone(),
            version: hosted.manifest.version.clone(),
            tags: hosted.manifest.tags.clone(),
            manifest_path: format!("{}/collection.json", hosted.manifest.id),
            checksum: sha256_hex(json.as_bytes()),
            tier: hosted.manifest.tier,
            docs_path: Some(docs_path_for(&hosted.manifest.id)),
        });
    }
    Ok(CollectionIndex {
        schema_version: SCHEMA_VERSION,
        // Intentionally omitted: a timestamp would make generation non-deterministic.
        generated_at: None,
        collections,
    })
}

impl DocGenerator for CollectionsManifestGenerator {
    fn name(&self) -> &'static str {
        "collections-manifest"
    }

    fn source(&self) -> &'static str {
        "src/collections/*/collection.json + collections/community/*/collection.json"
    }

    fn output_path(&self) -> &'static str {
        "collections/index.json"
    }

    fn generate(&self) -> Result<String> {
        let index = build_index()?;
        Ok(format!("{}\n", serde_json::to_string_pretty(&index)?))
    }

    fn write(&self, docs_dir: &Path) -> Result<()> {
        let collections_dir = docs_dir.join("collections");

        // Per-collection bundles.
        for hosted in all_hosted()? {
            let dir = collections_dir.join(&hosted.manifest.id);
            std::fs::create_dir_all(&dir)?;
            std::fs::write(
                dir.join("collection.json"),
                manifest_json(&hosted.manifest)?,
            )?;
            for (rel_path, bytes) in &hosted.files {
                std::fs::write(dir.join(rel_path), bytes)?;
            }
        }

        // Top-level index + the hosted manifest JSON Schema (served for validation).
        std::fs::create_dir_all(&collections_dir)?;
        std::fs::write(collections_dir.join("index.json"), self.generate()?)?;
        std::fs::write(
            collections_dir.join("schema.json"),
            include_str!("../schemas/issuetype_collection_schema.json"),
        )?;

        tracing::info!(
            generator = self.name(),
            output = %collections_dir.display(),
            "Generated hosted collection bundle"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::get_embedded_collection;
    use crate::collections::manifest::CollectionTier;

    #[test]
    fn test_index_lists_all_embedded_collections() {
        let index = build_index().unwrap();
        let ids: Vec<&str> = index.collections.iter().map(|c| c.id.as_str()).collect();
        for name in crate::collections::embedded_collection_names() {
            assert!(ids.contains(&name), "index missing {name}");
        }
        assert_eq!(index.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn test_index_publishes_community_collections() {
        let index = build_index().unwrap();
        let community: Vec<&str> = index
            .collections
            .iter()
            .filter(|c| c.tier == CollectionTier::Community && c.id == "example_chores")
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(
            community,
            vec!["example_chores"],
            "collections/community/ must reach the published index"
        );
    }

    #[test]
    fn test_index_orders_embedded_before_community_and_sorts_community() {
        let index = build_index().unwrap();
        let embedded_count = EMBEDDED_COLLECTIONS.len();
        let (embedded, community) = index.collections.split_at(embedded_count);

        let embedded_ids: Vec<&str> = embedded.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            embedded_ids,
            crate::collections::embedded_collection_names(),
            "embedded collections must keep EMBEDDED_COLLECTIONS order"
        );

        let community_ids: Vec<&str> = community.iter().map(|c| c.id.as_str()).collect();
        let mut sorted = community_ids.clone();
        sorted.sort_unstable();
        assert_eq!(
            community_ids, sorted,
            "community entries must be sorted by id"
        );
    }

    #[test]
    fn test_every_index_entry_links_to_its_docs_page() {
        for entry in build_index().unwrap().collections {
            assert_eq!(
                entry.docs_path.as_deref(),
                Some(format!("/workflows/{}/", entry.id).as_str()),
                "{} must deep-link to its docs page",
                entry.id
            );
        }
    }

    #[test]
    fn test_hosted_files_are_byte_identical_to_embedded() {
        let embedded = get_embedded_collection("dev_kanban").unwrap();
        let hosted = build_embedded(embedded).unwrap();
        for entry in &hosted.manifest.issue_types {
            let it = embedded
                .issuetypes
                .iter()
                .find(|it| it.key == entry.key)
                .unwrap();
            // schema_checksum matches a SHA-256 of the embedded bytes...
            assert_eq!(entry.schema_checksum, sha256_hex(it.schema_json.as_bytes()));
            // ...and the written file bytes equal the embedded bytes.
            let (_, bytes) = hosted
                .files
                .iter()
                .find(|(p, _)| p == &entry.schema_path)
                .unwrap();
            assert_eq!(bytes.as_slice(), it.schema_json.as_bytes());
        }
    }

    #[test]
    fn test_every_collection_publishes_its_icon() {
        for hosted in all_hosted().unwrap() {
            let icon_path = hosted
                .manifest
                .icon_path
                .clone()
                .unwrap_or_else(|| panic!("{} declares no icon_path", hosted.manifest.id));
            let (_, bytes) = hosted
                .files
                .iter()
                .find(|(p, _)| p == &icon_path)
                .unwrap_or_else(|| panic!("{} did not publish {icon_path}", hosted.manifest.id));
            let svg = std::str::from_utf8(bytes).unwrap();
            assert!(
                svg.contains(r#"viewBox="0 0 24 24""#),
                "{} published a non-Simple-Icons icon",
                hosted.manifest.id
            );
        }
    }

    #[test]
    fn test_icons_are_excluded_from_the_manifest_checksum() {
        // Icons are presentational and unverified; the derived checksum must
        // cover only the issue-type files the fetcher validates.
        let embedded = get_embedded_collection("ralph_loop").unwrap();
        let hosted = build_embedded(embedded).unwrap();
        assert_eq!(
            hosted.manifest.checksum.as_deref(),
            Some(derive_manifest_checksum(&hosted.manifest.issue_types).as_str())
        );
    }

    #[test]
    fn test_manifest_checksum_matches_verifier_derivation() {
        // The producer's manifest.checksum must equal the value the runtime
        // verifier derives from the same entries.
        for hosted in all_hosted().unwrap() {
            let derived = derive_manifest_checksum(&hosted.manifest.issue_types);
            assert_eq!(hosted.manifest.checksum.as_deref(), Some(derived.as_str()));
        }
    }

    #[test]
    fn test_generate_produces_parseable_index() {
        let generator = CollectionsManifestGenerator;
        let json = generator.generate().unwrap();
        let index: CollectionIndex = serde_json::from_str(&json).unwrap();
        assert!(!index.collections.is_empty());
        // Every index checksum must be a 64-char hex string.
        for entry in &index.collections {
            assert_eq!(entry.checksum.len(), 64);
        }
    }

    #[test]
    fn test_generation_is_deterministic() {
        let generator = CollectionsManifestGenerator;
        assert_eq!(generator.generate().unwrap(), generator.generate().unwrap());
    }
}
