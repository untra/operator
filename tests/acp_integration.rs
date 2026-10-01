//! End-to-end tests: spawn `operator acp` as a subprocess and roundtrip
//! real ACP messages over stdio.
//!
//! Phase A: `initialize` roundtrip. Phase B adds `session/new` and
//! `session/prompt` with `/bin/cat` as a stand-in delegator.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

const PER_LINE_TIMEOUT: Duration = Duration::from_secs(5);
const REGISTRY_ENV: &str = "OPERATOR_PROFILE_REGISTRY";
const REGISTRY_FILE: &str = "profiles.sqlite";
const INITIALIZE_REQUEST: &[u8] = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"acp-integration-test","version":"0.0.0"}}}"#;

/// Spawnable `operator` pointed at `registry` instead of the developer's own
/// profile registry, so tests neither depend on nor pollute it.
fn operator_command(registry: &Path, config: Option<&Path>, subcommand: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_operator"));
    command.env(REGISTRY_ENV, registry);
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    command
        .arg(subcommand)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

/// A directory nothing can be created in, restored on drop so the tempdir can
/// clean itself up even when an assertion panics.
#[cfg(unix)]
struct ReadOnlyDir(tempfile::TempDir);

#[cfg(unix)]
impl ReadOnlyDir {
    fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        Self(dir)
    }

    fn registry(&self) -> PathBuf {
        self.0.path().join("operator").join(REGISTRY_FILE)
    }
}

#[cfg(unix)]
impl Drop for ReadOnlyDir {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(self.0.path(), std::fs::Permissions::from_mode(0o755));
    }
}

async fn read_line<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut tokio::io::Lines<BufReader<R>>,
) -> String {
    tokio::time::timeout(PER_LINE_TIMEOUT, reader.next_line())
        .await
        .expect("timeout waiting for stdio response")
        .expect("stdio read error")
        .expect("eof before response")
}

#[tokio::test]
async fn test_operator_acp_stdio_initialize_roundtrip() {
    let tickets = tempfile::TempDir::new().unwrap();
    let registry_dir = tempfile::TempDir::new().unwrap();
    let registry = registry_dir.path().join(REGISTRY_FILE);
    let (_config_keep, config_path) = write_cat_delegator_config(tickets.path());
    let mut child = operator_command(&registry, Some(&config_path), "acp")
        .spawn()
        .expect("spawn operator acp");

    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    stdin
        .write_all(INITIALIZE_REQUEST)
        .await
        .expect("write request");
    stdin.write_all(b"\n").await.expect("terminate request");
    stdin.flush().await.expect("flush request");

    let line = read_line(&mut reader).await;
    let response: serde_json::Value =
        serde_json::from_str(&line).expect("response should be valid JSON");
    assert_eq!(response["jsonrpc"], "2.0", "response missing jsonrpc=2.0");
    assert_eq!(response["id"], 1, "response id should echo request id");
    let result = response["result"]
        .as_object()
        .expect("result should be an object");
    assert_eq!(
        result["protocolVersion"], 1,
        "should echo protocolVersion 1"
    );
    assert_eq!(
        result["agentInfo"]["name"], "operator",
        "agentInfo.name should identify operator: {result:?}"
    );
    assert!(
        registry.exists(),
        "{REGISTRY_ENV} should redirect registration to {}",
        registry.display()
    );

    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
}

#[cfg(unix)]
#[tokio::test]
async fn test_acp_initialize_survives_unwritable_registry() {
    let unwritable = ReadOnlyDir::new();
    let tickets = tempfile::TempDir::new().unwrap();
    let (_config_keep, config_path) = write_cat_delegator_config(tickets.path());
    let mut child = operator_command(&unwritable.registry(), Some(&config_path), "acp")
        .spawn()
        .expect("spawn operator acp");

    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    stdin.write_all(INITIALIZE_REQUEST).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let response: serde_json::Value =
        serde_json::from_str(&read_line(&mut reader).await).expect("response should be valid JSON");
    assert_eq!(
        response["result"]["agentInfo"]["name"], "operator",
        "acp must still serve when the registry is unwritable: {response}"
    );

    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
}

