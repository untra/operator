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

/// Static description of a provider CLI: the binary to probe plus everything
/// onboarding needs to install and authenticate it.
///
/// This is the single source of truth for provider binary names. Nothing else
/// may spell `gh`/`glab`/`tea` — enforced by
/// `provider_binaries_are_not_hardcoded_outside_the_registry`.
///
/// `provider` is `None` for the provider-agnostic `git` binary. Providers with
/// no onboarding story yet carry empty strings rather than `Option`s, so a
/// caller reads "nothing configured" without unwrapping.
pub struct CliSpec {
    provider: Option<GitProvider>,
    /// Human name of the tool ("GitHub CLI").
    pub name: &'static str,
    /// The binary as invoked on PATH.
    pub command: &'static str,
    /// Args that print an auth token on stdout, empty when unsupported.
    pub auth_args: &'static [&'static str],
    /// Where to download the CLI.
    pub install_url: &'static str,
    /// Where the user mints a personal access token.
    pub pat_url: &'static str,
    /// Provider brand name ("GitHub").
    pub display_name: &'static str,
    /// Placeholder shown in the token entry field.
    pub placeholder: &'static str,
}

const CLI_SPECS: &[CliSpec] = &[
    CliSpec {
        provider: None,
        name: "Git",
        command: "git",
        auth_args: &[],
        install_url: "https://git-scm.com/downloads",
        pat_url: "",
        display_name: "Git",
        placeholder: "",
    },
    CliSpec {
        provider: Some(GitProvider::GitHub),
        name: "GitHub CLI",
        command: "gh",
        auth_args: &["auth", "token"],
        install_url: "https://cli.github.com/",
        pat_url: "https://github.com/settings/personal-access-tokens/new",
        display_name: "GitHub",
        placeholder: "ghp_...",
    },
    CliSpec {
        provider: Some(GitProvider::GitLab),
        name: "GitLab CLI",
        command: "glab",
        auth_args: &["auth", "token"],
        install_url: "https://docs.gitlab.com/cli",
        pat_url: "https://gitlab.com/-/user_settings/personal_access_tokens",
        display_name: "GitLab",
        placeholder: "glpat-...",
    },
    CliSpec {
        provider: Some(GitProvider::Bitbucket),
        name: "Bitbucket CLI",
        command: "bb",
        auth_args: &[],
        install_url: "https://bitbucket.org/",
        pat_url: "",
        display_name: "Bitbucket",
        placeholder: "",
    },
    CliSpec {
        provider: Some(GitProvider::AzureDevOps),
        name: "Azure CLI",
        command: "az",
        auth_args: &[],
        install_url: "https://learn.microsoft.com/cli/azure/install-azure-cli",
        pat_url: "",
        display_name: "Azure DevOps",
        placeholder: "",
    },
    // Forgejo speaks Gitea's API, so `tea` drives it verbatim; there is no
    // separate `fj` dependency to install.
    CliSpec {
        provider: Some(GitProvider::Forgejo),
        name: "Gitea CLI (Forgejo-compatible)",
        command: "tea",
        auth_args: &[],
        install_url: "https://about.gitea.com/products/tea/",
        pat_url: "",
        display_name: "Forgejo",
        placeholder: "",
    },
    CliSpec {
        provider: Some(GitProvider::Gitea),
        name: "Gitea CLI",
        command: "tea",
        auth_args: &[],
        install_url: "https://about.gitea.com/products/tea/",
        pat_url: "https://gitea.com/user/settings/applications",
        display_name: "Gitea",
        placeholder: "Personal access token",
    },
];

/// The full spec for a provider.
pub fn spec_for(provider: GitProvider) -> &'static CliSpec {
    CLI_SPECS
        .iter()
        .find(|s| s.provider == Some(provider))
        .expect("CLI_SPECS covers every GitProvider variant")
}

/// The binary a provider's operations shell out to.
pub fn binary_for(provider: GitProvider) -> &'static str {
    spec_for(provider).command
}

