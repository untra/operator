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
    pub pid: u32,
    pub started_at: String,
    pub version: String,
}

/// Write API session file for client discovery
fn write_session_file(tickets_path: &Path, port: u16) -> std::io::Result<PathBuf> {
    let operator_dir = tickets_path.join("operator");
    std::fs::create_dir_all(&operator_dir)?;

    let session_file = operator_dir.join("api-session.json");
    let session = ApiSessionInfo {
        port,
        pid: std::process::id(),
        started_at: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let json = serde_json::to_string_pretty(&session)?;
    std::fs::write(&session_file, json)?;

    tracing::debug!(path = %session_file.display(), "Wrote API session file");
    Ok(session_file)
}

/// Remove API session file on shutdown
fn remove_session_file(tickets_path: &Path) {
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
    Running { port: u16 },
    Error(String),
}

impl RestApiStatus {
    /// Returns true if the server is running
    pub fn is_running(&self) -> bool {
        matches!(self, RestApiStatus::Running { .. })
    }
}

/// REST API server handle for lifecycle management
pub struct RestApiServer {
    config: Config,
    port: u16,
    tickets_path: PathBuf,
    status: Arc<Mutex<RestApiStatus>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    #[allow(dead_code)]
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl RestApiServer {
    /// Create a new server handle
    pub fn new(config: Config, port: u16) -> Self {
        let tickets_path = config.tickets_path();
        Self {
            config,
            port,
            tickets_path,
            status: Arc::new(Mutex::new(RestApiStatus::Stopped)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            task_handle: Arc::new(Mutex::new(None)),
        }
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
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        tokio::net::TcpListener::bind(addr).await.is_err()
    }

    /// Start the REST API server
    pub fn start(&self) -> Result<(), String> {
        if self.is_running() {
            return Err(format!("REST API already running on port {}", self.port));
        }

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        let state = ApiState::new(self.config.clone(), self.config.tickets_path());
        let router = build_router(state);
        let port = self.port;
        let status = self.status.clone();
        let tickets_path = self.tickets_path.clone();

        *status.lock().unwrap() = RestApiStatus::Starting;

        let handle = tokio::spawn(async move {
            use std::net::SocketAddr;
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    *status.lock().unwrap() = RestApiStatus::Running { port };
                    tracing::info!("REST API listening on http://{}", addr);

                    // Write session file for client discovery
                    if let Err(e) = write_session_file(&tickets_path, port) {
                        tracing::warn!(error = %e, "Failed to write API session file");
                    }

                    let _ = axum::serve(listener, router)
                        .with_graceful_shutdown(async {
                            let _ = shutdown_rx.await;
                        })
                        .await;

                    // Clean up session file on shutdown
                    remove_session_file(&tickets_path);
                }
                Err(e) => {
                    *status.lock().unwrap() = RestApiStatus::Error(e.to_string());
                    tracing::error!("Failed to start REST API: {}", e);
                }
            }

            *status.lock().unwrap() = RestApiStatus::Stopped;
        });

        *self.task_handle.lock().unwrap() = Some(handle);

        // Give the server a moment to start
        std::thread::sleep(std::time::Duration::from_millis(50));

        Ok(())
    }

    /// Stop the REST API server
    pub fn stop(&self) {
        // Set stopping status first for visual feedback
        *self.status.lock().unwrap() = RestApiStatus::Stopping;

        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }

        // Clean up session file
        remove_session_file(&self.tickets_path);

        // Brief delay to allow visual feedback
        std::thread::sleep(std::time::Duration::from_millis(100));

        *self.status.lock().unwrap() = RestApiStatus::Stopped;
        tracing::info!("REST API server stopped");
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

    #[test]
    fn test_rest_api_server_initial_status() {
        let config = Config::default();
        let server = RestApiServer::new(config, 7008);
        assert_eq!(server.status(), RestApiStatus::Stopped);
        assert!(!server.is_running());
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

        // Status should be stopped
        assert_eq!(server.status(), RestApiStatus::Stopped);
        // Shutdown tx should be taken (None)
        assert!(server.shutdown_tx.lock().unwrap().is_none());
    }

    #[test]
    fn test_write_session_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let port = 7008u16;

        let result = write_session_file(temp_dir.path(), port);
        assert!(result.is_ok());

        let session_file = temp_dir.path().join("operator").join("api-session.json");
        assert!(session_file.exists(), "Session file should exist");

        let content = std::fs::read_to_string(&session_file).unwrap();
        let session: ApiSessionInfo = serde_json::from_str(&content).unwrap();

        assert_eq!(session.port, port);
        assert!(!session.version.is_empty());
        assert!(session.pid > 0);
    }

    #[test]
    fn test_write_session_file_creates_operator_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Operator dir doesn't exist yet
        let operator_dir = temp_dir.path().join("operator");
        assert!(!operator_dir.exists());

        let result = write_session_file(temp_dir.path(), 7008);
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
