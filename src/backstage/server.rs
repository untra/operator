//! Backstage server lifecycle management.
//!
//! Provides a trait-based abstraction over server operations to enable:
//! - Unit testing without real binaries
//! - Mocking server behavior
//! - Compiled binary mode (no Bun required)
//! - Development mode with Bun

// M6 TUI integration complete - some methods still reserved for future Backstage API
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

use super::runtime::{BackstageRuntime, RuntimeError};
use crate::config::BrandingConfig;

/// Errors specific to Backstage operations
#[derive(Error, Debug)]
pub enum BackstageError {
    #[error("bun is not installed or not in PATH")]
    BunNotInstalled,

    #[error("bun version {0} is below minimum required version {1}")]
    #[allow(dead_code)] // Reserved for future version enforcement
    BunVersionTooOld(String, String),

    #[error("backstage scaffold not found at {0}")]
    ScaffoldNotFound(PathBuf),

    #[error("backstage server is not running")]
    ServerNotRunning,

    #[error("backstage server is already running on port {0}")]
    ServerAlreadyRunning(u16),

    #[error("failed to start backstage server: {0}")]
    StartFailed(String),

    #[error("failed to stop backstage server: {0}")]
    StopFailed(String),

    #[error("bun command failed: {0}")]
    CommandFailed(String),

    #[error("server not ready after {timeout_ms}ms on port {port}")]
    ServerNotReady { port: u16, timeout_ms: u64 },

    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

/// Version information for Bun
#[derive(Debug, Clone, PartialEq)]
pub struct BunVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub raw: String,
}

impl BunVersion {
    /// Parse a version string like "1.1.42" or "v1.1.42"
    pub fn parse(version_str: &str) -> Option<Self> {
        let clean = version_str.trim().trim_start_matches('v');
        let parts: Vec<&str> = clean.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        let major: u32 = parts[0].parse().ok()?;
        let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch,
            raw: version_str.to_string(),
        })
    }

    /// Check if this version meets the minimum requirement
    #[allow(dead_code)] // Used in tests, reserved for future version enforcement
    pub fn meets_minimum(&self, min: &BunVersion) -> bool {
        (self.major, self.minor, self.patch) >= (min.major, min.minor, min.patch)
    }
}

impl std::fmt::Display for BunVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Generate branding configuration file for backstage-server.
///
/// Creates a `theme.json` file in the branding directory that backstage-server
/// reads on startup to configure the portal's theme and branding.
///
/// # Arguments
/// * `branding_path` - Directory to write the theme.json file
/// * `config` - Branding configuration from Operator config
///
/// # Returns
/// * `Ok(())` on success
/// * `Err` if directory creation or file writing fails
pub fn generate_branding_config(
    branding_path: &Path,
    config: &BrandingConfig,
) -> std::io::Result<()> {
    // Create branding directory if it doesn't exist
    std::fs::create_dir_all(branding_path)?;

    // Build theme configuration JSON
    let theme = serde_json::json!({
        "appTitle": config.app_title,
        "orgName": config.org_name,
        "logoPath": config.logo_path,
        "colors": {
            "primary": config.colors.primary,
            "secondary": config.colors.secondary,
            "accent": config.colors.accent,
            "warning": config.colors.warning,
            "muted": config.colors.muted,
        }
    });

    // Write theme.json
    let theme_path = branding_path.join("theme.json");
    let theme_json = serde_json::to_string_pretty(&theme)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&theme_path, theme_json)?;

    tracing::info!(
        path = %theme_path.display(),
        app_title = %config.app_title,
        "Generated backstage branding config"
    );

    Ok(())
}

/// Copy default logo to branding directory if not already present.
///
/// Copies `img/operator_logo.svg` to the branding directory as `logo.svg`
/// on first setup if no logo already exists.
///
/// # Arguments
/// * `branding_path` - Directory containing branding assets
/// * `source_logo_path` - Path to the source logo file
pub fn copy_default_logo(branding_path: &Path, source_logo_path: &Path) -> std::io::Result<()> {
    let dest_logo = branding_path.join("logo.svg");

    // Only copy if destination doesn't exist
    if !dest_logo.exists() && source_logo_path.exists() {
        std::fs::create_dir_all(branding_path)?;
        std::fs::copy(source_logo_path, &dest_logo)?;
        tracing::info!(
            source = %source_logo_path.display(),
            dest = %dest_logo.display(),
            "Copied default logo to branding directory"
        );
    }

    Ok(())
}

