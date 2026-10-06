//! Connect links and attention signals for launched agents.

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use url::Url;
use utoipa::ToSchema;

use crate::agents::delegator_resolution::resolve_named_target;
use crate::agents::launcher::coder::workspace_alias;
use crate::agents::launcher::prompt::shell_escape_if_needed as shell_escape;
use crate::agents::{parse_launch_mode, LaunchModeKind};
use crate::config::{Config, ConnectInterface, RemoteHost, TargetKind};
use crate::state::AgentState;

/// URI authority of the Operator VS Code/Cursor extension.
pub const OPERATOR_EXTENSION: &str = "untra.operator-terminals";

/// Minutes without pane output before a running agent asks for attention.
pub const IDLE_ATTENTION_MINUTES: i64 = 10;

/// How a client acts on a connect option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ConnectKind {
    /// Open `uri` in the browser host (IDE deep link)
    Ide,
    /// Call `POST /api/v1/agents/{id}/focus`
    FocusApi,
    /// Copy `command` to run in a terminal
    Command,
}

/// Whether the agent's session can be attached or only resumed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SessionPhase {
    /// The terminal session is alive; attach to it
    Live,
    /// The session is gone; resume the LLM conversation by id
    Resumable,
    /// Nothing to attach or resume
    Gone,
}

/// One way to reach an agent's session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ConnectOption {
    /// Interface this option opens
    pub interface: ConnectInterface,
    /// How the client acts on it
    pub kind: ConnectKind,
    /// Button label
    pub label: String,
    /// Deep link for `ide` options
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub uri: Option<String>,
    /// Shell command for `command` options
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub command: Option<String>,
    /// Whether the option can be used right now
    pub available: bool,
    /// Why the option is unavailable
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unavailable_reason: Option<String>,
}

/// Every connect option for an agent, with the preferred one called out
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ConnectPlan {
    /// Attach vs resume
    pub phase: SessionPhase,
    /// Primary option (configured preference, else the session wrapper's)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferred: Option<ConnectOption>,
    /// All applicable options, preferred included
    pub options: Vec<ConnectOption>,
}

/// Why an agent needs the operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AttentionReason {
    /// The agent is waiting for input
    AwaitingInput,
    /// A review gate is pending
    Review,
    /// The session disappeared
    Orphaned,
    /// No output for a while
    Idle,
}

/// Attention nudge for an agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct Attention {
    /// Why attention is needed
    pub reason: AttentionReason,
    /// Short context (review state or last message)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub detail: Option<String>,
    /// Since when (ISO 8601)
    pub since: String,
}

/// What an IDE extension runs after opening the agent's workspace
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ConnectTerminal {
    /// Attach vs resume
    pub phase: SessionPhase,
    /// Ticket ID, for naming the terminal
    pub ticket_id: String,
    /// Existing terminal or session name
    pub terminal_name: String,
    /// Directory to start in
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cwd: Option<String>,
    /// Command to run inside the agent's environment; absent means focus `terminal_name`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub command: Option<String>,
}

/// Where a remote agent's session runs
#[derive(Debug, Clone, PartialEq)]
pub enum RemoteSession {
    /// A `[[hosts]]` entry or ssh target
    Ssh(RemoteHost),
    /// A Coder workspace reached over its provisioned alias
    Coder {
        host: RemoteHost,
        workspace: String,
        url: Option<String>,
    },
}

impl RemoteSession {
    fn host(&self) -> &RemoteHost {
        match self {
            RemoteSession::Ssh(host) | RemoteSession::Coder { host, .. } => host,
        }
    }
}

/// Facts about installed and running interfaces, gathered once per request
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectProbes {
    pub vscode_extension: bool,
    pub code: bool,
    pub cursor: bool,
    pub zed: bool,
}

/// Inputs for building connect options
#[derive(Debug, Clone)]
pub struct ConnectContext<'a> {
    pub agent: &'a AgentState,
    pub remote: Option<&'a RemoteSession>,
    pub tmux_socket: Option<&'a str>,
    pub preferred: Option<ConnectInterface>,
    pub can_open_terminal: bool,
    pub probes: &'a ConnectProbes,
}

