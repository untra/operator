//! Backstage runtime binary management.
//!
//! Downloads and manages the pre-compiled backstage-server binary
//! for the current platform.

use std::fs;
use std::path::PathBuf;

/// Supported platforms for backstage-server binary.
///
/// Variants are constructed via compile-time cfg attributes in `Platform::current()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants are selected at compile time via cfg attributes
pub enum Platform {
    DarwinArm64,
    DarwinX64,
    LinuxX64,
    LinuxArm64,
}

impl Platform {
    /// Detect the current platform at compile time.
    pub fn current() -> Option<Self> {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Some(Platform::DarwinArm64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Some(Platform::DarwinX64);

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Some(Platform::LinuxX64);

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Some(Platform::LinuxArm64);

        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
        )))]
        return None;
    }

    /// Get the Bun target identifier for this platform.
    pub fn bun_target(&self) -> &'static str {
        match self {
            Platform::DarwinArm64 => "bun-darwin-arm64",
            Platform::DarwinX64 => "bun-darwin-x64",
            Platform::LinuxX64 => "bun-linux-x64",
            Platform::LinuxArm64 => "bun-linux-arm64",
        }
    }

    /// Get a human-readable name for this platform.
    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::DarwinArm64 => "macOS ARM64",
            Platform::DarwinX64 => "macOS x64",
            Platform::LinuxX64 => "Linux x64",
            Platform::LinuxArm64 => "Linux ARM64",
        }
    }
}

/// Manages the backstage-server binary lifecycle.
pub struct BackstageRuntime {
    state_path: PathBuf,
    release_url: String,
    local_binary_path: Option<PathBuf>,
    platform: Platform,
}

