//! Issue type collection definitions

use serde::{Deserialize, Serialize};

/// A named collection of issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeCollection {
    /// Collection name (unique identifier)
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Ordered list of issue type keys in this collection
    pub types: Vec<String>,
    /// Priority order for queue sorting (first = highest priority)
    /// If empty, uses `types` order
    #[serde(default)]
    pub priority_order: Vec<String>,
}

impl IssueTypeCollection {
    /// Create a new collection
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            types: vec![],
            priority_order: vec![],
        }
    }

    /// Add an issue type to the collection
    pub fn with_type(mut self, key: impl Into<String>) -> Self {
        self.types.push(key.into());
        self
    }

    /// Add multiple issue types to the collection
    pub fn with_types(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.types.extend(keys.into_iter().map(|k| k.into()));
        self
    }

    /// Set the priority order
    pub fn with_priority_order(
        mut self,
        order: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.priority_order = order.into_iter().map(|k| k.into()).collect();
        self
    }

    /// Get priority index for a type (lower = higher priority)
    /// Returns usize::MAX if type not in priority order
    pub fn priority_index(&self, key: &str) -> usize {
        let order = if self.priority_order.is_empty() {
            &self.types
        } else {
            &self.priority_order
        };

        order.iter().position(|k| k == key).unwrap_or(usize::MAX)
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

/// Built-in collection presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinPreset {
    /// Simple: TASK only
    Simple,
    /// Dev Kanban: TASK, FEAT, FIX
    DevKanban,
    /// DevOps Kanban: TASK, SPIKE, INV, FEAT, FIX
    DevopsKanban,
    /// Operator: ASSESS, SYNC, INIT (Backstage operations)
    Operator,
    /// Backstage Full: DevOps + Operator types
    BackstageFull,
}