/// Interactive resume command for a tool's conversation.
pub fn resume_command(tool: &str, session_id: &str) -> Option<String> {
    let id = shell_escape(session_id);
    match tool {
        "claude" | "gemini" => Some(format!("{tool} --resume {id}")),
        "codex" => Some(format!("codex resume {id}")),
        _ => None,
    }
}

fn agent_resume_command(agent: &AgentState) -> Option<String> {
    let ctx = agent.step_launch_context.as_ref();
    let session_id = ctx.and_then(|c| c.session_id.as_deref())?;
    let tool = ctx.map(|c| c.tool.as_str()).or(agent.llm_tool.as_deref())?;
    resume_command(tool, session_id)
}

pub fn session_phase(agent: &AgentState) -> SessionPhase {
    if agent.session_name.is_some() && agent.status != "orphaned" {
        SessionPhase::Live
    } else if agent_resume_command(agent).is_some() {
        SessionPhase::Resumable
    } else {
        SessionPhase::Gone
    }
}

fn workdir(ctx: &ConnectContext) -> Option<String> {
    ctx.remote
        .map(|r| r.host().workdir.clone())
        .or_else(|| ctx.agent.worktree_path.clone())
}

fn wrapper(agent: &AgentState) -> &str {
    agent.session_wrapper.as_deref().unwrap_or("tmux")
}

fn wrapper_interface(agent: &AgentState) -> ConnectInterface {
    match wrapper(agent) {
        "vscode" => ConnectInterface::Vscode,
        "cmux" => ConnectInterface::Cmux,
        "zellij" => ConnectInterface::Zellij,
        _ => ConnectInterface::Tmux,
    }
}

fn tmux_attach(socket: Option<&str>, session: &str) -> String {
    let session = shell_escape(session);
    socket.map_or_else(
        || format!("tmux attach -t {session}"),
        |s| format!("tmux -L {} attach -t {session}", shell_escape(s)),
    )
}

fn in_dir(cwd: Option<&str>, command: &str) -> String {
    cwd.map_or_else(
        || command.to_string(),
        |dir| format!("cd {} && {command}", shell_escape(dir)),
    )
}

fn ssh_wrap(host: &RemoteHost, command: &str) -> String {
    let f_flag = host
        .ssh_config_path
        .as_deref()
        .map(|p| format!("-F {} ", shell_escape(p)))
        .unwrap_or_default();
    format!(
        "ssh -t {f_flag}{} {}",
        shell_escape(&host.ssh_alias),
        shell_escape(command)
    )
}

/// Command that enters the session from inside the agent's own environment.
pub fn session_command(ctx: &ConnectContext) -> Option<String> {
    let agent = ctx.agent;
    match session_phase(agent) {
        SessionPhase::Live => {
            let session = agent.session_name.as_deref()?;
            if ctx.remote.is_some() {
                return Some(tmux_attach(None, session));
            }
            match wrapper(agent) {
                "tmux" => Some(tmux_attach(ctx.tmux_socket, session)),
                "zellij" => Some(format!("zellij attach {}", shell_escape(session))),
                _ => None,
            }
        }
        SessionPhase::Resumable => {
            agent_resume_command(agent).map(|cmd| in_dir(workdir(ctx).as_deref(), &cmd))
        }
        SessionPhase::Gone => None,
    }
}

/// Command that enters the session from the operator's machine.
pub fn local_command(ctx: &ConnectContext) -> Option<String> {
    let inner = session_command(ctx)?;
    Some(
        ctx.remote
            .map_or(inner.clone(), |r| ssh_wrap(r.host(), &inner)),
    )
}

/// Expand `sessions.connect.terminal_command`, `{command}` becoming `sh -c '<command>'`.
pub fn terminal_invocation(template: &str, command: &str) -> String {
    template.replace(
        "{command}",
        &format!(
            "sh -c {}",
            crate::agents::launcher::prompt::shell_escape(command)
        ),
    )
}

