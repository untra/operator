//! Template initialization for first-time setup.
//!
//! This module handles copying embedded template files to the filesystem
//! when the templates directory doesn't exist.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

use crate::collections::manifest::{
    CollectionManifest, CollectionTier, IssueTypeEntry, SCHEMA_VERSION,
};
use crate::collections::{
    get_embedded_collection, EmbeddedCollection, EMBEDDED_COLLECTIONS, EMBEDDED_SCHEMAS,
};
use crate::issuetypes::IssueTypeRegistry;

/// Marker file written into the legacy user-types directory once its
/// contents have been migrated into `.tickets/templates/custom/`.
const MIGRATION_MARKER: &str = "MIGRATED.md";

/// Build an `IssueTypeRegistry` for a workspace using the canonical loading
/// priority, so every surface (REST API, CLI, TUI) resolves the same issue
/// types from the same place:
///
/// 1. Initialize default templates from embedded files if `.tickets/templates/`
///    is missing or empty.
/// 2. Migrate legacy user types (`.tickets/operator/issuetypes/*.json`) into a
///    `custom` collection, once (non-destructive, marker-guarded).
/// 3. Load from `.tickets/templates/` (collection-scoped structure), falling
///    back to embedded builtins if that fails or yields nothing.
/// 4. Load kanban-provider imports and honor a legacy `collections.toml`.
pub fn load_registry(tickets_path: &Path) -> IssueTypeRegistry {
    let mut registry = IssueTypeRegistry::new();
    let templates_path = tickets_path.join("templates");

    // Ensure schema files exist (runs every time, even if templates exist).
    if let Err(e) = ensure_schemas(tickets_path) {
        tracing::warn!("Failed to ensure schema files: {}", e);
    }

    if let Err(e) = init_default_templates(&templates_path) {
        tracing::warn!("Failed to initialize default templates: {}", e);
    }

    if let Err(e) = migrate_legacy_user_types(tickets_path, &templates_path) {
        tracing::warn!("Failed to migrate legacy user types: {}", e);
    }

    load_templates_or_builtins(&mut registry, &templates_path);
    load_legacy_extras(&mut registry, tickets_path);

    registry
}

