//! REST API server lifecycle management.
//!
//! Provides a lifecycle manager for the REST API server that can be started,
//! stopped, and queried for status. Designed to run alongside the TUI.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::rest::{build_router, ApiState};

/// Session info written when API server starts, for client discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSessionInfo {
    pub port: u16,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default)]
    pub profile_id: uuid::Uuid,
    pub pid: u32,
    pub started_at: String,
    pub version: String,
    /// State directory holding `local-token`, so a same-host client (the VS
    /// Code extension) can find the local credential when `paths.state` is
    /// not the default.
    pub state_dir: PathBuf,
}

/// Write API session file for client discovery
pub(super) fn write_session_file(
    tickets_path: &Path,
    state_path: &Path,
    addr: std::net::SocketAddr,
    profile_id: uuid::Uuid,
    local_token: Option<&str>,
) -> std::io::Result<PathBuf> {
    let operator_dir = tickets_path.join("operator");
    std::fs::create_dir_all(&operator_dir)?;

    let session_file = operator_dir.join("api-session.json");
    let origin = format!("http://{addr}");
    let registration_path = operator_dir.join(crate::auth::local::LOCAL_REGISTRATION_FILENAME);
    if let Some(token) = local_token {
        let registration = crate::auth::local::LocalRegistration {
            origin: origin.clone(),
            token: token.to_string(),
            tickets: tickets_path.canonicalize()?,
            pid: std::process::id(),
        };
        crate::auth::local::write_owner_only(
            &registration_path,
            &serde_json::to_string(&registration)?,
        )?;
    } else if let Err(error) = std::fs::remove_file(&registration_path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            return Err(error);
        }
    }
    let session = super::ApiSessionInfo {
        port: addr.port(),
        origin: local_token.map(|_| origin),
        profile_id,
        pid: std::process::id(),
        started_at: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        state_dir: state_path.to_path_buf(),
    };

    let json = serde_json::to_string_pretty(&session)?;
    crate::auth::local::write_owner_only(&session_file, &json)?;

    tracing::debug!(path = %session_file.display(), "Wrote API session file");
    Ok(session_file)
}

/// Remove API session file on shutdown
pub(super) fn remove_session_file(tickets_path: &Path) {
    let _ = std::fs::remove_file(
        tickets_path
            .join("operator")
            .join(crate::auth::local::LOCAL_REGISTRATION_FILENAME),
    );
    let session_file = tickets_path.join("operator").join("api-session.json");
    if session_file.exists() {
        if let Err(e) = std::fs::remove_file(&session_file) {
            tracing::warn!(error = %e, "Failed to remove API session file");
        } else {
            tracing::debug!(path = %session_file.display(), "Removed API session file");
        }
    }
}

/// Status of the REST API server
#[derive(Debug, Clone, PartialEq)]
pub enum RestApiStatus {
    Stopped,
    Starting,
    Stopping,
    /// This process owns the server listening on `port`.
    Running {
        port: u16,
    },
    Error(String),
}

impl RestApiStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, RestApiStatus::Running { .. })
    }
}

/// Result of probing an API already listening on the expected port, used to
/// decide whether to adopt it as "connected" or report a clear conflict.
#[derive(Debug, Clone, PartialEq)]
pub enum ExternalApiProbe {
    /// A same-version operator API serving the same project - safe to adopt.
    AdoptableSameProject { port: u16, version: String },
    /// An operator API of a different version is on the port.
    VersionMismatch { found: String },
    /// A same-version operator API serving a *different* project is on the port.
    DifferentProject { found_name: String },
    /// Something is listening, but it is not an operator API.
    NotOperator,
    /// Nothing answered (port held by a process that isn't accepting HTTP, etc.).
    Unreachable,
}

/// Tolerant view of `/api/v1/health` - older or foreign servers may omit fields.
#[derive(Debug, Clone, serde::Deserialize)]
struct ProbeHealth {
    #[serde(default)]
    version: String,
    #[serde(default)]
    directory_name: String,
    #[serde(default)]
    directory_id: String,
}