impl BuiltinPreset {
    /// Get all builtin presets
    pub fn all() -> &'static [BuiltinPreset] {
        &[
            BuiltinPreset::Simple,
            BuiltinPreset::DevKanban,
            BuiltinPreset::DevopsKanban,
            BuiltinPreset::Operator,
            BuiltinPreset::BackstageFull,
        ]
    }

    /// Get the collection name for this preset
    pub fn name(&self) -> &'static str {
        match self {
            BuiltinPreset::Simple => "simple",
            BuiltinPreset::DevKanban => "dev_kanban",
            BuiltinPreset::DevopsKanban => "devops_kanban",
            BuiltinPreset::Operator => "operator",
            BuiltinPreset::BackstageFull => "backstage_full",
        }
    }

    /// Get the description for this preset
    pub fn description(&self) -> &'static str {
        match self {
            BuiltinPreset::Simple => "Simple workflow with TASK only",
            BuiltinPreset::DevKanban => "Developer kanban with TASK, FEAT, FIX",
            BuiltinPreset::DevopsKanban => "DevOps kanban with TASK, SPIKE, INV, FEAT, FIX",
            BuiltinPreset::Operator => "Operator Backstage tasks: ASSESS, SYNC, INIT",
            BuiltinPreset::BackstageFull => "Full workflow plus Backstage: all types combined",
        }
    }

    /// Convert to an IssueTypeCollection
    pub fn into_collection(self) -> IssueTypeCollection {
        match self {
            BuiltinPreset::Simple => {
                IssueTypeCollection::new("simple", self.description()).with_types(["TASK"])
            }
            BuiltinPreset::DevKanban => IssueTypeCollection::new("dev_kanban", self.description())
                .with_types(["TASK", "FEAT", "FIX"])
                .with_priority_order(["FIX", "FEAT", "TASK"]),
            BuiltinPreset::DevopsKanban => {
                IssueTypeCollection::new("devops_kanban", self.description())
                    .with_types(["TASK", "SPIKE", "INV", "FEAT", "FIX"])
                    .with_priority_order(["INV", "FIX", "FEAT", "SPIKE", "TASK"])
            }
            BuiltinPreset::Operator => IssueTypeCollection::new("operator", self.description())
                .with_types(["ASSESS", "SYNC", "INIT"])
                .with_priority_order(["ASSESS", "SYNC", "INIT"]),
            BuiltinPreset::BackstageFull => {
                IssueTypeCollection::new("backstage_full", self.description())
                    .with_types([
                        "INV", "FIX", "FEAT", "SPIKE", "TASK", "ASSESS", "SYNC", "INIT",
                    ])
                    .with_priority_order([
                        "INV", "FIX", "FEAT", "ASSESS", "SPIKE", "TASK", "SYNC", "INIT",
                    ])
            }
        }
    }

    /// Parse preset name to variant
    pub fn from_name(name: &str) -> Option<BuiltinPreset> {
        match name.to_lowercase().as_str() {
            "simple" => Some(BuiltinPreset::Simple),
            "dev_kanban" | "devkanban" => Some(BuiltinPreset::DevKanban),
            "devops_kanban" | "devopskanban" => Some(BuiltinPreset::DevopsKanban),
            "operator" => Some(BuiltinPreset::Operator),
            "backstage_full" | "backstagefull" => Some(BuiltinPreset::BackstageFull),
            _ => None,
        }
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
        let collection = IssueTypeCollection::new("test", "")
            .with_types(["FEAT", "FIX", "TASK"])
            .with_priority_order(["FIX", "FEAT", "TASK"]);

        assert_eq!(collection.priority_index("FIX"), 0);
        assert_eq!(collection.priority_index("FEAT"), 1);
        assert_eq!(collection.priority_index("TASK"), 2);
        assert_eq!(collection.priority_index("SPIKE"), usize::MAX);
    }

    #[test]
    fn test_collection_priority_defaults_to_types() {
        let collection = IssueTypeCollection::new("test", "").with_types(["FEAT", "FIX", "TASK"]);
        // No priority_order set, should use types order

        assert_eq!(collection.priority_index("FEAT"), 0);
        assert_eq!(collection.priority_index("FIX"), 1);
        assert_eq!(collection.priority_index("TASK"), 2);
    }

    #[test]
    fn test_builtin_simple() {
        let collection = BuiltinPreset::Simple.into_collection();
        assert_eq!(collection.name, "simple");
        assert_eq!(collection.types, vec!["TASK"]);
    }

    #[test]
    fn test_builtin_dev_kanban() {
        let collection = BuiltinPreset::DevKanban.into_collection();
        assert_eq!(collection.name, "dev_kanban");
        assert_eq!(collection.types, vec!["TASK", "FEAT", "FIX"]);
        assert_eq!(collection.priority_order, vec!["FIX", "FEAT", "TASK"]);
    }

    #[test]
    fn test_builtin_devops_kanban() {
        let collection = BuiltinPreset::DevopsKanban.into_collection();
        assert_eq!(collection.name, "devops_kanban");
        assert_eq!(
            collection.types,
            vec!["TASK", "SPIKE", "INV", "FEAT", "FIX"]
        );
        assert_eq!(
            collection.priority_order,
            vec!["INV", "FIX", "FEAT", "SPIKE", "TASK"]
        );
    }

    #[test]
    fn test_builtin_from_name() {
        assert_eq!(
            BuiltinPreset::from_name("simple"),
            Some(BuiltinPreset::Simple)
        );
        assert_eq!(
            BuiltinPreset::from_name("dev_kanban"),
            Some(BuiltinPreset::DevKanban)
        );
        assert_eq!(
            BuiltinPreset::from_name("devops_kanban"),
            Some(BuiltinPreset::DevopsKanban)
        );
        assert_eq!(
            BuiltinPreset::from_name("operator"),
            Some(BuiltinPreset::Operator)
        );
        assert_eq!(
            BuiltinPreset::from_name("backstage_full"),
            Some(BuiltinPreset::BackstageFull)
        );
        assert_eq!(BuiltinPreset::from_name("unknown"), None);
    }

    #[test]
    fn test_builtin_operator() {
        let collection = BuiltinPreset::Operator.into_collection();
        assert_eq!(collection.name, "operator");
        assert_eq!(collection.types, vec!["ASSESS", "SYNC", "INIT"]);
        assert_eq!(collection.priority_order, vec!["ASSESS", "SYNC", "INIT"]);
    }

    #[test]
    fn test_builtin_backstage_full() {
        let collection = BuiltinPreset::BackstageFull.into_collection();
        assert_eq!(collection.name, "backstage_full");
        assert_eq!(
            collection.types,
            vec!["INV", "FIX", "FEAT", "SPIKE", "TASK", "ASSESS", "SYNC", "INIT"]
        );
        assert_eq!(
            collection.priority_order,
            vec!["INV", "FIX", "FEAT", "ASSESS", "SPIKE", "TASK", "SYNC", "INIT"]
        );
    }

    #[test]
    fn test_collections_file_parse() {
        let toml = r#"
[collections.agile]
name = "agile"
description = "Agile workflow"
types = ["STORY", "BUG", "TASK"]
priority_order = ["BUG", "STORY", "TASK"]

[collections.custom]
name = "custom"
description = "Custom workflow"
types = ["FEAT", "FIX"]
"#;

        let file = CollectionsFile::from_toml(toml).unwrap();
        assert_eq!(file.collections.len(), 2);

        let agile = file.collections.get("agile").unwrap();
        assert_eq!(agile.types, vec!["STORY", "BUG", "TASK"]);
        assert_eq!(agile.priority_order, vec!["BUG", "STORY", "TASK"]);

        let custom = file.collections.get("custom").unwrap();
        assert_eq!(custom.types, vec!["FEAT", "FIX"]);
        assert!(custom.priority_order.is_empty());
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