/// The spec for a provider slug, when that provider has an onboarding story
/// (a PAT URL to send the user to). Providers Operator can only *detect* have
/// no token flow and resolve to `None`.
pub fn onboarding_spec_for_slug(slug: &str) -> Option<&'static CliSpec> {
    let provider = GitProvider::ALL.into_iter().find(|p| p.slug() == slug)?;
    let spec = spec_for(provider);
    (!spec.pat_url.is_empty()).then_some(spec)
}

/// The provider-agnostic `git` binary's spec.
pub fn git_spec() -> &'static CliSpec {
    CLI_SPECS
        .iter()
        .find(|s| s.provider.is_none())
        .expect("CLI_SPECS carries the git binary")
}

/// Detect all provider CLIs (and `git` itself), in table order.
pub async fn detect_all_clis() -> Vec<CliInfo> {
    let checks = CLI_SPECS.iter().map(probe);
    futures_util::future::join_all(checks).await
}

/// Detect the CLI for a specific provider.
pub async fn detect_for(provider: GitProvider) -> CliInfo {
    probe(spec_for(provider)).await
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
        for expected in ["git", "gh", "glab", "bb", "az", "tea"] {
            assert!(
                clis.iter().any(|c| c.command == expected),
                "no probe for {expected}"
            );
        }
        // Forgejo and Gitea share `tea`, so the table is 7 rows over 6 binaries.
        assert!(!clis.iter().any(|c| c.command == "fj"));
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

    #[test]
    fn binary_for_covers_every_provider() {
        for provider in GitProvider::ALL {
            assert!(
                !binary_for(provider).is_empty(),
                "no CLI binary for provider {provider}"
            );
        }
    }

    #[test]
    fn forgejo_is_served_by_the_gitea_compatible_tea_cli() {
        assert_eq!(binary_for(GitProvider::Forgejo), "tea");
        assert_eq!(binary_for(GitProvider::Gitea), "tea");
    }

    #[test]
    fn onboarding_metadata_present_for_operational_providers() {
        for provider in [GitProvider::GitHub, GitProvider::GitLab, GitProvider::Gitea] {
            let spec = spec_for(provider);
            assert!(!spec.install_url.is_empty(), "{provider}: no install_url");
            assert!(!spec.pat_url.is_empty(), "{provider}: no pat_url");
            assert!(!spec.display_name.is_empty(), "{provider}: no display_name");
            assert!(!spec.placeholder.is_empty(), "{provider}: no placeholder");
        }
    }

    /// The registry is the only place a provider binary may be named. A second
    /// copy is how `gh`/`glab`/`tea` drifted apart in the first place.
    #[test]
    fn provider_binaries_are_not_hardcoded_outside_the_registry() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();
        visit(&src, &mut |path: &std::path::Path, body: &str| {
            if path.ends_with("api/cli_detection.rs") {
                return;
            }
            for (n, line) in body.lines().enumerate() {
                for bin in ["gh", "glab", "tea"] {
                    if line.contains(&format!("Command::new(\"{bin}\")")) {
                        offenders.push(format!("{}:{}", path.display(), n + 1));
                    }
                }
            }
        });
        assert!(
            offenders.is_empty(),
            "provider binaries must come from CLI_SPECS, not a literal: {offenders:#?}"
        );
    }

    fn visit(dir: &std::path::Path, f: &mut impl FnMut(&std::path::Path, &str)) {
        for entry in std::fs::read_dir(dir).expect("src should be readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                visit(&path, f);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let body = std::fs::read_to_string(&path).expect("rust source should be readable");
                f(&path, &body);
            }
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
        assert_eq!(info.command, "tea");
        assert_eq!(info.name, "Gitea CLI (Forgejo-compatible)");
    }

    #[tokio::test]
    async fn test_detect_for_gitea() {
        let info = detect_for(GitProvider::Gitea).await;
        assert_eq!(info.command, "tea");
        assert_eq!(info.name, "Gitea CLI");
    }
}