/// Server status information
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Stopping,
    Running {
        port: u16,
        pid: u32,
    },
    #[allow(dead_code)] // Used in StatusBar rendering
    Error(String),
}

impl ServerStatus {
    /// Returns true if the server is running
    pub fn is_running(&self) -> bool {
        matches!(self, ServerStatus::Running { .. })
    }
}

/// Trait abstracting Bun operations for testability
pub trait BunClient: Send + Sync {
    /// Check if Bun is available and return version info
    fn check_available(&self) -> Result<BunVersion, BackstageError>;

    /// Check if dependencies are installed
    fn check_dependencies(&self, scaffold_path: &Path) -> Result<bool, BackstageError>;

    /// Install dependencies (bun install)
    fn install_dependencies(&self, scaffold_path: &Path) -> Result<(), BackstageError>;

    /// Start the Backstage server, returns process handle
    fn start_server(&self, scaffold_path: &Path, port: u16) -> Result<Child, BackstageError>;

    /// Check if a process is still running
    fn is_process_running(&self, pid: u32) -> bool;
}

/// Real implementation using system Bun
pub struct SystemBunClient;

impl SystemBunClient {
    pub fn new() -> Self {
        Self
    }

    fn run_bun(
        &self,
        args: &[&str],
        cwd: Option<&Path>,
    ) -> Result<std::process::Output, BackstageError> {
        let mut cmd = Command::new("bun");
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BackstageError::BunNotInstalled
            } else {
                BackstageError::CommandFailed(e.to_string())
            }
        })
    }
}

impl Default for SystemBunClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BunClient for SystemBunClient {
    fn check_available(&self) -> Result<BunVersion, BackstageError> {
        let output = self.run_bun(&["--version"], None)?;

        if !output.status.success() {
            return Err(BackstageError::BunNotInstalled);
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        BunVersion::parse(version_str.trim()).ok_or_else(|| {
            BackstageError::CommandFailed(format!("Could not parse version: {}", version_str))
        })
    }

    fn check_dependencies(&self, scaffold_path: &Path) -> Result<bool, BackstageError> {
        let node_modules = scaffold_path.join("node_modules");
        let lockfile = scaffold_path.join("bun.lockb");

        Ok(node_modules.exists() && lockfile.exists())
    }

    fn install_dependencies(&self, scaffold_path: &Path) -> Result<(), BackstageError> {
        let output = self.run_bun(&["install"], Some(scaffold_path))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackstageError::CommandFailed(format!(
                "bun install failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    fn start_server(&self, scaffold_path: &Path, port: u16) -> Result<Child, BackstageError> {
        // Start backend with bun
        // Command: bun run packages/backend/src/index.ts
        let child = Command::new("bun")
            .args(["run", "packages/backend/src/index.ts"])
            .current_dir(scaffold_path)
            .env("PORT", port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    BackstageError::BunNotInstalled
                } else {
                    BackstageError::StartFailed(e.to_string())
                }
            })?;

        Ok(child)
    }

    fn is_process_running(&self, pid: u32) -> bool {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.process(sysinfo::Pid::from_u32(pid)).is_some()
    }
}

/// Client that uses a pre-compiled backstage-server binary.
///
/// Downloads the platform-specific binary on first use and executes it directly.
/// This eliminates the need for users to have Bun/Node installed.
pub struct RuntimeBinaryClient {
    runtime: BackstageRuntime,
}

impl RuntimeBinaryClient {
    /// Create a new runtime binary client.
    ///
    /// # Arguments
    /// * `state_path` - Directory to store the binary (e.g., .tickets/operator)
    /// * `release_url` - Base URL for downloading binaries
    /// * `local_binary_path` - Optional local path to binary (takes precedence over URL)
    pub fn new(
        state_path: PathBuf,
        release_url: String,
        local_binary_path: Option<String>,
    ) -> Result<Self, RuntimeError> {
        let runtime = BackstageRuntime::new(state_path, release_url, local_binary_path)?;
        Ok(Self { runtime })
    }
}

impl BunClient for RuntimeBinaryClient {
    fn check_available(&self) -> Result<BunVersion, BackstageError> {
        // For compiled binary, we return a synthetic version indicating binary mode
        // The binary is self-contained and doesn't need Bun
        Ok(BunVersion {
            major: 0,
            minor: 0,
            patch: 0,
            raw: "compiled-binary".to_string(),
        })
    }

