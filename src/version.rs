use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::time::Duration;

use crate::config::VersionCheckConfig;

/// Checks for updates by fetching the latest version from the configured URL.
///
/// Returns Some(version_string) if a newer version is available, None otherwise.
/// Fails gracefully on network errors, timeouts, or validation failures.
pub async fn check_for_updates(config: &VersionCheckConfig) -> Option<String> {
    // If URL is not configured, skip check
    let url = config.url.as_ref()?;

    // Fetch and validate the latest version
    match fetch_latest_version(url, config.timeout_secs).await {
        Ok(remote_version) => {
            let current_version = env!("CARGO_PKG_VERSION");

            // Compare versions and return remote if newer
            if is_newer_version(current_version, &remote_version) {
                tracing::info!(
                    current = %current_version,
                    remote = %remote_version,
                    "Update available"
                );
                Some(remote_version)
            } else {
                tracing::debug!(
                    current = %current_version,
                    remote = %remote_version,
                    "No update available"
                );
                None
            }
        }
        Err(e) => {
            tracing::debug!(error = %e, "Version check failed");
            None
        }
    }
}

/// Fetches the latest version from the specified URL with a timeout.
async fn fetch_latest_version(url: &str, timeout_secs: u64) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    validate_version_response(status, &body)
}

/// Validates that the HTTP response is acceptable for a version check.
///
/// Requirements:
/// - Status code must be 2xx
/// - Body must be a single line
/// - Body must resemble semver format (X.Y.Z or vX.Y.Z)
fn validate_version_response(status: StatusCode, body: &str) -> Result<String> {
    // Check for 2xx status code
    if !status.is_success() {
        return Err(anyhow!(
            "Non-success status code: {} (expected 2xx)",
            status.as_u16()
        ));
    }

    // Trim whitespace and check for single line
    let version = body.trim();
    if version.contains('\n') {
        return Err(anyhow!(
            "Response contains multiple lines (expected single line)"
        ));
    }

    // Validate semver format (allow optional 'v' prefix)
    if !is_semver_format(version) {
        return Err(anyhow!(
            "Response does not resemble semver format: {}",
            version
        ));
    }

    // Remove 'v' prefix if present
    let version = version.strip_prefix('v').unwrap_or(version);

    Ok(version.to_string())
}

/// Checks if a string resembles semver format (X.Y.Z where X, Y, Z are numbers).
fn is_semver_format(s: &str) -> bool {
    // Strip optional 'v' prefix
    let s = s.strip_prefix('v').unwrap_or(s);

    // Split by '.' and check we have exactly 3 parts
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    // Each part must be a valid number
    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Compares two semver strings and returns true if remote is newer than current.
///
/// Performs simple numeric comparison of X.Y.Z components.
/// Assumes both versions are valid semver strings (enforced by validation).
fn is_newer_version(current: &str, remote: &str) -> bool {
    // Parse versions into tuples of (major, minor, patch)
    let current_parts = parse_semver(current);
    let remote_parts = parse_semver(remote);

    match (current_parts, remote_parts) {
        (Some(c), Some(r)) => r > c,
        _ => false, // If parsing fails, assume no update
    }
}

/// Parses a semver string into (major, minor, patch) tuple.
fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    // Strip optional 'v' prefix
    let version = version.strip_prefix('v').unwrap_or(version);

    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Validation Tests ───────────────────────────────────────────────────

    #[test]
    fn test_validate_version_response_success() {
        let result = validate_version_response(StatusCode::OK, "0.1.10");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0.1.10");
    }

    #[test]
    fn test_validate_version_response_with_v_prefix() {
        let result = validate_version_response(StatusCode::OK, "v0.1.10");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0.1.10"); // Should strip 'v' prefix
    }

    #[test]
    fn test_validate_version_response_with_whitespace() {
        let result = validate_version_response(StatusCode::OK, "  0.1.10\n");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0.1.10");
    }

    #[test]
    fn test_validate_version_response_non_2xx() {
        let result = validate_version_response(StatusCode::NOT_FOUND, "0.1.10");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    #[test]
    fn test_validate_version_response_multiline() {
        let result = validate_version_response(StatusCode::OK, "0.1.10\n0.2.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("multiple lines"));
    }

    #[test]
    fn test_validate_version_response_invalid_semver() {
        let result = validate_version_response(StatusCode::OK, "invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("semver format"));
    }

    #[test]
    fn test_validate_version_response_partial_semver() {
        let result = validate_version_response(StatusCode::OK, "1.0");
        assert!(result.is_err());
    }

    // ─── Semver Format Tests ────────────────────────────────────────────────

    #[test]
    fn test_is_semver_format_valid() {
        assert!(is_semver_format("0.1.10"));
        assert!(is_semver_format("1.0.0"));
        assert!(is_semver_format("10.20.30"));
        assert!(is_semver_format("v0.1.10"));
        assert!(is_semver_format("v1.2.3"));
    }

    #[test]
    fn test_is_semver_format_invalid() {
        assert!(!is_semver_format("1.0"));
        assert!(!is_semver_format("1"));
        assert!(!is_semver_format("1.0.0.0"));
        assert!(!is_semver_format("abc.def.ghi"));
        assert!(!is_semver_format("1.0.x"));
        assert!(!is_semver_format(""));
    }

    // ─── Version Comparison Tests ───────────────────────────────────────────

    #[test]
    fn test_is_newer_version_patch() {
        assert!(is_newer_version("0.1.9", "0.1.10"));
        assert!(!is_newer_version("0.1.10", "0.1.9"));
        assert!(!is_newer_version("0.1.10", "0.1.10"));
    }

    #[test]
    fn test_is_newer_version_minor() {
        assert!(is_newer_version("0.1.10", "0.2.0"));
        assert!(!is_newer_version("0.2.0", "0.1.10"));
        assert!(is_newer_version("0.1.10", "0.2.5"));
    }

    #[test]
    fn test_is_newer_version_major() {
        assert!(is_newer_version("0.1.10", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "0.1.10"));
        assert!(is_newer_version("0.1.10", "2.0.0"));
    }

    #[test]
    fn test_is_newer_version_equal() {
        assert!(!is_newer_version("0.1.10", "0.1.10"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_version_with_v_prefix() {
        assert!(is_newer_version("v0.1.9", "v0.1.10"));
        assert!(is_newer_version("0.1.9", "v0.1.10"));
        assert!(is_newer_version("v0.1.9", "0.1.10"));
    }

    // ─── Semver Parsing Tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_semver_valid() {
        assert_eq!(parse_semver("0.1.10"), Some((0, 1, 10)));
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("10.20.30"), Some((10, 20, 30)));
        assert_eq!(parse_semver("v0.1.10"), Some((0, 1, 10)));
    }

    #[test]
    fn test_parse_semver_invalid() {
        assert_eq!(parse_semver("1.0"), None);
        assert_eq!(parse_semver("1"), None);
        assert_eq!(parse_semver("1.0.0.0"), None);
        assert_eq!(parse_semver("abc.def.ghi"), None);
        assert_eq!(parse_semver(""), None);
    }
}
