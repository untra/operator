//! Fetching and integrity verification for hosted collections.
//!
//! The pure helpers in this module ([`sha256_hex`], [`verify_files`],
//! [`derive_manifest_checksum`]) are HTTP-free so they can be unit tested and
//! reused by both the docs producer (which computes checksums) and the runtime
//! fetcher (which verifies them). The async fetch functions live alongside them.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use crate::collections::manifest::{
    CollectionIndex, CollectionIndexEntry, CollectionManifest, IssueTypeEntry, SCHEMA_VERSION,
};
use crate::collections::{get_embedded_collection, EmbeddedCollection, EMBEDDED_COLLECTIONS};

/// Compute the lowercase-hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Derive a manifest-level checksum from the per-issuetype file checksums.
///
/// SHA-256 over the issue-type checksums concatenated in `issue_types` order:
/// each entry contributes its `schema_checksum`, then its `template_checksum`
/// if present, every value on its own line. The docs producer and the runtime
/// verifier MUST compute this identically.
pub fn derive_manifest_checksum(entries: &[IssueTypeEntry]) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(entries.len() * 2);
    for e in entries {
        parts.push(&e.schema_checksum);
        if let Some(tc) = &e.template_checksum {
            parts.push(tc);
        }
    }
    sha256_hex(parts.join("\n").as_bytes())
}

/// Verify that every issue-type file's bytes match the checksum declared in its
/// manifest entry. `files` is keyed by the entry's relative path
/// (`schema_path` / `template_path`).
///
/// Returns `Err` on the first mismatch or missing file. Callers treat any error
/// as a verification failure and fall back to the embedded copy; unverified
/// bytes are never persisted.
pub fn verify_files(entries: &[IssueTypeEntry], files: &HashMap<String, Vec<u8>>) -> Result<()> {
    for e in entries {
        let schema = files
            .get(&e.schema_path)
            .ok_or_else(|| anyhow!("missing file for {} ({})", e.key, e.schema_path))?;
        let actual = sha256_hex(schema);
        if actual != e.schema_checksum {
            return Err(anyhow!(
                "checksum mismatch for {} ({}): expected {}, got {}",
                e.key,
                e.schema_path,
                e.schema_checksum,
                actual
            ));
        }
        if let (Some(path), Some(expected)) = (&e.template_path, &e.template_checksum) {
            let template = files
                .get(path)
                .ok_or_else(|| anyhow!("missing template for {} ({})", e.key, path))?;
            let actual = sha256_hex(template);
            if &actual != expected {
                return Err(anyhow!(
                    "template checksum mismatch for {} ({}): expected {expected}, got {actual}",
                    e.key,
                    path
                ));
            }
        }
    }
    Ok(())
}

/// Resolve a path relative to `base_url` by replacing the last URL segment.
///
/// `resolve_url("https://x/collections/index.json", "dev_kanban/collection.json")`
/// -> `https://x/collections/dev_kanban/collection.json`.
pub fn resolve_url(base_url: &str, relative: &str) -> String {
    match base_url.rfind('/') {
        Some(idx) => format!("{}/{}", &base_url[..idx], relative),
        None => relative.to_string(),
    }
}

/// A fully fetched and verified collection: its manifest plus the verified
/// bytes of each issue-type schema (and optional template).
pub struct FetchedCollection {
    pub manifest: CollectionManifest,
    /// (key, `schema_json`, `template_md`) for each issue type, in manifest order.
    pub files: Vec<(String, String, Option<String>)>,
}

fn http_client(timeout_secs: u64) -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?)
}

async fn get_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!("non-success status {} for {url}", status.as_u16()));
    }
    Ok(response.bytes().await?.to_vec())
}

/// Fetch the collection index. Returns `None` on any network/parse error or an
/// unknown schema version, so callers fall back to the embedded collections.
pub async fn fetch_index(url: &str, timeout_secs: u64) -> Option<CollectionIndex> {
    let client = http_client(timeout_secs)
        .map_err(|e| tracing::debug!(error = %e, "collection index client build failed"))
        .ok()?;
    let bytes = get_bytes(&client, url)
        .await
        .map_err(|e| tracing::debug!(error = %e, "collection index fetch failed"))
        .ok()?;
    let index: CollectionIndex = serde_json::from_slice(&bytes)
        .map_err(|e| tracing::debug!(error = %e, "collection index parse failed"))
        .ok()?;
    if index.schema_version != SCHEMA_VERSION {
        tracing::debug!(
            version = index.schema_version,
            "unsupported collection index schema version"
        );
        return None;
    }
    Some(index)
}

