//! Centralized editor environment variable detection and resolution.
//!
//! Resolves `$EDITOR`, `$VISUAL`, and `$IDE` with wrapper-aware defaults.
//! The session wrapper type must be known before detection, ensuring
//! wrapper inference precedes editor defaults.

use crate::config::SessionWrapperType;

/// Resolved editor environment variables, detected once at startup.
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// Resolved `$EDITOR` value (fallback/terminal editor)
    pub editor: String,
    /// Resolved `$VISUAL` value (full-screen/GUI editor)
    pub visual: String,
}

impl EditorConfig {
    /// Detect editor configuration from environment variables,
    /// falling back to wrapper-specific defaults.
    ///
    /// The `wrapper` parameter enforces that wrapper inference is resolved
    /// before editor defaults are computed.
    pub fn detect(wrapper: SessionWrapperType) -> Self {
        let (default_editor, default_visual) = match wrapper {
            SessionWrapperType::Vscode => ("vim", "code --wait"),
            SessionWrapperType::Tmux | SessionWrapperType::Cmux | SessionWrapperType::Zellij => {
                ("vim", "")
            }
        };

        Self {
            editor: std::env::var("EDITOR").unwrap_or_else(|_| default_editor.to_string()),
            visual: std::env::var("VISUAL").unwrap_or_else(|_| default_visual.to_string()),
        }
    }

    /// Returns the command to use for editing files.
    /// Follows the convention: `$VISUAL || $EDITOR || "vim"`.
    pub fn file_editor(&self) -> &str {
        if !self.visual.is_empty() {
            &self.visual
        } else if !self.editor.is_empty() {
            &self.editor
        } else {
            "vim"
        }
    }

    /// Split a command string like `"code --wait"` into program and args.
    /// Returns `(program, args)`.
    pub fn split_command(cmd: &str) -> (&str, Vec<&str>) {
        let mut parts = cmd.split_whitespace();
        let program = parts.next().unwrap_or("vim");
        let args: Vec<&str> = parts.collect();
        (program, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize env-var-mutating tests
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper: clear editor env vars, run closure, restore.
    fn with_clean_env<F: FnOnce() -> R, R>(f: F) -> R {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved_editor = std::env::var("EDITOR").ok();
        let saved_visual = std::env::var("VISUAL").ok();

        std::env::remove_var("EDITOR");
        std::env::remove_var("VISUAL");

        let result = f();

        // Restore
        match saved_editor {
            Some(v) => std::env::set_var("EDITOR", v),
            None => std::env::remove_var("EDITOR"),
        }
        match saved_visual {
            Some(v) => std::env::set_var("VISUAL", v),
            None => std::env::remove_var("VISUAL"),
        }

        result
    }

    #[test]
    fn test_vscode_wrapper_defaults_when_env_unset() {
        with_clean_env(|| {
            let config = EditorConfig::detect(SessionWrapperType::Vscode);
            assert_eq!(config.editor, "vim");
            assert_eq!(config.visual, "code --wait");
        });
    }

    #[test]
    fn test_tmux_wrapper_defaults_when_env_unset() {
        with_clean_env(|| {
            let config = EditorConfig::detect(SessionWrapperType::Tmux);
            assert_eq!(config.editor, "vim");
            assert_eq!(config.visual, "");
        });
    }

    #[test]
    fn test_wrapper_inference_precedes_editor_defaults() {
        // Same empty environment, different wrapper → different defaults.
        // This proves wrapper type drives the defaults.
        with_clean_env(|| {
            let vscode = EditorConfig::detect(SessionWrapperType::Vscode);
            let tmux = EditorConfig::detect(SessionWrapperType::Tmux);

            // Vscode gets GUI-aware defaults
            assert_eq!(vscode.visual, "code --wait");

            // Tmux gets terminal-only defaults
            assert_eq!(tmux.visual, "");

            // Both share the same terminal editor fallback
            assert_eq!(vscode.editor, tmux.editor);
        });
    }

    #[test]
    fn test_env_vars_override_wrapper_defaults() {
        with_clean_env(|| {
            std::env::set_var("EDITOR", "nano");
            std::env::set_var("VISUAL", "subl -w");

            let config = EditorConfig::detect(SessionWrapperType::Vscode);
            assert_eq!(config.editor, "nano");
            assert_eq!(config.visual, "subl -w");
        });
    }

    #[test]
    fn test_partial_env_override() {
        with_clean_env(|| {
            std::env::set_var("EDITOR", "nano");
            // VISUAL not set — should get vscode default

            let config = EditorConfig::detect(SessionWrapperType::Vscode);
            assert_eq!(config.editor, "nano");
            assert_eq!(config.visual, "code --wait");
        });
    }

    #[test]
    fn test_file_editor_prefers_visual() {
        let config = EditorConfig {
            editor: "vim".into(),
            visual: "code --wait".into(),
        };
        assert_eq!(config.file_editor(), "code --wait");
    }

    #[test]
    fn test_file_editor_falls_back_to_editor() {
        let config = EditorConfig {
            editor: "nano".into(),
            visual: String::new(),
        };
        assert_eq!(config.file_editor(), "nano");
    }

    #[test]
    fn test_file_editor_ultimate_fallback() {
        let config = EditorConfig {
            editor: String::new(),
            visual: String::new(),
        };
        assert_eq!(config.file_editor(), "vim");
    }

    #[test]
    fn test_split_command_with_args() {
        let (prog, args) = EditorConfig::split_command("code --wait");
        assert_eq!(prog, "code");
        assert_eq!(args, vec!["--wait"]);
    }

    #[test]
    fn test_split_command_no_args() {
        let (prog, args) = EditorConfig::split_command("vim");
        assert_eq!(prog, "vim");
        assert!(args.is_empty());
    }

    #[test]
    fn test_split_command_multiple_args() {
        let (prog, args) = EditorConfig::split_command("subl -w --new-window");
        assert_eq!(prog, "subl");
        assert_eq!(args, vec!["-w", "--new-window"]);
    }

    #[test]
    fn test_cmux_and_zellij_match_tmux_defaults() {
        with_clean_env(|| {
            let cmux = EditorConfig::detect(SessionWrapperType::Cmux);
            let zellij = EditorConfig::detect(SessionWrapperType::Zellij);
            let tmux = EditorConfig::detect(SessionWrapperType::Tmux);

            assert_eq!(cmux.editor, tmux.editor);
            assert_eq!(cmux.visual, tmux.visual);

            assert_eq!(zellij.editor, tmux.editor);
            assert_eq!(zellij.visual, tmux.visual);
        });
    }
}
