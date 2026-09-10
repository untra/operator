use std::{
    collections::BTreeMap,
    fs,
    future::Future,
    io::Write,
    path::{Path, PathBuf},
};

use super::identity::{credential_url, shell_quote, GitIdentity};
use crate::config::GitExecutionConfig;
use anyhow::{ensure, Context, Result};

tokio::task_local! {
    static ACTIVE: Option<GitExecutionConfig>;
    static AUTH: Option<ProviderAuth>;
}

#[derive(Clone)]
pub struct ProviderAuth {
    /// The provider this auth belongs to. Drives which env vars the CLI gets;
    /// never infer that from the binary name.
    pub provider: crate::types::pr::GitProvider,
    pub token_env: String,
    pub host: Option<String>,
}

/// The `(token, host)` env var names a provider's CLI reads, when it reads any.
/// `tea` (Gitea, Forgejo) authenticates through a private config file instead,
/// and the detect-only providers have no CLI stack yet.
fn auth_env_keys(provider: crate::types::pr::GitProvider) -> Option<(&'static str, &'static str)> {
    use crate::types::pr::GitProvider;
    match provider {
        GitProvider::GitHub => Some(("GH_TOKEN", "GH_HOST")),
        GitProvider::GitLab => Some(("GITLAB_TOKEN", "GITLAB_HOST")),
        GitProvider::Gitea
        | GitProvider::Forgejo
        | GitProvider::Bitbucket
        | GitProvider::AzureDevOps => None,
    }
}

pub async fn auth_scope<T>(auth: Option<ProviderAuth>, future: impl Future<Output = T>) -> T {
    AUTH.scope(auth, future).await
}

pub async fn scope<T>(config: Option<GitExecutionConfig>, future: impl Future<Output = T>) -> T {
    ACTIVE.scope(config, future).await
}

pub fn current() -> Option<GitExecutionConfig> {
    ACTIVE.try_with(Clone::clone).ok().flatten()
}

pub fn validate_remote(config: &GitExecutionConfig, remote: &str) -> Result<()> {
    if let Some(credentials) = &config.credentials {
        let expected = credential_url(&credentials.repository_url)?;
        let actual = credential_url(remote)?;
        ensure!(
            expected.origin() == actual.origin()
                && repository_path(&expected) == repository_path(&actual),
            "Git credential repository does not match origin"
        );
    }
    Ok(())
}

fn repository_path(url: &url::Url) -> &str {
    url.path().trim_matches('/').trim_end_matches(".git")
}

pub fn private_dir(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(path)
        .context("Creating private Git runtime directory")?;
    Ok(())
}

pub fn private_file(path: &Path, contents: &[u8], executable: bool) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if executable { 0o700 } else { 0o600 });
    }
    options.open(path)?.write_all(contents)?;
    Ok(())
}

pub struct GitRuntime {
    pub path: PathBuf,
    env: BTreeMap<String, String>,
    keep: bool,
}

impl Drop for GitRuntime {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

impl GitRuntime {
    pub fn create(config: &GitExecutionConfig) -> Result<Self> {
        Self::create_with_token(config, None)
    }

    pub fn create_with_token(config: &GitExecutionConfig, token: Option<&str>) -> Result<Self> {
        config.validate()?;
        let path = PathBuf::from("/tmp").join(format!("operator-git-{}", uuid::Uuid::new_v4()));
        private_dir(&path)?;
        let mut runtime = Self {
            path,
            env: BTreeMap::new(),
            keep: false,
        };
        runtime.populate(config, token)?;
        Ok(runtime)
    }