/// Decide how to treat an API on the port, given the local identity and the
/// health payload it returned. Pure so the decision is unit-testable.
fn classify_probe(
    port: u16,
    local_version: &str,
    local_directory_id: &str,
    health: &ProbeHealth,
) -> ExternalApiProbe {
    if health.version.is_empty() {
        // Responded, but without a version - not an operator health endpoint.
        ExternalApiProbe::NotOperator
    } else if health.version != local_version {
        ExternalApiProbe::VersionMismatch {
            found: health.version.clone(),
        }
    } else if health.directory_id != local_directory_id {
        ExternalApiProbe::DifferentProject {
            found_name: health.directory_name.clone(),
        }
    } else {
        ExternalApiProbe::AdoptableSameProject {
            port,
            version: health.version.clone(),
        }
    }
}

/// REST API server handle for lifecycle management
pub struct RestApiServer {
    auth: Option<Arc<crate::rest::state::AuthContext>>,
    config: Config,
    port: u16,
    tickets_path: PathBuf,
    status: Arc<Mutex<RestApiStatus>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    #[allow(dead_code)]
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Live `ApiState` once `start()` has been called. Exposed so the
    /// dashboard can read runtime info (e.g. active MCP SSE sessions).
    api_state: Arc<Mutex<Option<ApiState>>>,
}

