//! Generates copy-paste MCP client configuration snippets pointing at this
//! operator binary.
//!
//! Each `*_snippet(cwd)` returns a `serde_json::Value` shaped the way the
//! target client's config file expects. The dashboard writes one of these to
//! `<tickets>/operator/mcp/<client>.json` and opens it in the user's editor;
//! the user pastes the contents into their actual client config.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Path to the currently-running operator binary. Falls back to bare
/// "operator" if `current_exe` is unavailable (e.g. in some test contexts).
pub fn current_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("operator"))
}

/// Shape used by Claude Code (`~/.claude.json`), Claude Desktop, and Cursor
/// (`~/.cursor/mcp.json`). All three accept the same `mcpServers` block.
fn mcp_servers_shape(cwd: &Path) -> Value {
    json!({
        "mcpServers": {
            "operator": {
                "command": current_exe().to_string_lossy(),
                "args": ["mcp"],
                "cwd": cwd.to_string_lossy(),
            }
        }
    })
}

pub fn claude_code_snippet(cwd: &Path) -> Value {
    mcp_servers_shape(cwd)
}

pub fn claude_desktop_snippet(cwd: &Path) -> Value {
    mcp_servers_shape(cwd)
}

/// Cursor's `~/.cursor/mcp.json` uses the same `mcpServers` shape as Claude.
pub fn cursor_snippet(cwd: &Path) -> Value {
    mcp_servers_shape(cwd)
}

/// VS Code (1.94+) per-workspace `.vscode/mcp.json` uses a `servers` block
/// with an explicit `type: "stdio"` discriminator.
pub fn vscode_snippet(cwd: &Path) -> Value {
    json!({
        "servers": {
            "operator": {
                "type": "stdio",
                "command": current_exe().to_string_lossy(),
                "args": ["mcp"],
                "cwd": cwd.to_string_lossy(),
            }
        }
    })
}

/// Zed user settings under `context_servers`.
pub fn zed_snippet(cwd: &Path) -> Value {
    json!({
        "context_servers": {
            "operator": {
                "command": {
                    "path": current_exe().to_string_lossy(),
                    "args": ["mcp"],
                    "env": {}
                },
                "settings": { "cwd": cwd.to_string_lossy() }
            }
        }
    })
}

/// Dispatch by client name. Returns `None` for unknown clients.
pub fn snippet_for(client: &str, cwd: &Path) -> Option<Value> {
    match client {
        "claude-code" => Some(claude_code_snippet(cwd)),
        "claude-desktop" => Some(claude_desktop_snippet(cwd)),
        "cursor" => Some(cursor_snippet(cwd)),
        "vscode" => Some(vscode_snippet(cwd)),
        "zed" => Some(zed_snippet(cwd)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_code_snippet_shape() {
        let cfg = claude_code_snippet(&PathBuf::from("/work"));
        assert_eq!(cfg["mcpServers"]["operator"]["args"][0], "mcp");
        assert_eq!(cfg["mcpServers"]["operator"]["cwd"], "/work");
    }

    #[test]
    fn test_cursor_snippet_matches_claude_code() {
        let cursor = cursor_snippet(&PathBuf::from("/work"));
        let claude = claude_code_snippet(&PathBuf::from("/work"));
        assert_eq!(cursor, claude);
    }

    #[test]
    fn test_vscode_snippet_uses_servers_with_type() {
        let cfg = vscode_snippet(&PathBuf::from("/work"));
        assert_eq!(cfg["servers"]["operator"]["type"], "stdio");
        assert_eq!(cfg["servers"]["operator"]["args"][0], "mcp");
    }

    #[test]
    fn test_zed_snippet_uses_context_servers() {
        let cfg = zed_snippet(&PathBuf::from("/work"));
        assert!(cfg["context_servers"]["operator"]["command"]["path"].is_string());
    }

    #[test]
    fn test_snippet_for_unknown_client_is_none() {
        assert!(snippet_for("notepad++", &PathBuf::from("/w")).is_none());
    }

    #[test]
    fn test_snippet_for_dispatches_correctly() {
        let cwd = PathBuf::from("/w");
        assert!(snippet_for("claude-code", &cwd).is_some());
        assert!(snippet_for("claude-desktop", &cwd).is_some());
        assert!(snippet_for("cursor", &cwd).is_some());
        assert!(snippet_for("vscode", &cwd).is_some());
        assert!(snippet_for("zed", &cwd).is_some());
    }
}
