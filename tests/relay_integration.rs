//! Integration tests for the relay subsystem.
//!
//! Two layers:
//!
//! **Layer 1** — `ChannelSession` over a real `RelayHub` Unix socket.
//! Exercises ask/reply, broadcast, rename, timeout, and peer-gone flows
//! using only in-process async code (no external services needed).
//!
//! **Layer 2** — `relay-channel` binary driven via JSON-RPC stdio.
//! Verifies the MCP protocol surface: initialize, tools/list, relay_peers.
//! Binary tests skip gracefully if the binary hasn't been built yet.
//!
//! ## Running
//!
//! ```bash
//! # Build opr8r first (provides relay-channel subcommand for Layer 2)
//! cargo build --manifest-path opr8r/Cargo.toml
//!
//! # Run all relay integration tests
//! OPERATOR_RELAY_INTEGRATION_TEST_ENABLED=true \
//!   cargo test --test relay_integration -- --nocapture --test-threads=1
//!
//! # Layer 1 only (no binary needed)
//! OPERATOR_RELAY_INTEGRATION_TEST_ENABLED=true \
//!   cargo test --test relay_integration test_register -- --nocapture
//!
//! # Layer 2 only
//! OPERATOR_RELAY_INTEGRATION_TEST_ENABLED=true \
//!   cargo test --test relay_integration test_binary -- --nocapture
//! ```

use std::path::PathBuf;
use std::time::Duration;

use operator::relay::hub::RelayHub;
use operator_relay::channel_session::ChannelSession;
use operator_relay::protocol::ServerMsg;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use uuid::Uuid;

// ── Enable guard ──────────────────────────────────────────────────────────────

