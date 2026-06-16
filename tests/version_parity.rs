//! Asserts every managed manifest carries the canonical version from `VERSION`.
//!
//! Adding a new versioned manifest is a one-line addition to `MANAGED` below.
//! Keep this list in sync with the files revved by `bump-version.sh` and the
//! `release` job in `.github/workflows/build.yaml`.

use std::fs;
use std::path::{Path, PathBuf};

/// How to pull the version string out of a given file.
enum ExtractKind {
    /// First `version = "..."` line (Cargo.toml, zed extension.toml).
    TomlPackageVersion,
    /// Top-level `"version": "..."` (package.json, manifest.json, openapi.json).
    JsonDotVersion,
    /// `version: ...` line (docs/_config.yml).
    YamlVersion,
    /// `const VERSION = '...'` (webhook-server.ts).
    TsConst,
}

/// Repo root = crate manifest dir (tests run with CWD at the crate root).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Extract the first `"<key>": "<value>"`-style version from a string slice
/// once positioned at the start of the value. Returns the inner string.
fn between_quotes_after(haystack: &str, marker: &str) -> Option<String> {
    let start = haystack.find(marker)? + marker.len();
    let rest = &haystack[start..];
    let q = rest.find(['"', '\''])?;
    let rest = &rest[q + 1..];
    let end = rest.find(['"', '\''])?;
    Some(rest[..end].to_string())
}

fn extract(kind: &ExtractKind, content: &str) -> Option<String> {
    match kind {
        ExtractKind::TomlPackageVersion => content
            .lines()
            .find(|l| l.trim_start().starts_with("version ="))
            .and_then(|l| between_quotes_after(l, "version")),
        ExtractKind::JsonDotVersion => content
            .lines()
            .find(|l| l.trim_start().starts_with("\"version\""))
            .and_then(|l| between_quotes_after(l, ":")),
        ExtractKind::YamlVersion => content
            .lines()
            .find(|l| l.trim_start().starts_with("version:"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string()),
        ExtractKind::TsConst => content
            .lines()
            .find(|l| l.contains("const VERSION"))
            .and_then(|l| between_quotes_after(l, "=")),
    }
}

/// Every file whose version must match `VERSION`. One line per manifest.
const MANAGED: &[(&str, ExtractKind)] = &[
    ("Cargo.toml", ExtractKind::TomlPackageVersion),
    ("opr8r/Cargo.toml", ExtractKind::TomlPackageVersion),
    ("zed-extension/Cargo.toml", ExtractKind::TomlPackageVersion),
    (
        "zed-extension/extension.toml",
        ExtractKind::TomlPackageVersion,
    ),
    ("docs/_config.yml", ExtractKind::YamlVersion),
    ("vscode-extension/package.json", ExtractKind::JsonDotVersion),
    (
        "vscode-extension/src/webhook-server.ts",
        ExtractKind::TsConst,
    ),
    ("backstage-server/package.json", ExtractKind::JsonDotVersion),
    ("agnt-plugin/package.json", ExtractKind::JsonDotVersion),
    ("agnt-plugin/manifest.json", ExtractKind::JsonDotVersion),
    ("docs/schemas/openapi.json", ExtractKind::JsonDotVersion),
];

#[test]
fn test_all_managed_manifests_match_version_file() {
    let root = repo_root();
    let expected = read(&root.join("VERSION")).trim().to_string();
    assert!(!expected.is_empty(), "VERSION file is empty");

    let mut mismatches = Vec::new();
    for (rel, kind) in MANAGED {
        let path = root.join(rel);
        let content = read(&path);
        match extract(kind, &content) {
            Some(found) if found == expected => {}
            Some(found) => {
                mismatches.push(format!("  {rel}: found {found:?}, expected {expected:?}"));
            }
            None => mismatches.push(format!("  {rel}: no version string found")),
        }
    }

    assert!(
        mismatches.is_empty(),
        "version drift from VERSION={expected:?}:\n{}\nRun ./bump-version.sh or correct the files above; regenerate docs/schemas/openapi.json with `cargo run -- docs --only openapi`.",
        mismatches.join("\n")
    );
}
