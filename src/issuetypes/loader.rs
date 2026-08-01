//! Filesystem loading for issue types and collections

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use super::collection::{CollectionMetadata, CollectionsFile, IssueTypeCollection};
use super::schema::{IssueType, IssueTypeSource};

/// A loaded collection with its issue types
#[derive(Debug, Clone)]
pub struct LoadedCollection {
    /// Collection name (from directory name)
    pub name: String,
    /// Description (from collection.toml if present, or auto-generated)
    pub description: String,
    /// Issue types loaded from this collection directory
    pub types: HashMap<String, IssueType>,
    /// Ordered list of type keys (from collection.toml types field, or derived)
    pub type_order: Vec<String>,
    /// Descriptive metadata from collection.json (empty for legacy collections)
    pub metadata: CollectionMetadata,
}

/// Load imported issue types from the imports subdirectory
///
/// Structure: imports/{provider}/{project}/*.json
pub fn load_imported_types(imports_path: &Path) -> Result<HashMap<String, IssueType>> {
    let mut types = HashMap::new();

    if !imports_path.exists() {
        debug!(
            "Imports directory does not exist: {}",
            imports_path.display()
        );
        return Ok(types);
    }

    // Iterate over provider directories (jira, linear, etc.)
    let providers = fs::read_dir(imports_path).with_context(|| {
        format!(
            "Failed to read imports directory: {}",
            imports_path.display()
        )
    })?;

    for provider_entry in providers {
        let provider_entry = provider_entry?;
        let provider_path = provider_entry.path();

        if !provider_path.is_dir() {
            continue;
        }

        let provider_name = provider_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Iterate over project directories
        let projects = match fs::read_dir(&provider_path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!(
                    "Failed to read provider directory {}: {}",
                    provider_path.display(),
                    e
                );
                continue;
            }
        };

        for project_entry in projects {
            let project_entry = project_entry?;
            let project_path = project_entry.path();

            if !project_path.is_dir() {
                continue;
            }

            let project_name = project_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Load JSON files in this project directory
            let files = match fs::read_dir(&project_path) {
                Ok(entries) => entries,
                Err(e) => {
                    warn!(
                        "Failed to read project directory {}: {}",
                        project_path.display(),
                        e
                    );
                    continue;
                }
            };

            for file_entry in files {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                // Skip non-JSON files and mapping.toml
                if file_path.extension().is_none_or(|e| e != "json") {
                    continue;
                }

                match load_issuetype_file(&file_path) {
                    Ok(mut issue_type) => {
                        // Ensure source is marked correctly
                        issue_type.source = IssueTypeSource::Import {
                            provider: provider_name.to_string(),
                            project: project_name.to_string(),
                        };
                        debug!(
                            "Loaded imported issue type: {} from {}/{}",
                            issue_type.key, provider_name, project_name
                        );

                        // Use a prefixed key to avoid collisions
                        let full_key =
                            format!("{}_{}", project_name.to_uppercase(), issue_type.key);
                        types.insert(full_key, issue_type);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load imported type from {}: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(types)
}

/// Load a single issue type from a JSON file
pub fn load_issuetype_file(path: &Path) -> Result<IssueType> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let issue_type: IssueType = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

    // Validate the issue type
    if let Err(errors) = issue_type.validate() {
        let error_msgs: Vec<String> = errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        anyhow::bail!("Validation errors: {}", error_msgs.join("; "));
    }

    Ok(issue_type)
}

/// Load collections from collections.toml
pub fn load_collections(path: &Path) -> Result<HashMap<String, IssueTypeCollection>> {
    if !path.exists() {
        debug!("Collections file does not exist: {}", path.display());
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read collections file: {}", path.display()))?;

    let file = CollectionsFile::from_toml(&content)
        .with_context(|| format!("Failed to parse collections file: {}", path.display()))?;

    Ok(file.collections)
}

/// Load collections from directory structure
///
/// Structure (flattened - no issues/ subfolder):
/// ```text
/// templates/
/// ├── dev_kanban/
/// │   ├── collection.json  (optional: description, issue_types order)
/// │   ├── TASK.json
/// │   ├── TASK.md
/// │   ├── FEAT.json
/// │   ├── FEAT.md
/// │   ├── FIX.json
/// │   └── FIX.md
/// ├── devops_kanban/
/// │   ├── collection.json
/// │   └── ...
/// ```
///
/// Each subdirectory of `templates_path` is treated as a collection.
/// Issue types (*.json files) are loaded directly from the collection directory.
/// Optional `collection.json` (or legacy `collection.toml`) specifies the
/// description and issue type order.
pub fn load_collections_from_dir(
    templates_path: &Path,
) -> Result<HashMap<String, LoadedCollection>> {
    let mut collections = HashMap::new();

    if !templates_path.exists() {
        debug!(
            "Templates directory does not exist: {}",
            templates_path.display()
        );
        return Ok(collections);
    }

    let entries = fs::read_dir(templates_path).with_context(|| {
        format!(
            "Failed to read templates directory: {}",
            templates_path.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process directories
        if !path.is_dir() {
            continue;
        }

        let collection_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Skip hidden directories
        if collection_name.starts_with('.') {
            continue;
        }

        // Load issue types directly from collection directory (flattened structure)
        let types = load_types_from_collection_dir(&path, &collection_name)?;

        if types.is_empty() {
            debug!(
                "Collection '{}' has no valid issue types, skipping",
                collection_name
            );
            continue;
        }

        // Try to load collection metadata from collection.json / collection.toml
        let meta = load_collection_metadata(&path, &types);

        info!(
            "Loaded collection '{}' with {} issue types",
            collection_name,
            types.len()
        );

        collections.insert(
            collection_name.clone(),
            LoadedCollection {
                name: collection_name,
                description: meta.description,
                types,
                type_order: meta.type_order,
                metadata: meta.metadata,
            },
        );
    }

    Ok(collections)
}

/// Load issue types directly from a collection directory (flattened structure)
///
/// Looks for *.json files in the collection directory (excluding collection.toml).
fn load_types_from_collection_dir(
    collection_path: &Path,
    collection_name: &str,
) -> Result<HashMap<String, IssueType>> {
    let mut types = HashMap::new();

    let entries = fs::read_dir(collection_path).with_context(|| {
        format!(
            "Failed to read collection directory: {}",
            collection_path.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();

        // Only process JSON files
        if file_path.is_dir() || file_path.extension().is_none_or(|e| e != "json") {
            continue;
        }

        // Skip collection.toml's JSON equivalent if someone creates one
        if file_path
            .file_stem()
            .is_some_and(|s| s == "collection" || s == "issuetype_schema")
        {
            continue;
        }

        match load_issuetype_file(&file_path) {
            Ok(mut issue_type) => {
                // Mark source as from filesystem with collection name
                issue_type.source = IssueTypeSource::User;
                debug!(
                    "Loaded issue type '{}' from collection '{}'",
                    issue_type.key, collection_name
                );
                types.insert(issue_type.key.clone(), issue_type);
            }
            Err(e) => {
                warn!(
                    "Failed to load issue type from {}: {}",
                    file_path.display(),
                    e
                );
            }
        }
    }

    Ok(types)
}

/// Metadata for a loaded collection, sourced from `collection.json` (preferred)
/// or the legacy `collection.toml`.
struct LoadedMetadata {
    description: String,
    type_order: Vec<String>,
    /// Descriptive metadata; empty for legacy `collection.toml` collections,
    /// which predate the manifest format.
    metadata: CollectionMetadata,
}

/// Load optional collection metadata from `collection.json` (preferred) or the
/// legacy `collection.toml` (for workspaces scaffolded before the JSON migration).
fn load_collection_metadata(
    collection_path: &Path,
    types: &HashMap<String, IssueType>,
) -> LoadedMetadata {
    // Preferred: collection.json (current format).
    let json_path = collection_path.join("collection.json");
    if json_path.exists() {
        if let Ok(content) = fs::read_to_string(&json_path) {
            if let Ok(manifest) =
                crate::collections::manifest::CollectionManifest::from_json(&content)
            {
                let type_order = if manifest.issue_types.is_empty() {
                    derive_type_order(types)
                } else {
                    manifest.type_keys()
                };
                return LoadedMetadata {
                    metadata: CollectionMetadata::from(&manifest),
                    description: manifest.description,
                    type_order,
                };
            }
        }
    }

    // Back-compat: legacy collection.toml.
    let metadata_path = collection_path.join("collection.toml");
    if metadata_path.exists() {
        if let Ok(content) = fs::read_to_string(&metadata_path) {
            // toml 1.0: parse the document as a Table (parsing into Value expects
            // a bare value and rejects a document).
            if let Ok(toml_value) = toml::from_str::<toml::Table>(&content) {
                let description = toml_value
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Read `types` field from collection.toml for ordering
                let type_order = toml_value
                    .get("types")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_else(|| derive_type_order(types));

                return LoadedMetadata {
                    description,
                    type_order,
                    metadata: CollectionMetadata::default(),
                };
            }
        }
    }

    // Default: auto-generate description and derive order from types
    let collection_name = collection_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let description = format!(
        "Collection '{}' with {} issue types",
        collection_name,
        types.len()
    );

    LoadedMetadata {
        description,
        type_order: derive_type_order(types),
        metadata: CollectionMetadata::default(),
    }
}

/// Derive type order from issue types (alphabetical by key)
fn derive_type_order(types: &HashMap<String, IssueType>) -> Vec<String> {
    let mut keys: Vec<String> = types.keys().cloned().collect();
    keys.sort();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_collection_metadata_reads_collection_json() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();
        fs::write(
            dir.join("collection.json"),
            r#"{
                "schema_version": 1,
                "id": "demo",
                "name": "Demo",
                "description": "Demo collection",
                "issue_types": [
                    {"key": "TASK", "schema_path": "TASK.json"},
                    {"key": "FEAT", "schema_path": "FEAT.json"}
                ]
            }"#,
        )
        .unwrap();
        let meta = load_collection_metadata(dir, &HashMap::new());
        assert_eq!(meta.description, "Demo collection");
        assert_eq!(meta.type_order, vec!["TASK", "FEAT"]);
    }

    #[test]
    fn test_load_collection_metadata_falls_back_to_legacy_toml() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();
        fs::write(
            dir.join("collection.toml"),
            "description = \"Legacy collection\"\ntypes = [\"FIX\", \"INV\"]\n",
        )
        .unwrap();
        let meta = load_collection_metadata(dir, &HashMap::new());
        assert_eq!(meta.description, "Legacy collection");
        assert_eq!(meta.type_order, vec!["FIX", "INV"]);
    }

    #[test]
    fn test_load_collections() {
        let temp_dir = TempDir::new().unwrap();
        let toml = r#"
[collections.test]
name = "test"
description = "Test collection"
types = ["FEAT", "FIX"]
"#;
        let collections_path = temp_dir.path().join("collections.toml");
        fs::write(&collections_path, toml).unwrap();

        let collections = load_collections(&collections_path).unwrap();
        assert_eq!(collections.len(), 1);

        let test = collections.get("test").unwrap();
        assert_eq!(test.types, vec!["FEAT", "FIX"]);
    }

    #[test]
    fn test_load_collections_nonexistent() {
        let collections = load_collections(Path::new("/nonexistent/collections.toml")).unwrap();
        assert!(collections.is_empty());
    }

    #[test]
    fn test_load_imported_types() {
        let temp_dir = TempDir::new().unwrap();
        let imports_path = temp_dir.path().join("imports");
        let jira_proj = imports_path.join("jira").join("MYPROJ");
        fs::create_dir_all(&jira_proj).unwrap();

        let json = r#"{
            "key": "BUG",
            "name": "Bug",
            "description": "A bug",
            "mode": "autonomous",
            "glyph": "B",
            "fields": [
                {"name": "id", "description": "ID", "type": "string", "required": true, "auto": "id"}
            ],
            "steps": [
                {"name": "execute", "outputs": [], "prompt": "Fix it", "allowed_tools": ["*"]}
            ]
        }"#;
        fs::write(jira_proj.join("BUG.json"), json).unwrap();

        let types = load_imported_types(&imports_path).unwrap();
        assert_eq!(types.len(), 1);

        // Key should be prefixed with project name
        assert!(types.contains_key("MYPROJ_BUG"));

        let bug = types.get("MYPROJ_BUG").unwrap();
        assert!(matches!(
            &bug.source,
            IssueTypeSource::Import { provider, project }
            if provider == "jira" && project == "MYPROJ"
        ));
    }
}