fn relay_tests_enabled() -> bool {
    std::env::var("OPERATOR_RELAY_INTEGRATION_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

macro_rules! skip_if_not_configured {
    () => {
        if !relay_tests_enabled() {
            eprintln!("Skipping: set OPERATOR_RELAY_INTEGRATION_TEST_ENABLED=true to run relay integration tests");
            return;
        }
    };
}

// ── Test context ──────────────────────────────────────────────────────────────

struct RelayTestContext {
    hub: RelayHub,
    socket_path: PathBuf,
    _dir: TempDir,
}

impl RelayTestContext {
    async fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let socket_path = dir.path().join("hub.sock");
        let hub = RelayHub::start(socket_path.clone())
            .await
            .expect("failed to start relay hub");
        RelayTestContext {
            hub,
            socket_path,
            _dir: dir,
        }
    }

    async fn connect_as(&self, name: &str) -> (ChannelSession, mpsc::Receiver<ServerMsg>) {
        ChannelSession::connect(
            &self.socket_path,
            name.to_string(),
            "/tmp".to_string(),
            "main".to_string(),
        )
        .await
        .unwrap_or_else(|e| panic!("failed to connect as {name}: {e}"))
    }
}

// ── Layer 1: ChannelSession over real socket ──────────────────────────────────

#[tokio::test]
async fn test_register_and_list_peers() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (alice, _) = ctx.connect_as("alice").await;
    let (bob, _) = ctx.connect_as("bob").await;

    let alice_peers = alice.list_peers().await.expect("alice list_peers failed");
    let alice_sees: Vec<&str> = alice_peers.iter().map(|p| p.name.as_str()).collect();
    assert!(
        alice_sees.contains(&"bob"),
        "alice should see bob, got: {alice_sees:?}"
    );
    assert!(
        !alice_sees.contains(&"alice"),
        "alice should not see herself, got: {alice_sees:?}"
    );

    let bob_peers = bob.list_peers().await.expect("bob list_peers failed");
    let bob_sees: Vec<&str> = bob_peers.iter().map(|p| p.name.as_str()).collect();
    assert!(
        bob_sees.contains(&"alice"),
        "bob should see alice, got: {bob_sees:?}"
    );
    assert!(
        !bob_sees.contains(&"bob"),
        "bob should not see himself, got: {bob_sees:?}"
    );

    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_ask_reply_end_to_end() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (alice, mut alice_replies) = ctx.connect_as("alice").await;
    let (bob, mut bob_asks) = ctx.connect_as("bob").await;

    // Bob replies as soon as the ask arrives
    let ask_id = Uuid::new_v4().to_string();
    let ask_id_clone = ask_id.clone();
    tokio::spawn(async move {
        if let Some(ServerMsg::IncomingAsk { ask_id, .. }) = bob_asks.recv().await {
            bob.reply(ask_id, "pong".to_string());
        }
    });

    alice
        .send_ask(
            "bob".to_string(),
            "ping".to_string(),
            ask_id_clone,
            Some(5_000),
            None,
        )
        .await
        .expect("send_ask failed");

    let msg = tokio::time::timeout(Duration::from_secs(5), alice_replies.recv())
        .await
        .expect("timed out waiting for reply")
        .expect("alice_replies channel closed");

    if let ServerMsg::IncomingReply { text, .. } = msg {
        assert_eq!(text, "pong");
    } else {
        panic!("expected IncomingReply, got: {msg:?}");
    }

    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_broadcast_reaches_all_peers() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (broadcaster, _) = ctx.connect_as("broadcaster").await;
    let (_alice, mut alice_asks) = ctx.connect_as("alice").await;
    let (_bob, mut bob_asks) = ctx.connect_as("bob").await;

    let peer_count = broadcaster
        .broadcast("hello".to_string())
        .await
        .expect("broadcast failed");

    assert_eq!(peer_count, 2, "broadcast should reach 2 peers");

    // Both alice and bob should receive the ask
    tokio::time::timeout(Duration::from_secs(2), alice_asks.recv())
        .await
        .expect("alice did not receive broadcast within 2s")
        .expect("alice_asks channel closed");

    tokio::time::timeout(Duration::from_secs(2), bob_asks.recv())
        .await
        .expect("bob did not receive broadcast within 2s")
        .expect("bob_asks channel closed");

    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_rename_via_channel_session() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (alice, _) = ctx.connect_as("alice").await;
    let (watcher, _) = ctx.connect_as("watcher").await;

    alice
        .rename("alice-v2".to_string())
        .await
        .expect("rename failed");

    let peers = watcher.list_peers().await.expect("list_peers failed");
    let names: Vec<&str> = peers.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"alice-v2"),
        "new name should appear; got: {names:?}"
    );
    assert!(
        !names.contains(&"alice"),
        "old name should be gone; got: {names:?}"
    );

    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_ask_to_unknown_peer_returns_error() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (alice, mut alice_msgs) = ctx.connect_as("alice").await;

    let ask_id = Uuid::new_v4().to_string();
    alice
        .send_ask(
            "nobody".to_string(),
            "hello?".to_string(),
            ask_id.clone(),
            Some(2_000),
            None,
        )
        .await
        .expect("send_ask failed");

    let msg = tokio::time::timeout(Duration::from_secs(3), alice_msgs.recv())
        .await
        .expect("timed out waiting for error notification")
        .expect("alice_msgs channel closed");

    match msg {
        ServerMsg::Err {
            ask_id: Some(id), ..
        } => {
            assert_eq!(id, ask_id, "ask_id mismatch in error");
        }
        other => panic!("expected Err with ask_id, got: {other:?}"),
    }

    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_ask_timeout_propagates() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (alice, mut alice_msgs) = ctx.connect_as("alice").await;
    let (_bob, _bob_asks) = ctx.connect_as("bob").await;
    // Bob intentionally never replies

    let ask_id = Uuid::new_v4().to_string();
    alice
        .send_ask(
            "bob".to_string(),
            "are you there?".to_string(),
            ask_id.clone(),
            Some(200),
            None,
        )
        .await
        .expect("send_ask failed");

    // Hub delivers AskTimeout after 200ms; allow 3s total
    let msg = tokio::time::timeout(Duration::from_secs(3), alice_msgs.recv())
        .await
        .expect("timed out waiting for timeout error notification")
        .expect("alice_msgs channel closed");

    match msg {
        ServerMsg::Err {
            ask_id: Some(id), ..
        } => {
            assert_eq!(id, ask_id, "ask_id mismatch in timeout error");
        }
        other => panic!("expected Err with ask_id, got: {other:?}"),
    }

    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_peer_gone_on_disconnect() {
    skip_if_not_configured!();

    let ctx = RelayTestContext::new().await;
    let (alice, mut alice_msgs) = ctx.connect_as("alice").await;

    let (bob, _) = ctx.connect_as("bob").await;
    tokio::time::sleep(Duration::from_millis(20)).await;

    let ask_id = Uuid::new_v4().to_string();
    alice
        .send_ask(
            "bob".to_string(),
            "hi".to_string(),
            ask_id.clone(),
            Some(5_000),
            None,
        )
        .await
        .expect("send_ask failed");

    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(bob);

    // Hub detects bob gone, sends Err{ask_id} back to alice
    let msg = tokio::time::timeout(Duration::from_secs(3), alice_msgs.recv())
        .await
        .expect("timed out waiting for peer-gone error")
        .expect("alice_msgs channel closed");

    match msg {
        ServerMsg::Err {
            ask_id: Some(id), ..
        } => {
            assert_eq!(id, ask_id, "ask_id mismatch in peer-gone error");
        }
        other => panic!("expected Err with ask_id, got: {other:?}"),
    }

    ctx.hub.shutdown().await;
}

