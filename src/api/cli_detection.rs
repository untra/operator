//! CLI detection utilities for Git providers.
//!
//! Checks for the availability of provider CLI tools (gh, glab, etc.)
//! on the system PATH.

use std::process::Stdio;
use tokio::process::Command;

/// CLI tool information
#[derive(Debug, Clone)]
pub struct CliInfo {
    /// Name of the CLI tool
    pub name: &'static str,
    /// Command to run the tool
    pub command: &'static str,
    /// Whether the tool is installed
    pub installed: bool,
    /// Version string (if installed)
    pub version: Option<String>,
}

/// Check if the `git` CLI is installed
pub async fn detect_git() -> CliInfo {
    let (installed, version) = check_cli_version("git", &["--version"]).await;
    CliInfo {
        name: "Git",
        command: "git",
        installed,
        version,
    }
}

/// Check if the GitHub CLI (`gh`) is installed
pub async fn detect_github_cli() -> CliInfo {
    let (installed, version) = check_cli_version("gh", &["--version"]).await;
    CliInfo {
        name: "GitHub CLI",
        command: "gh",
        installed,
        version,
    }
}

/// Check if the GitLab CLI (`glab`) is installed
pub async fn detect_gitlab_cli() -> CliInfo {
    let (installed, version) = check_cli_version("glab", &["--version"]).await;
    CliInfo {
        name: "GitLab CLI",
        command: "glab",
        installed,
        version,
    }
}

/// Check if the Bitbucket CLI (`bb`) is installed
pub async fn detect_bitbucket_cli() -> CliInfo {
    let (installed, version) = check_cli_version("bb", &["--version"]).await;
    CliInfo {
        name: "Bitbucket CLI",
        command: "bb",
        installed,
        version,
    }
}

/// Check if the Azure CLI (`az`) is installed with repos extension
pub async fn detect_azure_cli() -> CliInfo {
    let (installed, version) = check_cli_version("az", &["--version"]).await;
    CliInfo {
        name: "Azure CLI",
        command: "az",
        installed,
        version,
    }
}

/// Detect all available provider CLIs
pub async fn detect_all_clis() -> Vec<CliInfo> {
    // Run all detections in parallel
    let (git, github, gitlab, bitbucket, azure) = tokio::join!(
        detect_git(),
        detect_github_cli(),
        detect_gitlab_cli(),
        detect_bitbucket_cli(),
        detect_azure_cli(),
    );

    vec![git, github, gitlab, bitbucket, azure]
}

/// Helper to check if a CLI is installed and get its version
async fn check_cli_version(command: &str, args: &[&str]) -> (bool, Option<String>) {
    let result = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;

    match result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string());
            (true, version)
        }
        _ => (false, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_git() {
        // git should be installed on most systems
        let info = detect_git().await;
        assert_eq!(info.name, "Git");
        assert_eq!(info.command, "git");
        // Don't assert installed=true as it depends on the system
    }

    #[tokio::test]
    async fn test_detect_github_cli() {
        let info = detect_github_cli().await;
        assert_eq!(info.name, "GitHub CLI");
        assert_eq!(info.command, "gh");
    }

    #[tokio::test]
    async fn test_detect_all_clis() {
        let clis = detect_all_clis().await;
        assert_eq!(clis.len(), 5);
        assert!(clis.iter().any(|c| c.command == "git"));
        assert!(clis.iter().any(|c| c.command == "gh"));
        assert!(clis.iter().any(|c| c.command == "glab"));
    }
}
