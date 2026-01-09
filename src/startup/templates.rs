//! Template initialization for first-time setup.
//!
//! This module handles copying embedded template files to the filesystem
//! when the templates directory doesn't exist.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

use crate::collections::{get_embedded_collection, EmbeddedCollection, EMBEDDED_COLLECTIONS};

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
    fs::write(collection_path.join("collection.toml"), collection.manifest)?;

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
        .ok_or_else(|| anyhow::anyhow!("Unknown embedded collection: {}", name))?;
    scaffold_collection(templates_path, embedded)
}

/// Scaffold all embedded collections
#[allow(dead_code)]
pub fn scaffold_all_collections(templates_path: &Path) -> Result<()> {
    for collection in EMBEDDED_COLLECTIONS.iter() {
        scaffold_collection(templates_path, collection)?;
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

        // Check collection.toml was created
        assert!(templates_path.join("dev_kanban/collection.toml").exists());
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

        assert!(templates_path.join("simple/collection.toml").exists());
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
        assert!(templates_path.join("backstage_full").exists());

        // backstage_full should have all 8 issuetypes
        assert!(templates_path.join("backstage_full/TASK.json").exists());
        assert!(templates_path.join("backstage_full/FEAT.json").exists());
        assert!(templates_path.join("backstage_full/FIX.json").exists());
        assert!(templates_path.join("backstage_full/SPIKE.json").exists());
        assert!(templates_path.join("backstage_full/INV.json").exists());
        assert!(templates_path.join("backstage_full/ASSESS.json").exists());
        assert!(templates_path.join("backstage_full/SYNC.json").exists());
        assert!(templates_path.join("backstage_full/INIT.json").exists());
    }
}
