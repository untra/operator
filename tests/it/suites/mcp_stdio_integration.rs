//! End-to-end test: spawn `operator mcp` as a subprocess and roundtrip
//! a real JSON-RPC handshake over stdio.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{ChildStdout, Command};

const PROCESS_TIMEOUT: Duration = Duration::from_secs(5);
const REGISTRY_ENV: &str = "OPERATOR_PROFILE_REGISTRY";
const REGISTRY_FILE: &str = "profiles.sqlite";
const INITIALIZE_REQUEST: &[u8] =
    b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n";

fn operator_mcp(workspace: &Path, registry: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_operator"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("OPERATOR_") {
            command.env_remove(key);
        }
    }
    command
        .current_dir(workspace)
        .env("HOME", workspace)
        .env("XDG_CONFIG_HOME", workspace)
        .env(REGISTRY_ENV, registry)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command
}

async fn read_response(
    reader: &mut Lines<BufReader<ChildStdout>>,
    method: &str,
    stderr_path: &Path,
) -> serde_json::Value {
    let line = match tokio::time::timeout(PROCESS_TIMEOUT, reader.next_line()).await {
        Ok(Ok(Some(line))) => line,
        result => panic!(
            "No MCP response for {method}: {result:?}\nstderr:\n{}",
            std::fs::read_to_string(stderr_path).unwrap_or_default()
        ),
    };
    serde_json::from_str(&line).unwrap_or_else(|err| {
        panic!("Invalid MCP response for {method}: {err}\n{line}");
    })
}

#[tokio::test]
async fn test_operator_mcp_stdio_initialize_and_list_tools() {
    let workspace = tempfile::TempDir::new().unwrap();
    let stderr_path = workspace.path().join("mcp.stderr");
    let stderr = std::fs::File::create(&stderr_path).unwrap();
    let mut child = operator_mcp(workspace.path(), &workspace.path().join(REGISTRY_FILE))
        .arg("mcp")
        .stderr(stderr)
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

    let resp1 = read_response(&mut reader, "initialize", &stderr_path).await;
    assert_eq!(resp1["id"], 1);
    assert_eq!(resp1["result"]["serverInfo"]["name"], "operator");
    assert!(
        resp1["result"]["capabilities"]["resources"].is_object(),
        "resources capability should be advertised"
    );

    let resp2 = read_response(&mut reader, "tools/list", &stderr_path).await;
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
    let exit = tokio::time::timeout(PROCESS_TIMEOUT, child.wait())
        .await
        .expect("timeout waiting for MCP to exit after stdin closed")
        .expect("wait for MCP to exit");
    assert!(
        exit.success(),
        "MCP exited with {exit}:\n{}",
        std::fs::read_to_string(&stderr_path).unwrap_or_default()
    );
}

#[tokio::test]
async fn test_mcp_startup_fails_before_creating_state_when_registry_is_unwritable() {
    let workspace = tempfile::TempDir::new().unwrap();
    let registry_parent = workspace.path().join("registry-parent");
    std::fs::File::create(&registry_parent).unwrap();
    let state = workspace.path().join("state");
    let config_path = workspace.path().join("config.toml");
    let mut config = operator::config::Config::default();
    config.paths.state = state.to_str().unwrap().to_owned();
    std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();

    let mut command = operator_mcp(workspace.path(), &registry_parent.join(REGISTRY_FILE));
    command.arg("--config").arg(config_path).arg("mcp");
    let output = tokio::time::timeout(PROCESS_TIMEOUT, command.output())
        .await
        .expect("timeout waiting for MCP to exit")
        .expect("run operator mcp");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("Creating ownership directory"), "{stderr}");
    assert!(stderr.contains("registry-parent"), "{stderr}");
    assert!(output.stdout.is_empty());
    assert!(!state.exists());
}