#[cfg(unix)]
#[tokio::test]
async fn test_unwritable_registry_still_fails_non_protocol_commands() {
    let unwritable = ReadOnlyDir::new();
    let tickets = tempfile::TempDir::new().unwrap();
    let (_config_keep, config_path) = write_cat_delegator_config(tickets.path());

    let output = operator_command(&unwritable.registry(), Some(&config_path), "queue")
        .output()
        .await
        .expect("run operator queue");

    assert!(
        !output.status.success(),
        "only stdio protocol servers may run unregistered; queue exited {}",
        output.status
    );
}

fn write_sleep_delegator_config(
    tickets_dir: &std::path::Path,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempfile::TempDir::new().unwrap();
    let config_path = temp.path().join("operator.toml");
    let body = format!(
        r#"
[paths]
tickets = "{tickets}"
projects = "."
state = "{tickets}/operator"
worktrees = "/tmp/operator-worktrees"

[llm_tools]

[[llm_tools.detected]]
name = "sleeper"
path = "/bin/sleep"
version = "noop"
command_template = "sleep 60"
health_ok = true

[[delegators]]
name = "test-sleeper"
llm_tool = "sleeper"
model = "noop"

[acp]
default_delegator = "test-sleeper"
"#,
        tickets = tickets_dir.display()
    );
    std::fs::write(&config_path, body).unwrap();
    (temp, config_path)
}

/// Write a minimal TOML config that makes `/bin/cat` look like a configured
/// LLM tool + delegator, then return its path (kept alive by the returned
/// `TempDir`).
fn write_cat_delegator_config(
    tickets_dir: &std::path::Path,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempfile::TempDir::new().unwrap();
    let config_path = temp.path().join("operator.toml");
    let body = format!(
        r#"
[paths]
tickets = "{tickets}"
projects = "."
state = "{tickets}/operator"
worktrees = "/tmp/operator-worktrees"

[llm_tools]

[[llm_tools.detected]]
name = "cat"
path = "/bin/cat"
version = "noop"
command_template = "cat {{{{prompt_file}}}}"
health_ok = true

[[delegators]]
name = "test-cat"
llm_tool = "cat"
model = "noop"

[acp]
default_delegator = "test-cat"
"#,
        tickets = tickets_dir.display()
    );
    std::fs::write(&config_path, body).unwrap();
    (temp, config_path)
}

#[tokio::test]
async fn test_operator_acp_session_new_and_prompt_with_cat_delegator() {
    let tickets = tempfile::TempDir::new().unwrap();
    let registry_dir = tempfile::TempDir::new().unwrap();
    let registry = registry_dir.path().join(REGISTRY_FILE);
    let cwd = tempfile::TempDir::new().unwrap();
    let canonical_cwd = std::fs::canonicalize(cwd.path()).unwrap();
    let (_config_keep, config_path) = write_cat_delegator_config(tickets.path());

    let mut child = operator_command(&registry, Some(&config_path), "acp")
        .spawn()
        .expect("spawn operator acp with cat-delegator config");

    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    // 1. initialize
    let init = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"acp-prompt-test","version":"0.0.0"}}}"#;
    stdin.write_all(init).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let init_line = read_line(&mut reader).await;
    let init_resp: serde_json::Value = serde_json::from_str(&init_line).unwrap();
    assert_eq!(init_resp["id"], 1, "initialize id mismatch: {init_resp}");

    // 2. session/new with our cwd
    let new_session = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"session/new","params":{{"cwd":"{}","mcpServers":[]}}}}"#,
        canonical_cwd.display()
    );
    stdin.write_all(new_session.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let new_line = read_line(&mut reader).await;
    let new_resp: serde_json::Value = serde_json::from_str(&new_line).unwrap();
    assert_eq!(new_resp["id"], 2, "session/new id mismatch: {new_resp}");
    let session_id = new_resp["result"]["sessionId"]
        .as_str()
        .expect("sessionId in response: {new_resp}")
        .to_string();
    assert!(!session_id.is_empty(), "sessionId must be non-empty");

    // 3. session/prompt with text "hello"
    let prompt = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{{"sessionId":"{session_id}","prompt":[{{"type":"text","text":"hello"}}]}}}}"#
    );
    stdin.write_all(prompt.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Read lines until we see (a) a session/update notification containing
    // "hello" and (b) the session/prompt response with id=3. Streaming
    // ordering isn't strict: assert both observed within ~10s budget.
    let mut saw_hello_update = false;
    let mut prompt_response: Option<serde_json::Value> = None;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline {
        let line = match tokio::time::timeout(Duration::from_secs(5), reader.next_line()).await {
            Ok(Ok(Some(l))) => l,
            Ok(Ok(None)) => break,
            _ => break,
        };
        let msg: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if msg["method"] == "session/update" {
            let text = msg["params"]["update"]["content"]["text"]
                .as_str()
                .unwrap_or("");
            if text.contains("hello") {
                saw_hello_update = true;
            }
        } else if msg["id"] == 3 {
            prompt_response = Some(msg);
            break;
        }
    }

    let resp = prompt_response.expect("session/prompt response must arrive");
    assert!(
        saw_hello_update,
        "expected at least one session/update containing 'hello'; final response: {resp}"
    );
    assert_eq!(
        resp["result"]["stopReason"], "end_turn",
        "cat exits 0, expected stopReason=end_turn: {resp}"
    );

    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
}

