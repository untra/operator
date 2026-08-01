//! Hosted/embedded collection manifest format
//!
//! A collection is described by a [`CollectionManifest`] (`collection.json`)
//! that references per-issuetype JSON schema files (and optional markdown
//! templates) by relative path, each with a SHA-256 checksum. A
//! [`CollectionIndex`] (the file the configurable manifest URL points at)
//! lists the available collections.
//!
//! This is the single collection format used both for hosted collections
//! served from the docs site and for the offline-fallback collections
//! embedded in the binary.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current manifest schema version. Manifests with an unknown version are
/// rejected by the fetcher and fall back to the embedded copy.
pub const SCHEMA_VERSION: u32 = 1;

/// Provenance tier of a collection: who authored and maintains it.
///
/// Orthogonal to distribution — curated community-authored collections may
/// ship embedded in the binary, while community submissions under
/// `collections/community/` are hosted-only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionTier {
    /// Authored and maintained by the operator project.
    #[default]
    Official,
    /// Community-authored (attribution via `author`/`url`/`license`).
    Community,
}

/// Top-level index listing the available collections. This is what the
/// configurable `collections_manifest_url` points at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndex {
    /// Schema version of this index document.
    pub schema_version: u32,
    /// RFC3339 generation timestamp (informational).
    #[serde(default)]
    pub generated_at: Option<String>,
    /// Available collections.
    #[serde(default)]
    pub collections: Vec<CollectionIndexEntry>,
}

/// A single entry in the [`CollectionIndex`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndexEntry {
    /// Stable collection id (e.g. `dev_kanban`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// One-line description.
    #[serde(default)]
    pub description: String,
    /// Collection semver.
    #[serde(default)]
    pub version: String,
    /// Free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Path to this collection's `collection.json`, relative to the index URL.
    pub manifest_path: String,
    /// SHA-256 (lowercase hex) of the referenced `collection.json` bytes.
    pub checksum: String,
    /// Provenance tier (defaults to official for older indexes).
    #[serde(default)]
    pub tier: CollectionTier,
    /// Path to the collection's docs page, relative to the site root
    /// `/collections/` (e.g. `dev_kanban/`). Informational, for deep links.
    #[serde(default)]
    pub docs_path: Option<String>,
}

/// A single collection manifest (`collection.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionManifest {
    /// Schema version of this manifest.
    pub schema_version: u32,
    /// Stable collection id (e.g. `dev_kanban`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// One-line description.
    #[serde(default)]
    pub description: String,
    /// Collection semver.
    #[serde(default)]
    pub version: String,
    /// Publisher identifier (e.g. `untra`).
    #[serde(default)]
    pub publisher: Option<String>,
    /// Human author/attribution shown in the setup picker.
    ///
    /// Built-in collections are authored by `Operator!`; a collection imported
    /// from a kanban provider lists the provider name + workspace/project.
    #[serde(default)]
    pub author: Option<String>,
    /// Link to the collection's source (GitHub repo or project page).
    #[serde(default)]
    pub url: Option<String>,
    /// SPDX license id.
    #[serde(default)]
    pub license: Option<String>,
    /// Free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Compatibility constraints.
    #[serde(default)]
    pub compatibility: Option<Compatibility>,
    /// Provenance tier (defaults to official for older manifests).
    #[serde(default)]
    pub tier: CollectionTier,
    /// Bare filename of the collection's Simple Icons-shaped SVG, next to the
    /// manifest. Deliberately not checksummed; icon must pass standards.
    #[serde(default)]
    pub icon_path: Option<String>,
    /// ISO-8601 date (`YYYY-MM-DD`) the collection was first published.
    #[serde(default)]
    pub created: Option<String>,
    /// ISO-8601 date (`YYYY-MM-DD`) of the last substantive revision.
    #[serde(default)]
    pub updated: Option<String>,
    /// Descriptive kanban onboarding defaults (v1: metadata only).
    #[serde(default)]
    pub kanban_defaults: Option<KanbanDefaults>,
    /// Issue types in this collection (display order).
    pub issue_types: Vec<IssueTypeEntry>,
    /// Descriptive workflow hints (v1: metadata only, no execution behavior).
    #[serde(default)]
    pub workflow_hints: Option<WorkflowHints>,
    /// Subset of `issue_types[].key` selected by default in the setup picker.
    #[serde(default)]
    pub default_selected: Vec<String>,
    /// SHA-256 (lowercase hex) derived from the issue-type file checksums.
    #[serde(default)]
    pub checksum: Option<String>,
}

