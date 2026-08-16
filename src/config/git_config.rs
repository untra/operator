use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::types::pr::GitProvider;

// ─── Git Provider Configuration ────────────────────────────────────────────

/// Git provider configuration for PR/MR operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct GitConfig {
    /// Default commit identity for delegated work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<GitIdentityConfig>,
    #[serde(default)]
    pub gitea: GiteaConfig,
    #[serde(default)]
    pub forgejo: ForgejoConfig,
    /// Active provider (auto-detected from remote URL if not specified)
    #[serde(default)]
    pub provider: Option<GitProviderConfig>,
    /// GitHub-specific configuration
    #[serde(default)]
    pub github: GitHubConfig,
    /// GitLab-specific configuration
    #[serde(default)]
    pub gitlab: GitLabConfig,
    /// Branch naming format (e.g., "{type}/{ticket_id}-{slug}")
    #[serde(default = "default_branch_format")]
    pub branch_format: String,
    /// Whether to use git worktrees for per-ticket isolation (default: false)
    /// When false, tickets work directly in the project directory with branches
    #[serde(default)]
    pub use_worktrees: bool,
}

fn default_branch_format() -> String {
    "{type}/{ticket_id}".to_string()
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            identity: None,
            gitea: GiteaConfig::default(),
            forgejo: ForgejoConfig::default(),
            provider: None,
            github: GitHubConfig::default(),
            gitlab: GitLabConfig::default(),
            branch_format: default_branch_format(),
            use_worktrees: false,
        }
    }
}

/// Git provider selection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum GitProviderConfig {
    /// GitHub (github.com)
    GitHub,
    /// GitLab (gitlab.com or self-hosted)
    GitLab,
    /// Bitbucket (bitbucket.org)
    Bitbucket,
    /// Azure DevOps (dev.azure.com)
    AzureDevOps,
    /// Forgejo (e.g. codeberg.org or self-hosted)
    Forgejo,
    /// Gitea (gitea.com or self-hosted)
    Gitea,
}

impl From<GitProviderConfig> for GitProvider {
    fn from(config: GitProviderConfig) -> Self {
        match config {
            GitProviderConfig::GitHub => GitProvider::GitHub,
            GitProviderConfig::GitLab => GitProvider::GitLab,
            GitProviderConfig::Bitbucket => GitProvider::Bitbucket,
            GitProviderConfig::AzureDevOps => GitProvider::AzureDevOps,
            GitProviderConfig::Forgejo => GitProvider::Forgejo,
            GitProviderConfig::Gitea => GitProvider::Gitea,
        }
    }
}

/// GitHub-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct GitHubConfig {
    /// Whether GitHub integration is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Environment variable containing the GitHub token (default: `GITHUB_TOKEN`)
    #[serde(default = "default_github_token_env")]
    pub token_env: String,
}

fn default_true() -> bool {
    true
}

fn default_github_token_env() -> String {
    "GITHUB_TOKEN".to_string()
}

/// GitLab-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct GitLabConfig {
    /// Whether GitLab integration is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable containing the GitLab token (default: `GITLAB_TOKEN`)
    #[serde(default = "default_gitlab_token_env")]
    pub token_env: String,
    /// GitLab host (default: gitlab.com, can be self-hosted)
    #[serde(default)]
    pub host: Option<String>,
}

fn default_gitlab_token_env() -> String {
    "GITLAB_TOKEN".to_string()
}

/// Commit identity template for delegated work.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema, PartialEq, Eq)]
#[ts(export)]
pub struct GitIdentityConfig {
    pub name: String,
    pub email: String,
}

/// Supplied HTTPS credential, bound to a repository; contains no secret value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema, PartialEq, Eq)]
#[ts(export)]
pub struct GitCredentialConfig {
    pub repository_url: String,
    pub username: String,
    pub token_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema, PartialEq, Eq)]
#[ts(export)]
pub struct GitConfigEntry {
    pub key: String,
    pub value: String,
}

/// Git settings owned by a named delegator.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema, PartialEq, Eq,
)]
#[ts(export)]
pub struct GitExecutionConfig {
    #[serde(default)]
    pub identity: Option<GitIdentityConfig>,
    #[serde(default)]
    pub credentials: Option<GitCredentialConfig>,
    #[serde(default)]
    pub settings: Vec<GitConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct GiteaConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_gitea_token_env")]
    pub token_env: String,
    /// HTTPS host or base URL; defaults to gitea.com.
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default = "default_wip_prefix")]
    pub wip_prefix: String,
}

fn default_gitea_token_env() -> String {
    "GITEA_TOKEN".into()
}

impl Default for GiteaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token_env: default_gitea_token_env(),
            host: None,
            wip_prefix: default_wip_prefix(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ForgejoConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_forgejo_token_env")]
    pub token_env: String,
    /// HTTPS host or base URL; defaults to codeberg.org.
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default = "default_wip_prefix")]
    pub wip_prefix: String,
}

fn default_forgejo_token_env() -> String {
    "FORGEJO_TOKEN".into()
}

impl Default for ForgejoConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token_env: default_forgejo_token_env(),
            host: None,
            wip_prefix: default_wip_prefix(),
        }
    }
}

fn default_wip_prefix() -> String {
    "WIP: ".into()
}

#[cfg(test)]
mod delegation_tests {
    use super::*;
    #[test]
    fn git_defaults_have_no_identity_and_provider_defaults_agree() {
        assert!(GitConfig::default().identity.is_none());
        let gitea: GiteaConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(gitea.token_env, GiteaConfig::default().token_env);
        assert!(!gitea.enabled);
        let forgejo: ForgejoConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(forgejo.token_env, ForgejoConfig::default().token_env);
        assert!(!forgejo.enabled);
    }
    #[test]
    fn credentials_reject_embedded_tokens_and_managed_config_overrides() {
        let credentials = GitCredentialConfig {
            repository_url: "https://user:secret@git.example/a/b".into(),
            username: "bot".into(),
            token_env: "TOKEN".into(),
        };
        assert!(GitExecutionConfig {
            credentials: Some(credentials),
            ..Default::default()
        }
        .validate()
        .is_err());
        for key in [
            "credential.helper",
            "user.name",
            "http.extraHeader",
            "include.path",
        ] {
            assert!(GitExecutionConfig {
                settings: vec![GitConfigEntry {
                    key: key.into(),
                    value: "value".into()
                }],
                ..Default::default()
            }
            .validate()
            .is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_provider_config_into_git_provider() {
        assert_eq!(
            GitProvider::from(GitProviderConfig::GitHub),
            GitProvider::GitHub
        );
        assert_eq!(
            GitProvider::from(GitProviderConfig::GitLab),
            GitProvider::GitLab
        );
        assert_eq!(
            GitProvider::from(GitProviderConfig::Bitbucket),
            GitProvider::Bitbucket
        );
        assert_eq!(
            GitProvider::from(GitProviderConfig::AzureDevOps),
            GitProvider::AzureDevOps
        );
        assert_eq!(
            GitProvider::from(GitProviderConfig::Forgejo),
            GitProvider::Forgejo
        );
        assert_eq!(
            GitProvider::from(GitProviderConfig::Gitea),
            GitProvider::Gitea
        );
    }
}
