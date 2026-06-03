//! Generates copy-paste ACP agent registrations pointing at this operator
//! binary.
//!
//! Each `*_snippet()` returns either a `serde_json::Value` (Zed, `JetBrains`)
//! or a plain `String` (Emacs elisp, Kiro TOML), shaped the way the target
//! editor expects it. The dashboard writes one of these to
//! `<tickets>/operator/acp/<editor>.{json,el,toml}` and opens it in the
//! user's editor; the user pastes the contents into their actual editor
//! configuration.

use serde_json::{json, Value};
use std::path::PathBuf;

/// Path to the currently-running operator binary. Falls back to bare
/// `"operator"` if `current_exe` is unavailable (e.g. in some test contexts).
pub fn current_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("operator"))
}

fn exe_string() -> String {
    current_exe().to_string_lossy().into_owned()
}

/// Zed `~/.config/zed/settings.json` — `agent_servers` block.
pub fn zed_snippet() -> Value {
    json!({
        "agent_servers": {
            "operator": {
                "command": exe_string(),
                "args": ["acp"],
                "env": {}
            }
        }
    })
}

/// `JetBrains` ACP Agent Registry JSON. Imported via the IDE's ACP plugin.
pub fn jetbrains_snippet() -> Value {
    json!({
        "name": "operator",
        "displayName": "Operator (Kanban Orchestrator)",
        "command": exe_string(),
        "args": ["acp"]
    })
}

/// Emacs `agent-shell` — elisp form to add to your init file.
pub fn emacs_snippet() -> String {
    format!(
        "(add-to-list 'agent-shell-acp-agents\n  '(:name \"operator\" :command \"{}\" :args (\"acp\")))",
        exe_string()
    )
}

/// Kiro `~/.kiro/agents.toml` entry.
pub fn kiro_snippet() -> String {
    format!(
        "[[agents]]\nname = \"operator\"\ncommand = \"{}\"\nargs = [\"acp\"]\n",
        exe_string()
    )
}

/// Dispatch by editor name. Returns `None` for unknown editors. JSON-shaped
/// editors (Zed, `JetBrains`) return their snippet directly; text-format
/// editors (Emacs, Kiro) are wrapped as `Value::String` so callers can treat
/// the result uniformly.
pub fn snippet_for(editor: &str) -> Option<Value> {
    match editor {
        "zed" => Some(zed_snippet()),
        "jetbrains" => Some(jetbrains_snippet()),
        "emacs" => Some(Value::String(emacs_snippet())),
        "kiro" => Some(Value::String(kiro_snippet())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zed_snippet_shape() {
        let cfg = zed_snippet();
        assert_eq!(cfg["agent_servers"]["operator"]["args"][0], "acp");
        assert!(cfg["agent_servers"]["operator"]["command"].is_string());
    }

    #[test]
    fn test_jetbrains_snippet_has_name_and_command() {
        let cfg = jetbrains_snippet();
        assert_eq!(cfg["name"], "operator");
        assert_eq!(cfg["args"][0], "acp");
    }

    #[test]
    fn test_emacs_snippet_is_valid_elisp_form() {
        let snippet = emacs_snippet();
        assert!(snippet.starts_with("(add-to-list 'agent-shell-acp-agents"));
        assert!(snippet.contains("operator"));
        assert!(snippet.contains(":args (\"acp\")"));
    }

    #[test]
    fn test_kiro_snippet_is_toml_array_entry() {
        let snippet = kiro_snippet();
        assert!(snippet.starts_with("[[agents]]"));
        assert!(snippet.contains("name = \"operator\""));
        assert!(snippet.contains("args = [\"acp\"]"));
    }

    #[test]
    fn test_snippet_for_unknown_editor_is_none() {
        assert!(snippet_for("notepad++").is_none());
    }

    #[test]
    fn test_snippet_for_dispatches_all_editors() {
        assert!(snippet_for("zed").is_some());
        assert!(snippet_for("jetbrains").is_some());
        assert!(snippet_for("emacs").is_some());
        assert!(snippet_for("kiro").is_some());
    }
}
