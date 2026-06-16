//! Feature Parity Integration Tests
//!
//! Ensures that Core Operations are available across all session management tools:
//! - TUI (keybindings)
//! - `VSCode` Extension (commands in package.json)
//! - REST API (endpoints)
//!
//! Core Operations:
//! - Sync Kanban Collections
//! - Pause Queue Processing
//! - Resume Queue Processing
//! - Approve Review
//! - Reject Review

// Note: We use include_str! to read source files directly since ui module
// is not exported from the library crate.

/// Read keybindings source file to extract descriptions
fn get_keybinding_descriptions() -> Vec<String> {
    let keybindings_src = include_str!("../src/ui/keybindings.rs");

    // Extract description strings from the source
    let mut descriptions = Vec::new();
    for line in keybindings_src.lines() {
        if let Some(start) = line.find("description:") {
            // Extract the string between quotes
            if let Some(quote_start) = line[start..].find('"') {
                let after_quote = &line[start + quote_start + 1..];
                if let Some(quote_end) = after_quote.find('"') {
                    descriptions.push(after_quote[..quote_end].to_string());
                }
            }
        }
    }
    descriptions
}

/// Core Operations that must be supported by all session management tools.
/// Each tuple: (TUI description pattern, `VSCode` command, API endpoint)
const CORE_OPERATIONS: &[(&str, &str, &str)] = &[
    (
        "Sync kanban",
        "operator.syncKanban",
        "POST /api/v1/queue/sync",
    ),
    (
        "Pause queue",
        "operator.pauseQueue",
        "POST /api/v1/queue/pause",
    ),
    (
        "Resume queue",
        "operator.resumeQueue",
        "POST /api/v1/queue/resume",
    ),
    (
        "Approve review",
        "operator.approveReview",
        "POST /api/v1/agents/{id}/approve",
    ),
    (
        "Reject review",
        "operator.rejectReview",
        "POST /api/v1/agents/{id}/reject",
    ),
];

/// Test that TUI has keybindings for all Core Operations
#[test]
fn test_tui_has_all_core_operations() {
    let descriptions = get_keybinding_descriptions();

    for (tui_pattern, _, _) in CORE_OPERATIONS {
        let found = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains(&tui_pattern.to_lowercase()));

        assert!(
            found,
            "TUI should have keybinding containing '{tui_pattern}'\nAvailable keybindings: {descriptions:?}"
        );
    }
}

/// Test that `VSCode` extension has commands for all Core Operations
#[test]
fn test_vscode_extension_has_all_core_operations() {
    // Read package.json from vscode-extension
    let package_json = include_str!("../vscode-extension/package.json");

    for (_, vscode_cmd, _) in CORE_OPERATIONS {
        assert!(
            package_json.contains(vscode_cmd),
            "VSCode extension should have command '{vscode_cmd}' in package.json"
        );
    }
}

/// Test that REST API routes are registered for all Core Operations.
///
/// Asserts against the generated OpenAPI spec rather than grepping the router
/// source: since the router was migrated to `utoipa_axum::OpenApiRouter`,
/// mounting a route *is* registering it in the spec, so the spec is the
/// authoritative list of live routes. Path params use OpenAPI `{param}` syntax.
#[test]
fn test_api_routes_are_registered() {
    let spec = operator::rest::ApiDoc::json().expect("generate OpenAPI spec");

    for route in [
        "/api/v1/queue/pause",
        "/api/v1/queue/resume",
        "/api/v1/queue/sync",
        "/api/v1/agents/{agent_id}/approve",
        "/api/v1/agents/{agent_id}/reject",
    ] {
        assert!(
            spec.contains(route),
            "REST API OpenAPI spec should document the {route} route"
        );
    }
}

/// Test that all Core Operations are documented in session management docs
#[test]
fn test_core_operations_documented() {
    // This test will fail until docs are updated, serving as a reminder
    let sessions_doc = include_str!("../docs/getting-started/sessions/index.md");

    // Check that feature parity section exists
    assert!(
        sessions_doc.contains("Feature Parity")
            || sessions_doc.contains("feature parity")
            || sessions_doc.contains("Core Operations"),
        "Session docs should document feature parity or Core Operations"
    );
}