/// Fetch and verify a single collection referenced by `entry`.
///
/// Verifies the manifest bytes against `entry.checksum`, rejects unknown schema
/// versions, and verifies every issue-type file against its declared checksum.
/// Any failure returns `Err`; callers fall back to the embedded copy and never
/// persist unverified bytes.
pub async fn fetch_collection(
    index_url: &str,
    entry: &CollectionIndexEntry,
    timeout_secs: u64,
) -> Result<FetchedCollection> {
    let client = http_client(timeout_secs)?;

    // 1. Manifest, verified against the index entry checksum.
    let manifest_url = resolve_url(index_url, &entry.manifest_path);
    let manifest_bytes = get_bytes(&client, &manifest_url).await?;
    let actual = sha256_hex(&manifest_bytes);
    if actual != entry.checksum {
        return Err(anyhow!(
            "manifest checksum mismatch for {}: expected {}, got {actual}",
            entry.id,
            entry.checksum
        ));
    }
    let manifest = CollectionManifest::from_json(&String::from_utf8(manifest_bytes)?)?;
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported manifest schema version {} for {}",
            manifest.schema_version,
            entry.id
        ));
    }

    // 2. Fetch every issue-type file; collect bytes keyed by relative path.
    let mut raw: HashMap<String, Vec<u8>> = HashMap::new();
    for it in &manifest.issue_types {
        let schema_url = resolve_url(&manifest_url, &it.schema_path);
        raw.insert(
            it.schema_path.clone(),
            get_bytes(&client, &schema_url).await?,
        );
        if let Some(template_path) = &it.template_path {
            let template_url = resolve_url(&manifest_url, template_path);
            raw.insert(
                template_path.clone(),
                get_bytes(&client, &template_url).await?,
            );
        }
    }

    // 3. Verify all checksums before trusting any bytes.
    verify_files(&manifest.issue_types, &raw)?;

    // 4. Project into UTF-8 file payloads in manifest order.
    let mut files = Vec::with_capacity(manifest.issue_types.len());
    for it in &manifest.issue_types {
        let schema_json = String::from_utf8(
            raw.get(&it.schema_path)
                .cloned()
                .ok_or_else(|| anyhow!("missing fetched bytes for {}", it.schema_path))?,
        )?;
        let template_md = match &it.template_path {
            Some(p) => Some(String::from_utf8(
                raw.get(p)
                    .cloned()
                    .ok_or_else(|| anyhow!("missing fetched bytes for {p}"))?,
            )?),
            None => None,
        };
        files.push((it.key.clone(), schema_json, template_md));
    }

    Ok(FetchedCollection { manifest, files })
}

/// Where a resolved collection's definition came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionOrigin {
    /// Fetched from the hosted manifest and checksum-verified.
    Hosted,
    /// Loaded from the embedded (offline) copy baked into the binary.
    Embedded,
}

/// A collection ready to present in the setup picker and scaffold from.
pub struct ResolvedCollection {
    pub manifest: CollectionManifest,
    /// (key, `schema_json`, `template_md`) in manifest order.
    pub files: Vec<(String, String, Option<String>)>,
    pub origin: CollectionOrigin,
    /// Why we fell back to embedded, if applicable (e.g. checksum failure).
    pub note: Option<String>,
}

/// Build a [`FetchedCollection`] from an embedded collection, computing
/// checksums from the compiled-in bytes. Infallible in practice (embedded
/// manifests are validated by tests) but returns `Result` for symmetry.
pub fn embedded_fetched(embedded: &EmbeddedCollection) -> Result<FetchedCollection> {
    let mut manifest = embedded
        .manifest_parsed()
        .map_err(|e| anyhow!("parsing embedded manifest for {}: {e}", embedded.name))?;
    let mut files = Vec::with_capacity(manifest.issue_types.len());
    for entry in &mut manifest.issue_types {
        let it = embedded
            .issuetypes
            .iter()
            .find(|it| it.key == entry.key)
            .ok_or_else(|| anyhow!("embedded file missing for {}", entry.key))?;
        entry.schema_checksum = sha256_hex(it.schema_json.as_bytes());
        let template = entry.template_path.as_ref().map(|_| {
            entry.template_checksum = Some(sha256_hex(it.template_md.as_bytes()));
            it.template_md.to_string()
        });
        files.push((entry.key.clone(), it.schema_json.to_string(), template));
    }
    manifest.checksum = Some(derive_manifest_checksum(&manifest.issue_types));
    Ok(FetchedCollection { manifest, files })
}