// ── Layer 2: relay-channel binary via JSON-RPC stdio ─────────────────────────

/// Returns `(binary_path, extra_args)` for invoking the relay-channel MCP server.
///
/// Preference order:
/// 1. `opr8r/target/debug/opr8r relay-channel` (primary distribution vehicle)
/// 2. `opr8r/target/release/opr8r relay-channel`
/// 3. `target/debug/relay-channel` (legacy standalone, kept for transition)
/// 4. `target/release/relay-channel`
fn relay_channel_command() -> (PathBuf, Vec<String>) {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let opr8r_debug = manifest.join("opr8r/target/debug/opr8r");
    if opr8r_debug.exists() {
        return (opr8r_debug, vec!["relay-channel".to_string()]);
    }
    let opr8r_release = manifest.join("opr8r/target/release/opr8r");
    if opr8r_release.exists() {
        return (opr8r_release, vec!["relay-channel".to_string()]);
    }
    let debug = manifest.join("target/debug/relay-channel");
    if debug.exists() {
        return (debug, vec![]);
    }
    (manifest.join("target/release/relay-channel"), vec![])
}

fn binary_available() -> bool {
    let (binary, _) = relay_channel_command();
    binary.exists()
}

/// Create a tokio Command pre-configured to run the relay-channel MCP server.
fn make_relay_channel_cmd() -> tokio::process::Command {
    let (binary, args) = relay_channel_command();
    let mut cmd = tokio::process::Command::new(binary);
    cmd.args(args);
    cmd
}

/// Send a JSON-RPC line to the process stdin (async).
async fn rpc_send(stdin: &mut (impl AsyncWriteExt + Unpin), msg: serde_json::Value) {
    let mut line = serde_json::to_string(&msg).unwrap();
    line.push('\n');
    stdin.write_all(line.as_bytes()).await.unwrap();
    stdin.flush().await.unwrap();
}