/// What the IDE extension runs once the workspace is open.
pub fn connect_terminal(ctx: &ConnectContext) -> ConnectTerminal {
    let agent = ctx.agent;
    ConnectTerminal {
        phase: session_phase(agent),
        ticket_id: agent.ticket_id.clone(),
        terminal_name: agent
            .session_name
            .clone()
            .unwrap_or_else(|| agent.ticket_id.clone()),
        cwd: workdir(ctx),
        command: session_command(ctx),
    }
}

/// `{scheme}://untra.operator-terminals/connect?agent=..&authority=..&path=..`
pub fn operator_connect_uri(
    scheme: &str,
    agent_id: &str,
    authority: Option<&str>,
    path: &str,
) -> Option<String> {
    let mut url = Url::parse(&format!("{scheme}://{OPERATOR_EXTENSION}/connect")).ok()?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("agent", agent_id);
        if let Some(authority) = authority {
            query.append_pair("authority", authority);
        }
        query.append_pair("path", path);
    }
    Some(url.to_string())
}

fn focus_session_uri(session: &str) -> Option<String> {
    let mut url = Url::parse(&format!("vscode://{OPERATOR_EXTENSION}/focus-session")).ok()?;
    url.query_pairs_mut().append_pair("name", session);
    Some(url.to_string())
}

fn zed_uri(remote: Option<&RemoteHost>, path: &str) -> Option<String> {
    let raw = remote.map_or_else(
        || format!("zed://file{path}"),
        |host| format!("zed://ssh/{}{path}", host.ssh_alias),
    );
    Url::parse(&raw).ok().map(|u| u.to_string())
}

fn coder_uri(workspace: &str, path: &str, url: &str) -> Option<String> {
    let mut uri = Url::parse("vscode://coder.coder-remote/open").ok()?;
    uri.query_pairs_mut()
        .append_pair("workspace", workspace)
        .append_pair("folder", path)
        .append_pair("url", url);
    Some(uri.to_string())
}

fn option(
    interface: ConnectInterface,
    kind: ConnectKind,
    label: String,
    uri: Option<String>,
    command: Option<String>,
) -> ConnectOption {
    ConnectOption {
        interface,
        kind,
        label,
        uri,
        command,
        available: true,
        unavailable_reason: None,
    }
}

fn host_suffix(remote: Option<&RemoteSession>) -> String {
    remote.map_or_else(String::new, |r| {
        let host = r.host();
        format!(
            " (ssh: {})",
            host.display_name.as_deref().unwrap_or(&host.name)
        )
    })
}

fn ide_options(ctx: &ConnectContext, phase: SessionPhase) -> Vec<ConnectOption> {
    let Some(path) = workdir(ctx) else {
        return Vec::new();
    };
    let verb = if phase == SessionPhase::Resumable {
        "Resume in"
    } else {
        "Open in"
    };

    if let Some(RemoteSession::Coder { workspace, url, .. }) = ctx.remote {
        let mut coder = option(
            ConnectInterface::Coder,
            ConnectKind::Ide,
            format!("Open coder/{workspace} in VS Code"),
            url.as_deref().and_then(|u| coder_uri(workspace, &path, u)),
            None,
        );
        if coder.uri.is_none() {
            coder.available = false;
            coder.unavailable_reason = Some("Coder deployment URL is not set".to_string());
        }
        return vec![coder];
    }

    let host = ctx.remote.map(RemoteSession::host);
    let authority = host.map(|h| format!("ssh-remote+{}", h.ssh_alias));
    let suffix = host_suffix(ctx.remote);
    let probes = ctx.probes;
    let mut out = Vec::new();

    let vscode = probes.vscode_extension || probes.code || wrapper(ctx.agent) == "vscode";
    let ides = [
        (vscode, ConnectInterface::Vscode, "vscode"),
        (probes.cursor, ConnectInterface::Cursor, "cursor"),
    ];
    for (enabled, interface, scheme) in ides {
        if enabled {
            out.push(option(
                interface,
                ConnectKind::Ide,
                format!("{verb} {}{suffix}", interface.label()),
                operator_connect_uri(scheme, &ctx.agent.id, authority.as_deref(), &path),
                None,
            ));
        }
    }
    if probes.zed {
        out.push(option(
            ConnectInterface::Zed,
            ConnectKind::Ide,
            format!("Open folder in Zed{suffix}"),
            zed_uri(host, &path),
            None,
        ));
    }
    out
}

