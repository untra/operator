//! Filesystem loading for issue types and collections

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use super::collection::{CollectionsFile, IssueTypeCollection};
use super::schema::{IssueType, IssueTypeSource};
use crate::templates::schema::TemplateSchema;
use crate::templates::TemplateType;

/// Load all built-in issue types
pub fn load_builtins() -> Result<HashMap<String, IssueType>> {
    let mut types = HashMap::new();

    for template_type in TemplateType::all() {
        let schema_json = template_type.schema();
        match TemplateSchema::from_json(schema_json) {
            Ok(schema) => {
                let issue_type = template_schema_to_issuetype(schema, IssueTypeSource::Builtin);
                debug!("Loaded builtin issue type: {}", issue_type.key);
                types.insert(issue_type.key.clone(), issue_type);
            }
            Err(e) => {
                warn!(
                    "Failed to parse builtin template {}: {}",
                    template_type.as_str(),
                    e
                );
            }
        }
    }

    Ok(types)
}

/// Convert a TemplateSchema to an IssueType
fn template_schema_to_issuetype(schema: TemplateSchema, source: IssueTypeSource) -> IssueType {
    IssueType {
        key: schema.key,
        name: schema.name,
        description: schema.description,
        mode: schema.mode,
        glyph: schema.glyph,
        color: schema.color,
        branch_prefix: schema.branch_prefix,
        project_required: schema.project_required,
        fields: schema.fields,
        steps: schema.steps,
        agent_prompt: schema.agent_prompt,
        source,
        external_id: None,
    }
}