    fn check_dependencies(&self, _scaffold_path: &Path) -> Result<bool, BackstageError> {
        // For compiled binary, dependencies are bundled in the binary
        // We just need to ensure the binary exists
        Ok(self.runtime.binary_exists())
    }

    fn install_dependencies(&self, _scaffold_path: &Path) -> Result<(), BackstageError> {
        // "Installing dependencies" means downloading the binary
        self.runtime.ensure_binary()?;
        Ok(())
    }

    fn start_server(&self, _scaffold_path: &Path, port: u16) -> Result<Child, BackstageError> {
        // Ensure binary is available
        let binary_path = self.runtime.ensure_binary()?;

        tracing::info!(
            binary = %binary_path.display(),
            port = port,
            platform = %self.runtime.platform().display_name(),
            "Starting backstage-server binary"
        );

        // Start the compiled binary
        let child = Command::new(&binary_path)
            .env("PORT", port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BackstageError::StartFailed(e.to_string()))?;

        Ok(child)
    }

    fn is_process_running(&self, pid: u32) -> bool {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.process(sysinfo::Pid::from_u32(pid)).is_some()
    }
}

/// Backstage server handle for lifecycle management
pub struct BackstageServer {
    client: Arc<dyn BunClient>,
    scaffold_path: PathBuf,
    port: u16,
    process: Mutex<Option<Child>>,
    status: Mutex<ServerStatus>,
}

impl BackstageServer {
    /// Create a new server handle
    pub fn new(client: Arc<dyn BunClient>, scaffold_path: PathBuf, port: u16) -> Self {
        Self {
            client,
            scaffold_path,
            port,
            process: Mutex::new(None),
            status: Mutex::new(ServerStatus::Stopped),
        }
    }

    /// Create with system Bun client (development mode)
    pub fn with_system_client(scaffold_path: PathBuf, port: u16) -> Self {
        Self::new(Arc::new(SystemBunClient::new()), scaffold_path, port)
    }

    /// Create with compiled binary client (production mode).
    ///
    /// Downloads and uses a pre-compiled backstage-server binary that doesn't
    /// require Bun/Node to be installed. The binary is downloaded on first use.
    ///
    /// # Arguments
    /// * `state_path` - Directory to store the binary (e.g., .tickets/operator)
    /// * `release_url` - Base URL for downloading binaries
    /// * `local_binary_path` - Optional local path to binary (takes precedence over URL)
    /// * `port` - Port to run the server on
    pub fn with_compiled_binary(
        state_path: PathBuf,
        release_url: String,
        local_binary_path: Option<String>,
        port: u16,
    ) -> Result<Self, BackstageError> {
        let client = RuntimeBinaryClient::new(state_path.clone(), release_url, local_binary_path)?;
        Ok(Self::new(Arc::new(client), state_path, port))
    }

    /// Get current server status
    pub fn status(&self) -> ServerStatus {
        self.status.lock().unwrap().clone()
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.status().is_running()
    }

