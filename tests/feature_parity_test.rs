//! Feature Parity Integration Tests
//!
//! Ensures that Core Operations are available across all session management tools:
//! - TUI (keybindings)
//! - VSCode Extension (commands in package.json)
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
/// Each tuple: (TUI description pattern, VSCode command, API endpoint)
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
            "TUI should have keybinding containing '{}'\nAvailable keybindings: {:?}",
            tui_pattern, descriptions
        );
    }
}

/// Test that VSCode extension has commands for all Core Operations
#[test]
fn test_vscode_extension_has_all_core_operations() {
    // Read package.json from vscode-extension
    let package_json = include_str!("../vscode-extension/package.json");

    for (_, vscode_cmd, _) in CORE_OPERATIONS {
        assert!(
            package_json.contains(vscode_cmd),
            "VSCode extension should have command '{}' in package.json",
            vscode_cmd
        );
    }
}

/// Test that REST API routes are registered for all Core Operations
#[test]
fn test_api_routes_are_registered() {
    // Read mod.rs to verify routes are registered
    let mod_rs = include_str!("../src/rest/mod.rs");

    // Check that pause/resume/sync endpoints are registered
    assert!(
        mod_rs.contains("/api/v1/queue/pause"),
        "REST API should have /api/v1/queue/pause route"
    );
    assert!(
        mod_rs.contains("/api/v1/queue/resume"),
        "REST API should have /api/v1/queue/resume route"
    );
    assert!(
        mod_rs.contains("/api/v1/queue/sync"),
        "REST API should have /api/v1/queue/sync route"
    );
    assert!(
        mod_rs.contains("/api/v1/agents/:agent_id/approve"),
        "REST API should have /api/v1/agents/:agent_id/approve route"
    );
    assert!(
        mod_rs.contains("/api/v1/agents/:agent_id/reject"),
        "REST API should have /api/v1/agents/:agent_id/reject route"
    );
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

    /// Verify VSCode commands have proper titles
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