/// Summary test: Print parity status for all Core Operations
#[test]
fn test_feature_parity_summary() {
    let descriptions = get_keybinding_descriptions();
    let package_json = include_str!("../vscode-extension/package.json");
    let mod_rs = include_str!("../src/rest/mod.rs");

    println!("\n=== Feature Parity Summary ===\n");
    println!(
        "{:<25} | {:<5} | {:<7} | {:<5}",
        "Operation", "TUI", "VSCode", "API"
    );
    println!("{:-<25}-+-{:-<5}-+-{:-<7}-+-{:-<5}", "", "", "", "");

    for (tui_pattern, vscode_cmd, api_path) in CORE_OPERATIONS {
        let has_tui = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains(&tui_pattern.to_lowercase()));
        let has_vscode = package_json.contains(vscode_cmd);
        // For API, check if the route path (without method) is in mod.rs
        let path_part = api_path.split_whitespace().last().unwrap_or("");
        // Convert {id} placeholder format to :agent_id format used in axum
        let axum_path = path_part.replace("{id}", ":agent_id");
        let has_api = mod_rs.contains(&axum_path);

        println!(
            "{:<25} | {:<5} | {:<7} | {:<5}",
            tui_pattern,
            if has_tui { "✓" } else { "✗" },
            if has_vscode { "✓" } else { "✗" },
            if has_api { "✓" } else { "✗" }
        );
    }
    println!();
}

// =============================================================================
// View Structure Parity
// =============================================================================

/// The four canonical views that all interfaces must implement.
/// Each tuple: (view name, TUI panel pattern in dashboard.rs, `VSCode` view ID in package.json)
const CANONICAL_VIEWS: &[(&str, &str, &str)] = &[
    ("Status", "StatusPanel", "operator-status"),
    ("Queue", "QueuePanel", "operator-queue"),
    ("In Progress", "InProgressPanel", "operator-in-progress"),
    ("Completed", "CompletedPanel", "operator-completed"),
];

