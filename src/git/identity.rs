use crate::config::{Config, GitExecutionConfig, GitIdentityConfig};
use crate::queue::Ticket;
use anyhow::{bail, ensure, Result};

pub const IDENTITY_ENV_NAMES: &[&str] = &[
    "GIT_AUTHOR_NAME",
    "GIT_AUTHOR_EMAIL",
    "GIT_COMMITTER_NAME",
    "GIT_COMMITTER_EMAIL",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitIdentity {
    pub name: String,
    pub email: String,
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

impl GitIdentity {
    pub fn to_export_block(&self) -> String {
        use std::fmt::Write;
        let mut exports = String::new();
        for key in IDENTITY_ENV_NAMES {
            let value = if key.ends_with("NAME") {
                &self.name
            } else {
                &self.email
            };
            writeln!(exports, "export {key}={}", shell_quote(value)).expect("writing to String");
        }
        exports
    }
}

fn interpolate(
    template: &str,
    ticket_id: &str,
    project: &str,
    ticket_type: &str,
) -> Result<String> {
    let mut result = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        ensure!(
            !rest[..start].contains('}'),
            "Invalid Git identity placeholder"
        );
        result.push_str(&rest[..start]);
        let end = rest[start..]
            .find('}')
            .ok_or_else(|| anyhow::anyhow!("Unclosed Git identity placeholder"))?
            + start;
        result.push_str(match &rest[start + 1..end] {
            "ticket_id" => ticket_id,
            "project" => project,
            "ticket_type" => ticket_type,
            _ => bail!("Unknown Git identity placeholder"),
        });
        rest = &rest[end + 1..];
    }
    ensure!(!rest.contains('}'), "Invalid Git identity placeholder");
    result.push_str(rest);
    ensure!(
        !result.trim().is_empty() && !result.contains(['\0', '\n', '\r']),
        "Git identity must be nonempty and single-line"
    );
    Ok(result)
}

pub fn valid_env_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
}

impl GitExecutionConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(identity) = &self.identity {
            interpolate(&identity.name, "ticket", "project", "type")?;
            interpolate(&identity.email, "ticket", "project", "type")?;
        }
        if let Some(credentials) = &self.credentials {
            let url = credential_url(&credentials.repository_url)?;
            ensure!(
                url.path().trim_matches('/').contains('/'),
                "Git credential URL must identify a repository"
            );
            ensure!(
                valid_env_name(&credentials.token_env),
                "Invalid Git token environment variable name"
            );
            ensure!(
                !credentials.username.is_empty()
                    && !credentials.username.contains(['\0', '\n', '\r']),
                "Invalid Git credential username"
            );
        }
        for entry in &self.settings {
            let key = entry.key.to_ascii_lowercase();
            ensure!(
                key.contains('.')
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')),
                "Invalid Git configuration key"
            );
            ensure!(
                !entry.value.contains('\0'),
                "Invalid Git configuration value"
            );
            ensure!(
                !key.starts_with("credential.")
                    && !key.starts_with("url.")
                    && !key.starts_with("http.")
                    && !key.starts_with("include")
                    && !matches!(
                        key.as_str(),
                        "user.name" | "user.email" | "core.askpass" | "core.sshcommand"
                    ),
                "Git configuration conflicts with managed identity or authentication"
            );
        }
        Ok(())
    }
}

pub fn credential_url(raw: &str) -> Result<url::Url> {
    let url = url::Url::parse(raw)
        .map_err(|_| anyhow::anyhow!("Invalid Git credential repository URL"))?;
    ensure!(url.scheme() == "https" && url.host_str().is_some() && url.username().is_empty() && url.password().is_none() && url.query().is_none() && url.fragment().is_none(), "Git credentials require an HTTPS repository URL without embedded credentials, query, or fragment");
    Ok(url)
}

pub fn resolve_config(
    config: &Config,
    ticket: &Ticket,
    delegator: Option<&str>,
) -> Result<Option<GitExecutionConfig>> {
    let mut resolved = match delegator {
        Some(name) => config
            .delegators
            .iter()
            .find(|d| d.name == name)
            .ok_or_else(|| anyhow::anyhow!("Unknown Git delegator: {name}"))?
            .git
            .clone()
            .unwrap_or_default(),
        None => GitExecutionConfig::default(),
    };
    if resolved.identity.is_none() {
        resolved.identity = config.git.identity.clone();
    }
    resolved.validate()?;
    if let Some(identity) = &mut resolved.identity {
        *identity = GitIdentityConfig {
            name: interpolate(
                &identity.name,
                &ticket.id,
                &ticket.project,
                &ticket.ticket_type,
            )?,
            email: interpolate(
                &identity.email,
                &ticket.id,
                &ticket.project,
                &ticket.ticket_type,
            )?,
        };
    }
    Ok((resolved != GitExecutionConfig::default()).then_some(resolved))
}

pub fn resolve_identity(config: &Config, ticket: &Ticket) -> Result<Option<GitIdentity>> {
    Ok(resolve_config(config, ticket, None)?
        .and_then(|c| c.identity)
        .map(|i| GitIdentity {
            name: i.name,
            email: i.email,
        }))
}

pub fn validate_config(config: &Config) -> Result<()> {
    crate::types::pr::ProviderHosts::from_config(&config.git)?;
    GitExecutionConfig {
        identity: config.git.identity.clone(),
        ..Default::default()
    }
    .validate()?;
    for delegator in &config.delegators {
        if let Some(git) = &delegator.git {
            git.validate()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_sets_both_pairs_and_escapes_shell() {
        let identity = GitIdentity {
            name: "Agent O'Neil".into(),
            email: "agent@example.org".into(),
        };
        let exports = identity.to_export_block();
        assert!(exports.contains("export GIT_AUTHOR_NAME='Agent O'\\''Neil'"));
        assert!(exports.contains("export GIT_COMMITTER_NAME='Agent O'\\''Neil'"));
        assert!(exports.contains("export GIT_AUTHOR_EMAIL='agent@example.org'"));
        assert!(exports.contains("export GIT_COMMITTER_EMAIL='agent@example.org'"));
    }

    #[test]
    fn interpolation_rejects_unknown_placeholders() {
        assert_eq!(
            interpolate(
                "bot-{ticket_id}-{project}-{ticket_type}",
                "42",
                "demo",
                "FIX"
            )
            .unwrap(),
            "bot-42-demo-FIX"
        );
        assert!(interpolate("{typo}", "42", "demo", "FIX").is_err());
    }
}
