use crate::api::cli_detection::binary_for;
use crate::config::{GitCredentialConfig, GitExecutionConfig, GiteaConfig};
use crate::git::runtime::{self, GitRuntime};
use crate::types::pr::{provider_base_url, GitProvider};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{io::AsyncWriteExt, process::Command};

pub struct TeaCli {
    config: GiteaConfig,
}

impl TeaCli {
    pub fn new(config: GiteaConfig) -> Self {
        Self { config }
    }

    pub fn base_url(&self) -> Result<url::Url> {
        provider_base_url(self.config.host.as_deref(), "gitea.com")
    }

    pub async fn available(&self) -> bool {
        Command::new(binary_for(GitProvider::Gitea))
            .args(["api", "--help"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status()
            .await
            .is_ok_and(|s| s.success())
    }

    pub async fn request(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<&Value>,
        cwd: Option<&Path>,
    ) -> Result<Value> {
        ensure!(
            !endpoint.contains("://") && !endpoint.starts_with("//"),
            "Tea endpoint must be relative"
        );
        let base = self.base_url()?;
        let selected = runtime::current().and_then(|c| c.credentials);
        if let Some(credentials) = &selected {
            let destination = crate::git::identity::credential_url(&credentials.repository_url)?;
            ensure!(
                destination.origin() == base.origin(),
                "Delegated Git credential does not match Gitea host"
            );
            if let Some(path) = endpoint.strip_prefix("repos/") {
                let repo = path
                    .split('?')
                    .next()
                    .unwrap_or(path)
                    .split('/')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join("/");
                ensure!(
                    destination
                        .path()
                        .trim_matches('/')
                        .trim_end_matches(".git")
                        == repo,
                    "Delegated credential does not match Gitea repository"
                );
            }
        }
        let credentials = selected.unwrap_or(GitCredentialConfig {
            repository_url: base.join("operator/authentication")?.to_string(),
            username: "operator".into(),
            token_env: self.config.token_env.clone(),
        });
        let private = GitRuntime::create(&GitExecutionConfig {
            credentials: Some(credentials),
            ..Default::default()
        })?;
        let mut command = Command::new(binary_for(GitProvider::Gitea));
        command.args(["api", "--login", "operator", "--method", method]);
        if body.is_some() {
            command.args(["--data", "@-"]);
        }
        command
            .arg(endpoint)
            .env("XDG_CONFIG_HOME", &private.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }
        let mut child = command
            .spawn()
            .context("Gitea requires tea with the api command on PATH")?;
        if let Some(mut input) = child.stdin.take() {
            if let Some(body) = body {
                input.write_all(&serde_json::to_vec(body)?).await?;
            }
        }
        let output = tokio::time::timeout(Duration::from_secs(30), child.wait_with_output())
            .await
            .context("Gitea CLI request timed out")??;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("401") || stderr.contains("403") {
                anyhow::bail!("Gitea authentication or repository permission denied");
            }
            if stderr.contains("429")
                || stderr.contains("502")
                || stderr.contains("503")
                || stderr.contains("504")
            {
                anyhow::bail!("Gitea transient server failure");
            }
            anyhow::bail!(
                "Gitea CLI request failed; check endpoint permissions and tea API support"
            );
        }
        serde_json::from_slice(&output.stdout).context("Gitea CLI returned invalid JSON")
    }
}