/// Compatibility constraints for a collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Compatibility {
    /// Minimum operator version this collection targets (e.g. `>=0.2.0`).
    #[serde(default)]
    pub operator_version: Option<String>,
}

/// A reference to a single issue-type file within a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeEntry {
    /// Issue type key (e.g. `TASK`).
    pub key: String,
    /// Path to the issuetype JSON, relative to the manifest.
    pub schema_path: String,
    /// SHA-256 (lowercase hex) of the issuetype JSON bytes.
    ///
    /// Embedded manifests omit this (the bytes are compiled in and trusted);
    /// the docs producer fills it for hosted manifests, which the fetcher verifies.
    #[serde(default)]
    pub schema_checksum: String,
    /// Optional path to the markdown template, relative to the manifest.
    #[serde(default)]
    pub template_path: Option<String>,
    /// SHA-256 (lowercase hex) of the markdown template bytes, if present.
    #[serde(default)]
    pub template_checksum: Option<String>,
}

/// Descriptive kanban onboarding defaults for a collection.
///
/// v1 is metadata only: suggestions seed the onboarding mapping UI but do
/// not drive sync behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KanbanDefaults {
    /// Suggested provider issue-type NAME -> collection issuetype key.
    /// `BTreeMap` keeps serialization order deterministic so the hosted
    /// bundle's manifest checksum is stable across generation runs.
    #[serde(default)]
    pub suggested_type_mappings: BTreeMap<String, String>,
}

/// Descriptive metadata about a collection's intended agentic loop shape.
///
/// v1 is metadata only: these fields are stored and displayed but do not
/// drive any execution-engine behavior. `runner_semantics` is always
/// `prompt_driven` in v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHints {
    /// Loop shape (e.g. `single_pass`, `ralph`, `review_loop`).
    #[serde(default)]
    pub loop_kind: Option<String>,
    /// Memory surfaces the loop reads/writes (e.g. `scratchpad`, `notes.md`).
    #[serde(default)]
    pub memory_surfaces: Vec<String>,
    /// Review gates between iterations (e.g. `human`, `test_suite`).
    #[serde(default)]
    pub review_gates: Vec<String>,
    /// External tools the loop expects (e.g. `gh`, `playwright`).
    #[serde(default)]
    pub external_tools: Vec<String>,
    /// Conditions under which the loop stops (e.g. `tests_green`, `max_iters`).
    #[serde(default)]
    pub stop_conditions: Vec<String>,
    /// How the runner interprets the hints. v1 is always `prompt_driven`.
    #[serde(default = "default_runner_semantics")]
    pub runner_semantics: String,
}

fn default_runner_semantics() -> String {
    "prompt_driven".to_string()
}

impl Default for WorkflowHints {
    fn default() -> Self {
        Self {
            loop_kind: None,
            memory_surfaces: Vec::new(),
            review_gates: Vec::new(),
            external_tools: Vec::new(),
            stop_conditions: Vec::new(),
            runner_semantics: default_runner_semantics(),
        }
    }
}