    /// Start the Backstage server
    pub fn start(&self) -> Result<(), BackstageError> {
        // Check if already running
        if self.is_running() {
            return Err(BackstageError::ServerAlreadyRunning(self.port));
        }

        // Check scaffold exists
        if !self.scaffold_path.exists() {
            return Err(BackstageError::ScaffoldNotFound(self.scaffold_path.clone()));
        }

        // Check Bun is available
        let version = self.client.check_available()?;
        tracing::info!(version = %version.raw, "Bun available");

        // Check/install dependencies
        if !self.client.check_dependencies(&self.scaffold_path)? {
            tracing::info!("Installing Backstage dependencies...");
            *self.status.lock().unwrap() = ServerStatus::Starting;
            self.client.install_dependencies(&self.scaffold_path)?;
        }

        // Start server
        *self.status.lock().unwrap() = ServerStatus::Starting;
        let mut child = self.client.start_server(&self.scaffold_path, self.port)?;
        let pid = child.id();

        // Spawn threads to forward stdout/stderr to tracing
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    tracing::info!(target: "backstage", "{}", line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    tracing::warn!(target: "backstage", "{}", line);
                }
            });
        }

        *self.process.lock().unwrap() = Some(child);
        *self.status.lock().unwrap() = ServerStatus::Running {
            port: self.port,
            pid,
        };

        tracing::info!(port = self.port, pid = pid, "Backstage server started");

        Ok(())
    }

    /// Stop the Backstage server
    pub fn stop(&self) -> Result<(), BackstageError> {
        let mut process_guard = self.process.lock().unwrap();

        if let Some(ref mut child) = *process_guard {
            // Set stopping status first for visual feedback
            *self.status.lock().unwrap() = ServerStatus::Stopping;

            child
                .kill()
                .map_err(|e| BackstageError::StopFailed(e.to_string()))?;
            child
                .wait()
                .map_err(|e| BackstageError::StopFailed(e.to_string()))?;
            tracing::info!("Backstage server stopped");
        }

        *process_guard = None;
        *self.status.lock().unwrap() = ServerStatus::Stopped;

        Ok(())
    }

    /// Toggle server state (start if stopped, stop if running)
    pub fn toggle(&self) -> Result<(), BackstageError> {
        if self.is_running() {
            self.stop()
        } else {
            self.start()
        }
    }

    /// Open Backstage in default browser
    ///
    /// Checks `$BROWSER` environment variable first, then falls back to
    /// platform-specific defaults (`open` on macOS, `xdg-open` on Linux).
    pub fn open_browser(&self) -> Result<(), BackstageError> {
        if !self.is_running() {
            return Err(BackstageError::ServerNotRunning);
        }

        let url = format!("http://localhost:{}", self.port);

        // Check $BROWSER environment variable first
        if let Ok(browser) = std::env::var("BROWSER") {
            let _ = Command::new(&browser).arg(&url).spawn();
            return Ok(());
        }

        // Fall back to platform-specific defaults
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(&url).spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(&url).spawn();
        }

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd").args(["/C", "start", &url]).spawn();
        }

        Ok(())
    }

    /// Wait for server to be ready (health endpoint responds).
    ///
    /// Polls the `/health` endpoint every 500ms until it responds with success
    /// or the timeout is reached.
    ///
    /// # Arguments
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    ///
    /// # Returns
    /// * `Ok(())` if server is ready
    /// * `Err(BackstageError::ServerNotReady)` if timeout reached
    pub fn wait_for_ready(&self, timeout_ms: u64) -> Result<(), BackstageError> {
        let url = format!("http://localhost:{}/health", self.port);
        let check_interval = Duration::from_millis(500);
        let max_attempts = (timeout_ms / 500) as usize;

        tracing::debug!(
            url = %url,
            max_attempts = max_attempts,
            "Waiting for server to be ready"
        );

        for attempt in 1..=max_attempts {
            match reqwest::blocking::Client::new()
                .get(&url)
                .timeout(Duration::from_secs(2))
                .send()
            {
                Ok(response) if response.status().is_success() => {
                    tracing::info!(attempts = attempt, "Server ready");
                    return Ok(());
                }
                Ok(response) => {
                    tracing::debug!(
                        attempt = attempt,
                        status = %response.status(),
                        "Health check returned non-success status"
                    );
                }
                Err(e) => {
                    tracing::debug!(
                        attempt = attempt,
                        error = %e,
                        "Health check failed, retrying..."
                    );
                }
            }
            std::thread::sleep(check_interval);
        }

        Err(BackstageError::ServerNotReady {
            port: self.port,
            timeout_ms,
        })
    }

    /// Refresh status by checking if process is still running
    pub fn refresh_status(&self) {
        let mut status_guard = self.status.lock().unwrap();

        if let ServerStatus::Running { pid, .. } = *status_guard {
            if !self.client.is_process_running(pid) {
                *status_guard = ServerStatus::Stopped;
                *self.process.lock().unwrap() = None;
                tracing::warn!(pid = pid, "Backstage server process died unexpectedly");
            }
        }
    }

    /// Get the server URL
    #[allow(dead_code)] // Used in tests, reserved for future Backstage API
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Get the port
    #[allow(dead_code)] // Used in tests, reserved for future API
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the scaffold path
    #[allow(dead_code)] // Used in tests
    pub fn scaffold_path(&self) -> &Path {
        &self.scaffold_path
    }
}

