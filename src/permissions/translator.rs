//! Permission translator for generating provider-specific configs

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::{ClaudeTranslator, CodexTranslator, GeminiTranslator, PermissionSet};

/// Result of generating provider config
#[derive(Debug)]
pub struct GeneratedConfig {
    /// Path where config was written (if applicable)
    pub config_path: Option<PathBuf>,
    /// CLI flags to add to the command
    pub cli_flags: Vec<String>,
    /// Full command for auditing purposes
    pub audit_info: String,
}

/// Trait for translating permissions to provider-specific format
pub trait PermissionTranslator: Send + Sync {
    /// Provider name (claude, gemini, codex)
    fn provider_name(&self) -> &str;

    /// Generate CLI flags from permissions
    fn generate_cli_flags(&self, permissions: &PermissionSet) -> Vec<String>;

    /// Generate config file content (if this provider uses config files)
    fn generate_config_content(&self, permissions: &PermissionSet) -> Option<String>;

    /// Get the relative path for the config file (if this provider uses config files)
    fn config_path(&self) -> Option<&str>;

    /// Whether this provider uses CLI args only (no config file)
    fn uses_cli_only(&self) -> bool {
        self.config_path().is_none()
    }
}

/// Manager for selecting and using the appropriate translator
pub struct TranslatorManager {
    translators: Vec<Box<dyn PermissionTranslator>>,
}

impl Default for TranslatorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslatorManager {
    /// Create a new TranslatorManager with all supported translators
    pub fn new() -> Self {
        Self {
            translators: vec![
                Box::new(ClaudeTranslator),
                Box::new(GeminiTranslator),
                Box::new(CodexTranslator),
            ],
        }
    }

    /// Get translator for a specific provider
    pub fn get(&self, provider: &str) -> Option<&dyn PermissionTranslator> {
        self.translators
            .iter()
            .find(|t| t.provider_name() == provider)
            .map(|t| t.as_ref())
    }

    /// Generate config and CLI flags for a provider
    ///
    /// # Arguments
    /// * `provider` - Provider name (claude, gemini, codex)
    /// * `permissions` - Merged permission set
    /// * `session_dir` - Directory to store session configs for auditing
    ///
    /// # Returns
    /// GeneratedConfig containing:
    /// - Optional config file path (for providers that use config files)
    /// - CLI flags to add to the command
    /// - Audit info string
    pub fn generate_config(
        &self,
        provider: &str,
        permissions: &PermissionSet,
        session_dir: &Path,
    ) -> Result<GeneratedConfig> {
        let translator = self
            .get(provider)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider))?;

        // Generate CLI flags
        let mut cli_flags = translator.generate_cli_flags(permissions);

        // Add provider-specific CLI args from step definition
        let step_cli_args = match provider {
            "claude" => &permissions.cli_args.claude,
            "gemini" => &permissions.cli_args.gemini,
            "codex" => &permissions.cli_args.codex,
            _ => &Vec::new(),
        };
        cli_flags.extend(step_cli_args.iter().cloned());

        // Generate config file if provider uses config files
        let config_path = if let Some(relative_path) = translator.config_path() {
            if let Some(content) = translator.generate_config_content(permissions) {
                let full_path = session_dir.join(relative_path);

                // Ensure parent directories exist
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create config dir: {:?}", parent))?;
                }

                fs::write(&full_path, &content)
                    .with_context(|| format!("Failed to write config file: {:?}", full_path))?;

                // Add CLI flag to point to config directory
                match provider {
                    "gemini" => {
                        cli_flags.push("--config-dir".to_string());
                        cli_flags.push(session_dir.display().to_string());
                    }
                    "codex" => {
                        cli_flags.push("--config-dir".to_string());
                        cli_flags.push(session_dir.display().to_string());
                    }
                    _ => {}
                }

                Some(full_path)
            } else {
                None
            }
        } else {
            None
        };

        // Create audit info
        let audit_info = format!(
            "Provider: {}\nCLI Flags: {:?}\nConfig Path: {:?}\n",
            provider, cli_flags, config_path
        );

        Ok(GeneratedConfig {
            config_path,
            cli_flags,
            audit_info,
        })
    }

    /// Save audit information for a session
    pub fn save_audit_info(
        session_dir: &Path,
        provider: &str,
        config: &GeneratedConfig,
        full_command: &str,
    ) -> Result<()> {
        fs::create_dir_all(session_dir)
            .with_context(|| format!("Failed to create session dir: {:?}", session_dir))?;

        // Save launch command
        let command_path = session_dir.join("launch-command.txt");
        fs::write(&command_path, full_command)
            .with_context(|| format!("Failed to write launch command: {:?}", command_path))?;

        // Save audit info
        let audit_path = session_dir.join(format!("{}-audit.txt", provider));
        fs::write(&audit_path, &config.audit_info)
            .with_context(|| format!("Failed to write audit info: {:?}", audit_path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_manager_get() {
        let manager = TranslatorManager::new();

        assert!(manager.get("claude").is_some());
        assert!(manager.get("gemini").is_some());
        assert!(manager.get("codex").is_some());
        assert!(manager.get("unknown").is_none());
    }

    #[test]
    fn test_translator_manager_provider_names() {
        let manager = TranslatorManager::new();

        assert_eq!(manager.get("claude").unwrap().provider_name(), "claude");
        assert_eq!(manager.get("gemini").unwrap().provider_name(), "gemini");
        assert_eq!(manager.get("codex").unwrap().provider_name(), "codex");
    }

    #[test]
    fn test_claude_uses_cli_only() {
        let manager = TranslatorManager::new();
        let claude = manager.get("claude").unwrap();
        assert!(claude.uses_cli_only());
    }

    #[test]
    fn test_gemini_uses_config_file() {
        let manager = TranslatorManager::new();
        let gemini = manager.get("gemini").unwrap();
        assert!(!gemini.uses_cli_only());
        assert!(gemini.config_path().is_some());
    }

    #[test]
    fn test_codex_uses_config_file() {
        let manager = TranslatorManager::new();
        let codex = manager.get("codex").unwrap();
        assert!(!codex.uses_cli_only());
        assert!(codex.config_path().is_some());
    }
}
