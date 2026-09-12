//! Remote (SSH) launch support.
//!
//! v1 execution shape: the *local* multiplexer pane runs a generated wrapper
//! script that ships the prompt and payload to the remote host, then execs
//! `ssh -t` into a *remote* tmux session running the agent. The pane Operator
//! tracks stays local (scraping/attach/send-keys unchanged); the remote tmux
//! session survives disconnects and `tmux new-session -A` reattaches on
//! relaunch. Completion callbacks flow through an SSH reverse tunnel via
//! `OPERATOR_API_URL`, keeping the REST API loopback-only on both ends.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{Config, RemoteHost};
use crate::llm::tool_config::{load_all_tool_configs, user_tools_dir};

use super::prompt::shell_escape;

/// `-F <config> ` fragment for hosts carrying a provisioned ssh config
/// (coder aliases); empty for plain `~/.ssh/config` hosts.
fn ssh_config_flag(host: &RemoteHost) -> String {
    host.ssh_config_path
        .as_ref()
        .map(|p| format!("-F {} ", shell_escape(p)))
        .unwrap_or_default()
}

pub(crate) fn reconcile_git_runtime(config: &Config, host: &RemoteHost) {
    for (pointer, path) in crate::git::runtime::abandoned_runtime_pointers(config) {
        let marker = pointer.with_extension("git-remote");
        if std::fs::read_to_string(&marker).ok().as_deref() != Some(&host.ssh_alias) {
            continue;
        }
        let target = shell_escape(&path.to_string_lossy());
        let script = format!("if [ -L {target} ]; then exit 1; fi; if [ -f {target}/pid ]; then read -r pid < {target}/pid; case \"$pid\" in ''|*[!0-9]*|0) exit 1;; esac; if kill -0 \"$pid\" 2>/dev/null; then exit 1; fi; fi; rm -rf -- {target}");
        let mut command = std::process::Command::new("ssh");
        if let Some(config) = &host.ssh_config_path {
            command.args(["-F", config]);
        }
        if command
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                &host.ssh_alias,
                &script,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
        {
            let _ = std::fs::remove_file(pointer);
            let _ = std::fs::remove_file(marker);
        }
    }
}

pub(crate) fn transfer_git_runtime(host: &RemoteHost, path: &Path) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    fn send(host: &RemoteHost, script: &str, data: &[u8]) -> Result<()> {
        let mut command = Command::new("ssh");
        if let Some(config) = &host.ssh_config_path {
            command.args(["-F", config]);
        }
        let mut child = command
            .args(["-o", "BatchMode=yes", &host.ssh_alias, script])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        child
            .stdin
            .take()
            .context("Opening SSH credential transport")?
            .write_all(data)?;
        anyhow::ensure!(child.wait()?.success(), "Git credential transport failed");
        Ok(())
    }
    fn copy(host: &RemoteHost, path: &Path) -> Result<()> {
        send(
            host,
            &format!(
                "umask 077; mkdir -- {}",
                shell_escape(&path.to_string_lossy())
            ),
            &[],
        )?;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                copy(host, &entry.path())?;
            } else {
                anyhow::ensure!(kind.is_file(), "Unexpected Git runtime file type");
                let mode = if entry.file_name() == "helper"
                    || entry.path().parent().is_some_and(|p| p.ends_with("bin"))
                {
                    "700"
                } else {
                    "600"
                };
                let target = shell_escape(&entry.path().to_string_lossy());
                send(
                    host,
                    &format!("umask 077; set -C; cat > {target} && chmod {mode} {target}"),
                    &std::fs::read(entry.path())?,
                )?;
            }
        }
        Ok(())
    }
    let result = copy(host, path);
    if result.is_err() {
        let _ = send(
            host,
            &format!("rm -rf -- {}", shell_escape(&path.to_string_lossy())),
            &[],
        );
    }
    result
}

/// Remote path the prompt file is shipped to.
pub(crate) fn remote_prompt_path(host: &RemoteHost, session_uuid: &str) -> String {
    format!(
        "{}/.tickets/operator/prompts/{session_uuid}.txt",
        host.workdir
    )
}

/// Remote path the payload (run) script is shipped to.
pub(crate) fn remote_payload_path(host: &RemoteHost, session_uuid: &str) -> String {
    format!(
        "{}/.tickets/operator/commands/{session_uuid}.sh",
        host.workdir
    )
}