/// Error types for runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Unsupported platform - backstage-server is not available for this OS/architecture")]
    UnsupportedPlatform,

    #[error("Failed to download binary: {0}")]
    DownloadFailed(String),

    #[error("Failed to write binary: {0}")]
    WriteFailed(String),

    #[error("Binary not found at {0}")]
    #[allow(dead_code)] // Reserved for future validation
    BinaryNotFound(PathBuf),

    #[error("Local file not found: {0}")]
    LocalFileNotFound(PathBuf),

    #[error("Local file is not executable: {0}")]
    #[allow(dead_code)] // Only used on Unix platforms
    LocalFileNotExecutable(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl BackstageRuntime {
    /// Create a new runtime manager.
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
        let platform = Platform::current().ok_or(RuntimeError::UnsupportedPlatform)?;

        Ok(Self {
            state_path,
            release_url,
            local_binary_path: local_binary_path.map(PathBuf::from),
            platform,
        })
    }

    /// Get the path where the binary should be stored.
    pub fn binary_path(&self) -> PathBuf {
        self.state_path.join("bin").join("backstage-server")
    }

    /// Check if the binary exists.
    pub fn binary_exists(&self) -> bool {
        self.binary_path().exists()
    }

    /// Get the current platform.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Ensure the binary is available, downloading if necessary.
    ///
    /// Returns the path to the binary.
    pub fn ensure_binary(&self) -> Result<PathBuf, RuntimeError> {
        let binary_path = self.binary_path();

        if binary_path.exists() {
            tracing::debug!(
                "Backstage binary already exists at {}",
                binary_path.display()
            );
            return Ok(binary_path);
        }

        self.download_binary()?;
        Ok(binary_path)
    }

    /// Download or copy the binary for the current platform.
    ///
    /// If `local_binary_path` is set, copies from local path.
    /// Otherwise, downloads from `release_url` with platform suffix appended.
    fn download_binary(&self) -> Result<(), RuntimeError> {
        // Create bin directory
        let bin_dir = self.state_path.join("bin");
        fs::create_dir_all(&bin_dir)?;

        let binary_path = self.binary_path();

        if let Some(ref local_path) = self.local_binary_path {
            self.copy_local_binary(local_path, &binary_path)?;
        } else {
            self.download_remote_binary(&binary_path)?;
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(())
    }

    /// Copy binary from a local path.
    fn copy_local_binary(
        &self,
        source_path: &PathBuf,
        dest_path: &PathBuf,
    ) -> Result<(), RuntimeError> {
        // Verify source exists
        if !source_path.exists() {
            return Err(RuntimeError::LocalFileNotFound(source_path.clone()));
        }

        // Verify source is executable (on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(source_path)?.permissions();
            if perms.mode() & 0o111 == 0 {
                return Err(RuntimeError::LocalFileNotExecutable(source_path.clone()));
            }
        }

        tracing::info!(
            "Copying local backstage-server from {} to {}",
            source_path.display(),
            dest_path.display()
        );

        // Copy the file (not symlink)
        fs::copy(source_path, dest_path).map_err(|e| RuntimeError::WriteFailed(e.to_string()))?;

        let bytes = fs::metadata(dest_path)?.len();
        tracing::info!(
            "Copied backstage-server ({} bytes) to {}",
            bytes,
            dest_path.display()
        );

        Ok(())
    }

    /// Download binary from a remote https:// URL.
    ///
    /// Appends platform suffix to the URL (e.g., /backstage-server-bun-darwin-arm64).
    fn download_remote_binary(&self, dest_path: &PathBuf) -> Result<(), RuntimeError> {
        let url = format!(
            "{}/backstage-server-{}",
            self.release_url,
            self.platform.bun_target()
        );

        tracing::info!(
            "Downloading backstage-server for {} from {}",
            self.platform.display_name(),
            url
        );

        // Download using reqwest blocking client
        let response = reqwest::blocking::get(&url)
            .map_err(|e| RuntimeError::DownloadFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RuntimeError::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| RuntimeError::DownloadFailed(e.to_string()))?;

        // Write binary
        fs::write(dest_path, &bytes).map_err(|e| RuntimeError::WriteFailed(e.to_string()))?;

        tracing::info!(
            "Downloaded backstage-server ({} bytes) to {}",
            bytes.len(),
            dest_path.display()
        );

        Ok(())
    }

    /// Remove the downloaded binary.
    #[allow(dead_code)] // Reserved for cleanup/maintenance operations
    pub fn remove_binary(&self) -> Result<(), RuntimeError> {
        let binary_path = self.binary_path();
        if binary_path.exists() {
            fs::remove_file(&binary_path)?;
            tracing::info!("Removed backstage-server binary");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        // This should return Some on supported platforms
        let platform = Platform::current();
        #[cfg(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
        ))]
        assert!(platform.is_some());
    }

    #[test]
    fn test_platform_bun_target() {
        assert_eq!(Platform::DarwinArm64.bun_target(), "bun-darwin-arm64");
        assert_eq!(Platform::DarwinX64.bun_target(), "bun-darwin-x64");
        assert_eq!(Platform::LinuxX64.bun_target(), "bun-linux-x64");
        assert_eq!(Platform::LinuxArm64.bun_target(), "bun-linux-arm64");
    }

    #[test]
    fn test_platform_display_name() {
        assert_eq!(Platform::DarwinArm64.display_name(), "macOS ARM64");
        assert_eq!(Platform::LinuxX64.display_name(), "Linux x64");
    }

    #[test]
    fn test_runtime_binary_path() {
        let runtime = BackstageRuntime::new(
            PathBuf::from("/tmp/test-state"),
            "https://example.com/releases".to_string(),
            None,
        );

        if let Ok(runtime) = runtime {
            let path = runtime.binary_path();
            assert_eq!(path, PathBuf::from("/tmp/test-state/bin/backstage-server"));
        }
    }

    #[test]
    fn test_runtime_with_local_path() {
        let runtime = BackstageRuntime::new(
            PathBuf::from("/tmp/test-state"),
            "https://example.com/releases".to_string(),
            Some("/path/to/local/binary".to_string()),
        );

        if let Ok(runtime) = runtime {
            assert_eq!(
                runtime.local_binary_path,
                Some(PathBuf::from("/path/to/local/binary"))
            );
        }
    }

    #[test]
    fn test_runtime_without_local_path() {
        let runtime = BackstageRuntime::new(
            PathBuf::from("/tmp/test-state"),
            "https://example.com/releases".to_string(),
            None,
        );

        if let Ok(runtime) = runtime {
            assert!(runtime.local_binary_path.is_none());
        }
    }

    #[test]
    fn test_runtime_unsupported_platform() {
        // This test verifies the error type exists
        let err = RuntimeError::UnsupportedPlatform;
        assert!(err.to_string().contains("Unsupported platform"));
    }

    #[test]
    fn test_local_file_not_found_error() {
        let err = RuntimeError::LocalFileNotFound(PathBuf::from("/nonexistent/path"));
        assert!(err.to_string().contains("Local file not found"));
    }

    #[test]
    fn test_local_file_not_executable_error() {
        let err = RuntimeError::LocalFileNotExecutable(PathBuf::from("/some/path"));
        assert!(err.to_string().contains("not executable"));
    }
}
