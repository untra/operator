//! Git provider onboarding logic.
//!
//! Detects CLI tools, grabs tokens, validates credentials, and resolves
//! the appropriate onboarding step for a given provider.

use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::config::{Config, GitProviderConfig};

/// Per-provider constants for onboarding.
struct ProviderMeta {
    cli_command: &'static str,
    cli_auth_args: &'static [&'static str],
    cli_install_url: &'static str,
    pat_url: &'static str,
    display_name: &'static str,
    placeholder: &'static str,
}

const GITHUB: ProviderMeta = ProviderMeta {
    cli_command: "gh",
    cli_auth_args: &["auth", "token"],
    cli_install_url: "https://cli.github.com/",
    pat_url: "https://github.com/settings/personal-access-tokens/new",
    display_name: "GitHub",
    placeholder: "ghp_...",
};

const GITLAB: ProviderMeta = ProviderMeta {
    cli_command: "glab",
    cli_auth_args: &["auth", "token"],
    cli_install_url: "https://docs.gitlab.com/cli",
    pat_url: "https://gitlab.com/-/user_settings/personal_access_tokens",
    display_name: "GitLab",
    placeholder: "glpat-...",
};

fn meta_for(provider: &str) -> Option<&'static ProviderMeta> {
    match provider {
        "github" => Some(&GITHUB),
        "gitlab" => Some(&GITLAB),
        _ => None,
    }
}

/// The resolved onboarding step for a provider.
#[derive(Debug)]
pub enum OnboardingStep {
    /// CLI not installed — open install page.
    InstallCli {
        install_url: String,
        provider_display: String,
    },
    /// CLI installed but no token — show PAT dialog.
    CollectToken {
        pat_url: String,
        provider: String,
        provider_display: String,
        placeholder: String,
    },
    /// CLI installed and authenticated — token ready to use.
    AutoConfigured {
        username: String,
        token: String,
        provider: String,
        provider_display: String,
    },
}

/// Check if a CLI tool is available on PATH (synchronous).
fn is_cli_installed(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Try to grab an auth token from a CLI tool (synchronous).
fn grab_cli_token(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            None
        } else {
            Some(token)
        }
    } else {
        None
    }
}

/// Validate a GitHub personal access token and return the username.
pub fn validate_github_token(token: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", "operator")
        .send()
        .context("Failed to reach GitHub API")?;

    if !resp.status().is_success() {
        anyhow::bail!("GitHub token validation failed (HTTP {})", resp.status());
    }

    let body: serde_json::Value = resp.json().context("Failed to parse GitHub response")?;
    body["login"]
        .as_str()
        .map(std::string::ToString::to_string)
        .context("GitHub response missing 'login' field")
}

/// Validate a GitLab personal access token and return the username.
pub fn validate_gitlab_token(token: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://gitlab.com/api/v4/user")
        .header("Private-Token", token)
        .header("User-Agent", "operator")
        .send()
        .context("Failed to reach GitLab API")?;

    if !resp.status().is_success() {
        anyhow::bail!("GitLab token validation failed (HTTP {})", resp.status());
    }

    let body: serde_json::Value = resp.json().context("Failed to parse GitLab response")?;
    body["username"]
        .as_str()
        .map(std::string::ToString::to_string)
        .context("GitLab response missing 'username' field")
}

/// Resolve the onboarding step for a provider.
///
/// Checks CLI installation → CLI authentication → returns the appropriate step.
pub fn resolve_onboarding(provider: &str) -> Option<OnboardingStep> {
    let meta = meta_for(provider)?;

    if !is_cli_installed(meta.cli_command) {
        return Some(OnboardingStep::InstallCli {
            install_url: meta.cli_install_url.to_string(),
            provider_display: meta.display_name.to_string(),
        });
    }

    if let Some(token) = grab_cli_token(meta.cli_command, meta.cli_auth_args) {
        // Validate the token
        let username = match provider {
            "github" => validate_github_token(&token),
            "gitlab" => validate_gitlab_token(&token),
            _ => return None,
        };

        if let Ok(username) = username {
            return Some(OnboardingStep::AutoConfigured {
                username,
                token,
                provider: provider.to_string(),
                provider_display: meta.display_name.to_string(),
            });
        }
        // CLI token is stale/invalid, fall through to manual entry
    }

    Some(OnboardingStep::CollectToken {
        pat_url: meta.pat_url.to_string(),
        provider: provider.to_string(),
        provider_display: meta.display_name.to_string(),
        placeholder: meta.placeholder.to_string(),
    })
}

/// Complete git onboarding by writing provider config and setting the env var.
pub fn complete_git_onboarding(config: &mut Config, provider: &str, token: &str) -> Result<()> {
    match provider {
        "github" => {
            config.git.provider = Some(GitProviderConfig::GitHub);
            config.git.github.enabled = true;
            config.save()?;
            std::env::set_var(&config.git.github.token_env, token);
        }
        "gitlab" => {
            config.git.provider = Some(GitProviderConfig::GitLab);
            config.git.gitlab.enabled = true;
            config.save()?;
            std::env::set_var(&config.git.gitlab.token_env, token);
        }
        _ => anyhow::bail!("Unsupported provider: {provider}"),
    }
    Ok(())
}

/// Validate a token for the given provider, returning the username on success.
pub fn validate_token(provider: &str, token: &str) -> Result<String> {
    match provider {
        "github" => validate_github_token(token),
        "gitlab" => validate_gitlab_token(token),
        _ => anyhow::bail!("Unsupported provider: {provider}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_for_github() {
        let meta = meta_for("github").unwrap();
        assert_eq!(meta.cli_command, "gh");
        assert_eq!(meta.display_name, "GitHub");
        assert_eq!(
            meta.pat_url,
            "https://github.com/settings/personal-access-tokens/new"
        );
    }

    #[test]
    fn test_meta_for_gitlab() {
        let meta = meta_for("gitlab").unwrap();
        assert_eq!(meta.cli_command, "glab");
        assert_eq!(meta.display_name, "GitLab");
        assert_eq!(
            meta.pat_url,
            "https://gitlab.com/-/user_settings/personal_access_tokens"
        );
    }

    #[test]
    fn test_meta_for_unknown_returns_none() {
        assert!(meta_for("bitbucket").is_none());
        assert!(meta_for("").is_none());
    }

    #[test]
    fn test_is_cli_installed_nonexistent() {
        assert!(!is_cli_installed("nonexistent-cli-tool-xyz-12345"));
    }

    #[test]
    fn test_grab_cli_token_nonexistent() {
        assert!(grab_cli_token("nonexistent-cli-tool-xyz-12345", &["auth", "token"]).is_none());
    }

    #[test]
    fn test_resolve_onboarding_unknown_provider() {
        assert!(resolve_onboarding("bitbucket").is_none());
    }
}
