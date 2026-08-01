//! Issue type collection definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metadata about where a collection was synced from (e.g., Jira, Linear)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionSyncSource {
    /// Provider type (e.g., "jira", "linear", "`github_projects`")
    pub provider: String,
    /// Base URL of the provider instance
    #[serde(default)]
    pub base_url: Option<String>,
    /// Project/board identifier in the external system
    #[serde(default)]
    pub project_id: Option<String>,
    /// Last sync timestamp
    #[serde(default)]
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// A named collection of issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeCollection {
    /// Collection name (unique identifier)
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Ordered list of issue type keys in this collection (display order)
    pub types: Vec<String>,
    /// Sync source metadata (if collection was synced from external provider)
    #[serde(default)]
    pub sync_source: Option<CollectionSyncSource>,
    /// Descriptive workflow hints (v1: metadata only, from a hosted manifest)
    #[serde(default)]
    pub workflow_hints: Option<crate::collections::manifest::WorkflowHints>,
    /// Collection semver (from a hosted manifest)
    #[serde(default)]
    pub version: Option<String>,
    /// Publisher identifier (from a hosted manifest)
    #[serde(default)]
    pub publisher: Option<String>,
    /// Author attribution (from a hosted manifest)
    #[serde(default)]
    pub author: Option<String>,
    /// Provenance tier (official when absent)
    #[serde(default)]
    pub tier: crate::collections::manifest::CollectionTier,
}

impl IssueTypeCollection {
    /// Create a new collection
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            types: vec![],
            sync_source: None,
            workflow_hints: None,
            version: None,
            publisher: None,
            author: None,
            tier: crate::collections::manifest::CollectionTier::default(),
        }
    }

    /// Attach manifest metadata (workflow hints, version, publisher, author, tier).
    pub fn with_manifest_metadata(
        mut self,
        workflow_hints: Option<crate::collections::manifest::WorkflowHints>,
        version: Option<String>,
        publisher: Option<String>,
        author: Option<String>,
        tier: crate::collections::manifest::CollectionTier,
    ) -> Self {
        self.workflow_hints = workflow_hints;
        self.version = version;
        self.publisher = publisher;
        self.author = author;
        self.tier = tier;
        self
    }

    /// Set the sync source for this collection
    pub fn with_sync_source(mut self, source: CollectionSyncSource) -> Self {
        self.sync_source = Some(source);
        self
    }

    /// Add an issue type to the collection
    pub fn with_type(mut self, key: impl Into<String>) -> Self {
        self.types.push(key.into());
        self
    }

    /// Add multiple issue types to the collection
    pub fn with_types(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.types
            .extend(keys.into_iter().map(std::convert::Into::into));
        self
    }

    /// Get position index for a type in the collection
    /// Returns `usize::MAX` if type not in collection
    pub fn priority_index(&self, key: &str) -> usize {
        self.types
            .iter()
            .position(|k| k == key)
            .unwrap_or(usize::MAX)
    }

    /// Check if a type is in this collection
    pub fn contains(&self, key: &str) -> bool {
        self.types.iter().any(|k| k == key)
    }

    /// Get the number of types in this collection
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

/// Wrapper struct for parsing collections.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionsFile {
    /// Map of collection name to collection definition
    #[serde(default)]
    pub collections: std::collections::HashMap<String, IssueTypeCollection>,
}

impl CollectionsFile {
    /// Parse from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_new() {
        let collection = IssueTypeCollection::new("test", "A test collection");
        assert_eq!(collection.name, "test");
        assert_eq!(collection.description, "A test collection");
        assert!(collection.is_empty());
    }

    #[test]
    fn test_collection_with_types() {
        let collection = IssueTypeCollection::new("test", "").with_types(["FEAT", "FIX", "TASK"]);
        assert_eq!(collection.len(), 3);
        assert!(collection.contains("FEAT"));
        assert!(collection.contains("FIX"));
        assert!(collection.contains("TASK"));
        assert!(!collection.contains("SPIKE"));
    }

    #[test]
    fn test_collection_priority_index() {
        let collection = IssueTypeCollection::new("test", "").with_types(["FEAT", "FIX", "TASK"]);

        assert_eq!(collection.priority_index("FEAT"), 0);
        assert_eq!(collection.priority_index("FIX"), 1);
        assert_eq!(collection.priority_index("TASK"), 2);
        assert_eq!(collection.priority_index("SPIKE"), usize::MAX);
    }

    #[test]
    fn test_collections_file_parse() {
        let toml = r#"
[collections.agile]
name = "agile"
description = "Agile workflow"
types = ["STORY", "BUG", "TASK"]

[collections.custom]
name = "custom"
description = "Custom workflow"
types = ["FEAT", "FIX"]
"#;

        let file = CollectionsFile::from_toml(toml).unwrap();
        assert_eq!(file.collections.len(), 2);

        let agile = file.collections.get("agile").unwrap();
        assert_eq!(agile.types, vec!["STORY", "BUG", "TASK"]);

        let custom = file.collections.get("custom").unwrap();
        assert_eq!(custom.types, vec!["FEAT", "FIX"]);
    }

    #[test]
    fn test_collections_file_serialize() {
        let mut file = CollectionsFile::default();
        file.collections.insert(
            "test".to_string(),
            IssueTypeCollection::new("test", "Test collection").with_types(["FEAT", "FIX"]),
        );

        let toml = file.to_toml().unwrap();
        assert!(toml.contains("[collections.test]"));
        assert!(toml.contains("FEAT"));
        assert!(toml.contains("FIX"));
    }
}
