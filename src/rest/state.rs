//! API state management for the REST server.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;
use crate::startup::templates::init_default_templates;

/// Shared state for the REST API
#[derive(Clone)]
pub struct ApiState {
    /// Issue type registry (thread-safe read-write access)
    pub registry: Arc<RwLock<IssueTypeRegistry>>,
    /// Application configuration (reserved for future CORS/auth settings)
    #[allow(dead_code)]
    pub config: Arc<Config>,
    /// Path to tickets directory for persistence
    pub tickets_path: PathBuf,
}

impl ApiState {
    /// Create new API state from config
    ///
    /// Loading priority:
    /// 1. Try to load from .tickets/templates/ directory (new collection-scoped structure)
    /// 2. If empty, initialize default templates from embedded files
    /// 3. Fallback to embedded builtins if filesystem loading fails
    pub fn new(config: Config, tickets_path: PathBuf) -> Self {
        let mut registry = IssueTypeRegistry::new();
        let templates_path = tickets_path.join("templates");

        // Try to load from templates directory first
        match registry.load_from_templates_dir(&templates_path) {
            Ok(()) if registry.type_count() > 0 => {
                tracing::info!(
                    "Loaded {} issue types from templates directory",
                    registry.type_count()
                );
            }
            Ok(()) => {
                // Templates directory empty or doesn't exist - initialize defaults
                tracing::info!("Templates directory empty, initializing defaults...");
                if let Err(e) = init_default_templates(&templates_path) {
                    tracing::warn!("Failed to initialize default templates: {}", e);
                } else {
                    // Try loading again after initialization
                    if let Err(e) = registry.load_from_templates_dir(&templates_path) {
                        tracing::warn!("Failed to load initialized templates: {}", e);
                    }
                }

                // If still empty, fallback to embedded builtins
                if registry.type_count() == 0 {
                    tracing::info!("Falling back to embedded builtin types");
                    if let Err(e) = registry.load_builtins() {
                        tracing::warn!("Failed to load builtin issue types: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load from templates directory: {}", e);
                // Fallback to embedded builtins
                if let Err(e) = registry.load_builtins() {
                    tracing::warn!("Failed to load builtin issue types: {}", e);
                }
            }
        }

        Self {
            registry: Arc::new(RwLock::new(registry)),
            config: Arc::new(config),
            tickets_path,
        }
    }

    /// Get the templates directory path
    #[allow(dead_code)] // Reserved for future use in REST API
    pub fn templates_path(&self) -> PathBuf {
        self.tickets_path.join("templates")
    }

    /// Get the issuetypes directory path (legacy)
    pub fn issuetypes_path(&self) -> PathBuf {
        self.tickets_path.join("operator/issuetypes")
    }

    /// Ensure the issuetypes directory exists
    pub async fn ensure_issuetypes_dir(&self) -> std::io::Result<()> {
        tokio::fs::create_dir_all(self.issuetypes_path()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_state_new() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        // Registry should have builtins loaded
        let registry = state.registry.blocking_read();
        assert!(registry.type_count() >= 5); // At least builtin types
    }

    #[test]
    fn test_issuetypes_path() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/tickets"));

        assert_eq!(
            state.issuetypes_path(),
            PathBuf::from("/tmp/tickets/operator/issuetypes")
        );
    }
}