impl RestApiServer {
    /// Create a new server handle
    pub fn new(config: Config, port: u16) -> Self {
        let tickets_path = config.tickets_path();
        Self {
            auth: None,
            config,
            port,
            tickets_path,
            status: Arc::new(Mutex::new(RestApiStatus::Stopped)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            task_handle: Arc::new(Mutex::new(None)),
            api_state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_auth(mut self, auth: Arc<crate::rest::state::AuthContext>) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Returns a clone of the live `ApiState` if the server has been started.
    /// `ApiState` is `Clone` with internal `Arc`s, so the clone shares the
    /// same `mcp_sessions` map as the running server.
    pub fn api_state(&self) -> Option<ApiState> {
        self.api_state.lock().unwrap().clone()
    }

    /// Get current server status
    pub fn status(&self) -> RestApiStatus {
        self.status.lock().unwrap().clone()
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.status().is_running()
    }

    /// Get the port
    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if the configured port is already in use by another process
    ///
    /// This is useful for detecting if another operator instance is already running.
    pub async fn is_port_in_use(&self) -> bool {
        use std::net::SocketAddr;
        let addr = SocketAddr::new(self.config.rest_api.host_ip(), self.port);
        tokio::net::TcpListener::bind(addr).await.is_err()
    }

    /// Probe an API already listening on the configured port to decide whether
    /// it is a same-version operator serving this project (safe to adopt) or a
    /// conflict we should report. Always connects over loopback.
    pub async fn probe_external(&self) -> ExternalApiProbe {
        let origin = format!(
            "http://{}",
            std::net::SocketAddr::new(self.config.rest_api.host_ip(), self.port)
        );
        let url = format!("{origin}/api/v1/health");
        let client = match crate::http_client::client_builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return ExternalApiProbe::Unreachable,
        };

        // `/api/v1/health` now requires a credential, so the probe presents the
        // local-unlock token from *this* project's state directory.
        let mut request = client.get(&url);
        if let Some(token) = crate::auth::local::registered_token(&self.config, &origin) {
            request = request.bearer_auth(token);
        }

        let response = match request.send().await {
            Ok(r) => r,
            Err(_) => return ExternalApiProbe::Unreachable,
        };

        if response.status() == reqwest::StatusCode::UNAUTHORIZED
            || response.status() == reqwest::StatusCode::FORBIDDEN
        {
            // An operator server we cannot authenticate to is, for adoption
            // purposes, someone else's workspace.
            return ExternalApiProbe::DifferentProject {
                found_name: String::new(),
            };
        }

        // A 2xx that doesn't deserialize into a health shape is "not operator".
        let health = match response.json::<ProbeHealth>().await {
            Ok(h) => h,
            Err(_) => return ExternalApiProbe::NotOperator,
        };

        let (_local_name, local_id) =
            crate::rest::directory::directory_identity(&self.tickets_path);
        classify_probe(self.port, env!("CARGO_PKG_VERSION"), &local_id, &health)
    }

    /// Start the REST API server
    pub fn start(&self) -> Result<(), String> {
        if self.is_running()
            || self
                .task_handle
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|task| !task.is_finished())
        {
            return Err(format!("REST API already running on port {}", self.port));
        }
        *self.status.lock().unwrap() = RestApiStatus::Starting;
        let result = self.start_owned();
        if let Err(error) = &result {
            *self.status.lock().unwrap() = RestApiStatus::Error(error.clone());
        }
        result
    }

    fn start_owned(&self) -> Result<(), String> {
        let state = match &self.auth {
            Some(auth) => ApiState::try_with_auth(
                self.config.clone(),
                self.config.tickets_path(),
                Arc::clone(auth),
            ),
            None => ApiState::try_new(self.config.clone(), self.config.tickets_path()),
        }
        .map_err(|error| error.to_string())?;
        crate::rest::state::AuthContext::validate_listener(
            &self.config,
            state.auth.deployment_mode,
        )
        .map_err(|e| e.to_string())?;
        let router = build_router(state.clone()).map_err(|error| error.to_string())?;
        let addr = std::net::SocketAddr::new(self.config.rest_api.host_ip(), self.port);
        let listener = std::net::TcpListener::bind(addr).map_err(|error| {
            *self.status.lock().unwrap() = RestApiStatus::Error(error.to_string());
            error.to_string()
        })?;
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let listener =
            tokio::net::TcpListener::from_std(listener).map_err(|error| error.to_string())?;
        let bound_addr = listener.local_addr().map_err(|error| error.to_string())?;
        write_session_file(
            &self.tickets_path,
            &state.config().auth_state_path(),
            bound_addr,
            state.config().profile.id,
            state.auth.local_token.as_deref(),
        )
        .map_err(|error| error.to_string())?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);
        *self.api_state.lock().unwrap() = Some(state.clone());
        let status = Arc::clone(&self.status);
        let tickets_path = self.tickets_path.clone();
        let port = bound_addr.port();
        let api_state_handle = Arc::clone(&self.api_state);
        *status.lock().unwrap() = RestApiStatus::Running { port };
        let handle = tokio::spawn(async move {
            state.auth.store.start_usage_tracking();
            let shutdown_state = state.clone();
            let result = axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
                shutdown_state.start_stopping();
            })
            .await;
            state.wait_until_idle().await;
            state.auth.store.stop_usage_tracking().await;
            if let Err(error) = state.auth.store.flush_usage() {
                tracing::warn!(%error, "Final credential usage flush failed");
            }
            remove_session_file(&tickets_path);
            *api_state_handle.lock().unwrap() = None;
            *status.lock().unwrap() = match result {
                Ok(()) => RestApiStatus::Stopped,
                Err(error) => RestApiStatus::Error(error.to_string()),
            };
        });
        *self.task_handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    /// Record that the port is occupied by something we cannot adopt (different
    /// version, different project, or a non-operator process). Surfaced to the
    /// operator via the dashboard's REST API status line.
    pub fn mark_conflict(&self, reason: impl Into<String>) {
        *self.status.lock().unwrap() = RestApiStatus::Error(reason.into());
    }

    /// Stop the REST API server
    pub fn stop(&self) {
        let mut status = self.status.lock().unwrap();
        if matches!(*status, RestApiStatus::Stopped | RestApiStatus::Error(_)) {
            return;
        }
        *status = RestApiStatus::Stopping;
        drop(status);
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
    }

    pub async fn shutdown(&self) {
        self.stop();
        let task = self.task_handle.lock().unwrap().take();
        if let Some(task) = task {
            let _ = task.await;
        }
    }

    /// Toggle server state (start if stopped, stop if running)
    #[allow(dead_code)]
    pub fn toggle(&self) -> Result<(), String> {
        if self.is_running() {
            self.stop();
            Ok(())
        } else {
            self.start()
        }
    }
}