/// Resolve the collections to offer in the setup picker.
///
/// Attempts to fetch the hosted index (when a URL is provided) and verifies each
/// collection; on any per-collection failure it falls back to the embedded copy.
/// Every embedded collection not covered by the index is appended, so the picker
/// is never empty even fully offline.
pub async fn resolve_for_setup(
    manifest_url: Option<&str>,
    timeout_secs: u64,
) -> Vec<ResolvedCollection> {
    let mut out: Vec<ResolvedCollection> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(url) = manifest_url {
        if let Some(index) = fetch_index(url, timeout_secs).await {
            for entry in &index.collections {
                match fetch_collection(url, entry, timeout_secs).await {
                    Ok(fc) => {
                        seen.insert(fc.manifest.id.clone());
                        out.push(ResolvedCollection {
                            manifest: fc.manifest,
                            files: fc.files,
                            origin: CollectionOrigin::Hosted,
                            note: None,
                        });
                    }
                    Err(e) => {
                        // Verification/network failure: fall back to embedded if we have it.
                        if let Some(embedded) = get_embedded_collection(&entry.id) {
                            if let Ok(fc) = embedded_fetched(embedded) {
                                seen.insert(entry.id.clone());
                                out.push(ResolvedCollection {
                                    manifest: fc.manifest,
                                    files: fc.files,
                                    origin: CollectionOrigin::Embedded,
                                    note: Some(e.to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Append embedded collections not provided by the hosted index.
    for embedded in EMBEDDED_COLLECTIONS {
        if seen.contains(embedded.name) {
            continue;
        }
        if let Ok(fc) = embedded_fetched(embedded) {
            out.push(ResolvedCollection {
                manifest: fc.manifest,
                files: fc.files,
                origin: CollectionOrigin::Embedded,
                note: None,
            });
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, schema_sum: &str) -> IssueTypeEntry {
        IssueTypeEntry {
            key: key.to_string(),
            schema_path: format!("{key}.json"),
            schema_checksum: schema_sum.to_string(),
            template_path: None,
            template_checksum: None,
        }
    }

    #[test]
    fn test_resolve_url_replaces_last_segment() {
        assert_eq!(
            resolve_url(
                "https://operator.untra.io/collections/index.json",
                "dev_kanban/collection.json"
            ),
            "https://operator.untra.io/collections/dev_kanban/collection.json"
        );
        assert_eq!(
            resolve_url(
                "https://operator.untra.io/collections/dev_kanban/collection.json",
                "TASK.json"
            ),
            "https://operator.untra.io/collections/dev_kanban/TASK.json"
        );
    }

    #[test]
    fn test_sha256_hex_empty_input_known_vector() {
        // SHA-256 of the empty string.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hex_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_verify_files_passes_when_checksums_match() {
        let task_bytes = b"task schema".to_vec();
        let entries = vec![entry("TASK", &sha256_hex(&task_bytes))];
        let mut files = HashMap::new();
        files.insert("TASK.json".to_string(), task_bytes);
        assert!(verify_files(&entries, &files).is_ok());
    }

    #[test]
    fn test_verify_files_fails_on_tampered_bytes() {
        let task_bytes = b"task schema".to_vec();
        let entries = vec![entry("TASK", &sha256_hex(&task_bytes))];
        let mut files = HashMap::new();
        // One byte flipped -> checksum no longer matches.
        files.insert("TASK.json".to_string(), b"task schemb".to_vec());
        let err = verify_files(&entries, &files).unwrap_err();
        assert!(err.to_string().contains("checksum mismatch"));
    }

    #[test]
    fn test_verify_files_fails_on_missing_file() {
        let entries = vec![entry("TASK", "deadbeef")];
        let files = HashMap::new();
        let err = verify_files(&entries, &files).unwrap_err();
        assert!(err.to_string().contains("missing file"));
    }

    #[test]
    fn test_verify_files_checks_template_checksum() {
        let schema_bytes = b"schema".to_vec();
        let template_bytes = b"template".to_vec();
        let mut e = entry("TASK", &sha256_hex(&schema_bytes));
        e.template_path = Some("TASK.md".to_string());
        e.template_checksum = Some("wrong".to_string());
        let mut files = HashMap::new();
        files.insert("TASK.json".to_string(), schema_bytes);
        files.insert("TASK.md".to_string(), template_bytes);
        let err = verify_files(&[e], &files).unwrap_err();
        assert!(err.to_string().contains("template checksum mismatch"));
    }

    #[test]
    fn test_embedded_fetched_computes_checksums_and_verifies() {
        let embedded = get_embedded_collection("dev_kanban").unwrap();
        let fc = embedded_fetched(embedded).unwrap();
        assert_eq!(fc.manifest.id, "dev_kanban");
        assert_eq!(fc.files.len(), 3);
        // The computed checksums must verify against the produced bytes.
        let mut map = HashMap::new();
        for (entry, (_, schema, template)) in fc.manifest.issue_types.iter().zip(&fc.files) {
            map.insert(entry.schema_path.clone(), schema.clone().into_bytes());
            if let (Some(p), Some(t)) = (&entry.template_path, template) {
                map.insert(p.clone(), t.clone().into_bytes());
            }
        }
        assert!(verify_files(&fc.manifest.issue_types, &map).is_ok());
    }

    #[tokio::test]
    async fn test_resolve_for_setup_offline_returns_all_embedded() {
        // No URL -> pure embedded fallback; picker is never empty.
        let resolved = resolve_for_setup(None, 1).await;
        let ids: Vec<&str> = resolved.iter().map(|r| r.manifest.id.as_str()).collect();
        assert!(ids.contains(&"dev_kanban"));
        assert!(ids.contains(&"devops_kanban"));
        assert!(resolved
            .iter()
            .all(|r| r.origin == CollectionOrigin::Embedded));
    }

    #[test]
    fn test_derive_manifest_checksum_is_order_sensitive_and_stable() {
        let a = vec![entry("TASK", "111"), entry("FEAT", "222")];
        let b = vec![entry("FEAT", "222"), entry("TASK", "111")];
        let sum_a = derive_manifest_checksum(&a);
        // Stable across calls.
        assert_eq!(sum_a, derive_manifest_checksum(&a));
        // Order matters.
        assert_ne!(sum_a, derive_manifest_checksum(&b));
    }
}