/// Load the collection-scoped templates directory, falling back to embedded
/// builtins when it fails or yields nothing.
fn load_templates_or_builtins(registry: &mut IssueTypeRegistry, templates_path: &Path) {
    match registry.load_from_templates_dir(templates_path) {
        Ok(()) if registry.type_count() > 0 => {
            info!(
                "Loaded {} issue types from templates directory",
                registry.type_count()
            );
        }
        Ok(()) => {
            info!("Falling back to embedded builtin types");
            if let Err(e) = registry.load_builtins() {
                tracing::warn!("Failed to load builtin issue types: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load from templates directory: {}", e);
            if let Err(e) = registry.load_builtins() {
                tracing::warn!("Failed to load builtin issue types: {}", e);
            }
        }
    }
}

/// Load the legacy extras that live outside the collection store: kanban
/// imports (provider/project-scoped, deliberately not shareable) and the
/// deprecated key-grouping `collections.toml`.
fn load_legacy_extras(registry: &mut IssueTypeRegistry, tickets_path: &Path) {
    let legacy_path = tickets_path.join("operator/issuetypes");

    let imports_path = legacy_path.join("imports");
    if imports_path.is_dir() {
        if let Err(e) = registry.load_imports(&imports_path) {
            tracing::warn!("Failed to load imported issue types: {}", e);
        }
    }

    let collections_toml = legacy_path.join("collections.toml");
    if collections_toml.is_file() {
        if let Err(e) = registry.load_collections(&collections_toml) {
            tracing::warn!("Failed to load collections.toml: {}", e);
        }
    }
}

/// Write a collection's icon under the filename its manifest declares.
///
/// Best-effort and non-fatal by design: the icon is presentational and never
/// executed, so a collection without one still installs cleanly. The bare
/// filename check mirrors `collections::validate::validate_path`, so a hostile
/// manifest cannot escape the collection directory.
fn write_collection_icon(dir: &Path, manifest: &CollectionManifest, icon_svg: Option<&str>) {
    let (Some(icon_path), Some(svg)) = (manifest.icon_path.as_deref(), icon_svg) else {
        return;
    };
    if icon_path.contains('/') || icon_path.contains('\\') || icon_path.starts_with('.') {
        tracing::warn!(
            collection = %manifest.id,
            icon_path,
            "ignoring collection icon with an unsafe path"
        );
        return;
    }
    if let Err(e) = fs::write(dir.join(icon_path), svg) {
        tracing::warn!(collection = %manifest.id, error = %e, "failed to write collection icon");
    }
}

/// Write a fetched (or synthesized) collection into its collection-scoped
/// directory: `templates/<id>/collection.json` + `<KEY>.json`/`<KEY>.md`.
///
/// `files` entries are `(key, schema_json, optional template_md)` — the shape
/// hosted fetches produce.
pub fn write_fetched_collection(
    templates_path: &Path,
    manifest: &CollectionManifest,
    files: &[(String, String, Option<String>)],
    icon_svg: Option<&str>,
) -> Result<()> {
    let dir = templates_path.join(&manifest.id);
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create collection directory: {}", dir.display()))?;

    fs::write(
        dir.join("collection.json"),
        format!("{}\n", manifest.to_json()?),
    )?;

    write_collection_icon(&dir, manifest, icon_svg);

    for (key, schema_json, template_md) in files {
        fs::write(dir.join(format!("{key}.json")), schema_json)?;
        if let Some(md) = template_md {
            fs::write(dir.join(format!("{key}.md")), md)?;
        }
    }

    info!(
        "Wrote collection '{}' with {} issue types",
        manifest.id,
        files.len()
    );
    Ok(())
}

/// One-time, non-destructive migration of legacy flat user types
/// (`.tickets/operator/issuetypes/*.json`) into a collection-scoped
/// `templates/custom/` collection.
///
/// Skipped when the marker file exists or `templates/custom/` is already
/// present. Originals are kept; a `MIGRATED.md` marker records the move.
fn migrate_legacy_user_types(tickets_path: &Path, templates_path: &Path) -> Result<()> {
    let legacy = tickets_path.join("operator/issuetypes");
    if !legacy.is_dir() || legacy.join(MIGRATION_MARKER).exists() {
        return Ok(());
    }
    let custom_dir = templates_path.join("custom");
    if custom_dir.exists() {
        // Never overwrite an existing custom collection.
        return Ok(());
    }

    let mut files: Vec<(String, String, Option<String>)> = Vec::new();
    for entry in fs::read_dir(&legacy)? {
        let path = entry?.path();
        if path.is_dir() || path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        if path
            .file_stem()
            .is_some_and(|s| s == "collection" || s == "issuetype_schema")
        {
            continue;
        }
        match crate::issuetypes::loader::load_issuetype_file(&path) {
            Ok(issue_type) => {
                let schema_json = fs::read_to_string(&path)?;
                let template_md = fs::read_to_string(path.with_extension("md")).ok();
                files.push((issue_type.key, schema_json, template_md));
            }
            Err(e) => {
                tracing::warn!(
                    "Skipping legacy user type {} during migration: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    if files.is_empty() {
        return Ok(());
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let manifest = CollectionManifest {
        schema_version: SCHEMA_VERSION,
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "User-defined issue types migrated from .tickets/operator/issuetypes/"
            .to_string(),
        version: "1.0.0".to_string(),
        publisher: None,
        author: None,
        url: None,
        license: None,
        tags: vec!["custom".to_string()],
        compatibility: None,
        tier: CollectionTier::default(),
        icon_path: None,
        created: None,
        updated: None,
        kanban_defaults: None,
        issue_types: files
            .iter()
            .map(|(key, _, md)| IssueTypeEntry {
                key: key.clone(),
                schema_path: format!("{key}.json"),
                schema_checksum: String::new(),
                template_path: md.as_ref().map(|_| format!("{key}.md")),
                template_checksum: None,
            })
            .collect(),
        workflow_hints: None,
        default_selected: files.iter().map(|(key, _, _)| key.clone()).collect(),
        checksum: None,
    };

    write_fetched_collection(templates_path, &manifest, &files, None)?;

    fs::write(
        legacy.join(MIGRATION_MARKER),
        "# Migrated\n\nThe issue types in this directory were copied into\n\
         `.tickets/templates/custom/` (the collection-scoped store that all\n\
         operator surfaces read). Edit them there; these originals are kept\n\
         for reference and are no longer loaded. Delete this file to re-run\n\
         the migration.\n",
    )?;

    info!(
        "Migrated {} legacy user type(s) into templates/custom/",
        files.len()
    );
    Ok(())
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
    let has_entries = templates_path.exists()
        && fs::read_dir(templates_path).is_ok_and(|mut d| d.next().is_some());
    if has_entries {
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

    // A scaffolded collection should be byte-identical to its hosted counterpart,
    // icon included.
    if let Ok(manifest) = collection.manifest_parsed() {
        write_collection_icon(&collection_path, &manifest, Some(collection.icon_svg));
    }

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
    use crate::collections::manifest::{CollectionManifest, IssueTypeEntry};
    use std::path::PathBuf;
    use tempfile::TempDir;

    const CUSTOM_TYPE_JSON: &str = r#"{
        "key": "GAST",
        "name": "Gastown",
        "description": "A custom user type",
        "mode": "autonomous",
        "glyph": "g",
        "fields": [],
        "steps": [{"name": "execute", "outputs": ["report"], "prompt": "Do it."}]
    }"#;

    fn fetched_manifest(id: &str) -> CollectionManifest {
        CollectionManifest::from_json(&format!(
            r#"{{
                "schema_version": 1,
                "id": "{id}",
                "name": "Fetched",
                "description": "A fetched collection",
                "issue_types": [
                    {{"key": "GAST", "schema_path": "GAST.json", "template_path": "GAST.md"}}
                ]
            }}"#
        ))
        .unwrap()
    }

    fn legacy_dir(tickets_path: &std::path::Path) -> PathBuf {
        tickets_path.join("operator/issuetypes")
    }

    #[test]
    fn test_write_fetched_collection_round_trips_through_loader() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        let manifest = fetched_manifest("fetched_loop");
        let files = vec![(
            "GAST".to_string(),
            CUSTOM_TYPE_JSON.to_string(),
            Some("# Gastown: {{ summary }}\n".to_string()),
        )];
        write_fetched_collection(&templates_path, &manifest, &files, None).unwrap();

        assert!(templates_path.join("fetched_loop/collection.json").exists());
        assert!(templates_path.join("fetched_loop/GAST.json").exists());
        assert!(templates_path.join("fetched_loop/GAST.md").exists());

        // The written layout must load through the canonical loader.
        let mut registry = IssueTypeRegistry::new();
        registry.load_from_templates_dir(&templates_path).unwrap();
        assert!(registry.get("GAST").is_some());
        assert!(registry.get_collection("fetched_loop").is_some());
    }

    #[test]
    fn test_write_fetched_collection_without_template_md() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        let mut manifest = fetched_manifest("fetched_loop");
        manifest.issue_types = vec![IssueTypeEntry {
            key: "GAST".to_string(),
            schema_path: "GAST.json".to_string(),
            schema_checksum: String::new(),
            template_path: None,
            template_checksum: None,
        }];
        let files = vec![("GAST".to_string(), CUSTOM_TYPE_JSON.to_string(), None)];
        write_fetched_collection(&templates_path, &manifest, &files, None).unwrap();

        assert!(templates_path.join("fetched_loop/GAST.json").exists());
        assert!(!templates_path.join("fetched_loop/GAST.md").exists());
    }

    #[test]
    fn test_load_registry_migrates_legacy_user_types() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path();
        let legacy = legacy_dir(tickets_path);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("gast.json"), CUSTOM_TYPE_JSON).unwrap();
        std::fs::write(legacy.join("gast.md"), "# Gastown\n").unwrap();

        let registry = load_registry(tickets_path);

        // Migrated into a collection-scoped `custom` collection, canonical names.
        let custom = tickets_path.join("templates/custom");
        assert!(custom.join("collection.json").exists());
        assert!(custom.join("GAST.json").exists());
        assert!(custom.join("GAST.md").exists());
        // Non-destructive: originals stay, marker written.
        assert!(legacy.join("gast.json").exists());
        assert!(legacy.join("MIGRATED.md").exists());
        // And the type is served by the unified registry.
        assert!(registry.get("GAST").is_some());
        assert!(registry.get_collection("custom").is_some());
    }

    #[test]
    fn test_load_registry_migration_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path();
        let legacy = legacy_dir(tickets_path);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("gast.json"), CUSTOM_TYPE_JSON).unwrap();

        let _ = load_registry(tickets_path);
        let first =
            std::fs::read_to_string(tickets_path.join("templates/custom/GAST.json")).unwrap();

        // Second run: no error, no re-write.
        let registry = load_registry(tickets_path);
        assert!(registry.get("GAST").is_some());

        // Marker blocks re-migration even if the migrated copy is deleted.
        std::fs::remove_dir_all(tickets_path.join("templates/custom")).unwrap();
        let registry = load_registry(tickets_path);
        assert!(!tickets_path.join("templates/custom").exists());
        assert!(registry.get("GAST").is_none());
        let _ = first;
    }

    #[test]
    fn test_migration_skips_invalid_legacy_files() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path();
        let legacy = legacy_dir(tickets_path);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("gast.json"), CUSTOM_TYPE_JSON).unwrap();
        std::fs::write(legacy.join("broken.json"), "{not valid json").unwrap();

        let registry = load_registry(tickets_path);

        // Valid type migrated; broken file skipped without aborting.
        assert!(tickets_path.join("templates/custom/GAST.json").exists());
        assert!(registry.get("GAST").is_some());
        assert!(legacy.join(MIGRATION_MARKER).exists());
    }

    #[test]
    fn test_load_registry_skips_migration_when_custom_exists() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path();
        let legacy = legacy_dir(tickets_path);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("gast.json"), CUSTOM_TYPE_JSON).unwrap();

        // A pre-existing custom collection must not be overwritten.
        let custom = tickets_path.join("templates/custom");
        std::fs::create_dir_all(&custom).unwrap();
        std::fs::write(
            custom.join("collection.json"),
            r#"{"schema_version": 1, "id": "custom", "name": "Custom", "issue_types": []}"#,
        )
        .unwrap();

        let _ = load_registry(tickets_path);
        assert!(!custom.join("GAST.json").exists());
    }

    #[test]
    fn test_load_registry_loads_kanban_imports() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path();
        let imports = legacy_dir(tickets_path).join("imports/jira/myproj");
        std::fs::create_dir_all(&imports).unwrap();
        std::fs::write(imports.join("GAST.json"), CUSTOM_TYPE_JSON).unwrap();

        let registry = load_registry(tickets_path);
        // Imports register under {PROJECT}_{KEY} and stay out of collections.
        assert!(registry.get("MYPROJ_GAST").is_some());
    }

    #[test]
    fn test_load_registry_honors_legacy_collections_toml() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path();
        let legacy = legacy_dir(tickets_path);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(
            legacy.join("collections.toml"),
            r#"
[collections.mygroup]
name = "mygroup"
description = "Legacy grouping"
types = ["TASK", "FEAT"]
"#,
        )
        .unwrap();

        let registry = load_registry(tickets_path);
        assert!(registry.get_collection("mygroup").is_some());
    }

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

        // Every embedded collection should exist
        assert!(templates_path.join("simple").exists());
        assert!(templates_path.join("dev_kanban").exists());
        assert!(templates_path.join("devops_kanban").exists());
        assert!(templates_path.join("operator").exists());
        assert!(templates_path.join("ralph_loop").exists());
        assert!(templates_path.join("jr_orchestration").exists());
        assert!(templates_path.join("elves_overnight").exists());
        assert!(templates_path.join("coder").exists());

        // `full` was demoted from the embedded set and must not scaffold
        assert!(!templates_path.join("full").exists());

        // Underscore-keyed operator types scaffold correctly
        assert!(templates_path.join("operator/AGENT_SETUP.json").exists());
        assert!(templates_path.join("operator/PROJECT_INIT.json").exists());
    }

    #[test]
    fn test_write_fetched_collection_persists_the_icon() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        let mut manifest = fetched_manifest("fetched_loop");
        manifest.icon_path = Some("icon.svg".to_string());
        let files = vec![(
            "GAST".to_string(),
            CUSTOM_TYPE_JSON.to_string(),
            Some("# Gastown\n".to_string()),
        )];
        let svg = r#"<svg role="img" viewBox="0 0 24 24"><title>Fetched</title><path d="M0 0h24v24H0z"/></svg>"#;
        write_fetched_collection(&templates_path, &manifest, &files, Some(svg)).unwrap();

        let written = templates_path.join("fetched_loop/icon.svg");
        assert_eq!(std::fs::read_to_string(written).unwrap(), svg);
    }

    #[test]
    fn test_write_fetched_collection_survives_a_missing_or_unsafe_icon() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");
        let files = vec![("GAST".to_string(), CUSTOM_TYPE_JSON.to_string(), None)];

        // No icon at all: the collection still installs.
        let manifest = fetched_manifest("no_icon");
        write_fetched_collection(&templates_path, &manifest, &files, None).unwrap();
        assert!(templates_path.join("no_icon/GAST.json").exists());

        // A traversal attempt is dropped, not written outside the collection.
        let mut hostile = fetched_manifest("hostile");
        hostile.icon_path = Some("../escaped.svg".to_string());
        write_fetched_collection(&templates_path, &hostile, &files, Some("<svg/>")).unwrap();
        assert!(templates_path.join("hostile/GAST.json").exists());
        assert!(!templates_path.join("escaped.svg").exists());
    }

    #[test]
    fn test_scaffold_writes_collection_icon_byte_identical_to_embedded() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        scaffold_all_collections(&templates_path).unwrap();

        for collection in EMBEDDED_COLLECTIONS {
            let icon_path = collection
                .manifest_parsed()
                .unwrap()
                .icon_path
                .unwrap_or_else(|| panic!("{} declares no icon_path", collection.name));
            let written = templates_path.join(collection.name).join(&icon_path);
            assert!(
                written.is_file(),
                "{} did not scaffold {icon_path}",
                collection.name
            );
            assert_eq!(
                std::fs::read_to_string(&written).unwrap(),
                collection.icon_svg,
                "{} scaffolded an icon that differs from the embedded bytes",
                collection.name
            );
        }
    }
}