    fn populate(
        &mut self,
        config: &GitExecutionConfig,
        supplied_token: Option<&str>,
    ) -> Result<()> {
        let mut settings = config
            .settings
            .iter()
            .map(|e| (e.key.clone(), e.value.clone()))
            .collect::<Vec<_>>();
        if let Some(identity) = &config.identity {
            for key in super::identity::IDENTITY_ENV_NAMES {
                self.env.insert(
                    (*key).into(),
                    if key.ends_with("NAME") {
                        identity.name.clone()
                    } else {
                        identity.email.clone()
                    },
                );
            }
        }
        if let Some(credentials) = &config.credentials {
            let token = match supplied_token {
                Some(token) => token.to_owned(),
                None => std::env::var(&credentials.token_env)
                    .context("Git credential environment variable is not set")?,
            };
            ensure!(
                !token.is_empty() && !token.contains(['\0', '\n', '\r']),
                "Git token must be nonempty and single-line"
            );
            let url = credential_url(&credentials.repository_url)?;
            let authority = &url[url::Position::BeforeHost..url::Position::AfterPort];
            let repo_path = repository_path(&url);
            private_file(
                &self.path.join("credential"),
                format!("username={}\npassword={}\n", credentials.username, token).as_bytes(),
                false,
            )?;
            let helper = format!(
                r#"#!/bin/sh
[ "$1" = get ] || exit 0
protocol= host= path=
while IFS='=' read -r key value; do
    case "$key" in protocol) protocol=$value;; host) host=$value;; path) path=$value;; esac
done
[ "$protocol" = https ] && [ "$host" = {host} ] || exit 0
case "$path" in {repo}|{repo_git}) cat "$(dirname "$0")/credential";; esac
"#,
                host = shell_quote(authority),
                repo = shell_quote(repo_path),
                repo_git = shell_quote(&format!("{repo_path}.git"))
            );
            private_file(&self.path.join("helper"), helper.as_bytes(), true)?;
            settings.extend([
                ("credential.helper".into(), String::new()),
                (
                    "credential.helper".into(),
                    self.path.join("helper").to_string_lossy().into_owned(),
                ),
                ("credential.useHttpPath".into(), "true".into()),
                ("http.followRedirects".into(), "false".into()),
                ("http.extraHeader".into(), String::new()),
            ]);
            self.env.insert("GIT_TERMINAL_PROMPT".into(), "0".into());
            self.env
                .insert("GIT_ASKPASS".into(), "/usr/bin/false".into());
            // These are private subprocess exports, never command arguments.
            for key in [
                "GH_TOKEN",
                "GITHUB_TOKEN",
                "GH_ENTERPRISE_TOKEN",
                "GITHUB_ENTERPRISE_TOKEN",
                "GITLAB_TOKEN",
                "GITLAB_ACCESS_TOKEN",
                "OAUTH_TOKEN",
            ] {
                self.env.insert(key.into(), token.clone());
            }
            self.env.insert("GH_HOST".into(), authority.into());
            self.env.insert("GITLAB_HOST".into(), authority.into());
            private_dir(&self.path.join("tea"))?;
            let tea = serde_json::json!({"logins": [{"name": "operator", "url": url.origin().ascii_serialization(), "user": credentials.username, "token": token, "default": true}], "preferences": {}});
            private_file(
                &self.path.join("tea/config.yml"),
                serde_json::to_string(&tea)?.as_bytes(),
                false,
            )?;
            private_dir(&self.path.join("bin"))?;
            // `OPERATOR_TEA_BINARY` is resolved by env.sh *on the target*, before
            // this directory joins PATH, so the shim never recurses into itself.
            // It is empty when the target has no `tea` -- Operator installs no
            // client binaries -- and `exec ""` would fail as an unreadable
            // `exec: : not found` that blames the shim. Say what is missing
            // instead; the launch preflight should have caught it first.
            let tea_wrapper = format!(
                "#!/bin/sh\nif [ -z \"${{OPERATOR_TEA_BINARY:-}}\" ]; then\n  echo \"operator: tea not found on PATH; install it on this machine to use Gitea or Forgejo\" >&2\n  exit 127\nfi\nXDG_CONFIG_HOME={} exec \"$OPERATOR_TEA_BINARY\" \"$@\"\n",
                shell_quote(&self.path.to_string_lossy())
            );
            private_file(&self.path.join("bin/tea"), tea_wrapper.as_bytes(), true)?;
        }
        let inherited = std::env::var("GIT_CONFIG_COUNT").unwrap_or_default();
        let offset: usize = if inherited.is_empty() {
            0
        } else {
            inherited
                .parse()
                .context("Invalid inherited GIT_CONFIG_COUNT")?
        };
        ensure!(
            offset <= 4096,
            "Inherited Git runtime configuration is too large"
        );
        for index in 0..offset {
            for part in ["KEY", "VALUE"] {
                let key = format!("GIT_CONFIG_{part}_{index}");
                self.env.insert(
                    key.clone(),
                    std::env::var(key).context("Incomplete inherited Git runtime configuration")?,
                );
            }
        }
        for (index, (key, value)) in settings.into_iter().enumerate() {
            self.env
                .insert(format!("GIT_CONFIG_KEY_{}", offset + index), key);
            self.env
                .insert(format!("GIT_CONFIG_VALUE_{}", offset + index), value);
        }
        let count = self
            .env
            .keys()
            .filter(|k| k.starts_with("GIT_CONFIG_KEY_"))
            .count();
        self.env
            .insert("GIT_CONFIG_COUNT".into(), count.to_string());
        use std::fmt::Write as _;
        let mut exports = String::new();
        for (key, value) in &self.env {
            writeln!(exports, "export {key}={}", shell_quote(value)).expect("writing to String");
        }
        if config.credentials.is_some() {
            exports.push_str(&format!(
                "export OPERATOR_TEA_BINARY=$(command -v tea || true)\nexport PATH={}:\"$PATH\"\n",
                shell_quote(&self.path.join("bin").to_string_lossy())
            ));
        }
        private_file(&self.path.join("env.sh"), exports.as_bytes(), false)?;
        Ok(())
    }

    pub fn apply(&self, command: &mut tokio::process::Command) {
        command.envs(&self.env);
    }

    pub fn persist(mut self) -> PathBuf {
        self.keep = true;
        self.path.clone()
    }
}

