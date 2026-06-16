//! Template initialization for first-time setup.
//!
//! This module handles copying embedded template files to the filesystem
//! when the templates directory doesn't exist.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

use crate::collections::{
    get_embedded_collection, EmbeddedCollection, EMBEDDED_COLLECTIONS, EMBEDDED_SCHEMAS,
};
use crate::issuetypes::IssueTypeRegistry;

/// Build an `IssueTypeRegistry` for a workspace using the canonical loading
/// priority, so every surface (REST API, CLI, TUI) resolves the same issue
/// types from the same place:
///
/// 1. Load from `.tickets/templates/` (collection-scoped structure).
/// 2. If empty, initialize default templates from embedded files, then reload.
/// 3. Fall back to embedded builtins if filesystem loading fails or yields none.
pub fn load_registry(tickets_path: &Path) -> IssueTypeRegistry {
    let mut registry = IssueTypeRegistry::new();
    let templates_path = tickets_path.join("templates");

    // Ensure schema files exist (runs every time, even if templates exist).
    if let Err(e) = ensure_schemas(tickets_path) {
        tracing::warn!("Failed to ensure schema files: {}", e);
    }

    match registry.load_from_templates_dir(&templates_path) {
        Ok(()) if registry.type_count() > 0 => {
            info!(
                "Loaded {} issue types from templates directory",
                registry.type_count()
            );
        }
        Ok(()) => {
            // Templates directory empty or absent — initialize defaults.
            info!("Templates directory empty, initializing defaults...");
            if let Err(e) = init_default_templates(&templates_path) {
                tracing::warn!("Failed to initialize default templates: {}", e);
            } else if let Err(e) = registry.load_from_templates_dir(&templates_path) {
                tracing::warn!("Failed to load initialized templates: {}", e);
            }

            if registry.type_count() == 0 {
                info!("Falling back to embedded builtin types");
                if let Err(e) = registry.load_builtins() {
                    tracing::warn!("Failed to load builtin issue types: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load from templates directory: {}", e);
            if let Err(e) = registry.load_builtins() {
                tracing::warn!("Failed to load builtin issue types: {}", e);
            }
        }
    }

    registry
}

/// Initialize the templates directory with default collections
///
/// Creates the directory structure:
/// ```text
/// .tickets/templates/
/// ├── dev_kanban/
/// │   ├── collection.toml
/// │   ├── TASK.json
/// │   ├── TASK.md
/// │   ├── FEAT.json
/// │   ├── FEAT.md
/// │   ├── FIX.json
/// │   └── FIX.md
/// └── ...
/// ```
pub fn init_default_templates(templates_path: &Path) -> Result<()> {
    if templates_path.exists() {
        info!(
            "Templates directory already exists: {}",
            templates_path.display()
        );
        return Ok(());
    }

    info!(
        "Initializing default templates at {}",
        templates_path.display()
    );

    // Default collections to scaffold
    let default_collections = ["dev_kanban", "devops_kanban", "simple", "operator"];

    for collection_name in default_collections {
        if let Some(embedded) = get_embedded_collection(collection_name) {
            scaffold_collection(templates_path, embedded)?;
        }
    }

    Ok(())
}

/// Scaffold a single embedded collection to the filesystem
pub fn scaffold_collection(templates_path: &Path, collection: &EmbeddedCollection) -> Result<()> {
    let collection_path = templates_path.join(collection.name);

    // Create collection directory
    fs::create_dir_all(&collection_path).with_context(|| {
        format!(
            "Failed to create collection directory: {}",
            collection_path.display()
        )
    })?;

    // Write collection manifest
    fs::write(collection_path.join("collection.json"), collection.manifest)?;

    // Write issuetype JSON and markdown files
    for issuetype in collection.issuetypes {
        let json_filename = format!("{}.json", issuetype.key);
        let md_filename = format!("{}.md", issuetype.key);

        fs::write(collection_path.join(&json_filename), issuetype.schema_json)?;
        fs::write(collection_path.join(&md_filename), issuetype.template_md)?;

        info!("Created template: {}/{}", collection.name, json_filename);
    }

    info!(
        "Scaffolded collection '{}' with {} issue types",
        collection.name,
        collection.issuetypes.len()
    );

    Ok(())
}

/// Scaffold a specific collection by name
#[allow(dead_code)]
pub fn scaffold_collection_by_name(templates_path: &Path, name: &str) -> Result<()> {
    let embedded = get_embedded_collection(name)
        .ok_or_else(|| anyhow::anyhow!("Unknown embedded collection: {name}"))?;
    scaffold_collection(templates_path, embedded)
}

/// Scaffold all embedded collections
#[allow(dead_code)]
pub fn scaffold_all_collections(templates_path: &Path) -> Result<()> {
    for collection in EMBEDDED_COLLECTIONS {
        scaffold_collection(templates_path, collection)?;
    }
    Ok(())
}

/// Ensure schema files exist in .tickets/schemas/
///
/// This function runs on every startup (not just first-time init) to ensure
/// schema files are available for issue types that need structured output.
/// Schema files are only written if they don't already exist.
pub fn ensure_schemas(tickets_path: &Path) -> Result<()> {
    let schemas_path = tickets_path.join("schemas");
    fs::create_dir_all(&schemas_path).with_context(|| {
        format!(
            "Failed to create schemas directory: {}",
            schemas_path.display()
        )
    })?;

    for schema in EMBEDDED_SCHEMAS {
        let schema_file = schemas_path.join(schema.name);
        // Only write if missing (don't overwrite user modifications)
        if !schema_file.exists() {
            fs::write(&schema_file, schema.content).with_context(|| {
                format!("Failed to write schema file: {}", schema_file.display())
            })?;
            info!("Created schema: {}", schema.name);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_default_templates() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        init_default_templates(&templates_path).unwrap();

        // Check that directories were created (flattened structure)
        assert!(templates_path.exists());
        assert!(templates_path.join("dev_kanban").exists());
        assert!(templates_path.join("devops_kanban").exists());
        assert!(templates_path.join("simple").exists());
        assert!(templates_path.join("operator").exists());

        // Check that template files were created (no issues/ subfolder)
        assert!(templates_path.join("dev_kanban/TASK.json").exists());
        assert!(templates_path.join("dev_kanban/TASK.md").exists());
        assert!(templates_path.join("dev_kanban/FEAT.json").exists());
        assert!(templates_path.join("dev_kanban/FEAT.md").exists());
        assert!(templates_path.join("dev_kanban/FIX.json").exists());
        assert!(templates_path.join("dev_kanban/FIX.md").exists());
        assert!(templates_path.join("devops_kanban/SPIKE.json").exists());
        assert!(templates_path.join("devops_kanban/INV.json").exists());

        // Check collection.json was created
        assert!(templates_path.join("dev_kanban/collection.json").exists());
    }

    #[test]
    fn test_init_skips_existing() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        // Create the directory first
        fs::create_dir_all(&templates_path).unwrap();
        fs::write(templates_path.join("marker.txt"), "existing").unwrap();

        // This should not fail and should not overwrite
        init_default_templates(&templates_path).unwrap();

        // Marker file should still exist
        assert!(templates_path.join("marker.txt").exists());
    }

    #[test]
    fn test_scaffold_collection_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        scaffold_collection_by_name(&templates_path, "simple").unwrap();

        assert!(templates_path.join("simple/collection.json").exists());
        assert!(templates_path.join("simple/TASK.json").exists());
        assert!(templates_path.join("simple/TASK.md").exists());
    }

    #[test]
    fn test_scaffold_unknown_collection() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        let result = scaffold_collection_by_name(&templates_path, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_scaffold_all_collections() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        scaffold_all_collections(&templates_path).unwrap();

        // All 5 collections should exist
        assert!(templates_path.join("simple").exists());
        assert!(templates_path.join("dev_kanban").exists());
        assert!(templates_path.join("devops_kanban").exists());
        assert!(templates_path.join("operator").exists());
        assert!(templates_path.join("full").exists());

        // full should have all 8 issuetypes
        assert!(templates_path.join("full/TASK.json").exists());
        assert!(templates_path.join("full/FEAT.json").exists());
        assert!(templates_path.join("full/FIX.json").exists());
        assert!(templates_path.join("full/SPIKE.json").exists());
        assert!(templates_path.join("full/INV.json").exists());
        assert!(templates_path.join("full/ASSESS.json").exists());
        assert!(templates_path.join("full/SYNC.json").exists());
        assert!(templates_path.join("full/INIT.json").exists());
    }
}