/// The canonical status-section ids, parsed from the committed ts-rs export
/// `bindings/SectionId.ts` — itself generated from the `SectionId` enum in
/// `src/ui/status_panel.rs` (the single source of truth). The VS Code copy under
/// `vscode-extension/src/generated/` is gitignored (it's produced by
/// `npm run copy-types`), so we read the tracked `bindings/` original to stay
/// reproducible on a fresh checkout. Parsing the file means this list can never
/// go stale: add a `SectionId` variant and it flows here automatically, so the
/// per-surface checks below catch any surface that forgot to add it.
fn canonical_section_ids() -> Vec<String> {
    let src = include_str!("../bindings/SectionId.ts");
    // `export type SectionId = "config" | "connections" | ...;`
    let body = src.split_once('=').map(|(_, b)| b).unwrap_or(src);
    let mut ids = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find('"') {
        let after = &rest[i + 1..];
        match after.find('"') {
            Some(end) => {
                ids.push(after[..end].to_string());
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
    ids
}

/// Extract the ordered ids listed in `ui/src/concepts.ts`'s `STATUS_KEYS` array.
fn concepts_status_keys() -> Vec<String> {
    let src = include_str!("../ui/src/concepts.ts");
    let start = src
        .find("STATUS_KEYS")
        .and_then(|i| src[i..].find('[').map(|j| i + j + 1))
        .expect("concepts.ts should declare STATUS_KEYS = [ ... ]");
    let end = start + src[start..].find(']').expect("STATUS_KEYS array end");
    let mut ids = Vec::new();
    let mut rest = &src[start..end];
    while let Some(i) = rest.find('\'') {
        let after = &rest[i + 1..];
        match after.find('\'') {
            Some(e) => {
                ids.push(after[..e].to_string());
                rest = &after[e + 1..];
            }
            None => break,
        }
    }
    ids
}

/// Verify TUI has all 4 canonical view panels
#[test]
fn test_tui_has_all_canonical_views() {
    let dashboard_src = include_str!("../src/ui/dashboard.rs");
    for (name, tui_pattern, _) in CANONICAL_VIEWS {
        assert!(
            dashboard_src.contains(tui_pattern),
            "TUI dashboard should contain {tui_pattern} for '{name}' view"
        );
    }
}

/// Verify `VSCode` extension has all 4 canonical views
#[test]
fn test_vscode_has_all_canonical_views() {
    let package_json = include_str!("../vscode-extension/package.json");
    for (name, _, vscode_id) in CANONICAL_VIEWS {
        assert!(
            package_json.contains(vscode_id),
            "VSCode extension should have view '{vscode_id}' for '{name}'"
        );
    }
}

/// The canonical section list must be non-trivial (guards a parsing regression
/// that would silently make the coverage checks vacuous).
#[test]
fn test_canonical_section_ids_present() {
    let ids = canonical_section_ids();
    assert!(
        ids.len() >= 9,
        "expected at least 9 canonical sections, parsed {ids:?}"
    );
    assert!(ids.contains(&"config".to_string()));
    assert!(ids.contains(&"projects".to_string()));
}

/// Every canonical section id must be referenced by the TUI status panel (as a
/// serde rename) so the TUI can't drop a section the source of truth declares.
#[test]
fn test_tui_has_all_status_sections() {
    let status_panel_src = include_str!("../src/ui/status_panel.rs");
    for id in canonical_section_ids() {
        assert!(
            status_panel_src.contains(&format!("\"{id}\"")),
            "TUI status_panel.rs is missing the serde rename for section '{id}'"
        );
    }
}

/// Every canonical section id must be referenced by the VS Code status provider.
#[test]
fn test_vscode_has_all_status_sections() {
    let status_provider_src = include_str!("../vscode-extension/src/status-provider.ts");
    for id in canonical_section_ids() {
        assert!(
            status_provider_src.contains(&id),
            "VSCode status-provider.ts is missing sectionId '{id}'"
        );
    }
}

/// The web UI sidebar (`STATUS_KEYS` in concepts.ts) must list exactly the
/// canonical sections, in the same order, as the TUI / VS Code surfaces.
#[test]
fn test_web_ui_status_keys_match_canonical_order() {
    assert_eq!(
        concepts_status_keys(),
        canonical_section_ids(),
        "ui/src/concepts.ts STATUS_KEYS must match the canonical SectionId order"
    );
}

/// Every `docsUrl` the web UI links to must resolve to a real docs page, so a
/// concept page never sends a reader to a 404.
#[test]
fn test_concepts_docs_urls_resolve() {
    const BASE: &str = "${DOCS_BASE}";
    let src = include_str!("../ui/src/concepts.ts");
    let docs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    let mut checked = 0;
    for line in src.lines() {
        let Some(i) = line.find("docsUrl:") else {
            continue;
        };
        // e.g. ``docsUrl: `${DOCS_BASE}/getting-started/git/`,``
        let after = &line[i..];
        let Some(b) = after.find(BASE) else {
            continue;
        };
        let rest = &after[b + BASE.len()..];
        let Some(end) = rest.find('`') else {
            continue;
        };
        let path = rest[..end].trim_matches('/');
        let resolves = if path.is_empty() {
            docs_dir.join("index.md").exists()
        } else {
            docs_dir.join(format!("{path}.md")).exists()
                || docs_dir.join(path).join("index.md").exists()
        };
        assert!(
            resolves,
            "concepts.ts docsUrl '/{path}/' does not resolve to a docs page on disk"
        );
        checked += 1;
    }
    assert!(
        checked >= 9,
        "expected to verify all concept docsUrls; only matched {checked}"
    );
}

/// View structure parity summary
#[test]
fn test_view_structure_parity_summary() {
    let dashboard_src = include_str!("../src/ui/dashboard.rs");
    let package_json = include_str!("../vscode-extension/package.json");

    println!("\n=== View Structure Parity ===\n");
    println!("{:<15} | {:<5} | {:<7}", "View", "TUI", "VSCode");
    println!("{:-<15}-+-{:-<5}-+-{:-<7}", "", "", "");

    for (name, tui_pattern, vscode_id) in CANONICAL_VIEWS {
        let has_tui = dashboard_src.contains(tui_pattern);
        let has_vscode = package_json.contains(vscode_id);
        println!(
            "{:<15} | {:<5} | {:<7}",
            name,
            if has_tui { "✓" } else { "✗" },
            if has_vscode { "✓" } else { "✗" },
        );
    }
    println!();
}

#[cfg(test)]
mod detailed_tests {
    use super::*;

    /// Verify specific TUI keybindings exist with expected descriptions
    #[test]
    fn test_tui_keybinding_descriptions() {
        let descriptions = get_keybinding_descriptions();

        // Sync kanban should exist
        let sync_exists = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains("sync kanban"));
        assert!(sync_exists, "Sync kanban shortcut should exist");

        // Pause queue should exist
        let pause_exists = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains("pause queue"));
        assert!(pause_exists, "Pause queue shortcut should exist");

        // Resume queue should exist
        let resume_exists = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains("resume queue"));
        assert!(resume_exists, "Resume queue shortcut should exist");

        // Approve review should exist
        let approve_exists = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains("approve review"));
        assert!(approve_exists, "Approve review shortcut should exist");

        // Reject review should exist
        let reject_exists = descriptions
            .iter()
            .any(|d| d.to_lowercase().contains("reject review"));
        assert!(reject_exists, "Reject review shortcut should exist");
    }

    /// Verify `VSCode` commands have proper titles
    #[test]
    fn test_vscode_command_titles() {
        let package_json = include_str!("../vscode-extension/package.json");

        assert!(
            package_json.contains("Pause Queue Processing"),
            "VSCode should have 'Pause Queue Processing' title"
        );
        assert!(
            package_json.contains("Resume Queue Processing"),
            "VSCode should have 'Resume Queue Processing' title"
        );
        assert!(
            package_json.contains("Sync Kanban Collections"),
            "VSCode should have 'Sync Kanban Collections' title"
        );
        assert!(
            package_json.contains("Approve Review"),
            "VSCode should have 'Approve Review' title"
        );
        assert!(
            package_json.contains("Reject Review"),
            "VSCode should have 'Reject Review' title"
        );
    }
}
