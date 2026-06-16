//! Asserts every AGNT plugin tool sets `this.name` to its manifest `type`.
//!
//! AGNT's `PluginManager` registers and routes each tool instance by its
//! `this.name` property. Ironclad rule: the constructor's `this.name` MUST equal
//! the tool's `type` in `agnt-plugin/manifest.json`. A class missing `this.name`
//! registers under `undefined` — it installs but the node never fires.
//!
//! This test reads files only (no JS runtime): it parses the manifest for the
//! source-of-truth `type` -> `entryPoint` pairs, then confirms each referenced
//! `.js` file assigns `this.name = "<type>"` in its constructor.

use std::fs;
use std::path::PathBuf;

/// Repo root = crate manifest dir (tests run with CWD at the crate root).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_every_agnt_tool_sets_this_name_to_manifest_type() {
    let root = repo_root();
    let plugin_dir = root.join("agnt-plugin");
    let manifest_path = plugin_dir.join("manifest.json");
    let manifest_raw = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", manifest_path.display()));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", manifest_path.display()));

    let tools = manifest["tools"]
        .as_array()
        .expect("manifest.json: `tools` must be an array");
    assert!(!tools.is_empty(), "manifest.json declares no tools");

    let mut problems = Vec::new();
    for tool in tools {
        let ty = tool["type"]
            .as_str()
            .expect("each tool must declare a string `type`");
        let entry = tool["entryPoint"]
            .as_str()
            .unwrap_or_else(|| panic!("tool {ty:?} missing string `entryPoint`"));

        let file_path = plugin_dir.join(entry.trim_start_matches("./"));
        let src = match fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                problems.push(format!("  {entry}: failed to read ({e})"));
                continue;
            }
        };

        // Accept single or double quotes around the type string.
        let dq = format!("this.name = \"{ty}\"");
        let sq = format!("this.name = '{ty}'");
        if !src.contains(&dq) && !src.contains(&sq) {
            problems.push(format!(
                "  {entry}: missing `this.name = \"{ty}\"` in constructor"
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "AGNT plugin tools must set this.name to their manifest type:\n{}\n\
         Add a constructor that sets `this.name` to the manifest `type` for each file above.",
        problems.join("\n")
    );
}