impl Drop for RestApiServer {
    fn drop(&mut self) {
        if self.is_running() {
            self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_api_status_is_running() {
        assert!(!RestApiStatus::Stopped.is_running());
        assert!(!RestApiStatus::Starting.is_running());
        assert!(!RestApiStatus::Stopping.is_running());
        assert!(RestApiStatus::Running { port: 7008 }.is_running());
        assert!(!RestApiStatus::Error("test".to_string()).is_running());
    }

    fn probe_health(version: &str, dir_id: &str, dir_name: &str) -> ProbeHealth {
        ProbeHealth {
            version: version.to_string(),
            directory_name: dir_name.to_string(),
            directory_id: dir_id.to_string(),
        }
    }

    #[test]
    fn test_classify_adoptable_same_version_same_project() {
        let h = probe_health("0.2.1", "abc123abc123", "acme");
        let result = classify_probe(7008, "0.2.1", "abc123abc123", &h);
        assert_eq!(
            result,
            ExternalApiProbe::AdoptableSameProject {
                port: 7008,
                version: "0.2.1".to_string()
            }
        );
    }

    #[test]
    fn test_classify_version_mismatch() {
        let h = probe_health("0.3.0", "abc123abc123", "acme");
        let result = classify_probe(7008, "0.2.1", "abc123abc123", &h);
        assert_eq!(
            result,
            ExternalApiProbe::VersionMismatch {
                found: "0.3.0".to_string()
            }
        );
    }

    #[test]
    fn test_classify_different_project() {
        // Same version, different directory id -> must NOT adopt.
        let h = probe_health("0.2.1", "deadbeef0000", "other-repo");
        let result = classify_probe(7008, "0.2.1", "abc123abc123", &h);
        assert_eq!(
            result,
            ExternalApiProbe::DifferentProject {
                found_name: "other-repo".to_string()
            }
        );
    }

    #[test]
    fn test_classify_not_operator_when_no_version() {
        let h = probe_health("", "", "");
        let result = classify_probe(7008, "0.2.1", "abc123abc123", &h);
        assert_eq!(result, ExternalApiProbe::NotOperator);
    }

    #[tokio::test]
    async fn test_probe_external_unreachable_when_nothing_listening() {
        // Port 1 is privileged/closed; nothing answers -> Unreachable.
        let config = Config::default();
        let server = RestApiServer::new(config, 1);
        assert_eq!(server.probe_external().await, ExternalApiProbe::Unreachable);
    }

    #[test]
    fn test_rest_api_server_initial_status() {
        let config = Config::default();
        let server = RestApiServer::new(config, 7008);
        assert_eq!(server.status(), RestApiStatus::Stopped);
        assert!(!server.is_running());
        assert!(
            server.api_state().is_none(),
            "api_state should be unset before start()"
        );
    }

    #[tokio::test]
    async fn test_both_listeners_reject_invalid_deployment_before_registration() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = dir.path().join("state").to_string_lossy().into_owned();
        config.paths.tickets = dir.path().join("tickets").to_string_lossy().into_owned();
        config.profile_registry = Some(dir.path().join("profiles.sqlite"));
        config.rest_api.host = "0.0.0.0".to_string();
        for mode in [
            crate::config::DeploymentMode::Local,
            crate::config::DeploymentMode::Hosted,
        ] {
            config.rest_api.deployment_mode = mode;
            let state = ApiState::in_memory(config.clone());
            assert!(crate::rest::serve(state, 0, false).await.is_err());
            let server = RestApiServer::new(config.clone(), 0);
            assert!(server.start().is_err());
            assert!(!server.is_running());
            assert!(!config
                .tickets_path()
                .join("operator/api-session.json")
                .exists());
            assert!(!crate::auth::local::local_token_path(&config.state_path()).exists());
        }
    }

    #[tokio::test]
    async fn test_start_exposes_api_state() {
        // Use port 0 to let the OS pick a free port (avoids collisions).
        let config = Config::default();
        let server = RestApiServer::new(config, 0);
        server.start().expect("start should succeed");
        assert!(server.is_running());
        assert!(
            server.api_state().is_some(),
            "api_state should be populated after start()"
        );
        server.shutdown().await;
    }

    #[test]
    fn test_rest_api_server_port() {
        let config = Config::default();
        let server = RestApiServer::new(config, 8080);
        assert_eq!(server.port(), 8080);
    }

    #[test]
    fn test_server_double_start_error() {
        let config = Config::default();
        let server = RestApiServer::new(config, 7008);

        // Simulate server already running by setting status
        *server.status.lock().unwrap() = RestApiStatus::Running { port: 7008 };

        let result = server.start();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("already running"));
    }

