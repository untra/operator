use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::api::cli_detection::onboarding_spec_for_slug;
use crate::config::{Config, GitProviderConfig};

#[derive(Debug)]
pub enum OnboardingStep {
    InstallCli {
        install_url: String,
        provider_display: String,
    },
    CollectToken {
        pat_url: String,
        provider: String,
        provider_display: String,
        placeholder: String,
    },
    AutoConfigured {
        username: String,
        token: String,
        provider: String,
        provider_display: String,
    },
}

fn is_cli_installed(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn grab_cli_token(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|token| !token.is_empty())
}

pub fn validate_github_token(token: &str) -> Result<String> {
    let response = reqwest::blocking::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", "operator")
        .send()
        .context("Failed to reach GitHub API")?;
    anyhow::ensure!(
        response.status().is_success(),
        "GitHub token validation failed (HTTP {})",
        response.status()
    );
    let body: serde_json::Value = response.json().context("Failed to parse GitHub response")?;
    body["login"]
        .as_str()
        .map(str::to_owned)
        .context("GitHub response missing 'login' field")
}

pub fn validate_gitlab_token(token: &str) -> Result<String> {
    let response = reqwest::blocking::Client::new()
        .get("https://gitlab.com/api/v4/user")
        .header("Private-Token", token)
        .header("User-Agent", "operator")
        .send()
        .context("Failed to reach GitLab API")?;
    anyhow::ensure!(
        response.status().is_success(),
        "GitLab token validation failed (HTTP {})",
        response.status()
    );
    let body: serde_json::Value = response.json().context("Failed to parse GitLab response")?;
    body["username"]
        .as_str()
        .map(str::to_owned)
        .context("GitLab response missing 'username' field")
}

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
    }
    Some(OnboardingStep::CollectToken {
        pat_url: meta.pat_url.to_string(),
        provider: provider.to_string(),
        provider_display: meta.display_name.to_string(),
        placeholder: meta.placeholder.to_string(),
    })
}

pub fn configure_git_provider(config: &mut Config, provider: &str, token_env: &str) -> Result<()> {
    match provider {
        "github" => {
            config.git.provider = Some(GitProviderConfig::GitHub);
            config.git.github.enabled = true;
            config.git.github.token_env = token_env.to_string();
        }
        "gitlab" => {
            config.git.provider = Some(GitProviderConfig::GitLab);
            config.git.gitlab.enabled = true;
            config.git.gitlab.token_env = token_env.to_string();
        }
        "gitea" => {
            config.git.provider = Some(GitProviderConfig::Gitea);
            config.git.gitea.enabled = true;
            config.git.gitea.token_env = token_env.to_string();
        }
        _ => anyhow::bail!("Unsupported provider: {provider}"),
    }
    Ok(())
}

pub fn apply_git_provider(config: &mut Config, provider: &str, token: &str) -> Result<()> {
    let token_env = token_env_for(config, provider)
        .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {provider}"))?;
    configure_git_provider(config, provider, &token_env)?;
    std::env::set_var(token_env, token);
    Ok(())
}

pub fn complete_git_onboarding(config: &mut Config, provider: &str, token: &str) -> Result<()> {
    apply_git_provider(config, provider, token)?;
    config.save()
}

pub fn token_env_for(config: &Config, provider: &str) -> Option<String> {
    match provider {
        "github" => Some(config.git.github.token_env.clone()),
        "gitlab" => Some(config.git.gitlab.token_env.clone()),
        "gitea" => Some(config.git.gitea.token_env.clone()),
        _ => None,
    }
}

pub fn shell_export_block(token_env: &str) -> String {
    format!("export {token_env}=<your-token>")
}

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

pub fn provider_is_configured(config: &Config, provider: &str) -> bool {
    match provider {
        "github" => config.git.provider == Some(GitProviderConfig::GitHub),
        "gitlab" => config.git.provider == Some(GitProviderConfig::GitLab),
        "gitea" => config.git.provider == Some(GitProviderConfig::Gitea),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_spec_for_supported_providers() {
        for provider in ["github", "gitlab", "gitea"] {
            assert!(onboarding_spec_for_slug(provider).is_some());
        }
        assert!(onboarding_spec_for_slug("bitbucket").is_none());
    }

    #[test]
    fn test_cli_helpers_reject_missing_binary() {
        assert!(!is_cli_installed("nonexistent-cli-tool-xyz-12345"));
        assert!(grab_cli_token("nonexistent-cli-tool-xyz-12345", &["auth", "token"]).is_none());
    }

    #[test]
    fn test_resolve_onboarding_rejects_unknown_provider() {
        assert!(resolve_onboarding("bitbucket").is_none());
    }
}