/// Load user-defined issue types from a directory
///
/// Scans for *.json files in the directory and attempts to parse each as an IssueType.
/// Invalid files are logged as warnings and skipped.
pub fn load_user_types(path: &Path) -> Result<HashMap<String, IssueType>> {
    let mut types = HashMap::new();

    if !path.exists() {
        debug!(
            "User issuetypes directory does not exist: {}",
            path.display()
        );
        return Ok(types);
    }

    let entries = fs::read_dir(path)
        .with_context(|| format!("Failed to read issuetypes directory: {}", path.display()))?;

    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();

        // Skip directories and non-JSON files
        if file_path.is_dir() || file_path.extension().is_none_or(|e| e != "json") {
            continue;
        }

        // Skip imports directory
        if file_path
            .file_stem()
            .is_some_and(|s| s == "imports" || s == "collections")
        {
            continue;
        }

        match load_issuetype_file(&file_path) {
            Ok(mut issue_type) => {
                // Ensure source is marked as User
                issue_type.source = IssueTypeSource::User;
                debug!(
                    "Loaded user issue type: {} from {}",
                    issue_type.key,
                    file_path.display()
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
        let error_msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
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

/// Validate a collection against available types, returning types that are missing
///
/// Returns a tuple of (valid_types, missing_types)
pub fn validate_collection_types(
    collection: &IssueTypeCollection,
    available_types: &HashMap<String, IssueType>,
) -> (Vec<String>, Vec<String>) {
    let mut valid = Vec::new();
    let mut missing = Vec::new();

    for type_key in &collection.types {
        if available_types.contains_key(type_key) {
            valid.push(type_key.clone());
        } else {
            missing.push(type_key.clone());
        }
    }

    (valid, missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_builtins() {
        let types = load_builtins().unwrap();
        assert!(types.contains_key("FEAT"));
        assert!(types.contains_key("FIX"));
        assert!(types.contains_key("TASK"));
        assert!(types.contains_key("SPIKE"));
        assert!(types.contains_key("INV"));

        // Verify source is set correctly
        let feat = types.get("FEAT").unwrap();
        assert_eq!(feat.source, IssueTypeSource::Builtin);
    }

    #[test]
    fn test_load_user_types_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let types = load_user_types(temp_dir.path()).unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_load_user_types_nonexistent_dir() {
        let types = load_user_types(Path::new("/nonexistent/path")).unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_load_user_types_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let json = r#"{
            "key": "STORY",
            "name": "User Story",
            "description": "A user story",
            "mode": "autonomous",
            "glyph": "S",
            "fields": [
                {"name": "id", "description": "ID", "type": "string", "required": true, "auto": "id"}
            ],
            "steps": [
                {"name": "execute", "outputs": [], "prompt": "Execute", "allowed_tools": ["*"]}
            ]
        }"#;
        fs::write(temp_dir.path().join("STORY.json"), json).unwrap();

        let types = load_user_types(temp_dir.path()).unwrap();
        assert_eq!(types.len(), 1);
        assert!(types.contains_key("STORY"));

        let story = types.get("STORY").unwrap();
        assert_eq!(story.name, "User Story");
        assert_eq!(story.source, IssueTypeSource::User);
    }

    #[test]
    fn test_load_user_types_skips_invalid() {
        let temp_dir = TempDir::new().unwrap();

        // Valid file
        let valid_json = r#"{
            "key": "VALID",
            "name": "Valid",
            "description": "Valid type",
            "mode": "autonomous",
            "glyph": "V",
            "fields": [
                {"name": "id", "description": "ID", "type": "string", "required": true, "auto": "id"}
            ],
            "steps": [
                {"name": "execute", "outputs": [], "prompt": "Execute", "allowed_tools": ["*"]}
            ]
        }"#;
        fs::write(temp_dir.path().join("VALID.json"), valid_json).unwrap();

        // Invalid file (lowercase key)
        let invalid_json = r#"{
            "key": "invalid",
            "name": "Invalid",
            "description": "Invalid type",
            "mode": "autonomous",
            "glyph": "I",
            "fields": [],
            "steps": [
                {"name": "execute", "outputs": [], "prompt": "Execute", "allowed_tools": ["*"]}
            ]
        }"#;
        fs::write(temp_dir.path().join("invalid.json"), invalid_json).unwrap();

        let types = load_user_types(temp_dir.path()).unwrap();
        assert_eq!(types.len(), 1);
        assert!(types.contains_key("VALID"));
    }

    #[test]
    fn test_load_collections() {
        let temp_dir = TempDir::new().unwrap();
        let toml = r#"
[collections.test]
name = "test"
description = "Test collection"
types = ["FEAT", "FIX"]
priority_order = ["FIX", "FEAT"]
"#;
        let collections_path = temp_dir.path().join("collections.toml");
        fs::write(&collections_path, toml).unwrap();

        let collections = load_collections(&collections_path).unwrap();
        assert_eq!(collections.len(), 1);

        let test = collections.get("test").unwrap();
        assert_eq!(test.types, vec!["FEAT", "FIX"]);
        assert_eq!(test.priority_order, vec!["FIX", "FEAT"]);
    }

    #[test]
    fn test_load_collections_nonexistent() {
        let collections = load_collections(Path::new("/nonexistent/collections.toml")).unwrap();
        assert!(collections.is_empty());
    }

    #[test]
    fn test_validate_collection_types() {
        let mut available = HashMap::new();
        available.insert(
            "FEAT".to_string(),
            IssueType::new_imported(
                "FEAT".to_string(),
                "Feature".to_string(),
                "".to_string(),
                "builtin".to_string(),
                "".to_string(),
                None,
            ),
        );
        available.insert(
            "FIX".to_string(),
            IssueType::new_imported(
                "FIX".to_string(),
                "Fix".to_string(),
                "".to_string(),
                "builtin".to_string(),
                "".to_string(),
                None,
            ),
        );

        let collection =
            IssueTypeCollection::new("test", "").with_types(["FEAT", "STORY", "FIX", "MISSING"]);

        let (valid, missing) = validate_collection_types(&collection, &available);
        assert_eq!(valid, vec!["FEAT", "FIX"]);
        assert_eq!(missing, vec!["STORY", "MISSING"]);
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
