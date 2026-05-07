use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ─── Git Provider Configuration ────────────────────────────────────────────

/// Git provider configuration for PR/MR operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct GitConfig {
    /// Active provider (auto-detected from remote URL if not specified)
    #[serde(default)]
    pub provider: Option<GitProviderConfig>,
    /// GitHub-specific configuration
    #[serde(default)]
    pub github: GitHubConfig,
    /// GitLab-specific configuration (planned)
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

/// GitLab-specific configuration (planned)
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
