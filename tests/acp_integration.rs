//! End-to-end tests: spawn `operator acp` as a subprocess and roundtrip
//! real ACP messages over stdio.
//!
//! Phase A: `initialize` roundtrip. Phase B adds `session/new` and
//! `session/prompt` with `/bin/cat` as a stand-in delegator.

use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

const PER_LINE_TIMEOUT: Duration = Duration::from_secs(5);

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
    let exe = env!("CARGO_BIN_EXE_operator");
    let mut child = Command::new(exe)
        .arg("acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn operator acp");

    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    let request = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"acp-integration-test","version":"0.0.0"}}}
"#;
    stdin.write_all(request).await.expect("write request");
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

    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
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
    let exe = env!("CARGO_BIN_EXE_operator");
    let tickets = tempfile::TempDir::new().unwrap();
    let cwd = tempfile::TempDir::new().unwrap();
    let canonical_cwd = std::fs::canonicalize(cwd.path()).unwrap();
    let (_config_keep, config_path) = write_cat_delegator_config(tickets.path());

    let mut child = Command::new(exe)
        .arg("--config")
        .arg(&config_path)
        .arg("acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
