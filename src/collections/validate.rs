//! Validation for shareable collection directories.
//!
//! Community collections (`collections/community/<id>/`) are validated at
//! docs-generation time so a broken submission fails a PR's CI, never a
//! user's install. Rules are intentionally stricter than what the loader
//! tolerates for local collections.

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::manifest::{CollectionManifest, CollectionTier, SCHEMA_VERSION};
use crate::issuetypes::loader::load_issuetype_file;

/// Collection ids: lowercase alphanumeric + underscore, 3-64 chars.
fn id_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9_]{3,64}$").expect("valid regex"))
}

/// Issue type keys: uppercase start, then uppercase/digit/underscore, 2-16 chars.
fn key_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9_]{1,15}$").expect("valid regex"))
}

/// Publication dates: ISO-8601 calendar dates, `YYYY-MM-DD`.
fn date_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("valid regex"))
}

/// Validate a manifest against the shareable-collection rules.
///
/// `dir_name` is the name of the directory holding `collection.json`; the
/// manifest `id` must match it so hosted paths stay consistent.
pub fn validate_manifest(manifest: &CollectionManifest, dir_name: &str) -> Result<()> {
    if !id_regex().is_match(&manifest.id) {
        bail!(
            "invalid collection id '{}': must match ^[a-z0-9_]{{3,64}}$",
            manifest.id
        );
    }
    if manifest.id != dir_name {
        bail!(
            "collection id '{}' does not match its directory name '{dir_name}'",
            manifest.id
        );
    }
    if manifest.schema_version != SCHEMA_VERSION {
        bail!(
            "unsupported schema_version {} (expected {SCHEMA_VERSION})",
            manifest.schema_version
        );
    }
    if manifest.name.trim().is_empty() {
        bail!("collection name must not be empty");
    }
    if manifest.description.trim().is_empty() {
        bail!("collection description must not be empty");
    }
    if manifest.issue_types.is_empty() || manifest.issue_types.len() > 32 {
        bail!(
            "collection must contain between 1 and 32 issue types (found {})",
            manifest.issue_types.len()
        );
    }

    let mut seen = std::collections::HashSet::new();
    for entry in &manifest.issue_types {
        if !key_regex().is_match(&entry.key) {
            bail!(
                "invalid issue type key '{}': must match ^[A-Z][A-Z0-9_]{{1,15}}$",
                entry.key
            );
        }
        if !seen.insert(entry.key.as_str()) {
            bail!("duplicate issue type key '{}'", entry.key);
        }
        validate_path(&entry.schema_path)?;
        if let Some(template) = &entry.template_path {
            validate_path(template)?;
        }
    }

    if let Some(icon) = &manifest.icon_path {
        validate_path(icon)?;
        // Extensions are matched exactly: hosted paths are served verbatim, so
        // `icon.SVG` would 404 against the manifest's own reference.
        if std::path::Path::new(icon)
            .extension()
            .is_none_or(|e| e != "svg")
        {
            bail!("icon_path '{icon}' must be an .svg file");
        }
    }
    for (value, field) in [
        (&manifest.created, "created"),
        (&manifest.updated, "updated"),
    ] {
        if let Some(date) = value {
            if !date_regex().is_match(date) {
                bail!("invalid {field} date '{date}': must be YYYY-MM-DD");
            }
        }
    }

    if manifest.tier == CollectionTier::Community {
        for (value, field) in [
            (&manifest.license, "license"),
            (&manifest.author, "author"),
            (&manifest.url, "url"),
            // Every hosted card needs an icon; community submissions are
            // hosted-only, so this is where the requirement bites.
            (&manifest.icon_path, "icon_path"),
        ] {
            if value.as_deref().is_none_or(|v| v.trim().is_empty()) {
                bail!("community collections require a {field}");
            }
        }
    }

    Ok(())
}

/// File references must be bare filenames next to the manifest — no
/// separators or traversal, matching the flat hosted/embedded layout.
fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.contains('/')
        || path.contains('\\')
        || path.contains("..")
        || path.starts_with('.')
    {
        bail!("unsafe path '{path}': must be a bare relative filename");
    }
    Ok(())
}

