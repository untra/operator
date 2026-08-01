//! # Dynamic Issue Type Registry
//!
//! Registry for loading, managing, and querying issue types with support
//! for collections and kanban-provider imports.
//!
//! Every surface (TUI, REST API, CLI) builds its registry through the single
//! canonical loader `crate::startup::templates::load_registry`, which reads
//! the collection-scoped `.tickets/templates/<collection>/` store, migrates
//! legacy flat user types into a `custom` collection, loads kanban imports
//! from `.tickets/operator/issuetypes/imports/`, and honors a legacy
//! `collections.toml`.
//!
//! ## Components
//!
//! - [`IssueType`]: Dynamic issue type definitions (extends `TemplateSchema`)
//! - [`IssueTypeCollection`]: Named groupings of issue types with priority ordering
//! - [`IssueTypeRegistry`]: Central manager for all issue types and collections
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut registry = crate::startup::templates::load_registry(&tickets_path);
//! registry.activate_collection("devops_kanban")?;
//!
//! for issue_type in registry.active_types() {
//!     println!("{}: {}", issue_type.key, issue_type.name);
//! }
//! ```

#![allow(dead_code)] // PARTIAL: Schema used internally, registry not yet exposed to UI

pub mod collection;
pub mod kanban_type;
pub mod loader;
pub mod schema;

pub use collection::IssueTypeCollection;
pub use schema::IssueType;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// One namespaced collection: its display metadata plus the issue types it
/// physically owns. Legacy key-groupings (collections.toml, `activate_custom`)
/// have an empty `types` map and resolve their keys across other collections.
#[derive(Debug, Clone)]
struct CollectionEntry {
    meta: IssueTypeCollection,
    types: HashMap<String, IssueType>,
}

