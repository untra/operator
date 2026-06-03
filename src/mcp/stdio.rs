//! Stdio transport for MCP — line-delimited JSON-RPC over stdin/stdout.
//!
//! Each line on stdin is one JSON-RPC request. Each response is one JSON
//! object written to stdout terminated by `\n`. Logs and diagnostics go to
//! stderr (via `tracing`). This is the transport MCP clients use when they
//! spawn `operator mcp` as a subprocess.

use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::mcp::handler::{handle_jsonrpc, JsonRpcRequest};
use crate::rest::state::ApiState;

/// Run the stdio MCP loop until stdin closes.
///
/// `reader`/`writer` are generic for testability; production callers pass
/// `tokio::io::stdin()` and `tokio::io::stdout()`.
pub async fn run<R, W>(state: ApiState, reader: R, mut writer: W) -> io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, line = %line, "Malformed JSON-RPC request");
                continue;
            }
        };
        let response = handle_jsonrpc(&request, &state).await;
        let json = serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialization failed"}}"#
                .to_string()
        });
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_state() -> ApiState {
        let temp = tempfile::TempDir::new().unwrap();
        // Leak the tempdir so the path survives for the duration of the test;
        // ApiState may keep references to files inside it.
        let path = temp.keep();
        ApiState::new(Config::default(), path)
    }

    #[tokio::test]
    async fn test_stdio_roundtrip_initialize() {
        let state = test_state();
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n";
        let mut output: Vec<u8> = Vec::new();
        run(state, &input[..], &mut output).await.unwrap();

        let response_str = std::str::from_utf8(&output).unwrap();
        let response: serde_json::Value = serde_json::from_str(response_str.trim()).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert_eq!(response["result"]["serverInfo"]["name"], "operator");
    }

    #[tokio::test]
    async fn test_stdio_ignores_blank_lines() {
        let state = test_state();
        let input = b"\n\n";
        let mut output: Vec<u8> = Vec::new();
        run(state, &input[..], &mut output).await.unwrap();
        assert!(output.is_empty());
    }

    #[tokio::test]
    async fn test_stdio_malformed_line_is_skipped() {
        let state = test_state();
        let input =
            b"not json\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n";
        let mut output: Vec<u8> = Vec::new();
        run(state, &input[..], &mut output).await.unwrap();
        let response_str = std::str::from_utf8(&output).unwrap();
        assert_eq!(response_str.matches('\n').count(), 1);
    }
}