// Ensure server is stopped when handle is dropped
impl Drop for BackstageServer {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation for testing
    #[derive(Default)]
    pub struct MockBunClient {
        pub installed: Mutex<bool>,
        pub version: Mutex<Option<BunVersion>>,
        pub deps_installed: Mutex<bool>,
        pub running_pids: Mutex<Vec<u32>>,
        pub next_pid: Mutex<u32>,
        pub fail_start: Mutex<bool>,
    }

    impl MockBunClient {
        pub fn new() -> Self {
            Self {
                installed: Mutex::new(true),
                version: Mutex::new(Some(BunVersion {
                    major: 1,
                    minor: 1,
                    patch: 42,
                    raw: "1.1.42".to_string(),
                })),
                deps_installed: Mutex::new(true),
                running_pids: Mutex::new(Vec::new()),
                next_pid: Mutex::new(1000),
                fail_start: Mutex::new(false),
            }
        }

        pub fn not_installed() -> Self {
            let mock = Self::new();
            *mock.installed.lock().unwrap() = false;
            mock
        }

        pub fn with_deps_not_installed() -> Self {
            let mock = Self::new();
            *mock.deps_installed.lock().unwrap() = false;
            mock
        }

        pub fn mark_pid_stopped(&self, pid: u32) {
            self.running_pids.lock().unwrap().retain(|&p| p != pid);
        }
    }

    impl BunClient for MockBunClient {
        fn check_available(&self) -> Result<BunVersion, BackstageError> {
            if !*self.installed.lock().unwrap() {
                return Err(BackstageError::BunNotInstalled);
            }

            self.version
                .lock()
                .unwrap()
                .clone()
                .ok_or(BackstageError::BunNotInstalled)
        }

        fn check_dependencies(&self, _scaffold_path: &Path) -> Result<bool, BackstageError> {
            Ok(*self.deps_installed.lock().unwrap())
        }

        fn install_dependencies(&self, _scaffold_path: &Path) -> Result<(), BackstageError> {
            *self.deps_installed.lock().unwrap() = true;
            Ok(())
        }

        fn start_server(&self, _scaffold_path: &Path, _port: u16) -> Result<Child, BackstageError> {
            if *self.fail_start.lock().unwrap() {
                return Err(BackstageError::StartFailed(
                    "Mock configured to fail".to_string(),
                ));
            }

            // Track the PID
            let mut next_pid = self.next_pid.lock().unwrap();
            let pid = *next_pid;
            *next_pid += 1;
            self.running_pids.lock().unwrap().push(pid);

            // Return a real dummy process (sleep) for testing
            // This allows us to test the Child handling
            Command::new("sleep")
                .arg("3600")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| BackstageError::StartFailed(e.to_string()))
        }