pub fn configure_command(command: &mut tokio::process::Command) -> Result<Option<GitRuntime>> {
    let config = current();
    if config.as_ref().is_none_or(|c| c.credentials.is_none()) {
        if let Some(auth) = AUTH.try_with(Clone::clone).ok().flatten() {
            if let Some((token_key, host_key)) = auth_env_keys(auth.provider) {
                if let Ok(token) = std::env::var(&auth.token_env) {
                    ensure!(!token.is_empty(), "Configured provider token is empty");
                    command.env(token_key, token);
                }
                if let Some(host) = auth.host {
                    command.env(host_key, host);
                }
            }
        }
    }
    let runtime = config.as_ref().map(GitRuntime::create).transpose()?;
    if let Some(runtime) = &runtime {
        runtime.apply(command);
    }
    Ok(runtime)
}

pub fn identity_exports(config: &GitExecutionConfig) -> String {
    config
        .identity
        .as_ref()
        .map(|i| {
            GitIdentity {
                name: i.name.clone(),
                email: i.email.clone(),
            }
            .to_export_block()
        })
        .unwrap_or_default()
}

pub fn managed_runtime_path(path: &Path) -> bool {
    path.parent() == Some(Path::new("/tmp"))
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_prefix("operator-git-"))
            .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
}

pub fn abandoned_runtime_pointers(config: &crate::config::Config) -> Vec<(PathBuf, PathBuf)> {
    const LAUNCH_GRACE: std::time::Duration = std::time::Duration::from_hours(1);
    let Ok(state) = crate::state::State::load(config) else {
        return Vec::new();
    };
    let active: std::collections::HashSet<_> = state
        .agents
        .iter()
        .filter_map(|a| {
            a.step_launch_context
                .as_ref()
                .and_then(|c| c.session_id.as_deref())
        })
        .collect();
    let Ok(entries) = fs::read_dir(config.tickets_path().join("operator/commands")) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| {
            let pointer = entry.ok()?.path();
            if pointer.extension()?.to_str()? != "git-runtime"
                || active.contains(pointer.file_stem()?.to_str()?)
            {
                return None;
            }
            if pointer.metadata().ok()?.modified().ok()?.elapsed().ok()? < LAUNCH_GRACE {
                return None;
            }
            let path = PathBuf::from(fs::read_to_string(&pointer).ok()?);
            managed_runtime_path(&path).then_some((pointer, path))
        })
        .collect()
}