    #[test]
    fn test_server_status_transitions() {
        let config = Config::default();
        let server = RestApiServer::new(config, 7008);

        // Initially stopped
        assert_eq!(server.status(), RestApiStatus::Stopped);

        // Manually set to starting
        *server.status.lock().unwrap() = RestApiStatus::Starting;
        assert!(!server.is_running());

        // Manually set to running
        *server.status.lock().unwrap() = RestApiStatus::Running { port: 7008 };
        assert!(server.is_running());

        // Manually set to stopping
        *server.status.lock().unwrap() = RestApiStatus::Stopping;
        assert!(!server.is_running());

        // Error state
        *server.status.lock().unwrap() = RestApiStatus::Error("test error".to_string());
        assert!(!server.is_running());
    }

    #[test]
    fn test_server_stop_clears_shutdown_tx() {
        let config = Config::default();
        let server = RestApiServer::new(config, 7008);

        // Simulate a running server with a shutdown channel
        let (tx, _rx) = tokio::sync::oneshot::channel();
        *server.shutdown_tx.lock().unwrap() = Some(tx);
        *server.status.lock().unwrap() = RestApiStatus::Running { port: 7008 };

        server.stop();

        // Admission closes before the server finishes draining.
        assert_eq!(server.status(), RestApiStatus::Stopping);
        // Shutdown tx should be taken (None)
        assert!(server.shutdown_tx.lock().unwrap().is_none());
    }

    #[test]
    fn test_write_session_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let port = 7008u16;

        let state_dir = temp_dir.path().join("custom-state");
        let profile_id = uuid::Uuid::new_v4();
        let result = write_session_file(
            temp_dir.path(),
            &state_dir,
            ([127, 0, 0, 1], port).into(),
            profile_id,
            None,
        );
        assert!(result.is_ok());

        let session_file = temp_dir.path().join("operator").join("api-session.json");
        assert!(session_file.exists(), "Session file should exist");

        let content = std::fs::read_to_string(&session_file).unwrap();
        let session: ApiSessionInfo = serde_json::from_str(&content).unwrap();

        assert_eq!(session.port, port);
        assert_eq!(session.profile_id, profile_id);
        assert!(!session.version.is_empty());
        assert!(session.pid > 0);
        assert_eq!(
            session.state_dir, state_dir,
            "clients locate local-token through the advertised state dir"
        );
    }

    #[test]
    fn test_write_session_file_creates_operator_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Operator dir doesn't exist yet
        let operator_dir = temp_dir.path().join("operator");
        assert!(!operator_dir.exists());

        let result = write_session_file(
            temp_dir.path(),
            &operator_dir,
            ([127, 0, 0, 1], 7008).into(),
            uuid::Uuid::nil(),
            None,
        );
        assert!(result.is_ok());

        // Should have created the operator directory
        assert!(operator_dir.exists());
        assert!(operator_dir.join("api-session.json").exists());
    }

    #[test]
    fn test_remove_session_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let operator_dir = temp_dir.path().join("operator");
        std::fs::create_dir_all(&operator_dir).unwrap();

        let session_file = operator_dir.join("api-session.json");
        std::fs::write(&session_file, "{}").unwrap();

        assert!(session_file.exists());
        remove_session_file(temp_dir.path());
        assert!(!session_file.exists(), "Session file should be removed");
    }

    #[test]
    fn test_remove_session_file_nonexistent() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Should not panic when file doesn't exist
        remove_session_file(temp_dir.path());
    }
}