/// Build the agent CLI command executed on the remote host.
///
/// Uses the *loaded* tool config template - builtin or user-provided (see
/// `crate::llm::tool_config`) - with the bare tool name, resolved via the
/// remote PATH, rather than the locally detected binary path, which would be
/// wrong on the remote machine. `{{config_flags}}` is dropped: permission
/// translation, MCP config, and statusline all write local files
pub(crate) fn build_remote_llm_command(
    tool_name: &str,
    model: &str,
    session_id: &str,
    remote_prompt: &str,
    yolo: bool,
    extra_flags: &[String],
) -> Result<String> {
    let tool = load_all_tool_configs()
        .into_iter()
        .find(|t| t.tool_name == tool_name)
        .ok_or_else(|| {
            let tools_dir = user_tools_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "~/.config/operator/tools".to_string());
            anyhow::anyhow!(
                "LLM tool '{tool_name}' has no tool config; builtins are claude/codex/gemini - \
                 add a JSON under {tools_dir} to support others"
            )
        })?;

    Ok(build_remote_llm_command_from(
        &tool,
        model,
        session_id,
        remote_prompt,
        yolo,
        extra_flags,
    ))
}

fn build_remote_llm_command_from(
    tool: &crate::llm::tool_config::ToolConfig,
    model: &str,
    session_id: &str,
    remote_prompt: &str,
    yolo: bool,
    extra_flags: &[String],
) -> String {
    let tool_name = tool.tool_name.as_str();
    let model_flag = if tool.arg_mapping.model.is_empty() {
        String::new()
    } else {
        format!("{} {model} ", tool.arg_mapping.model)
    };

    let mut cmd = tool
        .command_template
        .replace("{{config_flags}}", "")
        .replace("{{model_flag}}", &model_flag)
        .replace("{{model}}", model)
        .replace("{{session_id}}", session_id)
        .replace("{{prompt_file}}", remote_prompt);

    if yolo && !tool.yolo_flags.is_empty() {
        if let Some(pos) = cmd.find(tool_name) {
            let insert_pos = pos + tool_name.len();
            cmd.insert_str(insert_pos, &format!(" {}", tool.yolo_flags.join(" ")));
        }
    }

    if !extra_flags.is_empty() {
        cmd = format!("{cmd} {}", extra_flags.join(" "));
    }

    cmd
}

/// Build the local wrapper script content: ship prompt + payload over SSH,
/// then exec into the remote tmux session through a reverse tunnel.
///
/// Payloads travel as files via `ssh 'cat > …'` (portable on SFTP-only hosts,
/// and no payload quoting at all); only the short tmux line is quoted, through
/// exactly one escaping layer per shell that parses it. `exec` makes the pane
/// process *be* ssh, so pane-death ⇔ ssh-death and monitor semantics are
/// unchanged. `-A` makes relaunch idempotent (reattaches a surviving session).
/// `ExitOnForwardFailure` turns tunnel-port collisions into loud launch
/// failures instead of silently broken callbacks. The remote status bar is
/// switched off so its clock doesn't defeat content-hash idle detection.
pub(crate) fn build_remote_wrapper_script(
    host: &RemoteHost,
    session_name: &str,
    session_uuid: &str,
    local_prompt: &Path,
    local_payload: &Path,
    api_port: u16,
) -> String {
    let alias = shell_escape(&host.ssh_alias);
    let f_flag = ssh_config_flag(host);
    let r_prompt = remote_prompt_path(host, session_uuid);
    let r_payload = remote_payload_path(host, session_uuid);

    let mkdir_cmd = format!(
        "mkdir -p {} {}",
        shell_escape(&format!("{}/.tickets/operator/prompts", host.workdir)),
        shell_escape(&format!("{}/.tickets/operator/commands", host.workdir)),
    );
    let tmux_cmd = format!(
        "tmux new-session -A -s {} {} \\; set-option status off",
        shell_escape(session_name),
        shell_escape(&format!("bash {}", shell_escape(&r_payload))),
    );

    format!(
        "#!/bin/bash\nset -e\nssh {f}{alias} {mkdir}\nssh {f}{alias} {cat_prompt} < {local_prompt}\nssh {f}{alias} {cat_payload} < {local_payload}\nexec ssh -t {f}-R {port}:localhost:{port} -o ExitOnForwardFailure=yes {alias} {tmux}\n",
        f = f_flag,
        alias = alias,
        mkdir = shell_escape(&mkdir_cmd),
        cat_prompt = shell_escape(&format!("cat > {}", shell_escape(&r_prompt))),
        cat_payload = shell_escape(&format!("cat > {}", shell_escape(&r_payload))),
        local_prompt = shell_escape(&local_prompt.display().to_string()),
        local_payload = shell_escape(&local_payload.display().to_string()),
        port = api_port,
        tmux = shell_escape(&tmux_cmd),
    )
}

