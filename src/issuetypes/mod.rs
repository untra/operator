//! # Partially Integrated Module: Dynamic Issue Type Registry
//!
//! **Status**: Complete implementation, partially integrated
//!
//! **Purpose**: Dynamic registry system for loading, managing, and querying issue types
//! with support for user-defined types, collections, and preset configurations.
//!
//! **Current Integration**:
//! - Schema definitions used internally by `templates` module
//! - Builtin collections (simple, dev_kanban, devops_kanban) defined
//! - Registry loading and validation implemented
//!
//! **Not Yet Integrated**:
//! - Dynamic registry not exposed to TUI for runtime switching
//! - User-defined issue types from `.tickets/operator/issuetypes/` not loaded
//! - Collection switching not available in UI
//!
//! **Integration Point**: `templates/mod.rs` for runtime loading, `ui/create_dialog.rs` for selection
//!
//! **Milestone**: TBD - When custom issue type workflows are prioritized
//!
//! ## Components
//!
//! - [`IssueType`]: Dynamic issue type definitions (extends TemplateSchema)
//! - [`IssueTypeCollection`]: Named groupings of issue types with priority ordering
//! - [`IssueTypeRegistry`]: Central manager for all issue types and collections
//! - [`BuiltinPreset`]: Predefined collection configurations
//!
//! ## Usage When Fully Integrated
//!
//! ```rust,ignore
//! use crate::issuetypes::IssueTypeRegistry;
//!
//! let mut registry = IssueTypeRegistry::new();
//! registry.load_all(&tickets_path)?;
//! registry.activate_collection("devops_kanban")?;
//!
//! for issue_type in registry.active_types() {
//!     println!("{}: {}", issue_type.key, issue_type.name);
//! }
//! ```

#![allow(dead_code)] // PARTIAL: Schema used internally, registry not yet exposed to UI

pub mod collection;
pub mod loader;
pub mod schema;

pub use collection::{BuiltinPreset, IssueTypeCollection};
pub use schema::IssueType;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Central registry for all issue types and collections
#[derive(Debug, Clone)]
pub struct IssueTypeRegistry {
    /// All registered issue types by key
    types: HashMap<String, IssueType>,
    /// Named collections
    collections: HashMap<String, IssueTypeCollection>,
    /// Currently active collection name
    active_collection: String,
}

