//! MCP stdio relay-channel server mode for opr8r.
//!
//! Invoked via `opr8r relay-channel`. Connects to the relay hub and exposes
//! relay tools (relay_peers, relay_ask, relay_reply, relay_broadcast,
//! relay_rename) to LLM agents via the MCP stdio protocol.
//!
//! This is the distribution vehicle for relay-channel: since opr8r is signed,
//! notarized, and released on all platforms, relay functionality is available
//! to agents on any machine with a standard operator install.

use std::io::{BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;

use operator_relay::channel_session::ChannelSession;
use operator_relay::protocol::ServerMsg;
use operator_relay::socket_path::hub_socket_path;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use uuid::Uuid;

// ── Tool schemas ──────────────────────────────────────────────────────────────

fn tools_list() -> Value {
    json!([
        {
            "name": "relay_peers",
            "description": "List OTHER active sessions on this machine. Returns {me, peers} where me is your own session name and peers is every other session (excluding you). Each peer has cwd and git_branch for disambiguation.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "relay_ask",
            "description": "Ask a specific peer a question. Non-blocking: returns immediately with {ok, ask_id}; the reply arrives later as a channel notification whose meta carries the same ask_id. Errors tied to this ask (peer_not_found, peer_gone, timeout) also arrive as channel notifications. Correlate by ask_id. If multiple peers may share a similar name, call relay_peers first and match by cwd or git_branch to pick the right target.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "to": { "type": "string", "description": "Name of the peer to ask" },
                    "question": { "type": "string", "description": "The question to send" },
                    "thread_id": { "type": "string", "description": "Optional thread identifier to correlate multi-turn exchanges. If you received an ask with a thread_id and are replying or continuing, pass the same thread_id." },
                    "timeout_ms": { "type": "number", "description": "Timeout in milliseconds (default: 120000)" }
                },
                "required": ["to", "question"]
            }
        },
        {
            "name": "relay_reply",
            "description": "Reply to an incoming ask by its ask_id. text is a plain string. Replies are one-shot — no streaming, no cancellation, no structured payload. If you need structured data, serialize JSON inside the string; the asker parses it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ask_id": { "type": "string", "description": "The ask_id from the incoming ask notification" },
                    "text": { "type": "string", "description": "The reply text" }
                },
                "required": ["ask_id", "text"]
            }
        },
        {
            "name": "relay_broadcast",
            "description": "Broadcast a question to ALL other peers on this machine, including sessions on unrelated projects. Use ONLY when the user explicitly wants every session asked. Do NOT use as a fallback when relay_ask returns an error (peer_not_found, peer_gone, timeout); surface the error to the user and let them decide. If you want to reach a specific peer, use relay_ask.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": { "type": "string", "description": "The question to broadcast" },
                    "exclude_self": { "type": "boolean", "description": "If true (default), the sender is excluded from recipients." }
                },
                "required": ["question"]
            }
        },
        {
            "name": "relay_rename",
            "description": "Rename this session's registered name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "new_name": { "type": "string", "description": "New peer name (alphanumeric, ., -, _ only)" }
                },
                "required": ["new_name"]
            }
        }
    ])
}

// ── Stdout helpers ────────────────────────────────────────────────────────────

fn write_response(id: Option<&Value>, result: Value) {
    let resp = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    let mut out = std::io::stdout().lock();
    let _ = serde_json::to_writer(&mut out, &resp);
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

fn write_error(id: Option<&Value>, code: i64, message: &str) {
    let resp = json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } });
    let mut out = std::io::stdout().lock();
    let _ = serde_json::to_writer(&mut out, &resp);
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