pub fn reconcile_local(config: &crate::config::Config) {
    for (pointer, path) in abandoned_runtime_pointers(config) {
        if pointer.with_extension("git-remote").exists() {
            continue;
        }
        if fs::symlink_metadata(&path).is_ok_and(|m| !m.file_type().is_dir()) {
            continue;
        }
        if let Ok(pid) = fs::read_to_string(path.join("pid")) {
            let Ok(pid) = pid.trim().parse::<u32>() else {
                continue;
            };
            if pid == 0
                || std::process::Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success())
            {
                continue;
            }
        }
        if !path.exists() || fs::remove_dir_all(path).is_ok() {
            let _ = fs::remove_file(pointer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GitCredentialConfig, GitIdentityConfig};
    use std::process::{Command, Stdio};

    fn credential_config() -> GitExecutionConfig {
        GitExecutionConfig {
            credentials: Some(GitCredentialConfig {
                repository_url: "https://git.example/team/project.git".into(),
                username: "agent".into(),
                token_env: "OPERATOR_TEST_GIT_TOKEN".into(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn helper_returns_credentials_only_for_bound_repository() {
        let runtime =
            GitRuntime::create_with_token(&credential_config(), Some("secret-test-token")).unwrap();
        for (host, path, expected) in [
            ("git.example", "team/project.git", true),
            ("git.example", "team/other", false),
            ("evil.example", "team/project", false),
        ] {
            let mut child = Command::new(runtime.path.join("helper"))
                .arg("get")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(format!("protocol=https\nhost={host}\npath={path}\n\n").as_bytes())
                .unwrap();
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            assert_eq!(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("secret-test-token"),
                expected
            );
        }
    }

    /// With `tea` absent from the target, `OPERATOR_TEA_BINARY` is empty and
    /// the PATH shim used to `exec ""` -- an unreadable failure deep inside an
    /// agent run. Operator installs no client binaries, so this path is
    /// reachable whenever a target has not been provisioned with `tea`.
    #[test]
    fn tea_shim_reports_a_missing_binary_instead_of_execing_nothing() {
        let runtime =
            GitRuntime::create_with_token(&credential_config(), Some("secret-test-token")).unwrap();
        let shim = runtime.path.join("bin/tea");

        let output = Command::new(&shim)
            .arg("--version")
            .env_remove("OPERATOR_TEA_BINARY")
            .output()
            .unwrap();

        assert!(!output.status.success(), "shim must fail, not exec nothing");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("tea not found on PATH"),
            "shim must say what is missing, got: {stderr}"
        );
        // The bare `exec ""` failure; unreadable and blames the shim, not tea.
        assert!(
            !stderr.contains("exec: : not found"),
            "shim still execs the empty string: {stderr}"
        );
        assert!(
            !stderr.contains("secret-test-token"),
            "shim must not leak the token"
        );
    }

    #[test]
    fn runtime_is_private_and_removed_on_drop() {
        use std::os::unix::fs::PermissionsExt;
        let runtime =
            GitRuntime::create_with_token(&credential_config(), Some("secret-test-token")).unwrap();
        let path = runtime.path.clone();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        for file in ["credential", "env.sh", "tea/config.yml"] {
            assert_eq!(
                fs::metadata(path.join(file)).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        drop(runtime);
        assert!(!path.exists());
    }

    #[test]
    fn remote_validation_rejects_ssh_and_other_repositories() {
        let config = credential_config();
        assert!(validate_remote(&config, "git@git.example:team/project.git").is_err());
        assert!(validate_remote(&config, "https://git.example/team/other").is_err());
        assert!(validate_remote(&config, "https://git.example/team/project").is_ok());
    }

    /// Env injection must follow the provider the auth was built for, not the
    /// spelling of the binary. Sniffing the program name handed every non-`gh`
    /// tool a `GITLAB_TOKEN`, including `tea`.
    #[tokio::test]
    async fn provider_auth_injects_only_its_own_provider_env() {
        use crate::types::pr::GitProvider;
        std::env::set_var("OPERATOR_TEST_PROVIDER_TOKEN", "provider-secret");

        async fn envs_for(provider: GitProvider, program: &str) -> Vec<(String, String)> {
            let auth = ProviderAuth {
                provider,
                token_env: "OPERATOR_TEST_PROVIDER_TOKEN".into(),
                host: Some("git.example".into()),
            };
            auth_scope(Some(auth), async {
                let mut command = tokio::process::Command::new(program);
                let _runtime = configure_command(&mut command).unwrap();
                command
                    .as_std()
                    .get_envs()
                    .filter_map(|(k, v)| {
                        Some((
                            k.to_string_lossy().into_owned(),
                            v?.to_string_lossy().into_owned(),
                        ))
                    })
                    .collect()
            })
            .await
        }

        let github = envs_for(GitProvider::GitHub, "gh").await;
        assert!(github
            .iter()
            .any(|(k, v)| k == "GH_TOKEN" && v == "provider-secret"));
        assert!(github
            .iter()
            .any(|(k, v)| k == "GH_HOST" && v == "git.example"));
        assert!(!github.iter().any(|(k, _)| k == "GITLAB_TOKEN"));

        let gitlab = envs_for(GitProvider::GitLab, "glab").await;
        assert!(gitlab
            .iter()
            .any(|(k, v)| k == "GITLAB_TOKEN" && v == "provider-secret"));
        assert!(!gitlab.iter().any(|(k, _)| k == "GH_TOKEN"));

        // `tea` authenticates through its own private config, never these vars.
        let gitea = envs_for(GitProvider::Gitea, "tea").await;
        assert!(!gitea.iter().any(|(k, _)| k == "GITLAB_TOKEN"));
        assert!(!gitea.iter().any(|(k, _)| k == "GH_TOKEN"));

        std::env::remove_var("OPERATOR_TEST_PROVIDER_TOKEN");
    }

    #[tokio::test]
    async fn parallel_worktree_commits_use_separate_author_and_committer() {
        let root = tempfile::tempdir().unwrap();
        let repo = root.path().join("repo");
        fs::create_dir(&repo).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&repo)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init"]);
        git(&[
            "-c",
            "user.name=Human",
            "-c",
            "user.email=human@example.org",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ]);
        let one = root.path().join("one");
        let two = root.path().join("two");
        git(&["worktree", "add", "-b", "one", one.to_str().unwrap()]);
        git(&["worktree", "add", "-b", "two", two.to_str().unwrap()]);
        let original = fs::read(repo.join(".git/config")).unwrap();
        async fn commit(path: &Path, name: &str) {
            let config = GitExecutionConfig {
                identity: Some(GitIdentityConfig {
                    name: name.into(),
                    email: format!("{name}@example.org"),
                }),
                ..Default::default()
            };
            scope(Some(config), async {
                let mut command = tokio::process::Command::new("git");
                command
                    .args([
                        "-c",
                        "commit.gpgsign=false",
                        "commit",
                        "--allow-empty",
                        "-m",
                        "agent",
                    ])
                    .current_dir(path);
                let _runtime = configure_command(&mut command).unwrap();
                assert!(command.output().await.unwrap().status.success());
            })
            .await;
            let output = Command::new("git")
                .args(["log", "-1", "--format=%an|%ae|%cn|%ce"])
                .current_dir(path)
                .output()
                .unwrap();
            assert_eq!(
                String::from_utf8(output.stdout).unwrap().trim(),
                format!("{name}|{name}@example.org|{name}|{name}@example.org")
            );
        }
        tokio::join!(commit(&one, "one"), commit(&two, "two"));
        assert_eq!(fs::read(repo.join(".git/config")).unwrap(), original);
    }
}
