//! Integration tests enforcing strict packaging constraints on the embedded UI.
//!
//! These tests run without the `embed-ui` feature — they validate the source
//! artifacts (package.json, dist directory) directly from the filesystem.

use std::path::Path;

const ALLOWED_RUNTIME_DEPS: &[&str] = &[
    "react",
    "react-dom",
    "react-router-dom",
    "@dnd-kit/core",
    "@dnd-kit/sortable",
    "@dnd-kit/utilities",
    // Codicon webfont for the sidebar/page icon vocabulary. Zero-dependency,
    // Microsoft-maintained; the concept→icon mapping lives in ui/src/concepts.ts
    // and the canonical table at docs/design-system/.
    "@vscode/codicons",
];

const UNCOMPRESSED_BUDGET_BYTES: u64 = 10_485_760; // 10MB

#[test]
fn test_ui_package_json_dep_allowlist() {
    let pkg_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/package.json");
    assert!(pkg_path.exists(), "ui/package.json must exist");
    let content = std::fs::read_to_string(&pkg_path).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

    if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
        for dep_name in deps.keys() {
            assert!(
                ALLOWED_RUNTIME_DEPS.contains(&dep_name.as_str()),
                "Unauthorized runtime dependency '{dep_name}' in ui/package.json. \
                 Allowed: {ALLOWED_RUNTIME_DEPS:?}. Add to ALLOWED_RUNTIME_DEPS in tests/ui_packaging.rs if intentional.",
            );
        }
    }
}

#[test]
fn test_ui_package_json_no_css_in_js() {
    let pkg_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/package.json");
    if !pkg_path.exists() {
        return;
    }
    let content = std::fs::read_to_string(&pkg_path).unwrap();

    let banned = [
        "styled-components",
        "@emotion/react",
        "@emotion/styled",
        "tailwindcss",
        "@mui/material",
        "chakra-ui",
    ];
    for lib in &banned {
        assert!(
            !content.contains(lib),
            "Banned CSS-in-JS / heavy UI library '{lib}' found in ui/package.json",
        );
    }
}

#[test]
fn test_ui_dist_size_budget_uncompressed() {
    let dist_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/dist");
    if !dist_path.exists() || !dist_path.join("index.html").exists() {
        return; // Not built yet — skip
    }

    let total = walk_dir_size(&dist_path);
    assert!(
        total < UNCOMPRESSED_BUDGET_BYTES,
        "ui/dist/ is {}B ({:.1}MB) uncompressed — exceeds 5MB budget",
        total,
        total as f64 / 1_048_576.0
    );
}

#[test]
fn test_ui_dist_no_source_maps_in_production() {
    let dist_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/dist");
    if !dist_path.exists() {
        return;
    }

    let map_files = find_files_with_extension(&dist_path, "map");
    assert!(
        map_files.is_empty(),
        "Source maps found in ui/dist/ — these should not ship in the embedded binary: {map_files:?}",
    );
}

fn walk_dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += walk_dir_size(&path);
            } else if let Ok(meta) = path.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn find_files_with_extension(dir: &Path, ext: &str) -> Vec<String> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_files_with_extension(&path, ext));
            } else if path.extension().is_some_and(|e| e == ext) {
                results.push(path.display().to_string());
            }
        }
    }
    results
}
