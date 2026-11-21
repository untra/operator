#![cfg(unix)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn run_flow(hang: bool, chain: bool) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&requests);
    let server = tokio::spawn(async move {
        let mut tasks = tokio::task::JoinSet::new();
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let recorded = Arc::clone(&recorded);
            tasks.spawn(async move {
                let mut bytes = Vec::new();
                let mut chunk = [0; 4096];
                loop {
                    let count = stream.read(&mut chunk).await.unwrap();
                    if count == 0 { return; }
                    bytes.extend_from_slice(&chunk[..count]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let length = headers.lines().find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length: ").and_then(|value| value.parse::<usize>().ok())).unwrap_or(0);
                        if bytes.len() >= end + 4 + length { break; }
                    }
                }
                let request = String::from_utf8(bytes).unwrap();
                let callback = request.starts_with("POST /api/");
                recorded.lock().unwrap().push((Instant::now(), request));
                if hang && !callback { std::future::pending::<()>().await; }
                let body = if callback {
                    serde_json::json!({"status":"completed", "auto_proceed": chain,
                        "next_command": chain.then_some("test \"$OPERATOR_OTLP_TOKEN\" = rotated"),
                        "next_env": chain.then(|| serde_json::json!({"OPERATOR_OTLP_TOKEN":"rotated"}))}).to_string()
                } else { "{}".into() };
                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                stream.write_all(response.as_bytes()).await.unwrap();
            });
        }
    });
    let output = tokio::time::timeout(
        Duration::from_secs(15),
        tokio::process::Command::new(env!("CARGO_BIN_EXE_opr8r"))
            .args([
                "--ticket-id=TASK-1",
                "--step=build",
                "--api-url",
                &endpoint,
                "--",
                "true",
            ])
            .env_remove("OPERATOR_PROFILE_ID")
            .env_remove("TRACEPARENT")
            .env("OTEL_EXPORTER_OTLP_ENDPOINT", format!("{endpoint}/otlp"))
            .env("OPERATOR_OTLP_TOKEN", "ingest-token")
            .env("OPERATOR_API_TOKEN", "callback-token")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let requests = requests.lock().unwrap();
    let callback = requests
        .iter()
        .position(|(_, body)| body.starts_with("POST /api/"))
        .unwrap();
    let trace = requests
        .iter()
        .position(|(_, body)| body.starts_with("POST /otlp/v1/traces"))
        .unwrap();
    assert!(callback < trace);
    assert!(requests[callback].1.contains("traceparent"));
    assert!(requests[trace]
        .1
        .to_ascii_lowercase()
        .contains("authorization: bearer ingest-token"));
    if hang {
        assert!(requests[callback].0.elapsed() < Duration::from_millis(4500));
    }
    server.abort();
}

#[tokio::test]
async fn test_hanging_ingest_preserves_exit_and_reports_completion_first() {
    run_flow(true, false).await;
}

#[tokio::test]
async fn test_exec_receives_rotated_environment_after_telemetry_flush() {
    run_flow(false, true).await;
}