fn session_options(ctx: &ConnectContext, phase: SessionPhase) -> Vec<ConnectOption> {
    let agent = ctx.agent;
    let mut out = Vec::new();
    let command = local_command(ctx);

    if phase == SessionPhase::Live && ctx.remote.is_none() {
        match wrapper(agent) {
            "cmux" => out.push(option(
                ConnectInterface::Cmux,
                ConnectKind::FocusApi,
                "Focus in cmux".to_string(),
                None,
                None,
            )),
            "tmux" => out.push(option(
                ConnectInterface::Tmux,
                ConnectKind::FocusApi,
                "Switch tmux client".to_string(),
                None,
                None,
            )),
            "vscode" => {
                if let Some(session) = agent.session_name.as_deref() {
                    out.push(option(
                        ConnectInterface::Vscode,
                        ConnectKind::Ide,
                        "Focus VS Code terminal".to_string(),
                        focus_session_uri(session),
                        None,
                    ));
                }
            }
            _ => {}
        }
    }

    if let Some(cmd) = command {
        let interface = match (ctx.remote, phase) {
            (Some(_), _) => ConnectInterface::Ssh,
            (None, SessionPhase::Resumable) => ConnectInterface::Terminal,
            (None, _) => wrapper_interface(agent),
        };
        let verb = if phase == SessionPhase::Resumable {
            "Resume"
        } else {
            "Attach"
        };
        if ctx.can_open_terminal && interface != ConnectInterface::Tmux {
            out.push(option(
                ConnectInterface::Terminal,
                ConnectKind::FocusApi,
                format!("{verb} in terminal"),
                None,
                None,
            ));
        }
        out.push(option(
            interface,
            ConnectKind::Command,
            format!("{verb} with {}", interface.label()),
            None,
            Some(cmd),
        ));
    }
    out
}

fn pick_preferred(
    options: &[ConnectOption],
    preferred: Option<ConnectInterface>,
    fallback: ConnectInterface,
) -> Option<ConnectOption> {
    let usable =
        |iface: ConnectInterface| options.iter().find(|o| o.interface == iface && o.available);
    preferred
        .and_then(usable)
        .or_else(|| usable(fallback))
        .or_else(|| options.iter().find(|o| o.available))
        .cloned()
}

/// All connect options for an agent.
pub fn connect_plan(ctx: &ConnectContext) -> ConnectPlan {
    let phase = session_phase(ctx.agent);
    let mut options = Vec::new();
    if phase != SessionPhase::Gone {
        options.extend(ide_options(ctx, phase));
        options.extend(session_options(ctx, phase));
    }
    let fallback = if ctx.remote.is_some() {
        ConnectInterface::Ssh
    } else {
        wrapper_interface(ctx.agent)
    };
    ConnectPlan {
        phase,
        preferred: pick_preferred(&options, ctx.preferred, fallback),
        options,
    }
}

/// Whether and why an agent needs the operator.
pub fn attention(agent: &AgentState, now: DateTime<Utc>) -> Option<Attention> {
    let since = agent.last_activity.to_rfc3339();
    match agent.status.as_str() {
        "awaiting_input" => Some(match agent.review_state.as_deref() {
            Some(review) => Attention {
                reason: AttentionReason::Review,
                detail: Some(review.to_string()),
                since,
            },
            None => Attention {
                reason: AttentionReason::AwaitingInput,
                detail: agent.last_message.clone(),
                since,
            },
        }),
        "orphaned" => Some(Attention {
            reason: AttentionReason::Orphaned,
            detail: agent.last_message.clone(),
            since,
        }),
        "running" => agent
            .last_content_change
            .filter(|changed| now - *changed > Duration::minutes(IDLE_ATTENTION_MINUTES))
            .map(|changed| Attention {
                reason: AttentionReason::Idle,
                detail: None,
                since: changed.to_rfc3339(),
            }),
        _ => None,
    }
}

