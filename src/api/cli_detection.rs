//! CLI detection utilities for Git providers.
//!
//! Checks for the availability of provider CLI tools (gh, glab, etc.)
//! on the system PATH.

use std::process::Stdio;
use tokio::process::Command;

use crate::types::pr::GitProvider;

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

/// Static description of a provider CLI: display name and command to probe.
/// `provider` is `None` for the provider-agnostic `git` binary.
struct CliSpec {
    provider: Option<GitProvider>,
    name: &'static str,
    command: &'static str,
}

const CLI_SPECS: &[CliSpec] = &[
    CliSpec {
        provider: None,
        name: "Git",
        command: "git",
    },
    CliSpec {
        provider: Some(GitProvider::GitHub),
        name: "GitHub CLI",
        command: "gh",
    },
    CliSpec {
        provider: Some(GitProvider::GitLab),
        name: "GitLab CLI",
        command: "glab",
    },
    CliSpec {
        provider: Some(GitProvider::Bitbucket),
        name: "Bitbucket CLI",
        command: "bb",
    },
    CliSpec {
        provider: Some(GitProvider::AzureDevOps),
        name: "Azure CLI",
        command: "az",
    },
    CliSpec {
        provider: Some(GitProvider::Forgejo),
        name: "Forgejo CLI",
        command: "fj",
    },
    CliSpec {
        provider: Some(GitProvider::Gitea),
        name: "Gitea CLI",
        command: "tea",
    },
];

/// Detect all provider CLIs (and `git` itself), in table order.
pub async fn detect_all_clis() -> Vec<CliInfo> {
    let checks = CLI_SPECS.iter().map(probe);
    futures_util::future::join_all(checks).await
}

/// Detect the CLI for a specific provider.
pub async fn detect_for(provider: GitProvider) -> CliInfo {
    let spec = CLI_SPECS
        .iter()
        .find(|s| s.provider == Some(provider))
        .expect("CLI_SPECS covers every GitProvider variant");
    probe(spec).await
}

async fn probe(spec: &CliSpec) -> CliInfo {
    let (installed, version) = check_cli_version(spec.command, &["--version"]).await;
    CliInfo {
        name: spec.name,
        command: spec.command,
        installed,
        version,
    }
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
    async fn test_detect_all_clis() {
        let clis = detect_all_clis().await;
        assert_eq!(clis.len(), 7);
        assert!(clis.iter().any(|c| c.command == "git"));
        assert!(clis.iter().any(|c| c.command == "gh"));
        assert!(clis.iter().any(|c| c.command == "glab"));
        assert!(clis.iter().any(|c| c.command == "bb"));
        assert!(clis.iter().any(|c| c.command == "az"));
        assert!(clis.iter().any(|c| c.command == "fj"));
        assert!(clis.iter().any(|c| c.command == "tea"));
    }

    #[test]
    fn test_cli_specs_covers_all_providers() {
        for provider in GitProvider::ALL {
            assert!(
                CLI_SPECS.iter().any(|s| s.provider == Some(provider)),
                "no CliSpec for provider {provider}"
            );
        }
    }

    #[tokio::test]
    async fn test_detect_for_github() {
        let info = detect_for(GitProvider::GitHub).await;
        assert_eq!(info.command, "gh");
        assert_eq!(info.name, "GitHub CLI");
    }

    #[tokio::test]
    async fn test_detect_for_gitlab() {
        let info = detect_for(GitProvider::GitLab).await;
        assert_eq!(info.command, "glab");
        assert_eq!(info.name, "GitLab CLI");
    }

    #[tokio::test]
    async fn test_detect_for_forgejo() {
        let info = detect_for(GitProvider::Forgejo).await;
        assert_eq!(info.command, "fj");
        assert_eq!(info.name, "Forgejo CLI");
    }

    #[tokio::test]
    async fn test_detect_for_gitea() {
        let info = detect_for(GitProvider::Gitea).await;
        assert_eq!(info.command, "tea");
        assert_eq!(info.name, "Gitea CLI");
    }
}