/// Write the wrapper script to `.tickets/operator/commands/{uuid}-remote.sh`.
///
/// Regeneration is idempotent (keyed by session uuid) so relaunch can rebuild
/// it and `tmux new-session -A` reattaches the surviving remote session.
pub(crate) fn write_remote_wrapper_file(
    config: &Config,
    session_uuid: &str,
    content: &str,
) -> Result<PathBuf> {
    let commands_dir = config.tickets_path().join("operator/commands");
    std::fs::create_dir_all(&commands_dir).context("Failed to create commands directory")?;
    let wrapper_file = commands_dir.join(format!("{session_uuid}-remote.sh"));
    std::fs::write(&wrapper_file, content).context("Failed to write remote wrapper script")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper_file, std::fs::Permissions::from_mode(0o755))
            .context("Failed to set remote wrapper permissions")?;
    }
    Ok(wrapper_file)
}

/// Shared remote-launch tail for session backends (tmux, cmux).
///
/// Builds the remote agent command, writes the payload script (cd'ing to the
/// *remote* workdir, exporting `OPERATOR_API_URL` for the reverse tunnel) and
/// the local wrapper, then types `bash {wrapper}` into the already-created
/// local session via `send`. `cleanup` tears the session down if that fails.
#[allow(clippy::too_many_arguments)] // Cohesive launch context; a struct would just relabel it.
pub(crate) fn launch_remote_in_session(
    config: &Config,
    ticket: &crate::queue::Ticket,
    session_name: &str,
    session_uuid: &str,
    step_name: &str,
    host: &RemoteHost,
    tool_name: &str,
    model: &str,
    prompt_file: &Path,
    options: &super::options::LaunchOptions,
    operator_env: &super::prompt::OperatorEnvVars,
    is_resume: bool,
    send: impl FnOnce(&str) -> Result<()>,
    cleanup: impl FnOnce(),
) -> Result<String> {
    let r_prompt = remote_prompt_path(host, session_uuid);
    let mut llm_cmd = build_remote_llm_command(
        tool_name,
        model,
        session_uuid,
        &r_prompt,
        options.yolo_mode,
        &options.extra_flags,
    )?;

    // Resume the existing agent session if the remote tmux session died and
    // the payload actually runs fresh (a surviving session ignores it via -A).
    if is_resume {
        if let Some(pos) = llm_cmd.find(tool_name) {
            let insert_pos = pos + tool_name.len();
            llm_cmd.insert_str(insert_pos, &format!(" --resume {session_uuid}"));
        }
    }

    let mut provider_env = options
        .provider
        .as_ref()
        .map(|p| p.env.clone())
        .unwrap_or_default();
    // Tunnel default; a coder target's control-plane-reachable callback_url
    // overrides it so detached multi-step survives tunnel loss.
    let api_url = options
        .api_url_override
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{}", operator_env.ui_port));
    provider_env.insert("OPERATOR_API_URL".to_string(), api_url);

    let payload_file = super::prompt::write_command_file(
        config,
        session_uuid,
        &host.workdir,
        &llm_cmd,
        Some(operator_env),
        Some(&provider_env),
    )?;

    reconcile_git_runtime(config, host);
    let runtime_pointer = payload_file.with_extension("git-runtime");
    if runtime_pointer.exists() {
        let path = PathBuf::from(std::fs::read_to_string(&runtime_pointer)?);
        std::fs::write(
            runtime_pointer.with_extension("git-remote"),
            &host.ssh_alias,
        )?;
        let transferred = transfer_git_runtime(host, &path);
        let _ = std::fs::remove_dir_all(&path);
        transferred?;
    }

    let wrapper_content = build_remote_wrapper_script(
        host,
        session_name,
        session_uuid,
        prompt_file,
        &payload_file,
        operator_env.ui_port,
    );
    let wrapper_file = write_remote_wrapper_file(config, session_uuid, &wrapper_content)?;

    let bash_cmd = format!("bash {}", wrapper_file.display());
    if let Err(e) = send(&bash_cmd) {
        cleanup();
        anyhow::bail!("Failed to start remote agent in session: {e}");
    }

    tracing::info!(
        session = %session_name,
        session_uuid = %session_uuid,
        project = %ticket.project,
        ticket = %ticket.id,
        step = %step_name,
        tool = %tool_name,
        host = %host.name,
        remote_workdir = %host.workdir,
        wrapper_file = %wrapper_file.display(),
        "Launched agent on remote host"
    );

    Ok(session_name.to_string())
}

