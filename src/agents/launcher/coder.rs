//! Coder workspace target: lifecycle + SSH alias provisioning.
//!
//! A coder target's execution shape is an SSH target with a
//! dynamically-provisioned alias — the launch itself reuses `remote.rs`
//! unchanged. This module owns only what is Coder-specific: deterministic
//! workspace naming, create/start lifecycle (never delete), the SSH config
//! fragment (`ProxyCommand coder ssh --stdio`), and the git checkout on the
//! workspace. Identity is a plain user session token resolved from the
//! environment **by name** and never written to disk.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::config::{CoderConfig, Config, RemoteHost};

use super::prompt::shell_escape;

/// Coder caps workspace names at 32 characters.
const MAX_WORKSPACE_NAME: usize = 32;
/// Over budget: keep this much of the readable key, then `-` + 6 hex of hash.
const TRUNCATED_KEY_LEN: usize = 25;

/// Resolved Coder credentials — env values read at launch time, held only in
/// memory and injected into `coder` child processes under the CLI's standard
/// variable names.
#[derive(Debug)]
pub(crate) struct CoderSession {
    pub url: String,
    pub token: String,
}

/// Resolve URL and token from the environment by the configured variable
/// names. Missing either fails fast, naming the variable.
pub(crate) fn resolve_session(coder: &CoderConfig) -> Result<CoderSession> {
    let url = std::env::var(&coder.url_env).map_err(|_| {
        anyhow::anyhow!(
            "Coder target requires the '{}' environment variable (deployment URL); it is not set",
            coder.url_env
        )
    })?;
    let token = std::env::var(&coder.token_env).map_err(|_| {
        anyhow::anyhow!(
            "Coder target requires the '{}' environment variable (session token); it is not set",
            coder.token_env
        )
    })?;
    Ok(CoderSession { url, token })
}

/// Deterministic per-ticket workspace name: `{prefix}-{project}-{ticket_id}`,
/// lowercased, non-alphanumerics collapsed to single hyphens. Determinism is
/// what makes "never delete" an asset: the same ticket always maps to the
/// same workspace, so relaunch reuses it. Names over Coder's 32-char cap are
/// truncated and suffixed with 6 hex chars of a hash of the full key.
pub fn workspace_name(prefix: &str, project: &str, ticket_id: &str) -> String {
    let full = sanitize(&format!("{prefix}-{project}-{ticket_id}"));
    if full.len() <= MAX_WORKSPACE_NAME {
        return full;
    }
    let hash = fnv1a(&full);
    let truncated = full[..TRUNCATED_KEY_LEN].trim_end_matches('-');
    format!("{truncated}-{:06x}", hash & 0xFF_FFFF)
}

fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_hyphen = false;
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_hyphen = false;
        } else if !prev_hyphen {
            out.push('-');
            prev_hyphen = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// The provisioned ssh alias for a workspace.
pub fn workspace_alias(workspace: &str) -> String {
    format!("op-coder-{workspace}")
}

/// SSH config fragment content: `coder ssh --stdio` as a `ProxyCommand`, so
/// real `ssh` — with the full flag set (`-t`, `-R`) — works over the Coder
/// tailnet. Operator writes its own fragment rather than running
/// `coder config-ssh`, which rewrites the user's `~/.ssh/config`.
pub fn ssh_fragment(workspace: &str) -> String {
    format!(
        "Host {alias}\n  ProxyCommand coder ssh --stdio {workspace}\n  User coder\n",
        alias = workspace_alias(workspace),
    )
}

/// Write the per-workspace fragment under `.tickets/operator/ssh/` and return
/// its path. Idempotent — keyed by workspace name.
pub(crate) fn write_ssh_fragment(config: &Config, workspace: &str) -> Result<PathBuf> {
    let ssh_dir = config.tickets_path().join("operator/ssh");
    std::fs::create_dir_all(&ssh_dir).context("Failed to create ssh fragment directory")?;
    let path = ssh_dir.join(format!("{workspace}.config"));
    std::fs::write(&path, ssh_fragment(workspace)).context("Failed to write ssh fragment")?;
    Ok(path)
}

/// A workspace as reported by `coder list --output json` (fields we consume).
#[derive(Debug, serde::Deserialize)]
pub(crate) struct WorkspaceInfo {
    pub name: String,
    pub template_name: String,
}

/// What provisioning must do for a workspace, decided from `coder list`
/// output. Pure — directly unit-testable.
#[derive(Debug, PartialEq)]
pub(crate) enum WorkspaceAction {
    /// Exists on our template: `coder start` (no-op if running)
    Start,
    /// Absent: `coder create --template <t> -y`
    Create,
    /// Exists on a DIFFERENT template: refuse — guards against colliding
    /// with a human's workspace of the same name.
    Refuse { existing_template: String },
}

pub(crate) fn decide_workspace_action(
    existing: Option<&WorkspaceInfo>,
    template: &str,
) -> WorkspaceAction {
    match existing {
        None => WorkspaceAction::Create,
        Some(ws) if ws.template_name == template => WorkspaceAction::Start,
        Some(ws) => WorkspaceAction::Refuse {
            existing_template: ws.template_name.clone(),
        },
    }
}

/// Git checkout script run on the workspace over ssh: reuse a matching
/// checkout (fetch + ticket branch), otherwise clone then branch. Branch
/// naming stays in Rust (the caller passes the `git.branch_format`-derived
/// name) — never duplicated into a Coder template.
pub(crate) fn checkout_script(workdir: &str, remote_url: &str, branch: &str) -> String {
    let dir = shell_escape(workdir);
    let url = shell_escape(remote_url);
    let br = shell_escape(branch);
    format!(
        "set -e\nif [ -d {dir}/.git ] && [ \"$(git -C {dir} remote get-url origin)\" = {url} ]; then\n  git -C {dir} fetch origin\nelse\n  rm -rf {dir}\n  git clone {url} {dir}\nfi\ngit -C {dir} checkout -B {br}\n"
    )
}

/// Run a `coder` CLI invocation with the session injected under the CLI's
/// standard env names. Errors surface Coder's stderr verbatim — quota and
/// permission failures are the control plane's message, not ours to
/// reinterpret.
fn run_coder(session: &CoderSession, args: &[&str]) -> Result<String> {
    let output = Command::new("coder")
        .args(args)
        .env("CODER_URL", &session.url)
        .env("CODER_SESSION_TOKEN", &session.token)
        .output()
        .context("Failed to run the `coder` CLI")?;
    if !output.status.success() {
        anyhow::bail!(
            "`coder {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Look up a workspace by exact name via `coder list --output json`.
fn find_workspace(session: &CoderSession, name: &str) -> Result<Option<WorkspaceInfo>> {
    let out = run_coder(
        session,
        &[
            "list",
            "--search",
            &format!("name:{name}"),
            "--output",
            "json",
        ],
    )?;
    let all: Vec<WorkspaceInfo> = serde_json::from_str(out.trim()).unwrap_or_default();
    Ok(all.into_iter().find(|w| w.name == name))
}

/// Provision the workspace for a ticket and return the `RemoteHost` the
/// shared remote launch tail consumes. Blocking — workspace creation is
/// bounded by `create_timeout_secs`.
pub(crate) fn provision_workspace(
    config: &Config,
    coder: &CoderConfig,
    project: &str,
    ticket_id: &str,
    remote_url: Option<&str>,
    branch: Option<&str>,
) -> Result<RemoteHost> {
    // Fail fast before any lifecycle action: credentials, then CLI presence.
    let session = resolve_session(coder)?;
    if !cli_available() {
        anyhow::bail!(
            "Coder target requires the `coder` CLI on PATH; install it from your deployment"
        );
    }

    let workspace = workspace_name(&coder.name_prefix, project, ticket_id);
    match decide_workspace_action(
        find_workspace(&session, &workspace)?.as_ref(),
        &coder.template,
    ) {
        WorkspaceAction::Refuse { existing_template } => anyhow::bail!(
            "Workspace '{workspace}' exists on template '{existing_template}', not \
             '{}'; refusing to reuse a workspace Operator did not create",
            coder.template
        ),
        WorkspaceAction::Start => {
            run_coder(&session, &["start", &workspace, "--no-wait"]).map(|_| ())?;
        }
        WorkspaceAction::Create => {
            let mut args: Vec<String> = vec![
                "create".to_string(),
                workspace.clone(),
                "--template".to_string(),
                coder.template.clone(),
                "-y".to_string(),
            ];
            let mut params: Vec<_> = coder.parameters.iter().collect();
            params.sort();
            for (k, v) in params {
                args.push("--parameter".to_string());
                args.push(format!("{k}={v}"));
            }
            let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            run_coder(&session, &arg_refs).map(|_| ())?;
        }
    }

    let fragment = write_ssh_fragment(config, &workspace)?;
    let alias = workspace_alias(&workspace);
    let workdir = coder
        .workdir
        .clone()
        .unwrap_or_else(|| format!("/home/coder/{project}"));

    wait_for_ssh(&fragment, &alias, coder.create_timeout_secs)?;

    // Ensure the checkout before the agent lands in the workdir.
    if let (Some(url), Some(branch)) = (remote_url, branch) {
        let script = checkout_script(&workdir, url, branch);
        run_ssh(&fragment, &alias, &script)
            .with_context(|| format!("Failed to prepare checkout on workspace '{workspace}'"))?;
    }

    Ok(RemoteHost {
        name: workspace.clone(),
        ssh_alias: alias,
        workdir,
        display_name: Some(format!("coder/{workspace}")),
        ssh_config_path: Some(fragment.to_string_lossy().to_string()),
    })
}

/// Stop the workspace (never delete — reclamation is the Coder admin's
/// autostop/autodelete policy). Best-effort by design.
pub fn stop_workspace(coder: &CoderConfig, workspace: &str) -> Result<()> {
    let session = resolve_session(coder)?;
    run_coder(&session, &["stop", workspace, "--yes"]).map(|_| ())
}

fn cli_available() -> bool {
    Command::new("which")
        .arg("coder")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Poll ssh connectivity through the provisioned alias until the workspace
/// agent answers, bounded by `timeout_secs`.
fn wait_for_ssh(fragment: &std::path::Path, alias: &str, timeout_secs: u64) -> Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let ok = Command::new("ssh")
            .args(["-F".as_ref(), fragment.as_os_str()])
            .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=10"])
            .arg(alias)
            .arg("true")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            anyhow::bail!(
                "Workspace agent did not become reachable over ssh within {timeout_secs}s \
                 (alias '{alias}')"
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn run_ssh(fragment: &std::path::Path, alias: &str, script: &str) -> Result<()> {
    let status = Command::new("ssh")
        .args(["-F".as_ref(), fragment.as_os_str()])
        .args(["-o", "BatchMode=yes"])
        .arg(alias)
        .arg(script)
        .status()
        .context("Failed to run ssh against the workspace")?;
    if !status.success() {
        anyhow::bail!("ssh command on workspace '{alias}' exited with {status}");
    }
    Ok(())
}

/// Best-effort `stop_on_complete` for a finished agent: parses the persisted
/// launch mode, finds the coder target by the persisted target name, and
/// stops the workspace recorded as the agent's remote host.
pub fn stop_on_complete_for_agent(config: &Config, agent: &crate::state::AgentState) {
    use crate::agents::{parse_launch_mode, LaunchModeKind};
    let is_coder = agent
        .launch_mode
        .as_deref()
        .map(parse_launch_mode)
        .is_some_and(|m| m.kind == LaunchModeKind::Coder);
    if !is_coder {
        return;
    }
    let Some(workspace) = agent.remote_host.clone() else {
        return;
    };
    let coder = agent.target_name.as_deref().and_then(|name| {
        config
            .targets
            .iter()
            .find(|t| t.name == name)
            .and_then(|t| {
                if let crate::config::TargetKind::Coder(c) = &t.kind {
                    Some(c.clone())
                } else {
                    None
                }
            })
    });
    let Some(coder) = coder else {
        tracing::warn!(
            agent = %agent.id,
            "Cannot stop coder workspace: target no longer configured"
        );
        return;
    };
    if !coder.stop_on_complete {
        return;
    }
    match stop_workspace(&coder, &workspace) {
        Ok(()) => tracing::info!(workspace = %workspace, "Stopped coder workspace on completion"),
        Err(e) => tracing::warn!(workspace = %workspace, error = %e, "Failed to stop workspace"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_name_deterministic_and_sanitized() {
        let a = workspace_name("op", "MyProj", "FEAT-42");
        let b = workspace_name("op", "MyProj", "FEAT-42");
        assert_eq!(a, b, "same ticket must always map to the same workspace");
        assert_eq!(a, "op-myproj-feat-42");
    }

    #[test]
    fn test_workspace_name_collapses_special_chars() {
        assert_eq!(workspace_name("op", "a_b.c", "X--1"), "op-a-b-c-x-1");
    }

    #[test]
    fn test_workspace_name_respects_32_char_cap() {
        let name = workspace_name("op", "a-very-long-project-name-here", "FEATURE-12345");
        assert!(
            name.len() <= 32,
            "coder caps names at 32 chars, got {} ({name})",
            name.len()
        );
        // Truncated names stay deterministic and keep a readable prefix.
        let again = workspace_name("op", "a-very-long-project-name-here", "FEATURE-12345");
        assert_eq!(name, again);
        assert!(name.starts_with("op-a-very-long-project"));
        // Distinct long keys must not collide after truncation.
        let other = workspace_name("op", "a-very-long-project-name-here", "FEATURE-12346");
        assert_ne!(name, other);
    }

    #[test]
    fn test_ssh_fragment_shape() {
        let frag = ssh_fragment("op-proj-feat-1");
        assert!(frag.contains("Host op-coder-op-proj-feat-1"));
        assert!(frag.contains("ProxyCommand coder ssh --stdio op-proj-feat-1"));
        assert!(frag.contains("User coder"));
    }

    #[test]
    fn test_decide_workspace_action_template_mismatch_refuses() {
        let existing = WorkspaceInfo {
            name: "ws".to_string(),
            template_name: "someone-elses".to_string(),
        };
        assert_eq!(
            decide_workspace_action(Some(&existing), "operator-agent"),
            WorkspaceAction::Refuse {
                existing_template: "someone-elses".to_string()
            }
        );
    }

    #[test]
    fn test_decide_workspace_action_matching_starts_absent_creates() {
        let existing = WorkspaceInfo {
            name: "ws".to_string(),
            template_name: "operator-agent".to_string(),
        };
        assert_eq!(
            decide_workspace_action(Some(&existing), "operator-agent"),
            WorkspaceAction::Start
        );
        assert_eq!(
            decide_workspace_action(None, "operator-agent"),
            WorkspaceAction::Create
        );
    }

    #[test]
    fn test_resolve_session_missing_env_names_variable() {
        let coder = CoderConfig {
            template: "t".to_string(),
            url_env: "OPERATOR_TEST_CODER_URL_UNSET".to_string(),
            token_env: "OPERATOR_TEST_CODER_TOKEN_UNSET".to_string(),
            name_prefix: "op".to_string(),
            workdir: None,
            stop_on_complete: true,
            create_timeout_secs: 300,
            callback_url: None,
            parameters: std::collections::HashMap::new(),
        };
        let err = resolve_session(&coder).unwrap_err().to_string();
        assert!(
            err.contains("OPERATOR_TEST_CODER_URL_UNSET"),
            "failure must name the missing variable: {err}"
        );
    }

    #[test]
    fn test_checkout_script_reuses_matching_clone_and_branches_in_rust() {
        let script = checkout_script("/home/coder/proj", "git@github.com:u/r.git", "feat/x-42");
        assert!(script.contains("git clone 'git@github.com:u/r.git'"));
        assert!(script.contains("fetch origin"));
        assert!(
            script.contains("checkout -B 'feat/x-42'"),
            "branch name comes from Rust, never a template: {script}"
        );
        assert!(script.starts_with("set -e\n"));
    }

    #[test]
    fn test_write_ssh_fragment_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let config = Config {
            paths: crate::config::PathsConfig {
                tickets: temp.path().to_string_lossy().to_string(),
                projects: temp.path().to_string_lossy().to_string(),
                state: temp.path().join("s").to_string_lossy().to_string(),
                worktrees: temp.path().join("w").to_string_lossy().to_string(),
            },
            ..Default::default()
        };
        let p1 = write_ssh_fragment(&config, "ws-1").unwrap();
        let p2 = write_ssh_fragment(&config, "ws-1").unwrap();
        assert_eq!(p1, p2);
        assert!(std::fs::read_to_string(&p1)
            .unwrap()
            .contains("ProxyCommand coder ssh --stdio ws-1"));
    }
}
