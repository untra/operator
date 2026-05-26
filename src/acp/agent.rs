//! Operator's ACP agent over stdio.
//!
//! Wires the [`agent_client_protocol::Agent`] role builder up to a [`Stdio`]
//! transport. Editors that speak ACP launch `operator acp` as a subprocess
//! and exchange line-delimited JSON-RPC with this loop.

use std::process::Stdio as ProcStdio;
use std::sync::Arc;

use agent_client_protocol::schema::{
    AgentCapabilities, CancelNotification, ContentBlock, Implementation, InitializeRequest,
    InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse,
    SessionId, SessionNotification, StopReason,
};
use agent_client_protocol::{Agent, Client, ConnectionTo, Dispatch, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;

use crate::acp::session::SessionRegistry;
use crate::acp::translator;
use crate::config::{Config, Delegator};

/// Build the `InitializeResponse` operator advertises.
///
/// Echoes the client's protocol version (the ACP convention — the agent
/// accepts the protocol version requested unless it cannot satisfy it),
/// advertises default agent capabilities, and attaches `agentInfo` so
/// editors can identify operator in their UI.
pub fn build_initialize_response(request: &InitializeRequest) -> InitializeResponse {
    InitializeResponse::new(request.protocol_version)
        .agent_capabilities(AgentCapabilities::default())
        .agent_info(Implementation::new("operator", env!("CARGO_PKG_VERSION")).title("Operator"))
}

/// Run operator as an ACP agent over stdin/stdout until the client
/// disconnects.
///
/// Returns the protocol's `Result` so the binary entrypoint can surface
/// transport errors. Logs go to stderr via `tracing`; stdout is reserved
/// for line-delimited JSON-RPC (see `src/logging.rs` — global subscriber
/// writes to stderr).
pub async fn run_stdio(config: Config) -> agent_client_protocol::Result<()> {
    let registry = Arc::new(SessionRegistry::new());
    let config = Arc::new(config);

    let new_session_registry = Arc::clone(&registry);
    let new_session_config = Arc::clone(&config);
    let prompt_registry = Arc::clone(&registry);
    let prompt_config = Arc::clone(&config);
    let cancel_registry = Arc::clone(&registry);

    Agent
        .builder()
        .name("operator")
        .on_receive_request(
            async move |request: InitializeRequest, responder, _connection| {
                responder.respond(build_initialize_response(&request))
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: NewSessionRequest, responder, _connection| {
                match new_session_registry.create_or_attach(&new_session_config, &request.cwd) {
                    Ok(session_id) => {
                        tracing::info!(?session_id, cwd = %request.cwd.display(), "ACP session opened");
                        responder.respond(NewSessionResponse::new(session_id))
                    }
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(format!(
                            "session/new failed: {err}"
                        )),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: PromptRequest, responder, connection| {
                let reg = Arc::clone(&prompt_registry);
                let cfg = Arc::clone(&prompt_config);
                let cx = connection.clone();
                connection.spawn(async move {
                    let response = handle_prompt(&reg, &cfg, request, &cx).await;
                    match response {
                        Ok(resp) => responder.respond(resp)?,
                        Err(message) => responder.respond_with_error(
                            agent_client_protocol::util::internal_error(message),
                        )?,
                    }
                    Ok(())
                })?;
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_notification(
            async move |notif: CancelNotification, _connection| {
                tracing::info!(session_id = ?notif.session_id, "ACP cancel received");
                if let Some(tx) = cancel_registry.take_cancel_sender(&notif.session_id) {
                    let _ = tx.send(());
                    tracing::info!(session_id = ?notif.session_id, "ACP cancel signal sent to delegator");
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_dispatch(
            async move |message: Dispatch, cx: ConnectionTo<Client>| {
                let method = message.method().to_string();
                message.respond_with_error(
                    agent_client_protocol::util::internal_error(format!(
                        "ACP method not implemented: {method}"
                    )),
                    cx,
                )
            },
            agent_client_protocol::on_receive_dispatch!(),
        )
        .connect_to(Stdio::new())
        .await
}

/// Concatenate the text content blocks of a `PromptRequest`.
///
/// v1 supports text only; other `ContentBlock` variants (image, audio,
/// resource) are skipped. The result is one newline-joined string suitable
/// for piping to a CLI delegator's prompt file.
fn flatten_prompt(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pick the delegator to use for an ACP prompt: prefer `[acp].default_delegator`
/// by name, then fall back to `agents::delegator_resolution::resolve_default_delegator`.
fn resolve_acp_delegator(config: &Config) -> Option<&Delegator> {
    if let Some(name) = config.acp.default_delegator.as_deref() {
        if let Some(d) = config.delegators.iter().find(|d| d.name == name) {
            return Some(d);
        }
        tracing::warn!(
            requested = name,
            "acp.default_delegator name not found; falling back to default resolver"
        );
    }
    crate::agents::delegator_resolution::resolve_default_delegator(config)
}

/// Run the prompt → delegator → stream-back-to-editor pipeline.
///
/// Returns the prompt response on success, or an `Err(String)` message that
/// the caller will wrap in `internal_error`.
async fn handle_prompt(
    registry: &SessionRegistry,
    config: &Config,
    request: PromptRequest,
    connection: &ConnectionTo<Client>,
) -> Result<PromptResponse, String> {
    let session_id = request.session_id.clone();
    let session = registry
        .get(&session_id)
        .ok_or_else(|| format!("unknown ACP session: {}", session_id.0))?;

    let prompt_text = flatten_prompt(&request.prompt);
    let delegator = resolve_acp_delegator(config)
        .cloned()
        .ok_or_else(|| "no delegator configured for ACP prompts".to_string())?;

    let session_id_str = session_id.0.to_string();
    let prompt_file =
        crate::agents::launcher::prompt::write_prompt_file(config, &session_id_str, &prompt_text)
            .map_err(|e| format!("write_prompt_file: {e}"))?;

    let mut command_string =
        crate::agents::launcher::llm_command::build_llm_command_with_permissions_for_tool(
            config,
            &delegator.llm_tool,
            &delegator.model,
            &session_id_str,
            &prompt_file,
            None,
            None,
            Some(false),
        )
        .map_err(|e| format!("build_llm_command: {e}"))?;

    if delegator.llm_tool == "claude" {
        command_string.push_str(" --output-format stream-json");
    }

    tracing::info!(
        ?session_id,
        delegator = %delegator.name,
        cwd = %session.working_directory.display(),
        "ACP prompt: spawning delegator"
    );

    let (cancel_tx, cancel_rx) = oneshot::channel();
    registry.register_cancel_sender(&session_id, cancel_tx);

    let stop_reason = stream_delegator(
        &command_string,
        &session.working_directory,
        &session_id,
        connection,
        cancel_rx,
    )
    .await
    .map_err(|e| format!("delegator subprocess: {e}"))?;

    registry.take_cancel_sender(&session_id);

    Ok(PromptResponse::new(stop_reason))
}

/// Spawn the delegator via `bash -lc <cmd>` in `cwd`, stream stdout
/// line-by-line as ACP `AgentMessageChunk` notifications, and return the
/// final `StopReason` based on exit status. If `cancel_rx` fires, the
/// child process is killed and `StopReason::Cancelled` is returned.
async fn stream_delegator(
    command_string: &str,
    cwd: &std::path::Path,
    session_id: &SessionId,
    connection: &ConnectionTo<Client>,
    mut cancel_rx: oneshot::Receiver<()>,
) -> std::io::Result<StopReason> {
    let mut child = Command::new("bash")
        .arg("-lc")
        .arg(command_string)
        .current_dir(cwd)
        .stdin(ProcStdio::null())
        .stdout(ProcStdio::piped())
        .stderr(ProcStdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("failed to capture delegator stdout"))?;

    let mut lines = BufReader::new(stdout).lines();
    loop {
        tokio::select! {
            line_result = lines.next_line() => {
                match line_result? {
                    Some(line) => {
                        if let Some(update) = translator::line_to_update(&line) {
                            let notif = SessionNotification::new(session_id.clone(), update);
                            if let Err(e) = connection.send_notification(notif) {
                                tracing::warn!(error = %e, "ACP send_notification failed");
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = &mut cancel_rx => {
                tracing::info!(?session_id, "ACP cancel: killing delegator subprocess");
                child.kill().await.ok();
                return Ok(StopReason::Cancelled);
            }
        }
    }

    let status = child.wait().await?;
    Ok(if status.success() {
        StopReason::EndTurn
    } else {
        StopReason::Refusal
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::schema::ProtocolVersion;

    #[test]
    fn test_initialize_response_echoes_protocol_version() {
        let request = InitializeRequest::new(ProtocolVersion::V1);
        let response = build_initialize_response(&request);
        assert_eq!(response.protocol_version, ProtocolVersion::V1);
    }

    #[test]
    fn test_initialize_response_advertises_agent_info() {
        let request = InitializeRequest::new(ProtocolVersion::V1);
        let response = build_initialize_response(&request);
        let info = response.agent_info.expect("agent_info must be populated");
        assert_eq!(info.name, "operator");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    }
}
