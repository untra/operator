//! Git provider onboarding logic.
//!
//! Detects CLI tools, grabs tokens, validates credentials, and resolves
//! the appropriate onboarding step for a given provider.

use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::api::cli_detection::onboarding_spec_for_slug;
use crate::config::{Config, GitProviderConfig};

/// The resolved onboarding step for a provider.
#[derive(Debug)]
pub enum OnboardingStep {
    /// CLI not installed - open install page.
    InstallCli {
        install_url: String,
        provider_display: String,
    },
    /// CLI installed but no token - show PAT dialog.
    CollectToken {
        pat_url: String,
        provider: String,
        provider_display: String,
        placeholder: String,
    },
    /// CLI installed and authenticated - token ready to use.
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
    let meta = onboarding_spec_for_slug(provider)?;

    if !is_cli_installed(meta.command) {
        return Some(OnboardingStep::InstallCli {
            install_url: meta.install_url.to_string(),
            provider_display: meta.display_name.to_string(),
        });
    }

    if let Some(token) = (!meta.auth_args.is_empty())
        .then(|| grab_cli_token(meta.command, meta.auth_args))
        .flatten()
    {
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
        "gitea" => {
            config.git.provider = Some(GitProviderConfig::Gitea);
            config.git.gitea.enabled = true;
            config.save()?;
            std::env::set_var(&config.git.gitea.token_env, token);
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

pub fn resolve_onboarding_with_config(config: &Config, provider: &str) -> Option<OnboardingStep> {
    let mut step = resolve_onboarding(provider)?;
    if provider == "gitea" {
        let base =
            crate::types::pr::provider_base_url(config.git.gitea.host.as_deref(), "gitea.com")
                .ok()?;
        if let OnboardingStep::CollectToken { pat_url, .. } = &mut step {
            *pat_url = base.join("user/settings/applications").ok()?.to_string();
        }
    }
    Some(step)
}

pub fn validate_token_with_config(config: &Config, provider: &str, token: &str) -> Result<String> {
    if provider != "gitea" {
        return validate_token(provider, token);
    }
    let base = crate::types::pr::provider_base_url(config.git.gitea.host.as_deref(), "gitea.com")?;
    let git = crate::config::GitExecutionConfig {
        credentials: Some(crate::config::GitCredentialConfig {
            repository_url: base.join("operator/authentication")?.to_string(),
            username: "operator".into(),
            token_env: config.git.gitea.token_env.clone(),
        }),
        ..Default::default()
    };
    let runtime = crate::git::runtime::GitRuntime::create_with_token(&git, Some(token))?;
    let output = Command::new(crate::api::cli_detection::binary_for(
        crate::types::pr::GitProvider::Gitea,
    ))
    .args(["api", "--login", "operator", "user"])
    .env("XDG_CONFIG_HOME", &runtime.path)
    .output()
    .context("Gitea requires tea with the api command")?;
    anyhow::ensure!(output.status.success(), "Gitea token validation failed");
    let body: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    body["login"]
        .as_str()
        .map(str::to_owned)
        .context("Gitea response missing login")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_for_github() {
        let meta = onboarding_spec_for_slug("github").unwrap();
        assert_eq!(meta.command, "gh");
        assert_eq!(meta.display_name, "GitHub");
        assert_eq!(
            meta.pat_url,
            "https://github.com/settings/personal-access-tokens/new"
        );
    }

    #[test]
    fn test_meta_for_gitlab() {
        let meta = onboarding_spec_for_slug("gitlab").unwrap();
        assert_eq!(meta.command, "glab");
        assert_eq!(meta.display_name, "GitLab");
        assert_eq!(
            meta.pat_url,
            "https://gitlab.com/-/user_settings/personal_access_tokens"
        );
    }

    #[test]
    fn test_meta_for_unknown_returns_none() {
        assert!(onboarding_spec_for_slug("bitbucket").is_none());
        assert!(onboarding_spec_for_slug("").is_none());
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