/// The remote session an agent runs in, from its persisted target and host.
pub fn resolve_remote(config: &Config, agent: &AgentState) -> Option<RemoteSession> {
    let target = agent
        .target_name
        .as_deref()
        .and_then(|name| resolve_named_target(config, name).ok());
    let coder = target.as_ref().and_then(|t| match &t.kind {
        TargetKind::Coder(c) => Some(c.clone()),
        _ => None,
    });
    let is_coder = coder.is_some()
        || agent
            .launch_mode
            .as_deref()
            .map(parse_launch_mode)
            .is_some_and(|m| m.kind == LaunchModeKind::Coder);
    let name = agent.remote_host.as_deref()?;

    if is_coder {
        let workdir = coder
            .as_ref()
            .and_then(|c| c.workdir.clone())
            .unwrap_or_else(|| format!("/home/coder/{}", agent.project));
        let fragment = config
            .tickets_path()
            .join(format!("operator/ssh/{name}.config"));
        return Some(RemoteSession::Coder {
            host: RemoteHost {
                name: name.to_string(),
                ssh_alias: workspace_alias(name),
                workdir,
                display_name: Some(format!("coder/{name}")),
                ssh_config_path: Some(fragment.to_string_lossy().into_owned()),
            },
            workspace: name.to_string(),
            url: coder.and_then(|c| std::env::var(c.url_env).ok()),
        });
    }

    target
        .and_then(|t| t.as_remote_host())
        .or_else(|| {
            resolve_named_target(config, name)
                .ok()
                .and_then(|t| t.as_remote_host())
        })
        .map(RemoteSession::Ssh)
}

/// Probe which IDEs are installed and whether the VS Code extension is listening.
pub fn probe(config: &Config) -> ConnectProbes {
    let port = config.sessions.vscode.webhook_port;
    ConnectProbes {
        vscode_extension: std::net::TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            std::time::Duration::from_millis(100),
        )
        .is_ok(),
        code: which::which("code").is_ok(),
        cursor: which::which("cursor").is_ok(),
        zed: which::which("zed").is_ok(),
    }
}

