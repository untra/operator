//! Surface Parity Integration Tests
//!
//! Validates that operator capabilities stay aligned across all integration
//! surfaces: slash commands (Zed), MCP tools, REST routes, and TUI keybindings.
//!
//! Uses `include_str!` to scan source files (same pattern as feature_parity_test.rs)
//! and the capability inventory from `src/integrations/inventory.rs`.

use operator::integrations::all_capabilities;

// ============================================================================
// Source files scanned via include_str!
// ============================================================================

const EXTENSION_TOML: &str = include_str!("../zed-extension/extension.toml");
const MCP_TOOLS_RS: &str = include_str!("../src/mcp/tools.rs");
const REST_MOD_RS: &str = include_str!("../src/rest/mod.rs");
const KEYBINDINGS_RS: &str = include_str!("../src/ui/keybindings.rs");

// ============================================================================
// Helpers
// ============================================================================

/// Extract keybinding description strings from keybindings.rs source.
fn get_keybinding_descriptions() -> Vec<String> {
    let mut descriptions = Vec::new();
    for line in KEYBINDINGS_RS.lines() {
        if let Some(start) = line.find("description:") {
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

/// Extract the path portion from a "METHOD /path" REST endpoint string.
fn rest_path(endpoint: &str) -> &str {
    endpoint.split_whitespace().last().unwrap_or("")
}

// ============================================================================
// Per-surface validation
// ============================================================================

/// Every capability with a slash_command must appear in extension.toml
#[test]
fn test_slash_commands_present_in_extension_toml() {
    for cap in all_capabilities() {
        if let Some(cmd) = cap.slash_command {
            assert!(
                EXTENSION_TOML.contains(cmd),
                "Capability '{}': slash command '{}' not found in zed-extension/extension.toml",
                cap.name,
                cmd
            );
        }
    }
}

/// Every capability with an mcp_tool must appear in tools.rs
#[test]
fn test_mcp_tools_present_in_tools_rs() {
    for cap in all_capabilities() {
        if let Some(tool) = cap.mcp_tool {
            assert!(
                MCP_TOOLS_RS.contains(tool),
                "Capability '{}': MCP tool '{}' not found in src/mcp/tools.rs",
                cap.name,
                tool
            );
        }
    }
}

/// Every capability with a rest_endpoint must have its path in src/rest/mod.rs
#[test]
fn test_rest_endpoints_present_in_rest_mod() {
    for cap in all_capabilities() {
        if let Some(endpoint) = cap.rest_endpoint {
            let path = rest_path(endpoint);
            assert!(
                REST_MOD_RS.contains(path),
                "Capability '{}': REST path '{}' not found in src/rest/mod.rs",
                cap.name,
                path
            );
        }
    }
}

/// Every capability with a tui_action must match a keybinding description
#[test]
fn test_tui_actions_present_in_keybindings() {
    let descriptions = get_keybinding_descriptions();
    for cap in all_capabilities() {
        if let Some(action) = cap.tui_action {
            let found = descriptions
                .iter()
                .any(|d| d.to_lowercase().contains(&action.to_lowercase()));
            assert!(
                found,
                "Capability '{}': TUI action '{}' not found in keybinding descriptions.\nAvailable: {:?}",
                cap.name, action, descriptions
            );
        }
    }
}

// ============================================================================
// Cross-surface coverage
// ============================================================================

/// Every slash command in extension.toml should map to at least one capability
#[test]
fn test_no_orphan_slash_commands() {
    let caps = all_capabilities();
    let known_commands: Vec<&str> = caps.iter().filter_map(|c| c.slash_command).collect();

    // Extract slash command keys from extension.toml
    for line in EXTENSION_TOML.lines() {
        let trimmed = line.trim();
        // Slash command lines look like: op-status = { description = "..." }
        if trimmed.starts_with("op-") {
            if let Some(key) = trimmed.split('=').next() {
                let key = key.trim();
                assert!(
                    known_commands.contains(&key),
                    "Slash command '{}' in extension.toml has no matching capability in inventory",
                    key
                );
            }
        }
    }
}

/// No MCP tool name referenced in inventory should be missing from tools.rs
/// (redundant with per-capability test, but guards against typos in the inventory)
#[test]
fn test_inventory_mcp_tools_are_real() {
    for cap in all_capabilities() {
        if let Some(tool) = cap.mcp_tool {
            // Check that the tool name appears as a string literal in tools.rs
            let quoted = format!("\"{}\"", tool);
            assert!(
                MCP_TOOLS_RS.contains(&quoted),
                "Capability '{}': MCP tool '{}' does not appear as a quoted string in tools.rs (possible typo)",
                cap.name,
                tool
            );
        }
    }
}

// ============================================================================
// Schema validation
// ============================================================================

/// Validate extension.toml parses as a valid Zed extension manifest.
/// Struct mirrors Zed's actual schema: metadata at top level, not nested
/// under `[extension]`. Catches missing required fields before Zed does.
#[test]
fn test_extension_toml_schema_valid() {
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct ExtensionToml {
        id: String,
        name: String,
        version: String,
        schema_version: u32,
        #[serde(default)]
        slash_commands: std::collections::HashMap<String, SlashCommandEntry>,
        #[serde(default)]
        context_servers: std::collections::HashMap<String, toml::Value>,
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct SlashCommandEntry {
        description: String,
        requires_argument: bool,
    }

    let parsed: ExtensionToml =
        toml::from_str(EXTENSION_TOML).expect("extension.toml must parse as valid Zed manifest");

    assert!(!parsed.id.is_empty());
    assert!(parsed.schema_version >= 1);

    for (name, entry) in &parsed.slash_commands {
        assert!(
            !entry.description.is_empty(),
            "Slash command '{}' has empty description",
            name
        );
    }
}

// ============================================================================
// Summary
// ============================================================================

/// Print a human-readable surface parity matrix
#[test]
fn test_surface_parity_summary() {
    let descriptions = get_keybinding_descriptions();

    println!("\n=== Surface Parity Matrix ===\n");
    println!(
        "{:<22} | {:<7} | {:<7} | {:<7} | {:<7}",
        "Capability", "Slash", "MCP", "REST", "TUI"
    );
    println!(
        "{:-<22}-+-{:-<7}-+-{:-<7}-+-{:-<7}-+-{:-<7}",
        "", "", "", "", ""
    );

    for cap in all_capabilities() {
        let has_slash = cap
            .slash_command
            .map_or(false, |cmd| EXTENSION_TOML.contains(cmd));
        let has_mcp = cap
            .mcp_tool
            .map_or(false, |tool| MCP_TOOLS_RS.contains(tool));
        let has_rest = cap
            .rest_endpoint
            .map_or(false, |ep| REST_MOD_RS.contains(rest_path(ep)));
        let has_tui = cap.tui_action.map_or(false, |action| {
            descriptions
                .iter()
                .any(|d| d.to_lowercase().contains(&action.to_lowercase()))
        });

        let mark = |claimed: bool, present: bool| match (claimed, present) {
            (true, true) => "Y",
            (true, false) => "MISS",
            (false, _) => "-",
        };

        println!(
            "{:<22} | {:<7} | {:<7} | {:<7} | {:<7}",
            cap.name,
            mark(cap.slash_command.is_some(), has_slash),
            mark(cap.mcp_tool.is_some(), has_mcp),
            mark(cap.rest_endpoint.is_some(), has_rest),
            mark(cap.tui_action.is_some(), has_tui),
        );
    }
    println!();
}