fn write_notification(method: &str, params: Value) {
    let notif = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    let mut out = std::io::stdout().lock();
    let _ = serde_json::to_writer(&mut out, &notif);
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Run the relay-channel MCP stdio server.
pub async fn run() -> ExitCode {
    let socket_path = hub_socket_path();

    let name = determine_name();

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let git_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let (session, mut incoming_ask_rx) =
        match ChannelSession::connect(&socket_path, name, cwd, git_branch).await {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("[opr8r relay-channel] Failed to connect to relay hub: {e}");
                return ExitCode::from(1);
            }
        };
    let session = Arc::new(session);

    // If no explicit name, watch for Claude /rename and propagate
    #[cfg(unix)]
    if std::env::var("RELAY_AGENT_NAME").is_err() {
        use operator_relay::session_name::{ClaudeSessionNameSource, SessionNameSource};
        let src = ClaudeSessionNameSource::for_current_process();
        let s = session.clone();
        if let Err(e) = src
            .watch(move |new_name| {
                let s = s.clone();
                tokio::spawn(async move {
                    let _ = s.rename(new_name).await;
                });
            })
            .await
        {
            eprintln!("[opr8r relay-channel] Warning: could not watch for session renames: {e}");
        }
    }

    // Stdin reader thread feeding a tokio channel
    let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(32);
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) if !l.is_empty() => {
                    if stdin_tx.blocking_send(l).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    // Main event loop: interleave stdin requests and hub notifications.
    loop {
        tokio::select! {
            line = stdin_rx.recv() => {
                match line {
                    Some(l) => handle_request(&l, &session).await,
                    None => break,
                }
            }
            Some(msg) = incoming_ask_rx.recv() => {
                match msg {
                    ServerMsg::IncomingAsk { from, question, ask_id, broadcast_id, thread_id } => {
                        let mut meta = serde_json::Map::new();
                        meta.insert("from".into(), json!(from));
                        meta.insert("ask_id".into(), json!(ask_id));
                        if let Some(bid) = broadcast_id { meta.insert("broadcast_id".into(), json!(bid)); }
                        if let Some(tid) = thread_id { meta.insert("thread_id".into(), json!(tid)); }
                        write_notification("notifications/claude/channel", json!({
                            "content": question,
                            "meta": meta
                        }));
                    }
                    ServerMsg::IncomingReply { from, text, ask_id, broadcast_id, thread_id } => {
                        let mut meta = serde_json::Map::new();
                        meta.insert("from".into(), json!(from));
                        meta.insert("ask_id".into(), json!(ask_id));
                        if let Some(bid) = broadcast_id { meta.insert("broadcast_id".into(), json!(bid)); }
                        if let Some(tid) = thread_id { meta.insert("thread_id".into(), json!(tid)); }
                        write_notification("notifications/claude/channel", json!({
                            "content": text,
                            "meta": meta
                        }));
                    }
                    ServerMsg::Err { ask_id: Some(ask_id), code, .. } => {
                        let code_val = serde_json::to_value(&code).unwrap_or(json!("unknown"));
                        let code_str = code_val.as_str().unwrap_or("unknown");
                        write_notification("notifications/claude/channel", json!({
                            "content": format!("Ask error ({code_str}): the ask could not be delivered."),
                            "meta": { "ask_id": ask_id, "code": code_str }
                        }));
                    }
                    _ => {}
                }
            }
        }
    }

    ExitCode::SUCCESS
}

fn determine_name() -> String {
    if let Ok(explicit) = std::env::var("RELAY_AGENT_NAME") {
        return explicit;
    }
    #[cfg(unix)]
    {
        use operator_relay::session_name::{ClaudeSessionNameSource, SessionNameSource};
        let src = ClaudeSessionNameSource::for_current_process();
        if let Some(name) = src.initial_name() {
            return name;
        }
    }
    format!("channel-{}", std::process::id())
}

// ── Request dispatch ──────────────────────────────────────────────────────────

async fn handle_request(line: &str, session: &Arc<ChannelSession>) {
    let Ok(req) = serde_json::from_str::<Value>(line) else {
        return;
    };

    let id = req.get("id");
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(json!({}));

    match method {
        "initialize" => {
            write_response(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "experimental": { "claude/channel": {} }
                    },
                    "serverInfo": { "name": "relay-channel", "version": "0.1.0" },
                    "instructions": "Always reply to <channel> messages via relay_reply BEFORE \
                                     other work. Use relay_peers to pick targets. Use relay_ask \
                                     for one peer, relay_broadcast for all. Surface ask errors \
                                     to the user."
                }),
            );
        }
        "initialized" | "notifications/initialized" => {}
        "tools/list" => {
            write_response(id, json!({ "tools": tools_list() }));
        }
        "tools/call" => {
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            dispatch_tool(id, tool_name, &args, session).await;
        }
        "ping" => {
            write_response(id, json!({}));
        }
        _ => {
            if id.is_some() {
                write_error(id, -32601, &format!("Method not found: {method}"));
            }
        }
    }
}