#[tokio::test]
async fn test_cancel_kills_delegator() {
    let tickets = tempfile::TempDir::new().unwrap();
    let registry_dir = tempfile::TempDir::new().unwrap();
    let registry = registry_dir.path().join(REGISTRY_FILE);
    let cwd = tempfile::TempDir::new().unwrap();
    let canonical_cwd = std::fs::canonicalize(cwd.path()).unwrap();
    let (_config_keep, config_path) = write_sleep_delegator_config(tickets.path());

    let mut child = operator_command(&registry, Some(&config_path), "acp")
        .spawn()
        .expect("spawn operator acp with sleep-delegator config");

    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    // 1. initialize
    let init = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"acp-cancel-test","version":"0.0.0"}}}"#;
    stdin.write_all(init).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let init_line = read_line(&mut reader).await;
    let init_resp: serde_json::Value = serde_json::from_str(&init_line).unwrap();
    assert_eq!(init_resp["id"], 1);

    // 2. session/new
    let new_session = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"session/new","params":{{"cwd":"{}","mcpServers":[]}}}}"#,
        canonical_cwd.display()
    );
    stdin.write_all(new_session.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let new_line = read_line(&mut reader).await;
    let new_resp: serde_json::Value = serde_json::from_str(&new_line).unwrap();
    assert_eq!(new_resp["id"], 2);
    let session_id = new_resp["result"]["sessionId"]
        .as_str()
        .expect("sessionId")
        .to_string();

    // 3. session/prompt (delegator runs `sleep 60` - a long-running process)
    let prompt = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{{"sessionId":"{session_id}","prompt":[{{"type":"text","text":"ignored"}}]}}}}"#
    );
    stdin.write_all(prompt.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Give the delegator a moment to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 4. Send cancel notification (no id - it's a notification)
    let cancel = format!(
        r#"{{"jsonrpc":"2.0","method":"session/cancel","params":{{"sessionId":"{session_id}"}}}}"#
    );
    stdin.write_all(cancel.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // 5. The prompt response should arrive quickly with stopReason "cancelled"
    let mut prompt_response: Option<serde_json::Value> = None;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let line = match tokio::time::timeout(Duration::from_secs(3), reader.next_line()).await {
            Ok(Ok(Some(l))) => l,
            Ok(Ok(None)) => break,
            _ => break,
        };
        let msg: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if msg["id"] == 3 {
            prompt_response = Some(msg);
            break;
        }
    }

    let resp = prompt_response.expect("session/prompt response must arrive after cancel");
    assert_eq!(
        resp["result"]["stopReason"], "cancelled",
        "cancel should yield stopReason=cancelled: {resp}"
    );

    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
}