/// Distinct preflight failure exit codes used by [`preflight_script`].
const PREFLIGHT_NO_TMUX: i32 = 40;
const PREFLIGHT_NO_TOOL: i32 = 41;
const PREFLIGHT_NO_WORKDIR: i32 = 42;
const PREFLIGHT_NO_PROVIDER_CLI: i32 = 43;
const PREFLIGHT_NO_GIT: i32 = 44;

/// The check script run on the remote host by [`run_preflight`].
///
/// `provider` is the git provider the project resolves to, when it resolves to
/// one. Operator ships no client binaries, so the target supplies `gh`/`glab`/
/// `tea` itself and this is where a missing one is caught -- before a session
/// exists, rather than halfway through a ticket.
fn preflight_script(
    host: &RemoteHost,
    tool_name: &str,
    provider: Option<crate::types::pr::GitProvider>,
) -> String {
    let mut checks = format!(
        "command -v tmux >/dev/null || exit {PREFLIGHT_NO_TMUX}; command -v {tool} >/dev/null || exit {PREFLIGHT_NO_TOOL}; test -d {workdir} || exit {PREFLIGHT_NO_WORKDIR}; command -v git >/dev/null || exit {PREFLIGHT_NO_GIT}",
        tool = shell_escape(tool_name),
        workdir = shell_escape(&host.workdir),
    );
    if let Some(provider) = provider {
        checks.push_str(&format!(
            "; command -v {cli} >/dev/null || exit {PREFLIGHT_NO_PROVIDER_CLI}",
            cli = shell_escape(crate::api::cli_detection::binary_for(provider)),
        ));
    }
    checks
}