/// tmux socket Operator's sessions live on, when its generated config is in use.
pub fn tmux_socket(config: &Config) -> Option<&'static str> {
    (config.tmux.config_generated && config.tmux_config_path().exists())
        .then_some(crate::agents::tmux::OPERATOR_SOCKET)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::launcher::step_command::StepLaunchContext;

    fn agent() -> AgentState {
        let now = Utc::now();
        AgentState {
            git_context: None,
            id: "agent-1".to_string(),
            ticket_id: "FEAT-12".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "proj".to_string(),
            status: "running".to_string(),
            started_at: now,
            last_activity: now,
            last_message: None,
            paired: false,
            session_name: Some("op-FEAT-12".to_string()),
            session_wrapper: Some("tmux".to_string()),
            session_window_ref: None,
            session_context_ref: None,
            session_pane_ref: None,
            content_hash: None,
            current_step: None,
            step_started_at: None,
            last_content_change: None,
            pr_url: None,
            pr_number: None,
            repo: None,
            pr_status: None,
            completed_steps: vec![],
            llm_tool: Some("claude".to_string()),
            llm_model: None,
            launch_mode: None,
            review_state: None,
            dev_server_pid: None,
            worktree_path: Some("/wt/feat-12".to_string()),
            remote_host: None,
            step_launch_context: None,
            target_name: None,
            shutdown_recovery: None,
        }
    }

    fn with_session_id(mut a: AgentState) -> AgentState {
        a.step_launch_context = Some(StepLaunchContext {
            delegator: None,
            tool: "claude".to_string(),
            model: "opus".to_string(),
            yolo: false,
            session_id: Some("sess-1".to_string()),
            opr8r: "opr8r".to_string(),
            operator_relay: None,
            extra_flags: vec![],
        });
        a
    }

    fn ssh_remote() -> RemoteSession {
        RemoteSession::Ssh(RemoteHost {
            name: "devbox".to_string(),
            ssh_alias: "devbox".to_string(),
            workdir: "/home/me/proj".to_string(),
            display_name: None,
            ssh_config_path: None,
        })
    }

    fn ctx<'a>(
        agent: &'a AgentState,
        remote: Option<&'a RemoteSession>,
        probes: &'a ConnectProbes,
    ) -> ConnectContext<'a> {
        ConnectContext {
            agent,
            remote,
            tmux_socket: None,
            preferred: None,
            can_open_terminal: false,
            probes,
        }
    }

    fn vscode_probes() -> ConnectProbes {
        ConnectProbes {
            code: true,
            ..ConnectProbes::default()
        }
    }

    #[test]
    fn test_resume_command_per_tool() {
        assert_eq!(
            resume_command("claude", "abc").as_deref(),
            Some("claude --resume abc")
        );
        assert_eq!(
            resume_command("codex", "abc").as_deref(),
            Some("codex resume abc")
        );
        assert_eq!(resume_command("grok", "abc"), None);
    }

    #[test]
    fn test_session_phase_live_resumable_gone() {
        assert_eq!(session_phase(&agent()), SessionPhase::Live);
        let mut orphan = with_session_id(agent());
        orphan.status = "orphaned".to_string();
        assert_eq!(session_phase(&orphan), SessionPhase::Resumable);
        let mut gone = agent();
        gone.status = "orphaned".to_string();
        assert_eq!(session_phase(&gone), SessionPhase::Gone);
    }

    #[test]
    fn test_connect_plan_ssh_agent_builds_remote_vscode_uri_and_ssh_attach() {
        let a = agent();
        let remote = ssh_remote();
        let probes = vscode_probes();
        let plan = connect_plan(&ctx(&a, Some(&remote), &probes));

        let vscode = plan
            .options
            .iter()
            .find(|o| o.interface == ConnectInterface::Vscode)
            .unwrap();
        let uri = vscode.uri.as_deref().unwrap();
        assert!(uri.starts_with("vscode://untra.operator-terminals/connect?"));
        assert!(uri.contains("authority=ssh-remote%2Bdevbox"));
        assert!(uri.contains("path=%2Fhome%2Fme%2Fproj"));

        let ssh = plan
            .options
            .iter()
            .find(|o| o.interface == ConnectInterface::Ssh)
            .unwrap();
        assert_eq!(
            ssh.command.as_deref(),
            Some("ssh -t devbox 'tmux attach -t op-FEAT-12'")
        );
    }

    #[test]
    fn test_connect_plan_uris_never_carry_commands() {
        let a = with_session_id(agent());
        let remote = ssh_remote();
        let probes = ConnectProbes {
            code: true,
            cursor: true,
            zed: true,
            vscode_extension: true,
        };
        let plan = connect_plan(&ctx(&a, Some(&remote), &probes));
        for o in plan.options.iter().filter_map(|o| o.uri.as_deref()) {
            assert!(!o.contains("attach"), "{o}");
            assert!(!o.contains("resume"), "{o}");
        }
    }

    #[test]
    fn test_connect_plan_live_session_never_resumes() {
        let a = with_session_id(agent());
        let probes = vscode_probes();
        let plan = connect_plan(&ctx(&a, None, &probes));
        assert_eq!(plan.phase, SessionPhase::Live);
        assert!(plan
            .options
            .iter()
            .filter_map(|o| o.command.as_deref())
            .all(|c| !c.contains("--resume")));
    }

    #[test]
    fn test_connect_plan_orphaned_agent_offers_resume_in_worktree() {
        let mut a = with_session_id(agent());
        a.status = "orphaned".to_string();
        let probes = ConnectProbes::default();
        let plan = connect_plan(&ctx(&a, None, &probes));
        let cmd = plan
            .options
            .iter()
            .find(|o| o.kind == ConnectKind::Command)
            .and_then(|o| o.command.as_deref())
            .unwrap();
        assert_eq!(cmd, "cd /wt/feat-12 && claude --resume sess-1");
    }

    #[test]
    fn test_connect_plan_gone_has_no_options() {
        let mut a = agent();
        a.status = "orphaned".to_string();
        let probes = vscode_probes();
        let plan = connect_plan(&ctx(&a, None, &probes));
        assert!(plan.options.is_empty());
        assert!(plan.preferred.is_none());
    }

    #[test]
    fn test_connect_plan_preferred_honours_config_then_wrapper() {
        let a = agent();
        let probes = vscode_probes();
        let mut c = ctx(&a, None, &probes);
        assert_eq!(
            connect_plan(&c).preferred.unwrap().interface,
            ConnectInterface::Tmux
        );
        c.preferred = Some(ConnectInterface::Vscode);
        assert_eq!(
            connect_plan(&c).preferred.unwrap().interface,
            ConnectInterface::Vscode
        );
        c.preferred = Some(ConnectInterface::Zed);
        assert_eq!(
            connect_plan(&c).preferred.unwrap().interface,
            ConnectInterface::Tmux
        );
    }

    #[test]
    fn test_connect_plan_tmux_socket_in_attach_command() {
        let a = agent();
        let probes = ConnectProbes::default();
        let mut c = ctx(&a, None, &probes);
        c.tmux_socket = Some("operator");
        let plan = connect_plan(&c);
        assert!(plan
            .options
            .iter()
            .any(|o| o.command.as_deref() == Some("tmux -L operator attach -t op-FEAT-12")));
    }

    #[test]
    fn test_connect_plan_coder_without_url_is_unavailable() {
        let a = agent();
        let remote = RemoteSession::Coder {
            host: RemoteHost {
                name: "ws".to_string(),
                ssh_alias: "op-coder-ws".to_string(),
                workdir: "/home/coder/proj".to_string(),
                display_name: Some("coder/ws".to_string()),
                ssh_config_path: Some("/t/ws.config".to_string()),
            },
            workspace: "ws".to_string(),
            url: None,
        };
        let probes = vscode_probes();
        let plan = connect_plan(&ctx(&a, Some(&remote), &probes));
        let coder = plan
            .options
            .iter()
            .find(|o| o.interface == ConnectInterface::Coder)
            .unwrap();
        assert!(!coder.available);
        let ssh = plan
            .options
            .iter()
            .find(|o| o.interface == ConnectInterface::Ssh)
            .unwrap();
        assert!(ssh.command.as_deref().unwrap().contains("-F /t/ws.config"));
        assert_eq!(plan.preferred.unwrap().interface, ConnectInterface::Ssh);
    }

    #[test]
    fn test_connect_terminal_vscode_wrapper_focuses_existing() {
        let mut a = agent();
        a.session_wrapper = Some("vscode".to_string());
        let probes = ConnectProbes::default();
        let t = connect_terminal(&ctx(&a, None, &probes));
        assert_eq!(t.command, None);
        assert_eq!(t.terminal_name, "op-FEAT-12");
    }

    #[test]
    fn test_connect_terminal_remote_runs_inner_attach() {
        let a = agent();
        let remote = ssh_remote();
        let probes = ConnectProbes::default();
        let t = connect_terminal(&ctx(&a, Some(&remote), &probes));
        assert_eq!(t.command.as_deref(), Some("tmux attach -t op-FEAT-12"));
        assert_eq!(t.cwd.as_deref(), Some("/home/me/proj"));
    }

    #[test]
    fn test_terminal_invocation_wraps_command_in_sh() {
        assert_eq!(
            terminal_invocation("open -na Ghostty --args -e {command}", "tmux attach -t s"),
            "open -na Ghostty --args -e sh -c 'tmux attach -t s'"
        );
    }

    #[test]
    fn test_attention_reasons() {
        let now = Utc::now();
        let mut a = agent();
        assert_eq!(attention(&a, now), None);

        a.status = "awaiting_input".to_string();
        assert_eq!(
            attention(&a, now).unwrap().reason,
            AttentionReason::AwaitingInput
        );
        a.review_state = Some("pending_plan".to_string());
        assert_eq!(attention(&a, now).unwrap().reason, AttentionReason::Review);

        a.status = "orphaned".to_string();
        assert_eq!(
            attention(&a, now).unwrap().reason,
            AttentionReason::Orphaned
        );

        a.status = "running".to_string();
        a.last_content_change = Some(now - Duration::minutes(IDLE_ATTENTION_MINUTES + 1));
        assert_eq!(attention(&a, now).unwrap().reason, AttentionReason::Idle);
        a.last_content_change = Some(now);
        assert_eq!(attention(&a, now), None);
    }
}