/// Read lines from stdout until we find one that is valid JSON with a matching id (async, 5s timeout).
async fn rpc_recv(
    reader: &mut BufReader<impl tokio::io::AsyncRead + Unpin>,
    id: u64,
) -> serde_json::Value {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await.expect("read_line failed");
            if n == 0 {
                panic!("relay-channel process closed stdout waiting for id={id}");
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                if val.get("id").and_then(|v| v.as_u64()) == Some(id) {
                    return val;
                }
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for JSON-RPC response with id={id}"))
}

#[tokio::test]
async fn test_binary_initialize() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!(
            "Skipping: relay-channel binary not found (run `cargo build --bin relay-channel`)"
        );
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "test-agent-init")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // Give binary time to connect to hub
    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }),
    )
    .await;

    let resp = rpc_recv(&mut stdout, 1).await;

    assert_eq!(
        resp["result"]["serverInfo"]["name"].as_str(),
        Some("relay-channel"),
        "serverInfo.name mismatch: {resp}"
    );
    assert!(
        resp["result"]["protocolVersion"].as_str().is_some(),
        "missing protocolVersion in initialize response: {resp}"
    );
    let caps = resp["result"]["capabilities"]
        .as_object()
        .expect("capabilities should be an object");
    assert!(
        caps.contains_key("experimental"),
        "missing experimental capability in: {resp}"
    );
    assert!(
        resp["result"]["capabilities"]["experimental"]["claude/channel"].is_object(),
        "missing claude/channel in experimental: {resp}"
    );

    drop(stdin); // Close stdin → process exits
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_binary_tools_list() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "test-agent-tools")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Initialize first
    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    // Request tools list
    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;

    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    let expected = [
        "relay_peers",
        "relay_ask",
        "relay_reply",
        "relay_broadcast",
        "relay_rename",
    ];
    for name in &expected {
        assert!(
            tool_names.contains(name),
            "missing tool {name}, got: {tool_names:?}"
        );
    }
    assert_eq!(
        tools.len(),
        5,
        "expected exactly 5 tools, got: {tool_names:?}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_binary_relay_peers_empty() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "solo-agent")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "relay_peers", "arguments": {} }
        }),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
    let data: serde_json::Value = serde_json::from_str(text)
        .unwrap_or_else(|_| panic!("relay_peers text is not JSON: {text:?}"));
    assert_eq!(data["ok"], true, "ok should be true: {data}");
    assert!(data["me"].is_string(), "me should be a string: {data}");
    assert_eq!(
        data["peers"].as_array().map(Vec::len),
        Some(0),
        "peers should be empty array: {data}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_binary_relay_peers_with_peer() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    // Register alice via ChannelSession so the binary will see her
    let (alice, _) = ctx.connect_as("alice").await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "observer")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "relay_peers", "arguments": {} }
        }),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
    let data: serde_json::Value = serde_json::from_str(text)
        .unwrap_or_else(|_| panic!("relay_peers text is not JSON: {text:?}"));
    assert_eq!(data["ok"], true, "ok should be true: {data}");
    let peers = data["peers"].as_array().expect("peers should be array");
    let names: Vec<&str> = peers.iter().filter_map(|p| p["name"].as_str()).collect();
    assert!(
        names.contains(&"alice"),
        "expected alice in peer list, got: {names:?}"
    );

    drop(stdin);
    let _ = child.wait().await;
    drop(alice);
    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_binary_relay_ask_returns_immediately() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "asker")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    // Ask a non-existent peer — the tool call should return immediately with ask_id
    let before = std::time::Instant::now();
    rpc_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "relay_ask", "arguments": { "to": "nobody", "question": "hi?" } }
        }),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;
    let elapsed = before.elapsed();

    // Response should arrive well under 1 second (not blocking for reply)
    assert!(
        elapsed < Duration::from_secs(1),
        "relay_ask took too long ({elapsed:?}), should return immediately"
    );

    let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
    let data: serde_json::Value = serde_json::from_str(text)
        .unwrap_or_else(|_| panic!("relay_ask response not JSON: {text:?}"));
    assert_eq!(data["ok"], true, "ok should be true: {data}");
    assert!(
        data["ask_id"].is_string(),
        "ask_id should be a string: {data}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_binary_relay_rename_structured() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "old-name")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "relay_rename", "arguments": { "new_name": "new-name" } }
        }),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
    let data: serde_json::Value = serde_json::from_str(text)
        .unwrap_or_else(|_| panic!("relay_rename response not JSON: {text:?}"));
    assert_eq!(data["ok"], true, "ok should be true: {data}");
    assert_eq!(
        data["name"].as_str(),
        Some("new-name"),
        "name should be new-name: {data}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

#[tokio::test]
async fn test_binary_incoming_ask_notification_shape() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    // Spawn a binary that will receive the ask
    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "receiver")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let stdout_raw = child.stdout.take().unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;

    // Also connect an in-process asker
    let (asker, _) = ctx.connect_as("asker").await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let ask_id = Uuid::new_v4().to_string();
    asker
        .send_ask(
            "receiver".to_string(),
            "test question?".to_string(),
            ask_id.clone(),
            Some(5_000),
            None,
        )
        .await
        .expect("send_ask failed");

    // Read lines from binary stdout until we find a notification
    let mut reader = BufReader::new(stdout_raw);
    let notification = tokio::time::timeout(Duration::from_secs(5), async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await.expect("read_line failed");
            if n == 0 {
                panic!("stdout closed");
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                if val.get("method").and_then(|m| m.as_str())
                    == Some("notifications/claude/channel")
                {
                    return val;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for notifications/claude/channel");

    assert_eq!(
        notification["method"].as_str(),
        Some("notifications/claude/channel"),
        "notification method mismatch: {notification}"
    );
    let params = &notification["params"];
    // G1: content must be the raw question text, not prefixed with "{from} asks: "
    assert_eq!(
        params["content"].as_str(),
        Some("test question?"),
        "content should be the raw question text, got: {params}"
    );
    assert_eq!(
        params["meta"]["from"].as_str(),
        Some("asker"),
        "meta.from mismatch: {params}"
    );
    assert_eq!(
        params["meta"]["ask_id"].as_str(),
        Some(ask_id.as_str()),
        "meta.ask_id mismatch: {params}"
    );
    // G3: absent optional fields must be omitted from meta (sparse), not sent as null.
    // broadcast_id is only present for broadcast-sourced asks; omit it for direct asks.
    // thread_id is hub-generated so it may be present; only broadcast_id should be absent.
    assert!(
        params["meta"].get("broadcast_id").is_none(),
        "broadcast_id should be absent from meta for a direct ask, got: {params}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

// G1: incoming reply notification content is the raw reply text, not "{from} replied: {text}"
#[tokio::test]
async fn test_binary_incoming_reply_notification_content_is_raw_text() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut asker_child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "asker2")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut asker_stdin = asker_child.stdin.take().unwrap();
    let mut asker_stdout = BufReader::new(asker_child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut asker_stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut asker_stdout, 1).await;

    // In-process peer that will reply to the ask
    let (replier, mut replier_asks) = ctx.connect_as("replier2").await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    tokio::spawn(async move {
        if let Some(ServerMsg::IncomingAsk { ask_id, .. }) = replier_asks.recv().await {
            replier.reply(ask_id, "raw reply text".to_string());
        }
    });

    // Send the ask from the binary; it returns immediately with ask_id
    rpc_send(
        &mut asker_stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "relay_ask", "arguments": { "to": "replier2", "question": "what is up?" } }
        }),
    )
    .await;
    let _ = rpc_recv(&mut asker_stdout, 2).await;

    // Wait for the reply notification (no id — it's a JSON-RPC notification)
    let notification = tokio::time::timeout(Duration::from_secs(5), async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = asker_stdout
                .read_line(&mut line)
                .await
                .expect("read_line failed");
            if n == 0 {
                panic!("stdout closed");
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                if val.get("method").and_then(|m| m.as_str())
                    == Some("notifications/claude/channel")
                    && val["params"]["meta"]["ask_id"].is_string()
                {
                    return val;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for reply notification");

    let params = &notification["params"];
    // G1: reply content must be the raw text, not "{from} replied: {text}"
    assert_eq!(
        params["content"].as_str(),
        Some("raw reply text"),
        "reply notification content should be raw text, got: {params}"
    );

    drop(asker_stdin);
    let _ = asker_child.wait().await;
    ctx.hub.shutdown().await;
}

// G2: ask-error notification uses lowercase error code matching the protocol spec
#[tokio::test]
async fn test_binary_ask_error_notification_uses_lowercase_code() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "err-tester")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let stdout_raw = child.stdout.take().unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;

    // Ask a non-existent peer → peer_not_found error
    rpc_send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "relay_ask",
                "arguments": { "to": "ghost-peer", "question": "hello?" }
            }
        }),
    )
    .await;

    let mut reader = BufReader::new(stdout_raw);
    let notification = tokio::time::timeout(Duration::from_secs(5), async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await.expect("read_line failed");
            if n == 0 {
                panic!("stdout closed");
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                if val.get("method").and_then(|m| m.as_str())
                    == Some("notifications/claude/channel")
                {
                    return val;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for error notification");

    let params = &notification["params"];

    // G2: code must be lowercase snake_case, not Rust Debug format ("PeerNotFound")
    let code = params["meta"]["code"].as_str().unwrap_or("");
    assert_eq!(
        code, "peer_not_found",
        "error code should be lowercase 'peer_not_found', got: {code:?}"
    );

    // G2: content should be human-readable guidance, not "Ask {id} failed: PeerNotFound"
    let content = params["content"].as_str().unwrap_or("");
    assert!(
        !content.contains("PeerNotFound") && !content.contains("failed:"),
        "content should be human-readable, not debug repr, got: {content:?}"
    );
    assert!(
        !content.is_empty(),
        "error content should not be empty: {params}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

// G4: relay_ask tool schema must include optional thread_id property
#[tokio::test]
async fn test_binary_relay_ask_schema_has_thread_id() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "schema-tester")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;

    let tools = resp["result"]["tools"].as_array().expect("tools array");
    let ask_tool = tools
        .iter()
        .find(|t| t["name"] == "relay_ask")
        .expect("relay_ask tool not found");

    let props = &ask_tool["inputSchema"]["properties"];
    assert!(
        props.get("thread_id").is_some(),
        "relay_ask inputSchema should have thread_id property, got: {props}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}

// G4: thread_id passed to relay_ask must appear in the incoming_ask notification meta
#[tokio::test]
async fn test_binary_relay_ask_thread_id_propagated_to_notification() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut receiver_child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "thread-receiver")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel (receiver)");

    let mut receiver_stdin = receiver_child.stdin.take().unwrap();
    let receiver_stdout_raw = receiver_child.stdout.take().unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut receiver_stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;

    // In-process asker sends with explicit thread_id
    let (asker, _) = ctx.connect_as("thread-asker").await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let thread_id = "my-thread-001".to_string();
    let ask_id = Uuid::new_v4().to_string();
    asker
        .send_ask(
            "thread-receiver".to_string(),
            "thread-correlated question".to_string(),
            ask_id.clone(),
            Some(5_000),
            Some(thread_id.clone()),
        )
        .await
        .expect("send_ask with thread_id failed");

    let mut reader = BufReader::new(receiver_stdout_raw);
    let notification = tokio::time::timeout(Duration::from_secs(5), async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await.expect("read_line failed");
            if n == 0 {
                panic!("stdout closed");
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                if val.get("method").and_then(|m| m.as_str())
                    == Some("notifications/claude/channel")
                {
                    return val;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for ask notification with thread_id");

    let params = &notification["params"];
    assert_eq!(
        params["meta"]["thread_id"].as_str(),
        Some(thread_id.as_str()),
        "thread_id should appear in notification meta, got: {params}"
    );

    drop(receiver_stdin);
    let _ = receiver_child.wait().await;
    ctx.hub.shutdown().await;
}

// G5: relay_broadcast tool schema must include optional exclude_self property
#[tokio::test]
async fn test_binary_relay_broadcast_schema_has_exclude_self() {
    skip_if_not_configured!();
    if !binary_available() {
        eprintln!("Skipping: relay-channel binary not found");
        return;
    }

    let ctx = RelayTestContext::new().await;

    let mut child = make_relay_channel_cmd()
        .env("RELAY_HUB_SOCKET", ctx.socket_path.to_str().unwrap())
        .env("RELAY_AGENT_NAME", "broadcast-schema-tester")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn relay-channel");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    tokio::time::sleep(Duration::from_millis(200)).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    )
    .await;
    let _ = rpc_recv(&mut stdout, 1).await;

    rpc_send(
        &mut stdin,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    )
    .await;
    let resp = rpc_recv(&mut stdout, 2).await;

    let tools = resp["result"]["tools"].as_array().expect("tools array");
    let broadcast_tool = tools
        .iter()
        .find(|t| t["name"] == "relay_broadcast")
        .expect("relay_broadcast tool not found");

    let props = &broadcast_tool["inputSchema"]["properties"];
    assert!(
        props.get("exclude_self").is_some(),
        "relay_broadcast inputSchema should have exclude_self property, got: {props}"
    );

    drop(stdin);
    let _ = child.wait().await;
    ctx.hub.shutdown().await;
}
