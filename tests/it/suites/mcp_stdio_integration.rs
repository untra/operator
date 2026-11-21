//! End-to-end test: spawn `operator mcp` as a subprocess and roundtrip
//! a real JSON-RPC handshake over stdio.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

const REGISTRY_ENV: &str = "OPERATOR_PROFILE_REGISTRY";
const REGISTRY_FILE: &str = "profiles.sqlite";
const INITIALIZE_REQUEST: &[u8] =
    b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n";

/// Spawnable `operator mcp` pointed at `registry` instead of the developer's
/// own profile registry, so tests neither depend on nor pollute it.
fn operator_mcp(registry: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_operator"));
    command
        .env(REGISTRY_ENV, registry)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

#[tokio::test]
async fn test_operator_mcp_stdio_initialize_and_list_tools() {
    let registry_dir = tempfile::TempDir::new().unwrap();
    let mut child = operator_mcp(&registry_dir.path().join(REGISTRY_FILE))
        .spawn()
        .expect("spawn operator mcp");

    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    stdin.write_all(INITIALIZE_REQUEST).await.unwrap();
    stdin
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n")
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let line1 = tokio::time::timeout(Duration::from_secs(5), reader.next_line())
        .await
        .expect("timeout waiting for initialize response")
        .expect("read err")
        .expect("eof");
    let resp1: serde_json::Value = serde_json::from_str(&line1).unwrap();
    assert_eq!(resp1["id"], 1);
    assert_eq!(resp1["result"]["serverInfo"]["name"], "operator");
    assert!(
        resp1["result"]["capabilities"]["resources"].is_object(),
        "resources capability should be advertised"
    );

    let line2 = tokio::time::timeout(Duration::from_secs(5), reader.next_line())
        .await
        .expect("timeout waiting for tools/list response")
        .expect("read err")
        .expect("eof");
    let resp2: serde_json::Value = serde_json::from_str(&line2).unwrap();
    assert_eq!(resp2["id"], 2);
    let tools = resp2["result"]["tools"].as_array().expect("tools array");
    assert!(
        tools.len() >= 8,
        "expected at least 8 read tools, got {}",
        tools.len()
    );
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(
        names.contains(&"operator_list_tickets"),
        "operator_list_tickets should be in tool list, got: {names:?}"
    );

    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
}

#[cfg(unix)]
#[tokio::test]
async fn test_mcp_initialize_survives_unwritable_registry() {
    use std::os::unix::fs::PermissionsExt;
    let unwritable = tempfile::TempDir::new().unwrap();
    std::fs::set_permissions(unwritable.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let registry = unwritable.path().join("operator").join(REGISTRY_FILE);

    let mut child = operator_mcp(&registry).spawn().expect("spawn operator mcp");
    let mut stdin = child.stdin.take().expect("take stdin");
    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = BufReader::new(stdout).lines();

    stdin.write_all(INITIALIZE_REQUEST).await.unwrap();
    stdin.flush().await.unwrap();

    let line = tokio::time::timeout(Duration::from_secs(5), reader.next_line()).await;
    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
    std::fs::set_permissions(unwritable.path(), std::fs::Permissions::from_mode(0o755)).unwrap();

    let line = line
        .expect("timeout waiting for initialize response")
        .expect("read err")
        .expect("mcp must still serve when the registry is unwritable");
    let response: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(response["result"]["serverInfo"]["name"], "operator");
}