        fn is_process_running(&self, pid: u32) -> bool {
            self.running_pids.lock().unwrap().contains(&pid)
        }
    }

    use tempfile::TempDir;

    // ==================== BunVersion Tests ====================

    #[test]
    fn test_bun_version_parse_standard() {
        let v = BunVersion::parse("1.1.42").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 42);
    }

    #[test]
    fn test_bun_version_parse_with_v_prefix() {
        let v = BunVersion::parse("v1.0.0").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_bun_version_parse_two_part() {
        let v = BunVersion::parse("2.0").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_bun_version_parse_single_part() {
        let v = BunVersion::parse("3").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_bun_version_parse_with_whitespace() {
        let v = BunVersion::parse("  1.2.3  \n").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_bun_version_parse_invalid() {
        assert!(BunVersion::parse("").is_none());
        assert!(BunVersion::parse("not-a-version").is_none());
        assert!(BunVersion::parse("a.b.c").is_none());
    }

    #[test]
    fn test_bun_version_meets_minimum_equal() {
        let v = BunVersion::parse("1.0.0").unwrap();
        let min = BunVersion::parse("1.0.0").unwrap();
        assert!(v.meets_minimum(&min));
    }

    #[test]
    fn test_bun_version_meets_minimum_higher() {
        let v = BunVersion::parse("1.1.42").unwrap();
        let min = BunVersion::parse("1.0.0").unwrap();
        assert!(v.meets_minimum(&min));
    }

    #[test]
    fn test_bun_version_meets_minimum_lower() {
        let v = BunVersion::parse("1.0.0").unwrap();
        let min = BunVersion::parse("2.0.0").unwrap();
        assert!(!v.meets_minimum(&min));
    }

    #[test]
    fn test_bun_version_meets_minimum_minor() {
        let v = BunVersion::parse("1.5.0").unwrap();
        let min = BunVersion::parse("1.4.0").unwrap();
        assert!(v.meets_minimum(&min));
    }

    #[test]
    fn test_bun_version_display() {
        let v = BunVersion::parse("1.2.3").unwrap();
        assert_eq!(format!("{}", v), "1.2.3");
    }

    // ==================== ServerStatus Tests ====================

    #[test]
    fn test_server_status_is_running() {
        assert!(!ServerStatus::Stopped.is_running());
        assert!(!ServerStatus::Starting.is_running());
        assert!(!ServerStatus::Stopping.is_running());
        assert!(ServerStatus::Running {
            port: 7007,
            pid: 123
        }
        .is_running());
        assert!(!ServerStatus::Error("test".to_string()).is_running());
    }

    // ==================== MockBunClient Tests ====================

    #[test]
    fn test_mock_client_available() {
        let client = MockBunClient::new();
        let version = client.check_available().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 42);
    }

    #[test]
    fn test_mock_client_not_installed() {
        let client = MockBunClient::not_installed();
        assert!(matches!(
            client.check_available(),
            Err(BackstageError::BunNotInstalled)
        ));
    }

    #[test]
    fn test_mock_client_deps_installed() {
        let client = MockBunClient::new();
        let path = PathBuf::from("/test");
        assert!(client.check_dependencies(&path).unwrap());
    }

    #[test]
    fn test_mock_client_deps_not_installed() {
        let client = MockBunClient::with_deps_not_installed();
        let path = PathBuf::from("/test");
        assert!(!client.check_dependencies(&path).unwrap());
    }

    #[test]
    fn test_mock_client_install_deps() {
        let client = MockBunClient::with_deps_not_installed();
        let path = PathBuf::from("/test");
        assert!(!client.check_dependencies(&path).unwrap());

        client.install_dependencies(&path).unwrap();
        assert!(client.check_dependencies(&path).unwrap());
    }

    #[test]
    fn test_mock_client_process_running() {
        let client = MockBunClient::new();
        client.running_pids.lock().unwrap().push(123);
        assert!(client.is_process_running(123));
        assert!(!client.is_process_running(456));
    }

    #[test]
    fn test_mock_client_mark_pid_stopped() {
        let client = MockBunClient::new();
        client.running_pids.lock().unwrap().push(123);
        assert!(client.is_process_running(123));

        client.mark_pid_stopped(123);
        assert!(!client.is_process_running(123));
    }

    // ==================== BackstageServer Tests ====================

    #[test]
    fn test_server_status_initial() {
        let client = Arc::new(MockBunClient::new());
        let server = BackstageServer::new(client, PathBuf::from("/tmp/test"), 7007);

        assert_eq!(server.status(), ServerStatus::Stopped);
        assert!(!server.is_running());
    }

    #[test]
    fn test_server_url() {
        let client = Arc::new(MockBunClient::new());
        let server = BackstageServer::new(client, PathBuf::from("/tmp/test"), 7007);

        assert_eq!(server.url(), "http://localhost:7007");
    }

    #[test]
    fn test_server_port() {
        let client = Arc::new(MockBunClient::new());
        let server = BackstageServer::new(client, PathBuf::from("/tmp/test"), 8080);

        assert_eq!(server.port(), 8080);
    }

    #[test]
    fn test_server_scaffold_path() {
        let client = Arc::new(MockBunClient::new());
        let path = PathBuf::from("/my/scaffold");
        let server = BackstageServer::new(client, path.clone(), 7007);

        assert_eq!(server.scaffold_path(), path);
    }

    #[test]
    fn test_server_scaffold_not_found() {
        let client = Arc::new(MockBunClient::new());
        let server = BackstageServer::new(client, PathBuf::from("/nonexistent/path"), 7007);

        let result = server.start();
        assert!(matches!(result, Err(BackstageError::ScaffoldNotFound(_))));
    }

    #[test]
    fn test_server_bun_not_installed() {
        let client = Arc::new(MockBunClient::not_installed());
        let temp_dir = TempDir::new().unwrap();

        let server = BackstageServer::new(client, temp_dir.path().to_path_buf(), 7007);

        let result = server.start();
        assert!(matches!(result, Err(BackstageError::BunNotInstalled)));
    }

    #[test]
    fn test_server_start_and_stop() {
        let client = Arc::new(MockBunClient::new());
        let temp_dir = TempDir::new().unwrap();
        let server = BackstageServer::new(client, temp_dir.path().to_path_buf(), 7007);

        // Start
        server.start().unwrap();
        assert!(server.is_running());

        // Stop
        server.stop().unwrap();
        assert!(!server.is_running());
    }

    #[test]
    fn test_server_already_running() {
        let client = Arc::new(MockBunClient::new());
        let temp_dir = TempDir::new().unwrap();
        let server = BackstageServer::new(client, temp_dir.path().to_path_buf(), 7007);

        server.start().unwrap();

        let result = server.start();
        assert!(matches!(
            result,
            Err(BackstageError::ServerAlreadyRunning(7007))
        ));

        server.stop().unwrap();
    }

    #[test]
    fn test_server_toggle_start() {
        let client = Arc::new(MockBunClient::new());
        let temp_dir = TempDir::new().unwrap();
        let server = BackstageServer::new(client, temp_dir.path().to_path_buf(), 7007);

        assert!(!server.is_running());
        server.toggle().unwrap();
        assert!(server.is_running());

        server.stop().unwrap();
    }

    #[test]
    fn test_server_toggle_stop() {
        let client = Arc::new(MockBunClient::new());
        let temp_dir = TempDir::new().unwrap();
        let server = BackstageServer::new(client, temp_dir.path().to_path_buf(), 7007);

        server.start().unwrap();
        assert!(server.is_running());

        server.toggle().unwrap();
        assert!(!server.is_running());
    }

    #[test]
    fn test_open_browser_when_not_running() {
        let client = Arc::new(MockBunClient::new());
        let server = BackstageServer::new(client, PathBuf::from("/tmp/test"), 7007);

        let result = server.open_browser();
        assert!(matches!(result, Err(BackstageError::ServerNotRunning)));
    }

    #[test]
    fn test_server_with_system_client() {
        let temp_dir = TempDir::new().unwrap();
        let server = BackstageServer::with_system_client(temp_dir.path().to_path_buf(), 7007);

        assert_eq!(server.port(), 7007);
        assert!(!server.is_running());
    }

    #[test]
    fn test_server_installs_deps_if_missing() {
        let client = Arc::new(MockBunClient::with_deps_not_installed());
        let temp_dir = TempDir::new().unwrap();
        let server = BackstageServer::new(client.clone(), temp_dir.path().to_path_buf(), 7007);

        // Verify deps not installed initially
        assert!(!client.check_dependencies(temp_dir.path()).unwrap());

        // Start should install them
        server.start().unwrap();

        // Deps should now be installed
        assert!(client.check_dependencies(temp_dir.path()).unwrap());

        server.stop().unwrap();
    }

    // ==================== Integration Tests ====================

    #[test]
    #[ignore = "requires bun installation"]
    fn test_real_bun_version_check() {
        let client = SystemBunClient::new();
        let version = client.check_available().expect("Bun should be available");

        assert!(version.major >= 1, "Bun version should be 1.x or higher");
    }

    #[test]
    #[ignore = "requires bun installation"]
    fn test_real_check_dependencies_empty_dir() {
        let client = SystemBunClient::new();
        let temp_dir = TempDir::new().unwrap();

        // Empty directory should not have deps
        assert!(!client.check_dependencies(temp_dir.path()).unwrap());
    }
}