/// Check the remote host can run the agent before any session is created:
/// reachable over SSH (`BatchMode` so a password prompt can't wedge the TUI),
/// tmux and the tool on the remote PATH, and the workdir present.
pub(crate) fn run_preflight(
    host: &RemoteHost,
    tool_name: &str,
    provider: Option<crate::types::pr::GitProvider>,
) -> Result<()> {
    let mut cmd = std::process::Command::new("ssh");
    if let Some(ref frag) = host.ssh_config_path {
        cmd.args(["-F", frag]);
    }
    let status = cmd
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"])
        .arg(&host.ssh_alias)
        .arg(preflight_script(host, tool_name, provider))
        .status()
        .context("Failed to run ssh for remote preflight")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(c) if c == PREFLIGHT_NO_TMUX => anyhow::bail!(
            "Remote host '{}' has no tmux on PATH; install tmux there first",
            host.name
        ),
        Some(c) if c == PREFLIGHT_NO_TOOL => anyhow::bail!(
            "Remote host '{}' has no '{tool_name}' on PATH; install the agent CLI there first",
            host.name
        ),
        Some(c) if c == PREFLIGHT_NO_WORKDIR => anyhow::bail!(
            "Remote host '{}' is missing workdir '{}'; check out the project there first",
            host.name,
            host.workdir
        ),
        Some(c) if c == PREFLIGHT_NO_GIT => anyhow::bail!(
            "Remote host '{}' has no git on PATH; install git there first",
            host.name
        ),
        Some(c) if c == PREFLIGHT_NO_PROVIDER_CLI => {
            let spec = crate::api::cli_detection::spec_for(
                provider.expect("exit 43 is only emitted when a provider was checked"),
            );
            anyhow::bail!(
                "Remote host '{}' has no '{}' on PATH, needed to open {} pull requests. \
                 Operator does not install client binaries -- install it on the target \
                 (or bake it into the workspace image): {}",
                host.name,
                spec.command,
                spec.display_name,
                spec.install_url
            )
        }
        _ => anyhow::bail!(
            "Cannot reach remote host '{}' via `ssh {}` (BatchMode). Verify the alias in ~/.ssh/config and connect once manually to accept host keys",
            host.name,
            host.ssh_alias
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host() -> RemoteHost {
        RemoteHost {
            name: "gpu-vm".to_string(),
            ssh_alias: "gpu-alias".to_string(),
            workdir: "/srv/agents/proj".to_string(),
            display_name: None,
            ssh_config_path: None,
        }
    }

    #[test]
    fn test_remote_paths_are_under_workdir() {
        let h = host();
        assert_eq!(
            remote_prompt_path(&h, "uuid-1"),
            "/srv/agents/proj/.tickets/operator/prompts/uuid-1.txt"
        );
        assert_eq!(
            remote_payload_path(&h, "uuid-1"),
            "/srv/agents/proj/.tickets/operator/commands/uuid-1.sh"
        );
    }

    #[test]
    fn test_build_remote_llm_command_uses_bare_tool_and_remote_prompt() {
        let cmd = build_remote_llm_command(
            "claude",
            "opus",
            "uuid-1",
            "/srv/agents/proj/.tickets/operator/prompts/uuid-1.txt",
            false,
            &[],
        )
        .unwrap();
        assert!(cmd.starts_with("claude "), "bare tool name, got: {cmd}");
        assert!(cmd.contains("--session-id uuid-1"));
        assert!(cmd.contains("/srv/agents/proj/.tickets/operator/prompts/uuid-1.txt"));
        assert!(
            !cmd.contains("{{"),
            "all template variables substituted, got: {cmd}"
        );
    }

    #[test]
    fn test_build_remote_llm_command_applies_yolo_and_extra_flags() {
        let cmd = build_remote_llm_command(
            "claude",
            "opus",
            "uuid-1",
            "/tmp/p.txt",
            true,
            &["--verbose".to_string()],
        )
        .unwrap();
        assert!(cmd.contains("--dangerously-skip-permissions"));
        assert!(cmd.ends_with("--verbose"));
    }

    #[test]
    fn test_build_remote_llm_command_unknown_tool_errors() {
        let err =
            build_remote_llm_command("op-test-tool-does-not-exist", "m", "s", "/p", false, &[])
                .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("no tool config"), "got: {msg}");
        assert!(msg.contains("tools"), "points at the user tools dir: {msg}");
    }

    #[test]
    fn test_build_remote_llm_command_runtime_tool() {
        // A non-builtin tool config (as loaded from ~/.config/operator/tools)
        let tool: crate::llm::tool_config::ToolConfig = serde_json::from_str(
            r#"{
                "tool_name": "mux",
                "version_command": "mux --version",
                "capabilities": { "supports_sessions": true, "supports_headless": true },
                "model_aliases": ["default"],
                "arg_mapping": { "prompt": "", "model": "--model" },
                "command_template": "mux {{model_flag}}--resume {{session_id}} \"$(cat {{prompt_file}})\"",
                "yolo_flags": ["--yes"]
            }"#,
        )
        .unwrap();

        let cmd = build_remote_llm_command_from(&tool, "m1", "uuid-1", "/remote/p.txt", true, &[]);
        assert!(cmd.starts_with("mux "), "bare tool name, got: {cmd}");
        assert!(cmd.contains("--model m1"));
        assert!(cmd.contains("--resume uuid-1"));
        assert!(cmd.contains("/remote/p.txt"));
        assert!(cmd.contains("--yes"));
        assert!(
            !cmd.contains("{{"),
            "all template variables substituted: {cmd}"
        );
    }

    #[test]
    fn test_wrapper_ships_files_then_execs_tunneled_tmux() {
        let h = host();
        let script = build_remote_wrapper_script(
            &h,
            "op-FEAT-42",
            "uuid-1",
            Path::new("/local/.tickets/operator/prompts/uuid-1.txt"),
            Path::new("/local/.tickets/operator/commands/uuid-1.sh"),
            7008,
        );
        assert!(script.starts_with("#!/bin/bash\nset -e\n"));
        // Ships both files via `cat >` before the exec line.
        assert!(script.contains("cat > "));
        assert!(script.contains("uuid-1.txt"));
        assert!(script.contains("uuid-1.sh"));
        // Reverse tunnel with loud forward failure.
        assert!(script.contains("-R 7008:localhost:7008"));
        assert!(script.contains("-o ExitOnForwardFailure=yes"));
        // Exec so the pane process is ssh; -A so relaunch reattaches.
        assert!(script.contains("exec ssh -t"));
        assert!(script.contains("tmux new-session -A -s "));
        assert!(script.contains("op-FEAT-42"));
        // Remote status bar off so its clock can't defeat idle detection.
        assert!(script.contains("set-option status off"));
        let exec_pos = script.find("exec ssh").unwrap();
        let last_cat = script.rfind("cat > ").unwrap();
        assert!(last_cat < exec_pos, "files ship before exec");
    }

    #[test]
    fn test_wrapper_escapes_workdir_with_spaces() {
        let mut h = host();
        h.workdir = "/srv/agent workdir/proj".to_string();
        let script = build_remote_wrapper_script(
            &h,
            "op-X",
            "u1",
            Path::new("/l/p.txt"),
            Path::new("/l/c.sh"),
            7008,
        );
        // The workdir must never appear unquoted (space-split) in any remote command.
        assert!(!script.contains(" /srv/agent workdir/proj/"));
        assert!(script.contains("agent workdir"));
    }

    #[test]
    fn test_wrapper_passes_ssh_config_fragment_and_keeps_tunnel() {
        let mut h = host();
        h.ssh_config_path = Some("/local/.tickets/operator/ssh/ws.config".to_string());
        let script = build_remote_wrapper_script(
            &h,
            "op-FEAT-1",
            "u1",
            Path::new("/l/p.txt"),
            Path::new("/l/c.sh"),
            7008,
        );
        assert!(
            script.contains("-F '/local/.tickets/operator/ssh/ws.config'"),
            "provisioned fragment must be passed with -F: {script}"
        );
        assert!(
            script.contains("exec ssh -t -F '/local/.tickets/operator/ssh/ws.config' -R 7008:localhost:7008"),
            "-R reverse tunnel must be preserved alongside -F (ProxyCommand is transport only): {script}"
        );
    }

    #[test]
    fn test_preflight_script_distinct_exit_codes() {
        let s = preflight_script(&host(), "claude", None);
        assert!(s.contains("command -v tmux >/dev/null || exit 40"));
        assert!(s.contains("command -v 'claude' >/dev/null || exit 41"));
        assert!(s.contains("test -d '/srv/agents/proj' || exit 42"));
        assert!(s.contains("command -v git >/dev/null || exit 44"));
    }

    /// A missing provider CLI must be caught here, before a session exists.
    /// Operator installs no client binaries, so this check is the only thing
    /// standing between a BYO target and a PR that fails halfway through.
    #[test]
    fn preflight_requires_the_provider_cli_when_one_is_known() {
        use crate::types::pr::GitProvider;
        let s = preflight_script(&host(), "claude", Some(GitProvider::GitHub));
        assert!(s.contains("command -v 'gh' >/dev/null || exit 43"));

        let gitea = preflight_script(&host(), "claude", Some(GitProvider::Gitea));
        assert!(gitea.contains("command -v 'tea' >/dev/null || exit 43"));

        // Forgejo rides the Gitea-compatible tea CLI; never `fj`.
        let forgejo = preflight_script(&host(), "claude", Some(GitProvider::Forgejo));
        assert!(forgejo.contains("command -v 'tea' >/dev/null || exit 43"));
        assert!(!forgejo.contains("fj"));
    }

    /// No resolvable provider means no PR will be attempted, so requiring a
    /// provider CLI would block launches that never needed one.
    #[test]
    fn preflight_skips_the_provider_check_when_no_provider_is_resolved() {
        let s = preflight_script(&host(), "claude", None);
        assert!(!s.contains(&format!("exit {PREFLIGHT_NO_PROVIDER_CLI}")));
    }

    #[test]
    fn test_write_remote_wrapper_file_named_by_uuid() {
        use tempfile::tempdir;
        let temp = tempdir().unwrap();
        let config = Config {
            paths: crate::config::PathsConfig {
                tickets: temp.path().to_string_lossy().to_string(),
                projects: temp.path().to_string_lossy().to_string(),
                state: temp.path().join("operator").to_string_lossy().to_string(),
                worktrees: temp.path().join("wt").to_string_lossy().to_string(),
            },
            ..Default::default()
        };
        let path = write_remote_wrapper_file(&config, "uuid-9", "#!/bin/bash\n").unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "uuid-9-remote.sh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o755
            );
        }
    }
}
