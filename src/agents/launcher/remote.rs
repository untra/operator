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
use crate::llm::tool_config::load_all_tool_configs;

use super::prompt::shell_escape;

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
/// Uses the *embedded* tool config template (bare tool name, resolved via the
/// remote PATH) rather than the locally detected binary path, which would be
/// wrong on the remote machine. `{{config_flags}}` is dropped: permission
/// translation, MCP config, and statusline all write local files with local
/// paths — an accepted v1 degradation.
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
            anyhow::anyhow!(
                "LLM tool '{tool_name}' has no embedded tool config; remote launch supports claude/codex/gemini"
            )
        })?;

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

    Ok(cmd)
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
        "#!/bin/bash\nset -e\nssh {alias} {mkdir}\nssh {alias} {cat_prompt} < {local_prompt}\nssh {alias} {cat_payload} < {local_payload}\nexec ssh -t -R {port}:localhost:{port} -o ExitOnForwardFailure=yes {alias} {tmux}\n",
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
    provider_env.insert(
        "OPERATOR_API_URL".to_string(),
        format!("http://localhost:{}", operator_env.ui_port),
    );

    let payload_file = super::prompt::write_command_file(
        config,
        session_uuid,
        &host.workdir,
        &llm_cmd,
        Some(operator_env),
        Some(&provider_env),
    )?;

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

/// The check script run on the remote host by [`run_preflight`].
fn preflight_script(host: &RemoteHost, tool_name: &str) -> String {
    format!(
        "command -v tmux >/dev/null || exit {PREFLIGHT_NO_TMUX}; command -v {tool} >/dev/null || exit {PREFLIGHT_NO_TOOL}; test -d {workdir} || exit {PREFLIGHT_NO_WORKDIR}",
        tool = shell_escape(tool_name),
        workdir = shell_escape(&host.workdir),
    )
}

/// Check the remote host can run the agent before any session is created:
/// reachable over SSH (`BatchMode` so a password prompt can't wedge the TUI),
/// tmux and the tool on the remote PATH, and the workdir present.
pub(crate) fn run_preflight(host: &RemoteHost, tool_name: &str) -> Result<()> {
    let status = std::process::Command::new("ssh")
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"])
        .arg(&host.ssh_alias)
        .arg(preflight_script(host, tool_name))
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
        let err = build_remote_llm_command("agy", "m", "s", "/p", false, &[]).unwrap_err();
        assert!(err.to_string().contains("no embedded tool config"));
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
    fn test_preflight_script_distinct_exit_codes() {
        let s = preflight_script(&host(), "claude");
        assert!(s.contains("command -v tmux >/dev/null || exit 40"));
        assert!(s.contains("command -v 'claude' >/dev/null || exit 41"));
        assert!(s.contains("test -d '/srv/agents/proj' || exit 42"));
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