async fn dispatch_tool(
    id: Option<&Value>,
    tool_name: &str,
    args: &Value,
    session: &Arc<ChannelSession>,
) {
    match tool_name {
        "relay_peers" => match session.list_peers().await {
            Ok(peers) => {
                let payload = json!({
                    "ok": true,
                    "me": session.name(),
                    "peers": peers.iter().map(|p| json!({
                        "name": p.name,
                        "cwd": p.cwd,
                        "git_branch": p.git_branch,
                        "last_seen": p.last_seen
                    })).collect::<Vec<_>>()
                });
                write_response(
                    id,
                    json!({ "content": [{ "type": "text",
                        "text": serde_json::to_string(&payload).unwrap_or_default()
                    }]}),
                );
            }
            Err(e) => {
                let payload = json!({ "ok": false, "error": e.to_string() });
                write_response(
                    id,
                    json!({ "content": [{ "type": "text",
                        "text": serde_json::to_string(&payload).unwrap_or_default()
                    }]}),
                );
            }
        },

        "relay_ask" => {
            let to = args.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let question = args.get("question").and_then(|v| v.as_str()).unwrap_or("");
            let timeout_ms = args.get("timeout_ms").and_then(Value::as_u64);
            let thread_id = args
                .get("thread_id")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            if to.is_empty() || question.is_empty() {
                write_error(id, -32602, "relay_ask requires 'to' and 'question'");
                return;
            }
            let ask_id = Uuid::new_v4().to_string();
            match session
                .send_ask(
                    to.to_string(),
                    question.to_string(),
                    ask_id.clone(),
                    timeout_ms,
                    thread_id,
                )
                .await
            {
                Ok(()) => {
                    let payload = json!({ "ok": true, "ask_id": ask_id });
                    write_response(
                        id,
                        json!({ "content": [{ "type": "text",
                            "text": serde_json::to_string(&payload).unwrap_or_default()
                        }]}),
                    );
                }
                Err(e) => write_error(id, -32000, &e.to_string()),
            }
        }

        "relay_reply" => {
            let ask_id = args.get("ask_id").and_then(|v| v.as_str()).unwrap_or("");
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            if ask_id.is_empty() {
                write_error(id, -32602, "relay_reply requires 'ask_id'");
                return;
            }
            session.reply(ask_id.to_string(), text.to_string());
            let payload = json!({ "ok": true });
            write_response(
                id,
                json!({ "content": [{ "type": "text",
                    "text": serde_json::to_string(&payload).unwrap_or_default()
                }]}),
            );
        }

        "relay_broadcast" => {
            let question = args.get("question").and_then(|v| v.as_str()).unwrap_or("");
            if question.is_empty() {
                write_error(id, -32602, "relay_broadcast requires 'question'");
                return;
            }
            let exclude_self = args.get("exclude_self").and_then(Value::as_bool);
            let broadcast_id = Uuid::new_v4().to_string();
            match session
                .broadcast_with_id(question.to_string(), broadcast_id.clone(), exclude_self)
                .await
            {
                Ok(count) => {
                    let payload =
                        json!({ "ok": true, "broadcast_id": broadcast_id, "peer_count": count });
                    write_response(
                        id,
                        json!({ "content": [{ "type": "text",
                            "text": serde_json::to_string(&payload).unwrap_or_default()
                        }]}),
                    );
                }
                Err(e) => write_error(id, -32000, &e.to_string()),
            }
        }

        "relay_rename" => {
            let new_name = args.get("new_name").and_then(|v| v.as_str()).unwrap_or("");
            if new_name.is_empty() {
                write_error(id, -32602, "relay_rename requires 'new_name'");
                return;
            }
            match session.rename(new_name.to_string()).await {
                Ok(()) => {
                    let payload = json!({ "ok": true, "name": new_name });
                    write_response(
                        id,
                        json!({ "content": [{ "type": "text",
                            "text": serde_json::to_string(&payload).unwrap_or_default()
                        }]}),
                    );
                }
                Err(e) => write_error(id, -32000, &e.to_string()),
            }
        }

        other => {
            write_error(id, -32601, &format!("Unknown tool: {other}"));
        }
    }
}