/// Parse and validate a collection directory: manifest rules, referenced
/// files exist, and every issuetype schema parses and validates.
pub fn validate_collection_dir(dir: &Path) -> Result<CollectionManifest> {
    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("invalid collection directory: {}", dir.display()))?;

    let manifest_path = dir.join("collection.json");
    let manifest_json = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("missing collection.json in {}", dir.display()))?;
    let manifest = CollectionManifest::from_json(&manifest_json)
        .with_context(|| format!("invalid collection.json in {}", dir.display()))?;

    validate_manifest(&manifest, dir_name)?;

    for entry in &manifest.issue_types {
        let schema_path = dir.join(&entry.schema_path);
        if !schema_path.is_file() {
            bail!("missing schema file '{}'", entry.schema_path);
        }
        let issue_type = load_issuetype_file(&schema_path)
            .with_context(|| format!("invalid issuetype schema '{}'", entry.schema_path))?;
        if issue_type.key != entry.key {
            bail!(
                "schema key '{}' does not match manifest key '{}'",
                issue_type.key,
                entry.key
            );
        }
        if let Some(template) = &entry.template_path {
            if !dir.join(template).is_file() {
                bail!("missing template file '{template}'");
            }
        }
    }

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const VALID_TASK_JSON: &str = r#"{
        "key": "TASK",
        "name": "Task",
        "description": "A focused task",
        "mode": "autonomous",
        "glyph": ">",
        "fields": [],
        "steps": [{"name": "execute", "outputs": ["report"], "prompt": "Do the task."}]
    }"#;

    fn manifest_json(id: &str) -> String {
        format!(
            r#"{{
                "schema_version": 1,
                "id": "{id}",
                "name": "Example",
                "description": "An example collection",
                "tier": "community",
                "author": "someone",
                "url": "https://example.com/repo",
                "license": "MIT",
                "icon_path": "icon.svg",
                "created": "2026-01-15",
                "updated": "2026-08-01",
                "issue_types": [
                    {{"key": "TASK", "schema_path": "TASK.json", "template_path": "TASK.md"}}
                ]
            }}"#
        )
    }

    fn write_collection(dir: &Path, id: &str) {
        let cdir = dir.join(id);
        fs::create_dir_all(&cdir).unwrap();
        fs::write(cdir.join("collection.json"), manifest_json(id)).unwrap();
        fs::write(cdir.join("TASK.json"), VALID_TASK_JSON).unwrap();
        fs::write(cdir.join("TASK.md"), "# Task: {{ summary }}\n").unwrap();
    }

    fn parse(json: &str) -> CollectionManifest {
        CollectionManifest::from_json(json).unwrap()
    }

    #[test]
    fn test_valid_community_collection_dir_passes() {
        let tmp = TempDir::new().unwrap();
        write_collection(tmp.path(), "example_loop");
        let manifest = validate_collection_dir(&tmp.path().join("example_loop")).unwrap();
        assert_eq!(manifest.id, "example_loop");
        assert_eq!(manifest.tier, CollectionTier::Community);
    }

    #[test]
    fn test_id_must_match_directory() {
        let m = parse(&manifest_json("example_loop"));
        let err = validate_manifest(&m, "other_dir").unwrap_err();
        assert!(err.to_string().contains("does not match its directory"));
    }

    #[test]
    fn test_invalid_id_rejected() {
        for bad in ["ab", "Has-Hyphen", "UPPER", "a b"] {
            let mut m = parse(&manifest_json("example_loop"));
            m.id = bad.to_string();
            let err = validate_manifest(&m, bad).unwrap_err();
            assert!(
                err.to_string().contains("invalid collection id"),
                "id '{bad}' should be rejected, got: {err}"
            );
        }
    }

    #[test]
    fn test_unsupported_schema_version_rejected() {
        let mut m = parse(&manifest_json("example_loop"));
        m.schema_version = 99;
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("unsupported schema_version"));
    }

    #[test]
    fn test_empty_name_and_description_rejected() {
        let mut m = parse(&manifest_json("example_loop"));
        m.name = "  ".to_string();
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("name must not be empty"));

        let mut m = parse(&manifest_json("example_loop"));
        m.description = String::new();
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("description must not be empty"));
    }

    #[test]
    fn test_issue_type_count_bounds() {
        let mut m = parse(&manifest_json("example_loop"));
        m.issue_types.clear();
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("between 1 and 32"));

        let mut m = parse(&manifest_json("example_loop"));
        let entry = m.issue_types[0].clone();
        m.issue_types = (0..33)
            .map(|i| {
                let mut e = entry.clone();
                e.key = format!("K{i}");
                e
            })
            .collect();
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("between 1 and 32"));
    }

    #[test]
    fn test_invalid_and_duplicate_keys_rejected() {
        let mut m = parse(&manifest_json("example_loop"));
        m.issue_types[0].key = "bad-key".to_string();
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("invalid issue type key"));

        let mut m = parse(&manifest_json("example_loop"));
        let dup = m.issue_types[0].clone();
        m.issue_types.push(dup);
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("duplicate issue type key"));
    }

    #[test]
    fn test_unsafe_paths_rejected() {
        for bad in ["../TASK.json", "/etc/passwd", "sub/TASK.json", ".hidden"] {
            let mut m = parse(&manifest_json("example_loop"));
            m.issue_types[0].schema_path = bad.to_string();
            let err = validate_manifest(&m, "example_loop").unwrap_err();
            assert!(
                err.to_string().contains("unsafe path"),
                "path '{bad}' should be rejected, got: {err}"
            );
        }
    }

    #[test]
    fn test_community_requires_license_author_url_and_icon() {
        for field in ["license", "author", "url", "icon_path"] {
            let mut m = parse(&manifest_json("example_loop"));
            match field {
                "license" => m.license = None,
                "author" => m.author = Some("  ".to_string()),
                "icon_path" => m.icon_path = None,
                _ => m.url = None,
            }
            let err = validate_manifest(&m, "example_loop").unwrap_err();
            assert!(
                err.to_string()
                    .contains(&format!("community collections require a {field}")),
                "missing {field} should be rejected, got: {err}"
            );
        }
    }

    #[test]
    fn test_official_tier_does_not_require_attribution() {
        let mut m = parse(&manifest_json("example_loop"));
        m.tier = CollectionTier::Official;
        m.license = None;
        m.author = None;
        m.url = None;
        m.icon_path = None;
        assert!(validate_manifest(&m, "example_loop").is_ok());
    }

    #[test]
    fn test_unsafe_icon_path_rejected() {
        for bad in ["../icon.svg", "/etc/icon.svg", "sub/icon.svg", ".icon.svg"] {
            let mut m = parse(&manifest_json("example_loop"));
            m.icon_path = Some(bad.to_string());
            let err = validate_manifest(&m, "example_loop").unwrap_err();
            assert!(
                err.to_string().contains("unsafe path"),
                "icon path '{bad}' should be rejected, got: {err}"
            );
        }
    }

    #[test]
    fn test_non_svg_icon_path_rejected() {
        let mut m = parse(&manifest_json("example_loop"));
        m.icon_path = Some("icon.png".to_string());
        let err = validate_manifest(&m, "example_loop").unwrap_err();
        assert!(err.to_string().contains("must be an .svg file"));
    }

    #[test]
    fn test_malformed_dates_rejected() {
        for (field, bad) in [("created", "2026-1-5"), ("updated", "01/15/2026")] {
            let mut m = parse(&manifest_json("example_loop"));
            match field {
                "created" => m.created = Some(bad.to_string()),
                _ => m.updated = Some(bad.to_string()),
            }
            let err = validate_manifest(&m, "example_loop").unwrap_err();
            assert!(
                err.to_string()
                    .contains(&format!("invalid {field} date '{bad}'")),
                "date '{bad}' should be rejected, got: {err}"
            );
        }
    }

    #[test]
    fn test_dates_are_optional() {
        let mut m = parse(&manifest_json("example_loop"));
        m.created = None;
        m.updated = None;
        assert!(validate_manifest(&m, "example_loop").is_ok());
    }

    #[test]
    fn test_dir_missing_manifest_rejected() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("example_loop")).unwrap();
        let err = validate_collection_dir(&tmp.path().join("example_loop")).unwrap_err();
        assert!(err.to_string().contains("missing collection.json"));
    }

    #[test]
    fn test_dir_missing_schema_file_rejected() {
        let tmp = TempDir::new().unwrap();
        write_collection(tmp.path(), "example_loop");
        fs::remove_file(tmp.path().join("example_loop/TASK.json")).unwrap();
        let err = validate_collection_dir(&tmp.path().join("example_loop")).unwrap_err();
        assert!(err.to_string().contains("missing schema file"));
    }

    #[test]
    fn test_dir_broken_schema_rejected() {
        let tmp = TempDir::new().unwrap();
        write_collection(tmp.path(), "example_loop");
        // Steps are required: an issuetype without steps must fail validation.
        fs::write(
            tmp.path().join("example_loop/TASK.json"),
            r#"{"key": "TASK", "name": "Task", "description": "d", "mode": "autonomous", "glyph": ">", "fields": [], "steps": []}"#,
        )
        .unwrap();
        let err = validate_collection_dir(&tmp.path().join("example_loop")).unwrap_err();
        assert!(err.to_string().contains("invalid issuetype schema"));
    }

    #[test]
    fn test_dir_key_mismatch_rejected() {
        let tmp = TempDir::new().unwrap();
        write_collection(tmp.path(), "example_loop");
        fs::write(
            tmp.path().join("example_loop/TASK.json"),
            VALID_TASK_JSON.replace("\"TASK\"", "\"OTHER\""),
        )
        .unwrap();
        let err = validate_collection_dir(&tmp.path().join("example_loop")).unwrap_err();
        assert!(err.to_string().contains("does not match manifest key"));
    }

    #[test]
    fn test_dir_missing_template_rejected() {
        let tmp = TempDir::new().unwrap();
        write_collection(tmp.path(), "example_loop");
        fs::remove_file(tmp.path().join("example_loop/TASK.md")).unwrap();
        let err = validate_collection_dir(&tmp.path().join("example_loop")).unwrap_err();
        assert!(err.to_string().contains("missing template file"));
    }
}