impl CollectionManifest {
    /// Parse a manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize the manifest to pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Issue type keys in display order.
    pub fn type_keys(&self) -> Vec<String> {
        self.issue_types.iter().map(|e| e.key.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEV_KANBAN_JSON: &str = r#"{
        "schema_version": 1,
        "id": "dev_kanban",
        "name": "Dev Kanban",
        "description": "Developer kanban with TASK, FEAT, FIX",
        "version": "1.0.0",
        "publisher": "untra",
        "license": "MIT",
        "tags": ["kanban", "dev"],
        "issue_types": [
            {"key": "TASK", "schema_path": "TASK.json", "schema_checksum": "aaa", "template_path": "TASK.md", "template_checksum": "bbb"},
            {"key": "FEAT", "schema_path": "FEAT.json", "schema_checksum": "ccc"},
            {"key": "FIX", "schema_path": "FIX.json", "schema_checksum": "ddd"}
        ],
        "workflow_hints": {
            "loop_kind": "single_pass",
            "review_gates": ["test_suite"]
        },
        "default_selected": ["TASK", "FEAT", "FIX"]
    }"#;

    #[test]
    fn test_manifest_parses_full_fields() {
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        assert_eq!(m.id, "dev_kanban");
        assert_eq!(m.name, "Dev Kanban");
        assert_eq!(m.version, "1.0.0");
        assert_eq!(m.publisher.as_deref(), Some("untra"));
        assert_eq!(m.type_keys(), vec!["TASK", "FEAT", "FIX"]);
        assert_eq!(m.default_selected, vec!["TASK", "FEAT", "FIX"]);
        // First entry has a template, second does not.
        assert_eq!(m.issue_types[0].template_path.as_deref(), Some("TASK.md"));
        assert!(m.issue_types[1].template_path.is_none());
    }

    #[test]
    fn test_workflow_hints_runner_semantics_defaults_to_prompt_driven() {
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        let hints = m.workflow_hints.expect("hints present");
        // runner_semantics omitted in the fixture -> defaults applied.
        assert_eq!(hints.runner_semantics, "prompt_driven");
        assert_eq!(hints.loop_kind.as_deref(), Some("single_pass"));
        assert_eq!(hints.review_gates, vec!["test_suite"]);
    }

    #[test]
    fn test_workflow_hints_default_impl() {
        let hints = WorkflowHints::default();
        assert_eq!(hints.runner_semantics, "prompt_driven");
        assert!(hints.memory_surfaces.is_empty());
    }

    #[test]
    fn test_manifest_author_and_url_round_trip() {
        let json = r#"{
            "schema_version": 1,
            "id": "simple",
            "name": "Simple",
            "author": "Operator!",
            "url": "https://github.com/untra/operator",
            "issue_types": [
                {"key": "TASK", "schema_path": "TASK.json"}
            ]
        }"#;
        let m = CollectionManifest::from_json(json).unwrap();
        assert_eq!(m.author.as_deref(), Some("Operator!"));
        assert_eq!(m.url.as_deref(), Some("https://github.com/untra/operator"));
        // Survives a serialize/parse round trip.
        let m2 = CollectionManifest::from_json(&m.to_json().unwrap()).unwrap();
        assert_eq!(m2.author, m.author);
        assert_eq!(m2.url, m.url);
    }

    #[test]
    fn test_manifest_author_url_default_to_none() {
        // Omitted fields default to None (older manifests stay valid).
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        assert!(m.author.is_none());
        assert!(m.url.is_none());
    }

    #[test]
    fn test_manifest_round_trip() {
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        let json = m.to_json().unwrap();
        let m2 = CollectionManifest::from_json(&json).unwrap();
        assert_eq!(m.id, m2.id);
        assert_eq!(m.type_keys(), m2.type_keys());
    }

    #[test]
    fn test_manifest_tier_defaults_to_official() {
        // Older manifests without a tier field stay valid and are official.
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        assert_eq!(m.tier, CollectionTier::Official);
    }

