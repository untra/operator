//! API state management for the REST server.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use std::sync::RwLock as StdRwLock;

use tokio::sync::{Mutex, RwLock};

use crate::api::kanban_sync::KanbanBidirectionalSync;
use crate::auth::store::AuthStore;
use crate::auth::tokens::SigningKey;
use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;
use crate::rest::dto::auth::Scope;
use crate::rest::error::ApiError;
use crate::startup::templates::load_registry;

/// Shared state for the REST API
#[derive(Clone)]
pub struct ApiState {
    /// Issue type registry (thread-safe read-write access)
    pub registry: Arc<RwLock<IssueTypeRegistry>>,
    /// Live application configuration.
    config: Arc<StdRwLock<Arc<Config>>>,
    /// Serializes read-modify-write configuration updates.
    config_update: Arc<Mutex<()>>,
    /// Path to tickets directory for persistence
    pub tickets_path: PathBuf,
    /// Active MCP SSE sessions, each bound to the principal that opened it.
    pub mcp_sessions: Arc<Mutex<HashMap<String, McpSession>>>,
    /// Bidirectional kanban sync service (present only when at least one project has
    /// `bidirectional: true` in its sync config).
    pub kanban_sync: Option<Arc<KanbanBidirectionalSync>>,
    /// Authentication store and signing key.
    pub auth: Arc<AuthContext>,
}

/// An open MCP SSE session.
///
/// The principal is captured when the stream is established and re-checked on
/// every message: the session id alone is a bearer credential, and binding it
/// to an identity means a leaked id cannot be used by anyone else.
pub struct McpSession {
    /// Channel that relays JSON-RPC responses back over the SSE stream.
    pub tx: tokio::sync::mpsc::UnboundedSender<String>,
    /// Who opened the stream.
    pub subject: String,
    /// Scopes that principal held.
    pub scopes: Vec<Scope>,
}

/// Authentication state shared by every surface.
pub struct AuthContext {
    /// The credential database.
    pub store: AuthStore,
    /// Active token signing key.
    pub signing_key: SigningKey,
    /// The local-unlock token, when the server is bound to loopback.
    pub local_token: Option<String>,
}

impl AuthContext {
    /// Open the auth database and load or create the signing key.
    ///
    /// `bind_addr` decides whether a local-unlock token is issued: loopback
    /// gets one, anything else does not and must bootstrap.
    pub fn initialize(state_path: PathBuf, bind_addr: std::net::IpAddr) -> anyhow::Result<Self> {
        let store = AuthStore::open(&state_path)?;
        let signing_key = store.load_or_create_signing_key()?;

        let local_token = if crate::auth::local::is_loopback(bind_addr) {
            Some(crate::auth::local::issue(&state_path)?)
        } else {
            // Leaving a stale token behind would hand a local credential to a
            // publicly bound server.
            crate::auth::local::revoke(&state_path);
            None
        };

        Ok(Self {
            store,
            signing_key,
            local_token,
        })
    }
}

impl ApiState {
    /// Create new API state from config
    ///
    /// Loading priority:
    /// 1. Try to load from .tickets/templates/ directory (new collection-scoped structure)
    /// 2. If empty, initialize default templates from embedded files
    /// 3. Fallback to embedded builtins if filesystem loading fails
    pub fn new(config: Config, tickets_path: PathBuf) -> Self {
        let state_path = config.state_path();
        let bind_addr = config.rest_api.host_ip();
        let auth = AuthContext::initialize(state_path, bind_addr)
            .expect("auth store must be available; without it nothing can authenticate");
        Self::with_auth(config, tickets_path, Arc::new(auth))
    }

    /// Build with a caller-supplied auth context, so tests and the TUI can share
    /// one already-open store instead of racing to open the same database file.
    pub fn with_auth(config: Config, tickets_path: PathBuf, auth: Arc<AuthContext>) -> Self {
        // Shared loader; keeps the API's issue-type resolution identical to the CLI/TUI `workflow export` produces the same output on every surface.
        let registry = load_registry(&tickets_path);

        let config_arc = Arc::new(config);
        let kanban_sync = {
            let ks = KanbanBidirectionalSync::new(Arc::clone(&config_arc));
            if ks.has_any_bidirectional() {
                Some(Arc::new(ks))
            } else {
                None
            }
        };

        Self {
            registry: Arc::new(RwLock::new(registry)),
            config: Arc::new(StdRwLock::new(config_arc)),
            config_update: Arc::new(Mutex::new(())),
            tickets_path,
            mcp_sessions: Arc::new(Mutex::new(HashMap::new())),
            kanban_sync,
            auth,
        }
    }

    /// A snapshot of the live configuration.
    ///
    /// Clones the inner `Arc` so a handler reads a consistent view for the duration of a request.
    pub fn config(&self) -> Arc<Config> {
        match self.config.read() {
            Ok(guard) => Arc::clone(&guard),
            Err(poisoned) => Arc::clone(&poisoned.into_inner()),
        }
    }

    /// Replace the live configuration after a successful write to disk.
    ///
    /// This is the other half of fixing the stale-snapshot bug: writing
    /// `config.toml` without calling this leaves every handler reading the
    /// values the process started with.
    pub fn replace_config(&self, config: Config) {
        let next = Arc::new(config);
        match self.config.write() {
            Ok(mut guard) => *guard = next,
            Err(poisoned) => *poisoned.into_inner() = next,
        }
    }

    /// Mutate and persist the latest live configuration as one serialized operation.
    pub async fn mutate_config<T>(
        &self,
        mutate: impl FnOnce(&mut Config) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        let _guard = self.config_update.lock().await;
        let mut config = (*self.config()).clone();
        let result = mutate(&mut config)?;
        config
            .save()
            .map_err(|error| ApiError::InternalError(format!("Failed to save config: {error}")))?;
        self.replace_config(config);
        Ok(result)
    }

    /// Get the templates directory path
    pub fn templates_path(&self) -> PathBuf {
        self.tickets_path.join("templates")
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
    fn test_kanban_sync_none_when_no_bidirectional_projects() {
        // Default config has no kanban projects configured, so kanban_sync is None.
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));
        assert!(
            state.kanban_sync.is_none(),
            "kanban_sync should be None when no bidirectional projects are configured"
        );
    }

    #[test]
    fn test_kanban_sync_some_when_bidirectional_project_configured() {
        use crate::config::{JiraConfig, KanbanConfig, ProjectSyncConfig};
        use std::collections::HashMap;

        let mut project_sync = ProjectSyncConfig {
            sync_user_id: String::new(),
            status_mapping: crate::config::KanbanStatusMapping::default(),
            collection_name: None,
            type_mappings: HashMap::new(),
            bidirectional: true,
            ticket_project: None,
        };
        let _ = &mut project_sync; // suppress unused_mut if needed

        let mut projects = HashMap::new();
        projects.insert("MY-PROJECT".to_string(), project_sync);

        let jira_config = JiraConfig {
            enabled: true,
            api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
            email: "test@example.com".to_string(),
            projects,
        };

        let mut jira_map = HashMap::new();
        jira_map.insert("test.atlassian.net".to_string(), jira_config);

        let config = Config {
            kanban: KanbanConfig {
                jira: jira_map,
                linear: HashMap::new(),
                github: HashMap::new(),
                openspec: HashMap::new(),
            },
            ..Default::default()
        };

        let state = ApiState::new(config, PathBuf::from("/tmp/test"));
        assert!(
            state.kanban_sync.is_some(),
            "kanban_sync should be Some when at least one project has bidirectional: true"
        );
    }
}
