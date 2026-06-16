//! Hosted collection manifest generator.
//!
//! Emits the static collection bundle served from the docs site:
//!
//! ```text
//! docs/collections/
//! ├── index.json                 (CollectionIndex of all collections)
//! └── <id>/
//!     ├── collection.json         (CollectionManifest with checksums)
//!     ├── <KEY>.json              (issuetype schema, byte-identical to embedded)
//!     └── <KEY>.md                (issuetype template)
//! ```
//!
//! The per-issuetype files are written byte-for-byte from the embedded
//! collections, and their SHA-256 checksums are computed and recorded in the
//! manifest. The runtime fetcher verifies these checksums, so the hosted bundle
//! is guaranteed identical to the offline fallback.

use std::path::Path;

use anyhow::{anyhow, Result};

use super::DocGenerator;
use crate::collections::fetch::{derive_manifest_checksum, sha256_hex};
use crate::collections::manifest::{
    CollectionIndex, CollectionIndexEntry, CollectionManifest, SCHEMA_VERSION,
};
use crate::collections::{EmbeddedCollection, EMBEDDED_COLLECTIONS};

/// Generates the hosted collection bundle under `docs/collections/`.
pub struct CollectionsManifestGenerator;

/// A fully-resolved hosted manifest plus the byte payloads it references.
struct HostedCollection {
    manifest: CollectionManifest,
    /// (relative path, bytes) for each issuetype schema/template file.
    files: Vec<(String, Vec<u8>)>,
}

/// Build a hosted manifest for `embedded`: copy metadata from the embedded
/// `collection.json`, fill in per-file checksums from the embedded bytes, and
/// derive the manifest-level checksum.
fn build_hosted(embedded: &EmbeddedCollection) -> Result<HostedCollection> {
    let mut manifest = embedded
        .manifest_parsed()
        .map_err(|e| anyhow!("parsing embedded manifest for {}: {e}", embedded.name))?;
    let mut files = Vec::new();

    for entry in &mut manifest.issue_types {
        let it = embedded
            .issuetypes
            .iter()
            .find(|it| it.key == entry.key)
            .ok_or_else(|| {
                anyhow!(
                    "collection {} manifest references {} but no embedded file exists",
                    embedded.name,
                    entry.key
                )
            })?;

        let schema_bytes = it.schema_json.as_bytes().to_vec();
        entry.schema_checksum = sha256_hex(&schema_bytes);
        files.push((entry.schema_path.clone(), schema_bytes));

        if let Some(template_path) = entry.template_path.clone() {
            let md_bytes = it.template_md.as_bytes().to_vec();
            entry.template_checksum = Some(sha256_hex(&md_bytes));
            files.push((template_path, md_bytes));
        }
    }

    manifest.checksum = Some(derive_manifest_checksum(&manifest.issue_types));
    Ok(HostedCollection { manifest, files })
}

/// Serialize a hosted manifest to its canonical on-disk form (pretty JSON,
/// trailing newline).
fn manifest_json(manifest: &CollectionManifest) -> Result<String> {
    Ok(format!("{}\n", manifest.to_json()?))
}

/// Build the top-level index over all embedded collections.
fn build_index() -> Result<CollectionIndex> {
    let mut collections = Vec::new();
    for embedded in EMBEDDED_COLLECTIONS {
        let hosted = build_hosted(embedded)?;
        let json = manifest_json(&hosted.manifest)?;
        collections.push(CollectionIndexEntry {
            id: hosted.manifest.id.clone(),
            name: hosted.manifest.name.clone(),
            description: hosted.manifest.description.clone(),
            version: hosted.manifest.version.clone(),
            tags: hosted.manifest.tags.clone(),
            manifest_path: format!("{}/collection.json", hosted.manifest.id),
            checksum: sha256_hex(json.as_bytes()),
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
        "src/collections/*/collection.json (EMBEDDED_COLLECTIONS)"
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
        for embedded in EMBEDDED_COLLECTIONS {
            let hosted = build_hosted(embedded)?;
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
    fn test_hosted_files_are_byte_identical_to_embedded() {
        let embedded = get_embedded_collection("dev_kanban").unwrap();
        let hosted = build_hosted(embedded).unwrap();
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
    fn test_manifest_checksum_matches_verifier_derivation() {
        // The producer's manifest.checksum must equal the value the runtime
        // verifier derives from the same entries.
        for embedded in EMBEDDED_COLLECTIONS {
            let hosted = build_hosted(embedded).unwrap();
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
}