    #[test]
    fn test_manifest_tier_community_round_trip() {
        let json = r#"{
            "schema_version": 1,
            "id": "gastown",
            "name": "Gastown",
            "tier": "community",
            "author": "someone",
            "issue_types": [
                {"key": "TASK", "schema_path": "TASK.json"}
            ]
        }"#;
        let m = CollectionManifest::from_json(json).unwrap();
        assert_eq!(m.tier, CollectionTier::Community);
        let serialized = m.to_json().unwrap();
        assert!(serialized.contains("\"tier\": \"community\""));
        let m2 = CollectionManifest::from_json(&serialized).unwrap();
        assert_eq!(m2.tier, CollectionTier::Community);
    }

    #[test]
    fn test_index_entry_tier_and_docs_path_default() {
        let json = r#"{
            "schema_version": 1,
            "collections": [
                {"id": "simple", "name": "Simple", "manifest_path": "simple/collection.json", "checksum": "abc"}
            ]
        }"#;
        let index: CollectionIndex = serde_json::from_str(json).unwrap();
        let entry = &index.collections[0];
        assert_eq!(entry.tier, CollectionTier::Official);
        assert!(entry.docs_path.is_none());
    }

    #[test]
    fn test_index_entry_tier_and_docs_path_round_trip() {
        let json = r#"{
            "schema_version": 1,
            "collections": [
                {"id": "gastown", "name": "Gastown", "manifest_path": "gastown/collection.json", "checksum": "abc", "tier": "community", "docs_path": "gastown/"}
            ]
        }"#;
        let index: CollectionIndex = serde_json::from_str(json).unwrap();
        let entry = &index.collections[0];
        assert_eq!(entry.tier, CollectionTier::Community);
        assert_eq!(entry.docs_path.as_deref(), Some("gastown/"));
        let round: CollectionIndex =
            serde_json::from_str(&serde_json::to_string(&index).unwrap()).unwrap();
        assert_eq!(round.collections[0].tier, CollectionTier::Community);
        assert_eq!(round.collections[0].docs_path.as_deref(), Some("gastown/"));
    }

    #[test]
    fn test_manifest_icon_and_dates_default_to_none() {
        // Older manifests without icon/date fields stay valid.
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        assert!(m.icon_path.is_none());
        assert!(m.created.is_none());
        assert!(m.updated.is_none());
    }

    #[test]
    fn test_manifest_icon_and_dates_round_trip() {
        let json = r#"{
            "schema_version": 1,
            "id": "ralph_loop",
            "name": "Ralph Loop",
            "icon_path": "icon.svg",
            "created": "2026-05-01",
            "updated": "2026-08-01",
            "issue_types": [
                {"key": "PRD", "schema_path": "PRD.json"}
            ]
        }"#;
        let m = CollectionManifest::from_json(json).unwrap();
        assert_eq!(m.icon_path.as_deref(), Some("icon.svg"));
        assert_eq!(m.created.as_deref(), Some("2026-05-01"));
        assert_eq!(m.updated.as_deref(), Some("2026-08-01"));
        let m2 = CollectionManifest::from_json(&m.to_json().unwrap()).unwrap();
        assert_eq!(m2.icon_path, m.icon_path);
        assert_eq!(m2.created, m.created);
        assert_eq!(m2.updated, m.updated);
    }

    #[test]
    fn test_icon_and_dates_are_outside_the_manifest_checksum() {
        // The checksum derives from issue-type file checksums only, so changing
        // presentational metadata must not invalidate a hosted collection.
        let mut m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        let before = crate::collections::fetch::derive_manifest_checksum(&m.issue_types);

        m.icon_path = Some("icon.svg".to_string());
        m.created = Some("2026-05-01".to_string());
        m.updated = Some("2026-08-01".to_string());
        let after = crate::collections::fetch::derive_manifest_checksum(&m.issue_types);

        assert_eq!(before, after);
    }

    /// Every field of a fully-populated manifest, so the parity tests below
    /// compare complete key sets rather than whatever `DEV_KANBAN_JSON` happens
    /// to set.
    fn fully_populated() -> CollectionManifest {
        let mut m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        m.author = Some("Operator!".to_string());
        m.url = Some("https://github.com/untra/operator".to_string());
        m.license = Some("MIT".to_string());
        m.compatibility = Some(Compatibility {
            operator_version: Some(">=0.2.0".to_string()),
        });
        m.icon_path = Some("icon.svg".to_string());
        m.created = Some("2026-01-15".to_string());
        m.updated = Some("2026-08-01".to_string());
        m.kanban_defaults = Some(KanbanDefaults::default());
        m.checksum = Some("deadbeef".to_string());
        m
    }

    fn schema_properties(pointer: &str) -> serde_json::Map<String, serde_json::Value> {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../schemas/issuetype_collection_schema.json"))
                .expect("schema parses");
        schema
            .pointer(pointer)
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(|| panic!("schema has no properties at {pointer}"))
    }

    /// The published schema sets `additionalProperties: false`, so a Rust-side
    /// field addition that is not mirrored there silently breaks validation of
    /// every hosted manifest. Compare the key sets in both directions.
    #[test]
    fn test_manifest_fields_match_published_json_schema() {
        let serialized = serde_json::to_value(fully_populated()).unwrap();
        let rust_keys: Vec<&str> = serialized
            .as_object()
            .unwrap()
            .keys()
            .map(|k| &**k)
            .collect();
        let schema_props = schema_properties("/properties");

        for key in &rust_keys {
            assert!(
                schema_props.contains_key(*key),
                "CollectionManifest field '{key}' is missing from \
                 src/schemas/issuetype_collection_schema.json"
            );
        }
        for key in schema_props.keys() {
            assert!(
                rust_keys.contains(&&**key),
                "schema property '{key}' has no CollectionManifest field"
            );
        }
    }

    #[test]
    fn test_issue_type_entry_fields_match_published_json_schema() {
        let entry = IssueTypeEntry {
            key: "TASK".to_string(),
            schema_path: "TASK.json".to_string(),
            schema_checksum: "aaa".to_string(),
            template_path: Some("TASK.md".to_string()),
            template_checksum: Some("bbb".to_string()),
        };
        let serialized = serde_json::to_value(&entry).unwrap();
        let rust_keys: Vec<&str> = serialized
            .as_object()
            .unwrap()
            .keys()
            .map(|k| &**k)
            .collect();
        let schema_props = schema_properties("/properties/issue_types/items/properties");

        for key in &rust_keys {
            assert!(
                schema_props.contains_key(*key),
                "IssueTypeEntry field '{key}' is missing from the published schema"
            );
        }
        for key in schema_props.keys() {
            assert!(
                rust_keys.contains(&&**key),
                "schema property '{key}' has no IssueTypeEntry field"
            );
        }
    }

    #[test]
    fn test_kanban_defaults_default_to_none() {
        let m = CollectionManifest::from_json(DEV_KANBAN_JSON).unwrap();
        assert!(m.kanban_defaults.is_none());
    }

    #[test]
    fn test_kanban_defaults_round_trip() {
        let json = r#"{
            "schema_version": 1,
            "id": "dev_kanban",
            "name": "Dev Kanban",
            "issue_types": [
                {"key": "TASK", "schema_path": "TASK.json"}
            ],
            "kanban_defaults": {
                "suggested_type_mappings": {"Story": "FEAT", "Bug": "FIX"}
            }
        }"#;
        let m = CollectionManifest::from_json(json).unwrap();
        let defaults = m.kanban_defaults.as_ref().expect("kanban_defaults");
        assert_eq!(
            defaults.suggested_type_mappings.get("Story"),
            Some(&"FEAT".to_string())
        );
        let m2 = CollectionManifest::from_json(&m.to_json().unwrap()).unwrap();
        assert_eq!(m2.kanban_defaults.unwrap().suggested_type_mappings.len(), 2);
    }
}