impl Default for IssueTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl IssueTypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            collections: HashMap::new(),
            active_collection: "devops_kanban".to_string(),
        }
    }

    /// Load built-in issue types
    pub fn load_builtins(&mut self) -> Result<()> {
        let builtins = loader::load_builtins()?;
        for (key, issue_type) in builtins {
            self.types.insert(key, issue_type);
        }

        // Add builtin collections
        for preset in BuiltinPreset::all() {
            let collection = preset.into_collection();
            self.collections.insert(collection.name.clone(), collection);
        }

        info!("Loaded {} builtin issue types", self.types.len());
        Ok(())
    }

    /// Load user-defined issue types from a directory
    pub fn load_user_types(&mut self, path: &Path) -> Result<()> {
        let user_types = loader::load_user_types(path)?;
        let count = user_types.len();

        for (key, issue_type) in user_types {
            if self.types.contains_key(&key) {
                debug!("User type '{}' overrides builtin", key);
            }
            self.types.insert(key, issue_type);
        }

        if count > 0 {
            info!("Loaded {} user-defined issue types", count);
        }
        Ok(())
    }

    /// Load imported issue types from imports directory
    pub fn load_imports(&mut self, imports_path: &Path) -> Result<()> {
        let imported = loader::load_imported_types(imports_path)?;
        let count = imported.len();

        for (key, issue_type) in imported {
            self.types.insert(key, issue_type);
        }

        if count > 0 {
            info!("Loaded {} imported issue types", count);
        }
        Ok(())
    }

    /// Load collections from collections.toml
    pub fn load_collections(&mut self, path: &Path) -> Result<()> {
        let collections = loader::load_collections(path)?;
        let count = collections.len();

        for (name, collection) in collections {
            // Validate collection types, warn about missing ones
            let (valid, missing) = loader::validate_collection_types(&collection, &self.types);
            if !missing.is_empty() {
                warn!(
                    "Collection '{}' references unknown types: {:?}",
                    name, missing
                );
            }

            if valid.is_empty() {
                warn!("Collection '{}' has no valid types, skipping", name);
                continue;
            }

            // Update collection to only include valid types
            let mut validated_collection = collection;
            validated_collection.types = valid;
            self.collections.insert(name, validated_collection);
        }

        if count > 0 {
            info!("Loaded {} user-defined collections", count);
        }
        Ok(())
    }

    /// Load all issue types and collections from standard paths
    ///
    /// Standard paths:
    /// - `.tickets/operator/issuetypes/` for user types
    /// - `.tickets/operator/issuetypes/imports/` for imported types
    /// - `.tickets/operator/issuetypes/collections.toml` for collections
    pub fn load_all(&mut self, tickets_path: &Path) -> Result<()> {
        // First load builtins
        self.load_builtins()?;

        let issuetypes_path = tickets_path.join("operator/issuetypes");
        if issuetypes_path.exists() {
            // Load user types
            self.load_user_types(&issuetypes_path)?;

            // Load imports
            let imports_path = issuetypes_path.join("imports");
            if imports_path.exists() {
                self.load_imports(&imports_path)?;
            }

            // Load collections
            let collections_path = issuetypes_path.join("collections.toml");
            if collections_path.exists() {
                self.load_collections(&collections_path)?;
            }
        }

        Ok(())
    }

    /// Activate a builtin preset
    pub fn activate_preset(&mut self, preset: BuiltinPreset) -> Result<()> {
        let name = preset.name();
        if self.collections.contains_key(name) {
            self.active_collection = name.to_string();
            info!("Activated collection preset: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Preset '{}' not found in collections", name)
        }
    }

    /// Activate a named collection
    pub fn activate_collection(&mut self, name: &str) -> Result<()> {
        // First check if it's a builtin preset name
        if let Some(preset) = BuiltinPreset::from_name(name) {
            return self.activate_preset(preset);
        }

        // Otherwise check user collections
        if self.collections.contains_key(name) {
            self.active_collection = name.to_string();
            info!("Activated collection: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Collection '{}' not found", name)
        }
    }

    /// Activate a custom collection of types
    pub fn activate_custom(&mut self, type_keys: &[String]) -> Result<()> {
        // Validate that all types exist
        let mut valid_keys = Vec::new();
        for key in type_keys {
            if self.types.contains_key(key) {
                valid_keys.push(key.clone());
            } else {
                warn!(
                    "Custom collection references unknown type '{}', skipping",
                    key
                );
            }
        }

        if valid_keys.is_empty() {
            anyhow::bail!("No valid types in custom collection");
        }

        // Create or update the "custom" collection
        let collection = IssueTypeCollection::new("custom", "Custom collection")
            .with_types(valid_keys.iter().map(|s| s.as_str()));

        self.collections.insert("custom".to_string(), collection);
        self.active_collection = "custom".to_string();
        info!(
            "Activated custom collection with {} types",
            valid_keys.len()
        );
        Ok(())
    }

    /// Get an issue type by key
    pub fn get(&self, key: &str) -> Option<&IssueType> {
        self.types.get(key)
    }

    /// Get all registered issue types
    pub fn all_types(&self) -> impl Iterator<Item = &IssueType> {
        self.types.values()
    }

    /// Get the active collection
    pub fn active_collection(&self) -> Option<&IssueTypeCollection> {
        self.collections.get(&self.active_collection)
    }

    /// Get the name of the active collection
    pub fn active_collection_name(&self) -> &str {
        &self.active_collection
    }

    /// Get all issue types in the active collection (ordered)
    pub fn active_types(&self) -> Vec<&IssueType> {
        let Some(collection) = self.active_collection() else {
            return vec![];
        };

        collection
            .types
            .iter()
            .filter_map(|key| self.types.get(key))
            .collect()
    }

    /// Check if a key exists in the active collection
    pub fn is_active(&self, key: &str) -> bool {
        self.active_collection()
            .map(|c| c.contains(key))
            .unwrap_or(false)
    }

    /// Get priority index for a type in the active collection
    pub fn priority_index(&self, key: &str) -> usize {
        self.active_collection()
            .map(|c| c.priority_index(key))
            .unwrap_or(usize::MAX)
    }

    /// Get all available collections
    pub fn all_collections(&self) -> impl Iterator<Item = &IssueTypeCollection> {
        self.collections.values()
    }

    /// Get a collection by name
    pub fn get_collection(&self, name: &str) -> Option<&IssueTypeCollection> {
        self.collections.get(name)
    }

    /// Register a new issue type
    pub fn register(&mut self, issue_type: IssueType) -> Result<()> {
        issue_type.validate().map_err(|errors| {
            let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            anyhow::anyhow!("Validation errors: {}", msgs.join("; "))
        })?;

        let key = issue_type.key.clone();
        self.types.insert(key.clone(), issue_type);
        debug!("Registered issue type: {}", key);
        Ok(())
    }

    /// Register a new collection
    pub fn register_collection(&mut self, collection: IssueTypeCollection) -> Result<()> {
        let (_valid, missing) = loader::validate_collection_types(&collection, &self.types);
        if !missing.is_empty() {
            warn!(
                "Collection '{}' references unknown types: {:?}",
                collection.name, missing
            );
        }

        let name = collection.name.clone();
        self.collections.insert(name.clone(), collection);
        debug!("Registered collection: {}", name);
        Ok(())
    }

    /// Get the number of registered types
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    /// Get the number of registered collections
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = IssueTypeRegistry::new();
        assert_eq!(registry.type_count(), 0);
        assert_eq!(registry.collection_count(), 0);
    }

    #[test]
    fn test_registry_load_builtins() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        // Should have 5 builtin types
        assert!(registry.type_count() >= 5);
        assert!(registry.get("FEAT").is_some());
        assert!(registry.get("FIX").is_some());
        assert!(registry.get("TASK").is_some());
        assert!(registry.get("SPIKE").is_some());
        assert!(registry.get("INV").is_some());

        // Should have 3 builtin collections
        assert!(registry.collection_count() >= 3);
        assert!(registry.get_collection("simple").is_some());
        assert!(registry.get_collection("dev_kanban").is_some());
        assert!(registry.get_collection("devops_kanban").is_some());
    }

    #[test]
    fn test_registry_active_types() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        // Default is devops_kanban
        let active = registry.active_types();
        assert_eq!(active.len(), 5);

        // Switch to simple
        registry.activate_collection("simple").unwrap();
        let active = registry.active_types();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].key, "TASK");

        // Switch to dev_kanban
        registry.activate_collection("dev_kanban").unwrap();
        let active = registry.active_types();
        assert_eq!(active.len(), 3);
    }

    #[test]
    fn test_registry_priority() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();
        registry.activate_collection("devops_kanban").unwrap();

        // devops_kanban priority: INV, FIX, FEAT, SPIKE, TASK
        assert_eq!(registry.priority_index("INV"), 0);
        assert_eq!(registry.priority_index("FIX"), 1);
        assert_eq!(registry.priority_index("FEAT"), 2);
        assert_eq!(registry.priority_index("SPIKE"), 3);
        assert_eq!(registry.priority_index("TASK"), 4);
    }

    #[test]
    fn test_registry_is_active() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();
        registry.activate_collection("simple").unwrap();

        assert!(registry.is_active("TASK"));
        assert!(!registry.is_active("FEAT"));
        assert!(!registry.is_active("FIX"));
    }

    #[test]
    fn test_registry_activate_custom() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        registry
            .activate_custom(&["FEAT".to_string(), "FIX".to_string()])
            .unwrap();

        let active = registry.active_types();
        assert_eq!(active.len(), 2);
        assert_eq!(registry.active_collection_name(), "custom");
    }

    #[test]
    fn test_registry_activate_custom_warns_missing() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        // NONEXISTENT should be warned and skipped
        registry
            .activate_custom(&[
                "FEAT".to_string(),
                "NONEXISTENT".to_string(),
                "FIX".to_string(),
            ])
            .unwrap();

        let active = registry.active_types();
        assert_eq!(active.len(), 2); // Only FEAT and FIX
    }

    #[test]
    fn test_registry_register_type() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        let issue_type = IssueType::new_imported(
            "STORY".to_string(),
            "Story".to_string(),
            "A user story".to_string(),
            "custom".to_string(),
            "".to_string(),
            None,
        );

        registry.register(issue_type).unwrap();
        assert!(registry.get("STORY").is_some());
    }

    #[test]
    fn test_registry_all_types_iterator() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        let all: Vec<_> = registry.all_types().collect();
        assert!(all.len() >= 5);
    }

    #[test]
    fn test_registry_all_collections_iterator() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        let all: Vec<_> = registry.all_collections().collect();
        assert!(all.len() >= 3);
    }
}
