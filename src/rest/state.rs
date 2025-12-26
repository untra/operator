//! API state management for the REST server.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;

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
    pub fn new(config: Config, tickets_path: PathBuf) -> Self {
        let mut registry = IssueTypeRegistry::new();

        // Load builtins
        if let Err(e) = registry.load_builtins() {
            tracing::warn!("Failed to load builtin issue types: {}", e);
        }

        // Load user types and collections from tickets path
        if let Err(e) = registry.load_all(&tickets_path) {
            tracing::warn!("Failed to load user issue types: {}", e);
        }

        Self {
            registry: Arc::new(RwLock::new(registry)),
            config: Arc::new(config),
            tickets_path,
        }
    }

    /// Get the issuetypes directory path
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