/// Central registry for all issue types and collections.
///
/// Storage is **namespaced by collection**: an issue type key is unique only
/// within its collection, so the same key can carry different definitions in
/// different collections. Kanban-provider imports live in a reserved
/// namespace outside the collection map (not shareable, never listed).
#[derive(Debug, Clone)]
pub struct IssueTypeRegistry {
    /// Collections by name, each owning its types.
    collections: HashMap<String, CollectionEntry>,
    /// Collection load order (deterministic resolve fallback).
    order: Vec<String>,
    /// Kanban-provider imports, keyed `{PROJECT}_{KEY}`.
    imports: HashMap<String, IssueType>,
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
            collections: HashMap::new(),
            order: Vec::new(),
            imports: HashMap::new(),
            active_collection: "dev_kanban".to_string(),
        }
    }

    /// Get (or create) a collection entry, tracking load order.
    fn entry_mut(&mut self, name: &str) -> &mut CollectionEntry {
        if !self.collections.contains_key(name) {
            self.order.push(name.to_string());
            self.collections.insert(
                name.to_string(),
                CollectionEntry {
                    meta: IssueTypeCollection::new(name, ""),
                    types: HashMap::new(),
                },
            );
        }
        self.collections.get_mut(name).expect("entry just ensured")
    }

    /// Load built-in issue types from the embedded collection manifests.
    pub fn load_builtins(&mut self) -> Result<()> {
        for embedded in crate::collections::EMBEDDED_COLLECTIONS {
            let manifest = embedded.manifest_parsed()?;
            let mut types = HashMap::new();
            for it in embedded.issuetypes {
                let mut issue_type = IssueType::from_json(it.schema_json)?;
                issue_type.source = schema::IssueTypeSource::Builtin;
                types.insert(issue_type.key.clone(), issue_type);
            }
            let meta = IssueTypeCollection::new(&manifest.id, &manifest.description)
                .with_types(manifest.type_keys().iter().map(String::as_str))
                .with_manifest_metadata(
                    manifest.workflow_hints.clone(),
                    (!manifest.version.is_empty()).then(|| manifest.version.clone()),
                    manifest.publisher.clone(),
                    manifest.author.clone(),
                    manifest.tier,
                );
            let entry = self.entry_mut(&manifest.id);
            entry.meta = meta;
            entry.types = types;
        }

        info!(
            "Loaded {} embedded collections ({} issue types)",
            self.collections.len(),
            self.type_count()
        );
        Ok(())
    }

    /// Load imported issue types (reserved namespace, not a collection).
    pub fn load_imports(&mut self, imports_path: &Path) -> Result<()> {
        let imported = loader::load_imported_types(imports_path)?;
        let count = imported.len();

        for (key, issue_type) in imported {
            self.imports.insert(key, issue_type);
        }

        if count > 0 {
            info!("Loaded {} imported issue types", count);
        }
        Ok(())
    }

    /// Register a single imported type under its prefixed key.
    pub fn register_import(&mut self, prefixed_key: &str, issue_type: IssueType) {
        self.imports.insert(prefixed_key.to_string(), issue_type);
    }

    /// Load collections from collections.toml (legacy key-groupings)
    pub fn load_collections(&mut self, path: &Path) -> Result<()> {
        let collections = loader::load_collections(path)?;
        let count = collections.len();

        for (_, collection) in collections {
            self.register_collection(collection)?;
        }

        if count > 0 {
            info!("Loaded {} user-defined collections", count);
        }
        Ok(())
    }

    /// Load issue types and collections from directory structure
    ///
    /// Flattened directory structure (no issues/ subfolder):
    /// ```text
    /// .tickets/templates/
    /// ├── dev_kanban/
    /// │   ├── collection.toml  (optional)
    /// │   ├── TASK.json
    /// │   ├── TASK.md
    /// │   ├── FEAT.json
    /// │   ├── FEAT.md
    /// │   ├── FIX.json
    /// │   └── FIX.md
    /// ├── devops_kanban/
    /// │   └── ...
    /// ```
    ///
    /// Each collection is self-contained with its own issue types.
    pub fn load_from_templates_dir(&mut self, templates_path: &Path) -> Result<()> {
        let mut loaded: Vec<_> = loader::load_collections_from_dir(templates_path)?
            .into_iter()
            .collect();

        if loaded.is_empty() {
            debug!("No collections found in templates directory");
            return Ok(());
        }
        // HashMap iteration order is random; sort for a deterministic
        // load order (the resolve fallback depends on it).
        loaded.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, loaded_collection) in loaded {
            let meta = IssueTypeCollection::new(&name, &loaded_collection.description)
                .with_types(
                    loaded_collection
                        .type_order
                        .iter()
                        .map(std::string::String::as_str),
                )
                .with_manifest_metadata(
                    loaded_collection.workflow_hints,
                    loaded_collection.version,
                    loaded_collection.publisher,
                    loaded_collection.author,
                    loaded_collection.tier,
                );

            let entry = self.entry_mut(&name);
            entry.meta = meta;
            entry.types = loaded_collection.types;
        }

        info!(
            "Loaded {} issue types in {} collections from templates directory",
            self.type_count(),
            self.collections.len()
        );

        Ok(())
    }

    /// Activate a named collection
    pub fn activate_collection(&mut self, name: &str) -> Result<()> {
        if self.collections.contains_key(name) {
            self.active_collection = name.to_string();
            info!("Activated collection: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Collection '{name}' not found")
        }
    }

    /// Activate a custom grouping of types (keys resolve across collections)
    pub fn activate_custom(&mut self, type_keys: &[String]) -> Result<()> {
        // Validate that all keys resolve somewhere
        let mut valid_keys = Vec::new();
        for key in type_keys {
            if self.resolve(None, key).is_some() {
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

        // Create or update the "custom" grouping (owns no types itself)
        let meta = IssueTypeCollection::new("custom", "Custom collection")
            .with_types(valid_keys.iter().map(std::string::String::as_str));
        let entry = self.entry_mut("custom");
        entry.meta = meta;
        entry.types.clear();

        self.active_collection = "custom".to_string();
        info!(
            "Activated custom collection with {} types",
            valid_keys.len()
        );
        Ok(())
    }

    /// Get an issue type by key within a specific collection only.
    pub fn get_in(&self, collection: &str, key: &str) -> Option<&IssueType> {
        self.collections.get(collection)?.types.get(key)
    }

    /// Resolve an issue type by key with collection precedence:
    /// explicit collection → active collection → deterministic search across
    /// collections in load order → kanban imports. Misses at an earlier level
    /// fall through to the next (logged at debug).
    pub fn resolve(&self, collection: Option<&str>, key: &str) -> Option<&IssueType> {
        if let Some(name) = collection {
            if let Some(it) = self.get_in(name, key) {
                return Some(it);
            }
            debug!("key '{key}' not in collection '{name}', falling back");
        }
        if let Some(it) = self.get_in(&self.active_collection, key) {
            return Some(it);
        }
        for name in &self.order {
            if name == &self.active_collection {
                continue;
            }
            if let Some(it) = self.get_in(name, key) {
                debug!("key '{key}' resolved via load-order fallback to '{name}'");
                return Some(it);
            }
        }
        self.imports.get(key)
    }

    /// Get an issue type by key (resolution-order lookup).
    pub fn get(&self, key: &str) -> Option<&IssueType> {
        self.resolve(None, key)
    }

    /// All issue types, deduplicated by key in resolution order: the active
    /// collection's definitions win, then other collections in load order,
    /// then imports.
    pub fn all_types(&self) -> impl Iterator<Item = &IssueType> {
        self.types_with_collections().map(|(_, it)| it)
    }

    /// All issue types with their owning collection, deduplicated by key in
    /// resolution order (imports report the reserved `imports` namespace).
    pub fn types_with_collections(&self) -> impl Iterator<Item = (&str, &IssueType)> {
        let mut seen = std::collections::HashSet::new();
        let mut out: Vec<(&str, &IssueType)> = Vec::new();

        // Active collection first, then the rest in load order.
        let mut names: Vec<&String> = Vec::new();
        if self.collections.contains_key(&self.active_collection) {
            names.push(&self.active_collection);
        }
        names.extend(self.order.iter().filter(|n| **n != self.active_collection));

        for name in names {
            let Some(entry) = self.collections.get(name) else {
                continue;
            };
            // Meta order first, then any stragglers deterministically.
            for key in &entry.meta.types {
                if let Some(it) = entry.types.get(key) {
                    if seen.insert(key.clone()) {
                        out.push((name.as_str(), it));
                    }
                }
            }
            let mut rest: Vec<&String> = entry
                .types
                .keys()
                .filter(|k| !entry.meta.contains(k))
                .collect();
            rest.sort();
            for key in rest {
                if seen.insert(key.clone()) {
                    out.push((name.as_str(), &entry.types[key]));
                }
            }
        }

        let mut import_keys: Vec<&String> = self.imports.keys().collect();
        import_keys.sort();
        for key in import_keys {
            if seen.insert(key.clone()) {
                out.push(("imports", &self.imports[key]));
            }
        }

        out.into_iter()
    }

    /// The types of a single collection, in its display order. Keys the
    /// collection doesn't own (legacy groupings) resolve across the others.
    /// `None` when the collection doesn't exist.
    pub fn types_in(&self, collection: &str) -> Option<Vec<&IssueType>> {
        let entry = self.collections.get(collection)?;
        Some(
            entry
                .meta
                .types
                .iter()
                .filter_map(|key| entry.types.get(key).or_else(|| self.resolve(None, key)))
                .collect(),
        )
    }

    /// The collection that owns `key` under resolution-order lookup
    /// (`imports` for kanban-imported types).
    pub fn collection_of(&self, key: &str) -> Option<&str> {
        if self.get_in(&self.active_collection, key).is_some() {
            return self
                .collections
                .get_key_value(&self.active_collection)
                .map(|(name, _)| name.as_str());
        }
        for name in &self.order {
            if name == &self.active_collection {
                continue;
            }
            if self.get_in(name, key).is_some() {
                return Some(name.as_str());
            }
        }
        self.imports.contains_key(key).then_some("imports")
    }

    /// Get the active collection
    pub fn active_collection(&self) -> Option<&IssueTypeCollection> {
        self.collections
            .get(&self.active_collection)
            .map(|e| &e.meta)
    }

    /// Get the name of the active collection
    pub fn active_collection_name(&self) -> &str {
        &self.active_collection
    }

    /// Get all issue types in the active collection (ordered). Keys the
    /// active collection doesn't own (legacy groupings) resolve across the
    /// other collections.
    pub fn active_types(&self) -> Vec<&IssueType> {
        let Some(entry) = self.collections.get(&self.active_collection) else {
            return vec![];
        };

        entry
            .meta
            .types
            .iter()
            .filter_map(|key| entry.types.get(key).or_else(|| self.resolve(None, key)))
            .collect()
    }

    /// Check if a key exists in the active collection
    pub fn is_active(&self, key: &str) -> bool {
        self.active_collection().is_some_and(|c| c.contains(key))
    }

    /// Get priority index for a type in the active collection
    pub fn priority_index(&self, key: &str) -> usize {
        self.active_collection()
            .map_or(usize::MAX, |c| c.priority_index(key))
    }

    /// Get all available collections in load order
    pub fn all_collections(&self) -> impl Iterator<Item = &IssueTypeCollection> {
        self.order
            .iter()
            .filter_map(|name| self.collections.get(name))
            .map(|entry| &entry.meta)
    }

    /// Get a collection by name
    pub fn get_collection(&self, name: &str) -> Option<&IssueTypeCollection> {
        self.collections.get(name).map(|e| &e.meta)
    }

    /// Register a new issue type into a specific collection.
    pub fn register_in(&mut self, collection: &str, issue_type: IssueType) -> Result<()> {
        issue_type.validate().map_err(|errors| {
            let msgs: Vec<String> = errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            anyhow::anyhow!("Validation errors: {}", msgs.join("; "))
        })?;

        let key = issue_type.key.clone();
        let entry = self.entry_mut(collection);
        entry.types.insert(key.clone(), issue_type);
        if !entry.meta.contains(&key) {
            entry.meta.types.push(key.clone());
        }
        debug!("Registered issue type: {collection}/{key}");
        Ok(())
    }

    /// Register a new issue type into the active collection.
    pub fn register(&mut self, issue_type: IssueType) -> Result<()> {
        let active = self.active_collection.clone();
        self.register_in(&active, issue_type)
    }

    /// Remove an issue type from a specific collection. Returns whether the
    /// key was present.
    pub fn remove_from(&mut self, collection: &str, key: &str) -> bool {
        let Some(entry) = self.collections.get_mut(collection) else {
            return false;
        };
        let removed = entry.types.remove(key).is_some();
        entry.meta.types.retain(|k| k != key);
        removed
    }

    /// Register a new collection grouping (owns no types itself; its keys
    /// resolve across the other collections).
    pub fn register_collection(&mut self, collection: IssueTypeCollection) -> Result<()> {
        let missing: Vec<&String> = collection
            .types
            .iter()
            .filter(|key| self.resolve(None, key).is_none())
            .collect();
        if !missing.is_empty() {
            warn!(
                "Collection '{}' references unknown types: {:?}",
                collection.name, missing
            );
        }

        let name = collection.name.clone();
        let entry = self.entry_mut(&name);
        entry.meta = collection;
        debug!("Registered collection: {}", name);
        Ok(())
    }

    /// Get the number of registered types across all collections and imports
    /// (same key in two collections counts twice).
    pub fn type_count(&self) -> usize {
        self.collections
            .values()
            .map(|e| e.types.len())
            .sum::<usize>()
            + self.imports.len()
    }

    /// Get the number of registered collections
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant(key: &str, name: &str) -> IssueType {
        IssueType::new_imported(
            key.to_string(),
            name.to_string(),
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
        )
    }

    #[test]
    fn test_registry_new() {
        let registry = IssueTypeRegistry::new();
        assert_eq!(registry.type_count(), 0);
        assert_eq!(registry.collection_count(), 0);
    }

    #[test]
    fn test_same_key_coexists_across_collections() {
        let mut registry = IssueTypeRegistry::new();
        registry
            .register_in("alpha", variant("TASK", "Alpha Task"))
            .unwrap();
        registry
            .register_in("beta", variant("TASK", "Beta Task"))
            .unwrap();

        assert_eq!(registry.get_in("alpha", "TASK").unwrap().name, "Alpha Task");
        assert_eq!(registry.get_in("beta", "TASK").unwrap().name, "Beta Task");
        assert_eq!(registry.type_count(), 2);
    }

    #[test]
    fn test_resolve_precedence_explicit_then_active_then_load_order() {
        let mut registry = IssueTypeRegistry::new();
        registry
            .register_in("alpha", variant("TASK", "Alpha Task"))
            .unwrap();
        registry
            .register_in("beta", variant("TASK", "Beta Task"))
            .unwrap();
        registry
            .register_in("beta", variant("ONLY", "Beta Only"))
            .unwrap();

        // Explicit collection wins.
        assert_eq!(
            registry.resolve(Some("beta"), "TASK").unwrap().name,
            "Beta Task"
        );

        // Active collection next.
        registry.activate_collection("alpha").unwrap();
        assert_eq!(registry.resolve(None, "TASK").unwrap().name, "Alpha Task");
        assert_eq!(registry.get("TASK").unwrap().name, "Alpha Task");

        // Deterministic load-order fallback for keys outside the active collection.
        assert_eq!(registry.resolve(None, "ONLY").unwrap().name, "Beta Only");

        // Explicit miss falls back rather than failing.
        assert_eq!(
            registry.resolve(Some("alpha"), "ONLY").unwrap().name,
            "Beta Only"
        );
    }

    #[test]
    fn test_all_types_dedups_by_resolution_order() {
        let mut registry = IssueTypeRegistry::new();
        registry
            .register_in("alpha", variant("TASK", "Alpha Task"))
            .unwrap();
        registry
            .register_in("beta", variant("TASK", "Beta Task"))
            .unwrap();
        registry
            .register_in("beta", variant("ONLY", "Beta Only"))
            .unwrap();
        registry.activate_collection("beta").unwrap();

        let names: Vec<&str> = registry.all_types().map(|t| t.name.as_str()).collect();
        // One entry per key; active collection's definition wins the dedup.
        assert_eq!(names.iter().filter(|n| n.contains("Task")).count(), 1);
        assert!(names.contains(&"Beta Task"));
        assert!(names.contains(&"Beta Only"));
    }

    #[test]
    fn test_imports_are_not_a_collection() {
        let mut registry = IssueTypeRegistry::new();
        registry.register_import("MYPROJ_STORY", variant("STORY", "Imported Story"));

        // Resolvable via the normal lookup path...
        assert!(registry.get("MYPROJ_STORY").is_some());
        // ...but not listed as a collection.
        assert!(registry.get_collection("imports").is_none());
        assert_eq!(registry.collection_count(), 0);
    }

    #[test]
    fn test_load_builtins_registers_embedded_collections() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        // Every embedded collection becomes a namespaced entry.
        for name in [
            "simple",
            "dev_kanban",
            "devops_kanban",
            "operator",
            "ralph_loop",
            "jr_orchestration",
            "elves_overnight",
        ] {
            assert!(registry.get_collection(name).is_some(), "missing {name}");
        }
        // `full` was demoted and presets are retired.
        assert!(registry.get_collection("full").is_none());
        // Underscore keys from the operator collection resolve.
        assert!(registry.get("AGENT_SETUP").is_some());
        assert!(registry.get_in("operator", "PROJECT_INIT").is_some());
    }

    #[test]
    fn test_activate_collection_has_no_preset_fallback() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();
        assert!(registry.activate_collection("full").is_err());
        assert!(registry.activate_collection("dev_kanban").is_ok());
    }

    #[test]
    fn test_collection_of_reports_owner_in_resolution_order() {
        let mut registry = IssueTypeRegistry::new();
        registry
            .register_in("alpha", variant("TASK", "Alpha Task"))
            .unwrap();
        registry
            .register_in("beta", variant("TASK", "Beta Task"))
            .unwrap();
        registry
            .register_in("beta", variant("ONLY", "Beta Only"))
            .unwrap();
        registry.register_import("MYPROJ_STORY", variant("STORY", "Imported"));
        registry.activate_collection("beta").unwrap();

        assert_eq!(registry.collection_of("TASK"), Some("beta"));
        assert_eq!(registry.collection_of("ONLY"), Some("beta"));
        registry.activate_collection("alpha").unwrap();
        assert_eq!(registry.collection_of("TASK"), Some("alpha"));
        assert_eq!(registry.collection_of("MYPROJ_STORY"), Some("imports"));
        assert_eq!(registry.collection_of("NOPE"), None);
    }

    #[test]
    fn test_types_with_collections_carries_owner() {
        let mut registry = IssueTypeRegistry::new();
        registry
            .register_in("alpha", variant("TASK", "Alpha Task"))
            .unwrap();
        registry
            .register_in("beta", variant("ONLY", "Beta Only"))
            .unwrap();
        registry.activate_collection("alpha").unwrap();

        let pairs: Vec<(String, String)> = registry
            .types_with_collections()
            .map(|(c, t)| (c.to_string(), t.key.clone()))
            .collect();
        assert!(pairs.contains(&("alpha".to_string(), "TASK".to_string())));
        assert!(pairs.contains(&("beta".to_string(), "ONLY".to_string())));
    }

    #[test]
    fn test_legacy_group_resolves_types_across_collections() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();

        // collections.toml-style grouping: references keys owned elsewhere.
        let group =
            IssueTypeCollection::new("mygroup", "Legacy grouping").with_types(["TASK", "FEAT"]);
        registry.register_collection(group).unwrap();
        registry.activate_collection("mygroup").unwrap();

        let active = registry.active_types();
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].key, "TASK");
        assert_eq!(active[1].key, "FEAT");
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

        // Default is dev_kanban (3 types: TASK, FEAT, FIX)
        let active = registry.active_types();
        assert_eq!(active.len(), 3);

        // Switch to simple
        registry.activate_collection("simple").unwrap();
        let active = registry.active_types();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].key, "TASK");

        // Switch to devops_kanban
        registry.activate_collection("devops_kanban").unwrap();
        let active = registry.active_types();
        assert_eq!(active.len(), 5);
    }

    #[test]
    fn test_registry_priority() {
        let mut registry = IssueTypeRegistry::new();
        registry.load_builtins().unwrap();
        registry.activate_collection("devops_kanban").unwrap();

        // devops_kanban priority: TASK, FEAT, FIX, SPIKE, INV
        assert_eq!(registry.priority_index("TASK"), 0);
        assert_eq!(registry.priority_index("FEAT"), 1);
        assert_eq!(registry.priority_index("FIX"), 2);
        assert_eq!(registry.priority_index("SPIKE"), 3);
        assert_eq!(registry.priority_index("INV"), 4);
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
            String::new(),
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
