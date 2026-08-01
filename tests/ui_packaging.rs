//! Integration tests enforcing strict packaging constraints on the frontend.
//!
//! Two packages are covered: `ui/` (the embedded SPA) and `webcomponents/` (the
//! components `ui/` and the docs site share). Both ship to users — the SPA
//! inside the binary, the web components on the docs site — so both are held to
//! the same dependency, size, and source-map rules.
//!
//! These tests run without the `embed-ui` feature: they validate the source
//! artifacts (package.json, dist directory) directly from the filesystem.

use std::path::{Path, PathBuf};

/// Runtime dependencies allowed in `ui/package.json`.
const ALLOWED_UI_DEPS: &[&str] = &[
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

/// Runtime dependencies allowed in `webcomponents/package.json`.
const ALLOWED_WEBCOMPONENT_DEPS: &[&str] = &[
    // Workflow-graph rendering. First-party (same author as operator): renders
    // a flat node/edge graph with React Flow. Operator supplies the graph itself
    // via webcomponents/src/workflow/issuetype-to-ir.ts, which projects an
    // issue type's native steps — no workflow source is parsed or executed.
    "@untra/naiveworkflow-react",
    // React Flow — peer dependency of @untra/naiveworkflow-react; the graph
    // renderer/canvas. (dagre, its layout engine, comes in transitively.)
    "@xyflow/react",
];

/// CSS-in-JS and heavyweight component libraries banned from every frontend
/// package: the brand is expressed through the shared tokens in
/// `docs/assets/css/tokens.css`, not a second styling system.
const BANNED_LIBRARIES: &[&str] = &[
    "styled-components",
    "@emotion/react",
    "@emotion/styled",
    "tailwindcss",
    "@mui/material",
    "chakra-ui",
];

const UNCOMPRESSED_BUDGET_BYTES: u64 = 10_485_760; // 10MB

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn assert_dep_allowlist(package_json: &str, allowed: &[&str]) {
    let pkg_path = repo_path(package_json);
    assert!(pkg_path.exists(), "{package_json} must exist");
    let content = std::fs::read_to_string(&pkg_path).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

    if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
        for dep_name in deps.keys() {
            assert!(
                allowed.contains(&dep_name.as_str()),
                "Unauthorized runtime dependency '{dep_name}' in {package_json}. \
                 Allowed: {allowed:?}. Update the allowlist in tests/ui_packaging.rs \
                 if intentional.",
            );
        }
    }
}

fn assert_no_banned_libraries(package_json: &str) {
    let pkg_path = repo_path(package_json);
    if !pkg_path.exists() {
        return;
    }
    let content = std::fs::read_to_string(&pkg_path).unwrap();
    for lib in BANNED_LIBRARIES {
        assert!(
            !content.contains(lib),
            "Banned CSS-in-JS / heavy UI library '{lib}' found in {package_json}",
        );
    }
}

fn assert_no_source_maps(dist: &str) {
    let dist_path = repo_path(dist);
    if !dist_path.exists() {
        return;
    }
    let map_files = find_files_with_extension(&dist_path, "map");
    assert!(
        map_files.is_empty(),
        "Source maps found in {dist}/ — these should not ship to users: {map_files:?}",
    );
}

#[test]
fn test_ui_package_json_dep_allowlist() {
    assert_dep_allowlist("ui/package.json", ALLOWED_UI_DEPS);
}

#[test]
fn test_webcomponents_package_json_dep_allowlist() {
    assert_dep_allowlist("webcomponents/package.json", ALLOWED_WEBCOMPONENT_DEPS);
}

#[test]
fn test_frontend_packages_have_no_css_in_js() {
    assert_no_banned_libraries("ui/package.json");
    assert_no_banned_libraries("webcomponents/package.json");
}

/// The SPA must not depend on the graph stack directly — that would let the
/// in-app graph drift from the one the docs site draws.
#[test]
fn test_ui_does_not_depend_on_the_graph_stack_directly() {
    let content = std::fs::read_to_string(repo_path("ui/package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();
    let deps = pkg.get("dependencies").and_then(|d| d.as_object());

    for dep in ALLOWED_WEBCOMPONENT_DEPS {
        assert!(
            deps.is_none_or(|d| !d.contains_key(*dep)),
            "ui/package.json depends on '{dep}' directly. The graph stack belongs to \
             webcomponents/ so the SPA and the docs site render from one implementation.",
        );
    }
}

#[test]
fn test_ui_dist_size_budget_uncompressed() {
    let dist_path = repo_path("ui/dist");
    if !dist_path.exists() || !dist_path.join("index.html").exists() {
        return; // Not built yet — skip
    }

    let total = walk_dir_size(&dist_path);
    assert!(
        total < UNCOMPRESSED_BUDGET_BYTES,
        "ui/dist/ is {}B ({:.1}MB) uncompressed — exceeds the {:.0}MB budget",
        total,
        total as f64 / 1_048_576.0,
        UNCOMPRESSED_BUDGET_BYTES as f64 / 1_048_576.0
    );
}

#[test]
fn test_dist_directories_have_no_source_maps() {
    assert_no_source_maps("ui/dist");
    assert_no_source_maps("webcomponents/dist");
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
