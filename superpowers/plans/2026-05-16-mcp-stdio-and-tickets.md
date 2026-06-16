# Operator MCP — Stdio Transport, Ticket Tools, and Status Integration

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Commit policy:** User handles all git commits manually. Where steps say "Commit", surface the diff to the user and let them run `git commit`. Do not commit automatically.

**Goal:** Add stdio transport for operator's MCP server, expand the tool surface to cover ticket queue read/write operations, advertise the stdio entrypoint through the existing `McpDescriptorResponse` so the vscode-extension can pick it up, and surface MCP lifecycle through operator's existing `StatusSection` pattern so users can toggle it and copy client configs from the dashboard.

**Architecture:** The HTTP/SSE MCP transport (`src/mcp/transport.rs`) already implements the protocol bound to `ApiState` and exposes seven read-only tools. `src/mcp/descriptor.rs` already publishes a discovery endpoint that the vscode-extension consumes. The structural move is to (1) extract the JSON-RPC dispatch core into a transport-agnostic module, (2) add a stdio transport that reads line-delimited JSON-RPC from stdin and writes to stdout, (3) add an `operator mcp` CLI subcommand as the entrypoint MCP clients launch, (4) expand the tool surface with ticket-queue operations that call into the existing `src/queue/Queue` (sync API, wrapped via `tokio::task::spawn_blocking`) and `TicketCreator`, (5) extend the existing descriptor with an optional `stdio: StdioCommand` field so IDE extensions can switch transports without a new endpoint, and (6) add a row to `ConnectionsSection` mirroring the `Operator API` lifecycle pattern. Stdio is the dominant MCP transport across Claude Code, Cursor, VS Code, Zed, and JetBrains; the existing HTTP transport stays as-is for network use.

**Tech Stack:** Rust 1.88+, tokio, serde_json, axum (existing for HTTP), clap (CLI), ratatui (status integration), ts-rs + schemars (for config + descriptor binding regeneration). No new top-level dependencies required — `tokio::io::{AsyncBufReadExt, AsyncWriteExt}` is sufficient for the stdio loop, and `EditFile` action + existing file I/O cover the client-config snippet delivery (no clipboard dependency).

---

## Pre-Flight (verify before starting)

Run each of these from the operator project root and confirm the output matches the assumption. If any differ, fix the relevant task before implementing.

1. `rg -n "pub fn claim_ticket|pub fn complete_ticket|pub fn list_queue" src/queue/`
   → should find sync methods in `src/queue/mod.rs` around lines 134-158.
2. `rg -n "pub struct McpDescriptorResponse" src/mcp/`
   → should find `src/mcp/descriptor.rs:14`. Confirms the descriptor already exists (do not re-create it).
3. `rg -n "mcp_sessions" src/rest/state.rs`
   → should find the field on `ApiState` typed `Arc<Mutex<HashMap<...>>>` where `Mutex` is `tokio::sync::Mutex` (line 7 imports).
4. `head -50 vscode-extension/src/mcp-connect.ts`
   → confirm the consumer reads `server_name`, `transport_url` from the descriptor. The descriptor extension in Task 6.5 must stay additive (Option field with `skip_serializing_if`).
5. `cargo build --release && ls target/release/operator`
   → confirms the binary path that `client_configs::current_exe()` will return.
6. `rg -n "pub struct TicketCreator|pub fn create_ticket_with_values" src/queue/creator.rs`
   → should find the existing creator at lines 16-68. Confirms the headless variant in Task 5.5 is additive.

---

## Critical Structural Approach

Three decisions lock the rest of the plan:

1. **Transport-agnostic handler.** The function `handle_jsonrpc(&JsonRpcRequest, &ApiState) -> JsonRpcResponse` in `src/mcp/transport.rs` already has the right shape but lives in a file named after HTTP. Extract it to `src/mcp/handler.rs` unchanged. Both transports import it.

2. **`ApiState` is the shared substrate; `Queue` is the ticket-write surface.** Existing tools call `routes::*` handlers, which take `State<ApiState>`. New ticket-queue tools should construct `crate::queue::Queue::new(&state.config)` and call its **sync** methods (`list_queue`, `claim_ticket(&Ticket)`, `complete_ticket(&Ticket)`, `return_to_queue(&Ticket)`) inside `tokio::task::spawn_blocking`. Creation uses `crate::queue::creator::TicketCreator` via a new headless variant (Task 5.5). Do **not** introduce an in-process HTTP roundtrip.

3. **No new server lifecycle for stdio.** Stdio MCP is spawned by the client (Claude Code, Cursor, VS Code, …) as a subprocess — it does not run inside the operator TUI. The HTTP-MCP toggle (`config.mcp.http_enabled`) is implemented by conditionally including the MCP routes in `build_router`; flipping it requires an API restart. There is **no** `McpStdioServer` struct, no shutdown channels for stdio. Status display in `ConnectionsSection` reflects (a) whether HTTP MCP routes are mounted on the current API server and (b) whether the stdio entrypoint is advertised in the descriptor.

---

## File Structure

**Create:**
- `src/mcp/handler.rs` — transport-agnostic `handle_jsonrpc` + JSON-RPC types
- `src/mcp/stdio.rs` — line-delimited stdio JSON-RPC loop
- `src/mcp/tickets.rs` — ticket-queue tools (separated from REST-wrapping `tools.rs`)
- `src/mcp/resources.rs` — MCP resources (tickets exposed as URIs)
- `src/mcp/client_configs.rs` — generates copy-paste config snippets for Claude Code, Claude Desktop, Cursor, VS Code, Zed
- `tests/mcp_stdio_integration.rs` — end-to-end test: spawn `operator mcp`, send init + tools/list over a pipe

**Modify:**
- `src/mcp/mod.rs` — add `handler`, `stdio`, `tickets`, `resources`, `client_configs` modules
- `src/mcp/transport.rs` — import `handle_jsonrpc` from `handler.rs`; delete the local copy
- `src/mcp/tools.rs` — merge ticket tools from `tickets.rs` into `all_tool_definitions` and `execute_tool`; update tool-count assertion
- `src/mcp/descriptor.rs` — extend existing `McpDescriptorResponse` with `stdio: Option<StdioCommand>`; inject `State<ApiState>` into the handler so it can read `config.mcp.stdio_advertised`
- `src/rest/mod.rs` — gate MCP route mounting on `config.mcp.http_enabled`
- `src/queue/creator.rs` — add `create_ticket_headless` (no editor launch)
- `src/main.rs` — add `Commands::Mcp` variant and `cmd_mcp` async fn
- `src/config.rs` — add `McpConfig` struct (fields: `http_enabled`, `stdio_advertised`, `expose_ticket_write_tools`) with `JsonSchema + TS` derives, and field on `Config`
- `src/ui/status_panel.rs` — add `mcp_http_status: McpHttpStatus` + `mcp_stdio_advertised: bool` + `mcp_active_sessions: usize` to `StatusSnapshot`; add new `StatusAction` variants: `ToggleMcpHttp`, `WriteAndOpenMcpClientConfig { client: String }`, `OpenMcpDocs`
- `src/ui/sections/connections_section.rs` — add an "MCP" row after the "Operator API" row
- `src/ui/dashboard.rs` — populate the new `StatusSnapshot` fields at the construction site
- `src/app/status_actions.rs` — handle the three new `StatusAction` variants

Session files live under `<tickets_path>/operator/` (see `src/rest/server.rs:27`). Generated client-config snippets go to `<tickets_path>/operator/mcp/<client>.json`.

---

## Tasks

### Task 1: Extract the JSON-RPC handler to a transport-agnostic module

**Files:**
- Create: `src/mcp/handler.rs`
- Modify: `src/mcp/transport.rs`
- Modify: `src/mcp/mod.rs:7-9`

- [ ] **Step 1: Move types and dispatch to `handler.rs`**

Create `src/mcp/handler.rs` with the contents below. These are the existing types from `transport.rs:24-51` plus the existing `handle_jsonrpc` fn from `transport.rs:136-233`, with `pub` added to `handle_jsonrpc` and `JsonRpcResponse`/`JsonRpcError` so other transports can use them.

```rust
//! Transport-agnostic JSON-RPC handler for MCP.
//!
//! Both the HTTP/SSE transport (`transport.rs`) and the stdio transport
//! (`stdio.rs`) dispatch through `handle_jsonrpc`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mcp::tools;
use crate::rest::state::ApiState;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

pub async fn handle_jsonrpc(request: &JsonRpcRequest, state: &ApiState) -> JsonRpcResponse {
    let id = request.id.clone().unwrap_or(Value::Null);
    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {}, "resources": { "subscribe": false, "listChanged": false } },
                "serverInfo": { "name": "operator", "version": env!("CARGO_PKG_VERSION") }
            })),
            error: None,
        },
        "notifications/initialized" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        },
        "tools/list" => {
            let tool_defs = tools::all_tool_definitions();
            let tools_json: Vec<Value> = tool_defs.into_iter().map(|t| json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })).collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "tools": tools_json })),
                error: None,
            }
        }
        "tools/call" => {
            let tool_name = request.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = request.params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match tools::execute_tool(tool_name, arguments, state).await {
                Ok(result) => {
                    let text = serde_json::to_string_pretty(&result).unwrap_or_default();
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({ "content": [{ "type": "text", "text": text }] })),
                        error: None,
                    }
                }
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError { code: -32000, message: e }),
                },
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    }
}
```

Note: the `initialize` capabilities object already advertises `resources` here so Task 6 doesn't need to re-edit it.

- [ ] **Step 2: Update `transport.rs` to import from `handler.rs`**

Replace lines 24-51 and the entire `handle_jsonrpc` function (lines 136-233) in `src/mcp/transport.rs` with:

```rust
use crate::mcp::handler::{handle_jsonrpc, JsonRpcRequest};
```

at the top, and update the `message_handler` call site (line 125) to use the imported `handle_jsonrpc`. The local `JsonRpcResponse` and `JsonRpcError` types are no longer needed in `transport.rs` — delete them.

- [ ] **Step 3: Wire the new module into `src/mcp/mod.rs`**

Edit `src/mcp/mod.rs` to add the new modules (some don't exist yet — comment them out until the corresponding task creates the file, or add them all and let `cargo check` fail until the files land):

```rust
//! Model Context Protocol (MCP) integration for Operator.

pub mod client_configs;
pub mod descriptor;
pub mod handler;
pub mod resources;
pub mod stdio;
pub mod tickets;
pub mod tools;
pub mod transport;
```

- [ ] **Step 4: Move the existing handler tests to `handler.rs`**

The six tests in `src/mcp/transport.rs:236-371` (`test_handle_initialize`, `test_handle_tools_list`, `test_handle_tools_call_health`, `test_handle_tools_call_unknown`, `test_handle_unknown_method`, `test_handle_notifications_initialized`) all test `handle_jsonrpc` directly. Move them verbatim to a `#[cfg(test)] mod tests { ... }` block in `src/mcp/handler.rs`. Update `test_handle_initialize` to also assert the `resources` capability is present.

- [ ] **Step 5: Verify**

Run: `cargo test mcp::handler`
Expected: All six tests PASS (with the updated capabilities assertion).

Run: `cargo test mcp::transport`
Expected: Compiles, no tests left in transport.rs.

Run: `cargo clippy -- -D warnings`
Expected: No warnings.

- [ ] **Step 6: Stop for user commit review**

This task is structurally complete (refactor only, plus the additive resources capability). Surface the diff to the user.

---

### Task 2: Add stdio transport

**Files:**
- Create: `src/mcp/stdio.rs`
- Test: inline `#[cfg(test)]` block

- [ ] **Step 1: Write a failing test for one round-trip**

In `src/mcp/stdio.rs`, write the function shell and a test that pipes a JSON-RPC request through it. The test uses a `Vec<u8>` for both input and output. Tests use `tempfile::TempDir` because `ApiState::new` initializes templates on disk.

```rust
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
        let json = serde_json::to_string(&response)
            .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialization failed"}}"#.to_string());
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
        // ApiState::new writes default templates into tickets_path; tempdir handles cleanup.
        ApiState::new(Config::default(), temp.path().to_path_buf())
    }

    #[tokio::test]
    async fn test_stdio_roundtrip_initialize() {
        let state = test_state();
        let input = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
"#;
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
        let input = b"not json\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n";
        let mut output: Vec<u8> = Vec::new();
        run(state, &input[..], &mut output).await.unwrap();
        let response_str = std::str::from_utf8(&output).unwrap();
        // Only one response should be present (the valid one)
        assert_eq!(response_str.matches('\n').count(), 1);
    }
}
```

Confirm `tempfile` is already a dev-dependency (it's used elsewhere in the project). If not, add `tempfile = "3"` under `[dev-dependencies]`.

- [ ] **Step 2: Run the tests**

Run: `cargo test mcp::stdio`
Expected: 3 tests PASS.

- [ ] **Step 3: Stop for commit review**

---

### Task 3: Add `operator mcp` CLI subcommand

**Files:**
- Modify: `src/main.rs` (add variant, match arm, async fn)

- [ ] **Step 1: Add the `Mcp` variant to the `Commands` enum**

Edit `src/main.rs`. Insert after the `Api { port: Option<u16> }` variant (around line 230):

```rust
    /// Run as an MCP stdio server (for use by Claude Code, Cursor, Zed, JetBrains, etc.).
    ///
    /// Reads line-delimited JSON-RPC from stdin and writes responses to stdout.
    /// Log output goes to stderr. Intended to be spawned by an MCP-capable client.
    Mcp,
```

- [ ] **Step 2: Add the match arm**

In the main `match cli.command` block (around `src/main.rs:281`), add a new arm before `Some(Commands::Setup { ... })`:

```rust
        Some(Commands::Mcp) => {
            cmd_mcp(&config).await?;
        }
```

- [ ] **Step 3: Implement `cmd_mcp`**

Add to the bottom of `src/main.rs`, alongside `cmd_api`:

```rust
async fn cmd_mcp(config: &Config) -> Result<()> {
    use crate::rest::state::ApiState;
    let state = ApiState::new(config.clone(), config.tickets_path());
    tracing::info!("Starting MCP stdio server");
    crate::mcp::stdio::run(state, tokio::io::stdin(), tokio::io::stdout()).await?;
    tracing::info!("MCP stdio server stopped (stdin closed)");
    Ok(())
}
```

- [ ] **Step 4: Verify it runs and responds**

Run: `cargo build --release`
Run interactively in a shell:
```
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/operator mcp
```
Expected: One line of JSON output containing `"serverInfo":{"name":"operator"`. Process exits cleanly after stdin closes.

- [ ] **Step 5: Stop for commit review**

---

### Task 4: Add ticket-queue MCP read tool (`operator_list_tickets`)

**Files:**
- Create: `src/mcp/tickets.rs`
- Modify: `src/mcp/tools.rs:23-99` (definitions), `:101-150` (dispatch), `:162` (count assertion)

- [ ] **Step 1: Write a failing test for `operator_list_tickets`**

In `src/mcp/tickets.rs`:

```rust
//! Ticket-queue MCP tools.
//!
//! Reads/writes via `crate::queue::Queue` which uses blocking `std::fs`,
//! so all calls are wrapped in `tokio::task::spawn_blocking`.

use serde_json::{json, Value};

use crate::queue::ticket::Ticket;
use crate::queue::Queue;
use crate::rest::state::ApiState;

fn ticket_to_json(t: &Ticket) -> Value {
    json!({
        "id": t.id,
        "filename": t.filename,
        "project": t.project,
        "ticket_type": t.ticket_type,
        "summary": t.summary,
        "priority": t.priority,
        "status": t.status,
        "branch": t.branch,
        "external_id": t.external_id,
        "external_url": t.external_url,
        "external_provider": t.external_provider,
    })
}

pub async fn list_tickets(args: Value, state: &ApiState) -> Result<Value, String> {
    let status = args.get("status").and_then(|v| v.as_str()).unwrap_or("queue").to_string();
    let config = (*state.config).clone();
    let tickets = tokio::task::spawn_blocking(move || -> Result<Vec<Ticket>, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        match status.as_str() {
            "queue" => queue.list_queue().map_err(|e| e.to_string()),
            "in-progress" => queue.list_in_progress().map_err(|e| e.to_string()),
            "completed" => queue.list_completed().map_err(|e| e.to_string()),
            other => Err(format!("Unknown ticket status: {other}")),
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let json_tickets: Vec<Value> = tickets.iter().map(ticket_to_json).collect();
    Ok(json!({ "tickets": json_tickets, "count": json_tickets.len() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_state() -> ApiState {
        let temp = tempfile::TempDir::new().unwrap();
        // Leak the tempdir so it survives the test; in real test isolation use a guard.
        let path = temp.into_path();
        ApiState::new(Config::default(), path)
    }

    #[tokio::test]
    async fn test_list_tickets_empty_queue() {
        let state = test_state();
        let result = list_tickets(json!({}), &state).await.unwrap();
        assert_eq!(result["count"], 0);
    }

    #[tokio::test]
    async fn test_list_tickets_unknown_status_errors() {
        let state = test_state();
        let err = list_tickets(json!({ "status": "bogus" }), &state).await.unwrap_err();
        assert!(err.contains("Unknown ticket status"));
    }
}
```

Verify the actual `Queue::new` signature first (Pre-Flight #1). If it takes a different argument (e.g. `&Config` vs. owned `Config`), adjust the closure capture. The `(*state.config).clone()` pattern handles the `Arc<Config>` deref.

Run: `cargo test mcp::tickets::tests::test_list_tickets_empty_queue`
Expected: PASS.

- [ ] **Step 2: Register the tool in `tools.rs`**

In `src/mcp/tools.rs:23-99`, append to the `vec!` in `all_tool_definitions`:

```rust
        McpToolDefinition {
            name: "operator_list_tickets".to_string(),
            description: "List tickets in the operator queue. Filter by status: queue, in-progress, completed. Returns id, project, type, summary, priority, branch, and external links — not body content.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string", "enum": ["queue", "in-progress", "completed"], "default": "queue" }
                },
                "required": []
            }),
        },
```

In `execute_tool` (around `src/mcp/tools.rs:103`), add the dispatch arm before the catch-all `_ =>`:

```rust
        "operator_list_tickets" => crate::mcp::tickets::list_tickets(args, state).await,
```

- [ ] **Step 3: Update the tool-count assertions**

In `src/mcp/handler.rs::tests::test_handle_tools_list` (moved in Task 1, Step 4) and `src/mcp/tools.rs:162`'s count assertion, change the expected count from `7` to `8`.

- [ ] **Step 4: Run everything**

Run: `cargo test mcp::`
Expected: All MCP tests PASS.

- [ ] **Step 5: Stop for commit review**

---

### Task 5: Add ticket-queue MCP write tools (claim, complete, return-to-queue)

**Files:**
- Modify: `src/mcp/tickets.rs` (three new fns + tests)
- Modify: `src/mcp/tools.rs` (three definitions, three dispatch arms, count assertion → 11)

All three write tools follow the same pattern: look up the ticket by `id` in the appropriate source list, call the corresponding `Queue` method on it, return the new path. They share a permission gate on `config.mcp.expose_ticket_write_tools` (added in Task 7).

- [ ] **Step 1: Add the shared lookup helper in `tickets.rs`**

```rust
async fn find_ticket(state: &ApiState, id: &str, in_status: &str) -> Result<Ticket, String> {
    let id = id.to_string();
    let in_status = in_status.to_string();
    let config = (*state.config).clone();
    tokio::task::spawn_blocking(move || -> Result<Ticket, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        let list = match in_status.as_str() {
            "queue" => queue.list_queue(),
            "in-progress" => queue.list_in_progress(),
            "completed" => queue.list_completed(),
            other => return Err(format!("Unknown status: {other}")),
        }
        .map_err(|e| e.to_string())?;
        list.into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Ticket {id} not found in {in_status}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
```

- [ ] **Step 2: Implement `claim_ticket`**

```rust
pub async fn claim_ticket(args: Value, state: &ApiState) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).ok_or("Missing required arg: id")?;
    let ticket = find_ticket(state, id, "queue").await?;
    let config = (*state.config).clone();
    let id_str = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        queue.claim_ticket(&ticket).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(json!({ "id": id_str, "moved_to": "in-progress" }))
}
```

Add test that creates a temp tickets dir, writes a fake ticket file into `queue/`, calls `claim_ticket`, then asserts the file exists in `in-progress/` and not in `queue/`. Use a real timestamped filename matching the project's expected pattern (see `src/queue/ticket.rs` for parse rules).

- [ ] **Step 3: Implement `complete_ticket` and `return_to_queue`**

Identical shape — source status is `"in-progress"` for both, target differs:

```rust
pub async fn complete_ticket(args: Value, state: &ApiState) -> Result<Value, String> { /* lookup in-progress, call queue.complete_ticket */ }
pub async fn return_to_queue(args: Value, state: &ApiState) -> Result<Value, String> { /* lookup in-progress, call queue.return_to_queue */ }
```

Add a test for each.

- [ ] **Step 4: Register the three tools in `tools.rs` with the permission gate**

Append three `McpToolDefinition` entries to `all_tool_definitions`:

```rust
        McpToolDefinition {
            name: "operator_claim_ticket".to_string(),
            description: "Move a ticket from queue to in-progress. Disabled unless [mcp].expose_ticket_write_tools = true.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": { "id": { "type": "string", "description": "Ticket id (e.g. FEAT-1234)" } },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_complete_ticket".to_string(),
            description: "Move a ticket from in-progress to completed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "operator_return_to_queue".to_string(),
            description: "Move a ticket from in-progress back to queue (un-claim).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }),
        },
```

In `execute_tool`, add three dispatch arms with the shared permission gate. Extract the gate to a helper:

```rust
fn require_write_tools(state: &ApiState) -> Result<(), String> {
    if !state.config.mcp.expose_ticket_write_tools {
        Err("Ticket write tools disabled in config ([mcp].expose_ticket_write_tools = true to enable)".to_string())
    } else {
        Ok(())
    }
}
```

```rust
        "operator_claim_ticket" => {
            require_write_tools(state)?;
            crate::mcp::tickets::claim_ticket(args, state).await
        }
        "operator_complete_ticket" => {
            require_write_tools(state)?;
            crate::mcp::tickets::complete_ticket(args, state).await
        }
        "operator_return_to_queue" => {
            require_write_tools(state)?;
            crate::mcp::tickets::return_to_queue(args, state).await
        }
```

- [ ] **Step 5: Add a gate test**

In `tickets.rs::tests`, assert that `claim_ticket` returns the gate error when `config.mcp.expose_ticket_write_tools = false`. Construct the state with a config where the flag is false, then call `execute_tool("operator_claim_ticket", ...)` and assert the error string.

- [ ] **Step 6: Update tool-count assertions to 11**

(8 existing + 3 new write tools = 11. Task 5.5 will add a 12th, Task 6 doesn't add tools.)

- [ ] **Step 7: Verify**

Run: `cargo test mcp::`
Expected: All MCP tests PASS.

- [ ] **Step 8: Stop for commit review**

---

### Task 5.5: Add `operator_create_ticket` write tool

**Files:**
- Modify: `src/queue/creator.rs` (add `create_ticket_headless`)
- Modify: `src/mcp/tickets.rs` (add `create_ticket` MCP fn)
- Modify: `src/mcp/tools.rs` (one definition, one dispatch arm, count → 12)

The existing `TicketCreator::create_ticket_with_values` (lines 33-68) opens `$EDITOR` after writing the file. That's wrong for MCP — there's no terminal. Add a headless variant that returns the path without launching an editor.

- [ ] **Step 1: Add `create_ticket_headless` to `TicketCreator`**

In `src/queue/creator.rs`, beside the existing `create_ticket_with_values`, add:

```rust
/// Create a ticket without opening it in an editor (for MCP / API use).
pub fn create_ticket_headless(
    &self,
    template_type: TemplateType,
    values: &HashMap<String, String>,
) -> Result<PathBuf> {
    let now = Utc::now();
    let timestamp = now.format("%Y%m%d-%H%M").to_string();
    let type_str = template_type.as_str();
    let project = values
        .get("project")
        .filter(|p| !p.is_empty())
        .cloned()
        .unwrap_or_else(|| "global".to_string());

    let filename = format!("{timestamp}-{type_str}-{project}-new-ticket.md");
    let filepath = self.queue_path.join(&filename);

    let template = template_type.template_content();
    let content = render_template(template, values)?;
    fs::create_dir_all(&self.queue_path).context("Failed to create queue directory")?;
    fs::write(&filepath, &content).context("Failed to write ticket file")?;

    Ok(filepath)
}
```

Refactor `create_ticket_with_values` to call `create_ticket_headless` and then `open_in_editor` (DRY). Run existing tests to confirm no regression.

- [ ] **Step 2: Add MCP fn `create_ticket` in `tickets.rs`**

```rust
pub async fn create_ticket(args: Value, state: &ApiState) -> Result<Value, String> {
    use crate::queue::creator::TicketCreator;
    use crate::templates::TemplateType;
    use std::collections::HashMap;

    let template_str = args.get("template").and_then(|v| v.as_str()).ok_or("Missing required arg: template")?;
    let template_type = TemplateType::from_str(template_str).map_err(|e| e.to_string())?;
    let mut values: HashMap<String, String> = HashMap::new();
    if let Some(obj) = args.get("values").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                values.insert(k.clone(), s.to_string());
            }
        }
    }

    let config = (*state.config).clone();
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf, String> {
        let creator = TicketCreator::new(&config);
        creator.create_ticket_headless(template_type, &values).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(json!({ "path": path.to_string_lossy(), "filename": path.file_name().and_then(|n| n.to_str()).unwrap_or("") }))
}
```

Verify `TemplateType::from_str` exists. If not, use the project's actual enum-parsing convention.

- [ ] **Step 3: Register `operator_create_ticket` in `tools.rs`**

```rust
        McpToolDefinition {
            name: "operator_create_ticket".to_string(),
            description: "Create a new ticket from a template (FEAT, FIX, INV, SPIKE, etc.) and write it to the queue. Returns the filename. Gated by [mcp].expose_ticket_write_tools.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "template": { "type": "string", "description": "Template type (FEAT, FIX, INV, SPIKE, ...)" },
                    "values": { "type": "object", "description": "Handlebars values for the template (project, summary, etc.)" }
                },
                "required": ["template"]
            }),
        },
```

```rust
        "operator_create_ticket" => {
            require_write_tools(state)?;
            crate::mcp::tickets::create_ticket(args, state).await
        }
```

- [ ] **Step 4: Add test**

Temp tickets dir, call `create_ticket` with `template = "FEAT", values = { "summary": "test", "project": "demo" }`, assert a `.md` file lands in `queue/` with a name containing `demo`.

- [ ] **Step 5: Update tool-count assertions to 12**

- [ ] **Step 6: Verify**

Run: `cargo test mcp::`
Expected: PASS.

- [ ] **Step 7: Stop for commit review**

---

### Task 6: MCP resources capability — expose tickets as resources

**Files:**
- Modify: `src/mcp/handler.rs` (add `resources/list` and `resources/read` handlers; the capability is already advertised after Task 1 Step 1)
- Create: `src/mcp/resources.rs`

MCP clients can subscribe to resources to read context. Expose each ticket as a resource with URI `operator://tickets/{status}/{id}`. This is the highest-leverage capability for IDEs that want to surface tickets natively.

- [ ] **Step 1: Add `resources/list` and `resources/read` handlers**

After the `tools/call` arm in `handle_jsonrpc`:

```rust
        "resources/list" => {
            let resources = crate::mcp::resources::list_resources(state).await
                .unwrap_or_else(|_| vec![]);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "resources": resources })),
                error: None,
            }
        }
        "resources/read" => {
            let uri = request.params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            match crate::mcp::resources::read_resource(uri, state).await {
                Ok(contents) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({ "contents": [{ "uri": uri, "mimeType": "text/markdown", "text": contents }] })),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError { code: -32000, message: e }),
                },
            }
        }
```

- [ ] **Step 2: Implement `src/mcp/resources.rs` via `Queue`**

```rust
//! MCP resources — exposes tickets as URI-addressable resources.

use serde_json::{json, Value};

use crate::queue::Queue;
use crate::rest::state::ApiState;

pub async fn list_resources(state: &ApiState) -> Result<Vec<Value>, String> {
    let config = (*state.config).clone();
    tokio::task::spawn_blocking(move || -> Result<Vec<Value>, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        let mut all = Vec::new();
        for (status, list) in [
            ("queue", queue.list_queue()),
            ("in-progress", queue.list_in_progress()),
            ("completed", queue.list_completed()),
        ] {
            for t in list.map_err(|e| e.to_string())? {
                all.push(json!({
                    "uri": format!("operator://tickets/{status}/{}", t.id),
                    "name": t.filename,
                    "mimeType": "text/markdown",
                    "description": t.summary,
                }));
            }
        }
        Ok(all)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn read_resource(uri: &str, state: &ApiState) -> Result<String, String> {
    let prefix = "operator://tickets/";
    let rest = uri.strip_prefix(prefix).ok_or_else(|| format!("Unknown URI scheme: {uri}"))?;
    let (status, id) = rest.split_once('/').ok_or_else(|| format!("Malformed URI: {uri}"))?;

    let config = (*state.config).clone();
    let status = status.to_string();
    let id = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        let list = match status.as_str() {
            "queue" => queue.list_queue(),
            "in-progress" => queue.list_in_progress(),
            "completed" => queue.list_completed(),
            other => return Err(format!("Unknown status: {other}")),
        }
        .map_err(|e| e.to_string())?;
        let ticket = list.into_iter().find(|t| t.id == id).ok_or_else(|| format!("Ticket {id} not found"))?;
        std::fs::read_to_string(&ticket.filepath).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_state() -> ApiState {
        let temp = tempfile::TempDir::new().unwrap();
        ApiState::new(Config::default(), temp.into_path())
    }

    #[tokio::test]
    async fn test_list_resources_empty() {
        let state = test_state();
        let resources = list_resources(&state).await.unwrap();
        assert!(resources.is_empty());
    }

    #[tokio::test]
    async fn test_read_resource_unknown_scheme() {
        let state = test_state();
        let err = read_resource("file:///tmp/x", &state).await.unwrap_err();
        assert!(err.contains("Unknown URI scheme"));
    }

    #[tokio::test]
    async fn test_read_resource_malformed() {
        let state = test_state();
        let err = read_resource("operator://tickets/queue", &state).await.unwrap_err();
        assert!(err.contains("Malformed URI"));
    }
}
```

- [ ] **Step 3: Verify**

Run: `cargo test mcp::`
Expected: All PASS.

- [ ] **Step 4: Stop for commit review**

---

### Task 6.5: Extend `McpDescriptorResponse` with stdio command (vscode-extension Phase 2 enabler)

**Files:**
- Modify: `src/mcp/descriptor.rs`

The existing descriptor at `src/mcp/descriptor.rs:14-30` is consumed by `vscode-extension/src/mcp-connect.ts` to register operator as an SSE MCP server. Extending it with an optional `stdio` field is purely additive (gated by `skip_serializing_if`) and unlocks the Phase 2 work where the extension can choose to spawn `operator mcp` instead of (or alongside) the SSE transport.

- [ ] **Step 1: Add `StdioCommand` and the optional field**

Edit `src/mcp/descriptor.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StdioCommand {
    /// Absolute path to the operator binary (the same binary serving this descriptor)
    pub command: String,
    /// Args to pass: typically ["mcp"]
    pub args: Vec<String>,
    /// Working directory the client should set when spawning. Defaults to the
    /// operator process's current working directory.
    pub cwd: String,
}

// Add to McpDescriptorResponse:
    /// Stdio transport entrypoint. Present when [mcp].stdio_advertised = true.
    /// Clients may spawn this as a subprocess instead of using transport_url.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdio: Option<StdioCommand>,
```

- [ ] **Step 2: Inject `State<ApiState>` into the handler and populate `stdio`**

Change the handler signature from `descriptor(Host(host): Host)` to `descriptor(State(state): State<ApiState>, Host(host): Host)` and populate:

```rust
pub async fn descriptor(
    State(state): State<ApiState>,
    Host(host): Host,
) -> Json<McpDescriptorResponse> {
    let base = format!("http://{host}");

    let stdio = if state.config.mcp.stdio_advertised {
        let command = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "operator".to_string());
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_default();
        Some(StdioCommand {
            command,
            args: vec!["mcp".to_string()],
            cwd,
        })
    } else {
        None
    };

    Json(McpDescriptorResponse {
        server_name: "operator".to_string(),
        server_id: "operator-mcp".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        transport_url: format!("{base}/api/v1/mcp/sse"),
        label: "Operator MCP Server".to_string(),
        openapi_url: Some(format!("{base}/api-docs/openapi.json")),
        stdio,
    })
}
```

The route registration in `src/rest/mod.rs` already passes `ApiState` (other handlers use it), but verify the descriptor's route line and add the `State` extractor if it currently uses a different shape.

- [ ] **Step 3: Update existing descriptor tests + add stdio coverage**

Update both `test_descriptor_response` and `test_descriptor_custom_port` to construct a test `ApiState` and pass it. Add two new tests:
- `test_descriptor_stdio_present_when_advertised` — config with `stdio_advertised = true` → `resp.stdio.is_some()` and `resp.stdio.unwrap().args == vec!["mcp"]`.
- `test_descriptor_stdio_absent_when_disabled` — config with `stdio_advertised = false` → `resp.stdio.is_none()`.

(Both tests need Task 7's `McpConfig` to exist. Either land Task 7 first, or temporarily inline the field default behind a feature flag. **Preferred order:** Task 7 before Task 6.5 — see ordering note below.)

- [ ] **Step 4: Regenerate TypeScript bindings**

Run: `cargo test` — `ts-rs` regenerates `bindings/` (or wherever the project configures `#[ts(export)]` output) including the new `StdioCommand` type.

- [ ] **Step 5: Verify**

Run: `cargo test mcp::descriptor`
Expected: PASS, including the two new stdio tests.

Run: `cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 6: Stop for commit review**

> **Ordering note:** This task reads `config.mcp.stdio_advertised`, which is defined in Task 7. If you're executing strictly in numeric order, swap: do Task 7 first, then return to Task 6.5. Tasks 1-6 are independent of `McpConfig`.

---

### Task 7: Add `[mcp]` config section

**Files:**
- Modify: `src/config.rs` (Config struct around lines 28-74; new `McpConfig` struct alongside `RestApiConfig` and `RelayConfig`)

- [ ] **Step 1: Add the `McpConfig` struct**

Add in `src/config.rs`, near other sub-structs like `RestApiConfig` (`src/config.rs:263-291`) and `RelayConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct McpConfig {
    /// Whether to mount MCP HTTP/SSE endpoints on the REST API server.
    /// Toggling requires an API restart (no hot-swap of the axum router).
    #[serde(default = "default_true")]
    pub http_enabled: bool,
    /// Whether the descriptor endpoint advertises the `operator mcp` stdio
    /// command. Set to false on multi-tenant/remote deployments where clients
    /// shouldn't spawn local subprocesses.
    #[serde(default = "default_true")]
    pub stdio_advertised: bool,
    /// Whether to expose ticket-mutating tools (claim, complete, return-to-queue,
    /// create) over MCP. Defaults to `false` because any MCP client can call them.
    #[serde(default)]
    pub expose_ticket_write_tools: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            http_enabled: true,
            stdio_advertised: true,
            expose_ticket_write_tools: false,
        }
    }
}

fn default_true() -> bool { true }
```

The `JsonSchema + TS` derive pair matches the rest of the codebase (`src/config.rs`'s other structs). The `TS` derive triggers TypeScript binding regeneration consumed by `vscode-extension/scripts/copy-types.js`.

- [ ] **Step 2: Add the field to `Config`**

Add to the `Config` struct (alongside `relay: RelayConfig`):

```rust
    #[serde(default)]
    pub mcp: McpConfig,
```

Update `Config::default()` to include `mcp: McpConfig::default()`.

- [ ] **Step 3: Wire `http_enabled` into the router build**

In `src/rest/mod.rs` (around lines 175-181, where MCP routes are currently mounted unconditionally), wrap the MCP route registrations:

```rust
if state.config.mcp.http_enabled {
    router = router
        .route("/api/v1/mcp/descriptor", get(descriptor::descriptor))
        .route("/api/v1/mcp/sse", get(transport::sse_handler))
        .route("/api/v1/mcp/message", post(transport::message_handler));
}
```

(Exact shape depends on the existing router-building pattern. Verify by reading `src/rest/mod.rs:175-181`.)

- [ ] **Step 4: Verify the gate from Task 5 now compiles**

Run: `cargo test mcp::`
Expected: PASS, including the disabled-write-tools gate test from Task 5.

- [ ] **Step 5: Regen docs and TS bindings**

```
cargo run -- docs --only config    # regenerates docs/configuration/index.md with [mcp] section
cargo test                          # ts-rs regenerates bindings
```

Then refresh the vscode-extension's copy (Phase 2 will need this):

```
cd vscode-extension && npm run copy-types
```

If `copy-types` isn't yet a script in `package.json`, fall back to the manual path the project uses (`scripts/copy-types.js`).

Verify the generated `docs/configuration/index.md` now describes `[mcp]`.

- [ ] **Step 6: Stop for commit review**

---

### Task 8: Client config snippet generator

**Files:**
- Create: `src/mcp/client_configs.rs`

Operator users adopting MCP need to be told "paste this into your client config." Generate snippets at runtime so they always carry the correct absolute path to the operator binary and the project's working directory.

- [ ] **Step 0: Verify modern client config shapes**

Before writing snippets, confirm the current expected shape for each:
- Run: `head -100 vscode-extension/src/mcp-connect.ts` — verify what the extension currently writes to workspace `mcp.servers` (or the modern `.vscode/mcp.json` `servers` shape). The snippet for VS Code must match what the extension expects.
- Cursor: `~/.cursor/mcp.json` uses the `mcpServers` shape (same as Claude Code's `claude.json`).
- Claude Desktop: `~/Library/Application Support/Claude/claude_desktop_config.json` uses `mcpServers`.
- Zed: `settings.json` under `context_servers`.

If any shape has drifted, update the snippet in Step 1 before testing.

- [ ] **Step 1: Implement `client_configs.rs`**

```rust
//! Generates copy-paste MCP client configuration snippets pointing at this operator binary.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub fn current_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("operator"))
}

fn mcp_servers_shape(cwd: &Path) -> Value {
    // Used by Claude Code (~/.claude.json), Claude Desktop, and Cursor (~/.cursor/mcp.json).
    json!({
        "mcpServers": {
            "operator": {
                "command": current_exe().to_string_lossy(),
                "args": ["mcp"],
                "cwd": cwd.to_string_lossy(),
            }
        }
    })
}

pub fn claude_code_snippet(cwd: &Path) -> Value { mcp_servers_shape(cwd) }
pub fn claude_desktop_snippet(cwd: &Path) -> Value { mcp_servers_shape(cwd) }

/// Cursor's `~/.cursor/mcp.json` uses the same `mcpServers` shape as Claude Code.
pub fn cursor_snippet(cwd: &Path) -> Value { mcp_servers_shape(cwd) }

/// VS Code (1.94+) per-workspace `.vscode/mcp.json` uses a `servers` block with explicit `type`.
pub fn vscode_snippet(cwd: &Path) -> Value {
    json!({
        "servers": {
            "operator": {
                "type": "stdio",
                "command": current_exe().to_string_lossy(),
                "args": ["mcp"],
                "cwd": cwd.to_string_lossy(),
            }
        }
    })
}

/// Zed config under `context_servers` in user settings.
pub fn zed_snippet(cwd: &Path) -> Value {
    json!({
        "context_servers": {
            "operator": {
                "command": { "path": current_exe().to_string_lossy(), "args": ["mcp"], "env": {} },
                "settings": { "cwd": cwd.to_string_lossy() }
            }
        }
    })
}

/// Dispatch by client name. Returns `None` for unknown clients.
pub fn snippet_for(client: &str, cwd: &Path) -> Option<Value> {
    match client {
        "claude-code" => Some(claude_code_snippet(cwd)),
        "claude-desktop" => Some(claude_desktop_snippet(cwd)),
        "cursor" => Some(cursor_snippet(cwd)),
        "vscode" => Some(vscode_snippet(cwd)),
        "zed" => Some(zed_snippet(cwd)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_code_snippet_shape() {
        let cfg = claude_code_snippet(&PathBuf::from("/work"));
        assert_eq!(cfg["mcpServers"]["operator"]["args"][0], "mcp");
        assert_eq!(cfg["mcpServers"]["operator"]["cwd"], "/work");
    }

    #[test]
    fn test_cursor_snippet_matches_claude_code() {
        let cursor = cursor_snippet(&PathBuf::from("/work"));
        let claude = claude_code_snippet(&PathBuf::from("/work"));
        assert_eq!(cursor, claude);
    }

    #[test]
    fn test_vscode_snippet_uses_servers_with_type() {
        let cfg = vscode_snippet(&PathBuf::from("/work"));
        assert_eq!(cfg["servers"]["operator"]["type"], "stdio");
        assert_eq!(cfg["servers"]["operator"]["args"][0], "mcp");
    }

    #[test]
    fn test_zed_snippet_uses_context_servers() {
        let cfg = zed_snippet(&PathBuf::from("/work"));
        assert!(cfg["context_servers"]["operator"]["command"]["path"].is_string());
    }

    #[test]
    fn test_snippet_for_unknown_client_is_none() {
        assert!(snippet_for("notepad++", &PathBuf::from("/w")).is_none());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test mcp::client_configs`
Expected: PASS.

- [ ] **Step 3: Stop for commit review**

---

### Task 9: Integrate MCP into `StatusSnapshot` and `ConnectionsSection`

**Files:**
- Modify: `src/ui/status_panel.rs:401-431` (StatusSnapshot fields), `:143-174` (StatusAction)
- Modify: `src/ui/sections/connections_section.rs:70-138` (children)
- Modify: `src/ui/dashboard.rs:203-317` (snapshot construction)
- Modify: `src/app/status_actions.rs` (action handlers)

Mirror the existing `Operator API` row pattern exactly. No new clipboard dependency — `WriteAndOpenMcpClientConfig` writes the snippet to a file and dispatches the existing `EditFile(path)` action.

- [ ] **Step 1: Add `McpHttpStatus` enum**

In `src/rest/server.rs` (alongside `RestApiStatus`) or a new `src/mcp/status.rs` if you prefer to keep MCP types together:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum McpHttpStatus {
    /// MCP HTTP routes mounted on the REST API server on the given port.
    Mounted { port: u16 },
    /// MCP HTTP routes disabled via [mcp].http_enabled = false.
    NotMounted,
}
```

- [ ] **Step 2: Add MCP fields to `StatusSnapshot`**

In `src/ui/status_panel.rs` around line 425, before the closing `}`:

```rust
    /// MCP HTTP transport status (mounted on the API server, or disabled by config).
    pub mcp_http_status: McpHttpStatus,
    /// Whether the descriptor advertises the stdio entrypoint.
    pub mcp_stdio_advertised: bool,
    /// Currently active MCP SSE sessions on the HTTP transport.
    pub mcp_active_sessions: usize,
```

- [ ] **Step 3: Add new `StatusAction` variants**

In `src/ui/status_panel.rs:143-174`, add before `None`:

```rust
    /// Toggle [mcp].http_enabled (requires API restart to take effect).
    ToggleMcpHttp,
    /// Generate a client config snippet, write it to <tickets>/operator/mcp/<client>.json,
    /// and open it in $EDITOR. `client` is one of: "claude-code", "claude-desktop", "cursor", "vscode", "zed".
    WriteAndOpenMcpClientConfig { client: String },
    /// Open the operator MCP docs page in the default browser.
    OpenMcpDocs,
```

- [ ] **Step 4: Add the "MCP" row in `ConnectionsSection::children`**

In `src/ui/sections/connections_section.rs` after the "Operator API" row push (around line 117), insert:

```rust
        rows.push(TreeRow {
            section_id: SectionId::Connections,
            depth: 1,
            label: "MCP".into(),
            description: match (&snapshot.mcp_http_status, snapshot.mcp_stdio_advertised, snapshot.mcp_active_sessions) {
                (McpHttpStatus::Mounted { port }, true, n) if n > 0 => format!(":{port} + stdio · {n} sessions"),
                (McpHttpStatus::Mounted { port }, true, _) => format!(":{port} + stdio"),
                (McpHttpStatus::Mounted { port }, false, _) => format!(":{port} (HTTP only)"),
                (McpHttpStatus::NotMounted, true, _) => "stdio only".into(),
                (McpHttpStatus::NotMounted, false, _) => "Disabled".into(),
            },
            icon: match (&snapshot.mcp_http_status, snapshot.mcp_stdio_advertised) {
                (McpHttpStatus::Mounted { .. }, _) | (_, true) => StatusIcon::Plug,
                _ => StatusIcon::Cross,
            },
            is_header: false,
            actions: ActionSet {
                primary: StatusAction::WriteAndOpenMcpClientConfig { client: "claude-code".to_string() },
                back: StatusAction::None,
                special: StatusAction::ToggleMcpHttp,
                special_meta: Some(ActionMeta { title: "HTTP", tooltip: "Toggle the MCP HTTP transport (restart required)" }),
                refresh: StatusAction::OpenMcpDocs,
                refresh_meta: Some(ActionMeta { title: "Docs", tooltip: "Open MCP setup docs in browser" }),
            },
            health: SectionHealth::Gray,
        });
```

`McpHttpStatus` and `StatusIcon::Plug` need imports added at the top of the file.

- [ ] **Step 5: Populate the snapshot in `dashboard.rs`**

In `src/ui/dashboard.rs:203-317`'s `build_status_snapshot`, populate the three new fields:

```rust
    mcp_http_status: if self.config.mcp.http_enabled {
        match &self.rest_api_status {
            RestApiStatus::Running { port } => McpHttpStatus::Mounted { port: *port },
            _ => McpHttpStatus::NotMounted,
        }
    } else {
        McpHttpStatus::NotMounted
    },
    mcp_stdio_advertised: self.config.mcp.stdio_advertised,
    mcp_active_sessions: self.api_state.as_ref()
        .map(|s| s.mcp_sessions.try_lock().map(|m| m.len()).unwrap_or(0))
        .unwrap_or(0),
```

Verify the `Dashboard` struct's field that holds `ApiState` (might not be `api_state`; grep for `ApiState` in `src/ui/dashboard.rs`). If the dashboard doesn't currently hold an `ApiState` reference, route the session count through the `RestApiServer` lifecycle handle (which already holds the state) or default to `0` until the API is running.

- [ ] **Step 6: Wire the action handlers in `src/app/status_actions.rs`**

Add three new match arms in the existing dispatcher (around `src/app/status_actions.rs:66`):

```rust
StatusAction::ToggleMcpHttp => {
    // Flip config.mcp.http_enabled in the running Config and surface a notice.
    // No hot-swap: tell the user to restart the API.
    self.config.mcp.http_enabled = !self.config.mcp.http_enabled;
    self.dashboard.set_status(if self.config.mcp.http_enabled {
        "MCP HTTP enabled — restart the API to mount routes"
    } else {
        "MCP HTTP disabled — restart the API to unmount routes"
    });
}

StatusAction::WriteAndOpenMcpClientConfig { client } => {
    use crate::mcp::client_configs;
    let cwd = std::env::current_dir().unwrap_or_default();
    let Some(snippet) = client_configs::snippet_for(&client, &cwd) else {
        self.dashboard.set_status(&format!("Unknown MCP client: {client}"));
        return;
    };
    let dir = self.config.tickets_path().join("operator/mcp");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        self.dashboard.set_status(&format!("Failed to create {}: {e}", dir.display()));
        return;
    }
    let path = dir.join(format!("{client}.json"));
    let body = serde_json::to_string_pretty(&snippet).unwrap_or_default();
    if let Err(e) = std::fs::write(&path, body) {
        self.dashboard.set_status(&format!("Failed to write {}: {e}", path.display()));
        return;
    }
    // Reuse the existing EditFile dispatcher.
    self.dispatch(StatusAction::EditFile(path.to_string_lossy().into_owned()));
}

StatusAction::OpenMcpDocs => {
    // Use the existing open_in_browser helper.
    if let Err(e) = open_in_browser("https://operator.untra.io/mcp/") {
        self.dashboard.set_status(&format!("Failed to open docs: {e}"));
    }
}
```

Confirm the docs URL before merging (TODO marker; pick the actual operator docs URL).

- [ ] **Step 7: Update all test snapshots**

Search test files for `StatusSnapshot {` (`rg "StatusSnapshot \{" --type rust`) and add the three new fields with defaults:

```rust
    mcp_http_status: McpHttpStatus::Mounted { port: 7008 },
    mcp_stdio_advertised: true,
    mcp_active_sessions: 0,
```

- [ ] **Step 8: Add a test for the new MCP row**

Append to `src/ui/sections/connections_section.rs::tests`:

```rust
    #[test]
    fn test_connections_mcp_row_present() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        let mcp_row = children.iter().find(|r| r.label == "MCP");
        assert!(mcp_row.is_some(), "MCP row should always be present");
    }

    #[test]
    fn test_connections_mcp_row_description_disabled() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.mcp_http_status = McpHttpStatus::NotMounted;
        snap.mcp_stdio_advertised = false;
        let children = section.children(&snap);
        let mcp_row = children.iter().find(|r| r.label == "MCP").unwrap();
        assert_eq!(mcp_row.description, "Disabled");
    }
```

- [ ] **Step 9: Verify**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test`
Expected: All PASS, no warnings.

- [ ] **Step 10: Stop for commit review**

---

### Task 10: End-to-end integration test

**Files:**
- Create: `tests/mcp_stdio_integration.rs`

- [ ] **Step 1: Write the test**

```rust
//! End-to-end test: spawn `operator mcp` as a subprocess and roundtrip
//! a real JSON-RPC handshake.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[tokio::test]
async fn test_operator_mcp_stdio_initialize_and_list_tools() {
    let exe = env!("CARGO_BIN_EXE_operator");
    let mut child = Command::new(exe)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn operator mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    stdin.write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n").await.unwrap();
    stdin.write_all(b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n").await.unwrap();
    stdin.flush().await.unwrap();

    let line1 = tokio::time::timeout(std::time::Duration::from_secs(5), reader.next_line()).await.unwrap().unwrap().unwrap();
    let resp1: serde_json::Value = serde_json::from_str(&line1).unwrap();
    assert_eq!(resp1["id"], 1);
    assert_eq!(resp1["result"]["serverInfo"]["name"], "operator");

    let line2 = tokio::time::timeout(std::time::Duration::from_secs(5), reader.next_line()).await.unwrap().unwrap().unwrap();
    let resp2: serde_json::Value = serde_json::from_str(&line2).unwrap();
    assert_eq!(resp2["id"], 2);
    // 8 read + 4 write tools = 12 (or whichever count Task 5.5 left it at)
    assert!(resp2["result"]["tools"].as_array().unwrap().len() >= 8);

    drop(stdin);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;
}
```

- [ ] **Step 2: Run**

Run: `cargo test --test mcp_stdio_integration -- --nocapture`
Expected: PASS (a few seconds to build the binary the first time).

- [ ] **Step 3: Final verification**

```
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- docs --only config       # confirm [mcp] appears in regenerated docs
cd vscode-extension && npm run copy-types && cd ..  # confirm new TS types regenerated
```

Expected: green across the board, generated docs and TS bindings updated.

- [ ] **Step 4: Stop for user to commit**

---

## Integration Handoff (sets up Phase 2 + Phase 3)

This plan does **not** modify vscode-extension or write a Cursor integration. It sets the stage so the next two plans can be written and executed independently.

**Phase 2 — vscode-extension refinement (separate follow-up plan):**
- The existing `vscode-extension/src/mcp-connect.ts:34-97` already consumes `/api/v1/mcp/descriptor`. After Task 6.5, the response carries an optional `stdio: StdioCommand` field.
- Phase 2 will modify `mcp-connect.ts` to: (a) detect the new field, (b) offer a workspace setting `operator.mcpTransport: "sse" | "stdio" | "auto"`, (c) when stdio is chosen/auto-selected, register operator via VS Code's modern MCP API as a stdio server using `descriptor.stdio.command` + `descriptor.stdio.args` + `descriptor.stdio.cwd`. Fallback remains SSE.
- The TypeScript binding for `StdioCommand` will be available via the existing `scripts/copy-types.js` flow once Task 7 Step 5 runs.

**Phase 3 — Cursor integration (separate follow-up plan):**
- Cursor has no extension; it consumes `~/.cursor/mcp.json` directly. Task 8's `cursor_snippet()` already produces the right shape.
- Phase 3 will add either: (a) a `operator mcp install --client cursor` CLI subcommand that writes the snippet to `~/.cursor/mcp.json` (merging with existing servers), or (b) a docs page rendering the snippet with the current binary path, or (c) both. The dashboard's `WriteAndOpenMcpClientConfig { client: "cursor" }` action (Task 9) already covers the local-workspace path.
- Phase 3 should also document the JetBrains/Claude Desktop install flow using the same `snippet_for(client, cwd)` dispatch since those clients use the same `mcpServers` shape.

---

## Self-Review

**Spec coverage:**
- Stdio transport — Tasks 2, 3
- Expanded tool surface (read + 4 write tools) — Tasks 4, 5, 5.5
- Resources capability — Task 6
- Descriptor stdio handoff for vscode-extension — Task 6.5
- Config section — Task 7
- Client config snippets — Task 8
- Status integration (toggle + write-and-open snippet + docs link) — Task 9
- End-to-end test — Task 10
- Phase 2/3 handoff — Integration Handoff section

**Open assumptions verified in Pre-Flight:**
1. ✓ `Queue` API at `src/queue/mod.rs:134-158` (sync methods).
2. ✓ `McpDescriptorResponse` exists at `src/mcp/descriptor.rs:14`.
3. ✓ `mcp_sessions: Arc<tokio::sync::Mutex<...>>` — `.await` correct.
4. ⚠ Docs URL for `OpenMcpDocs` — placeholder used (`https://operator.untra.io/mcp/`); confirm before merging.
5. ⚠ VS Code MCP shape — verify against current extension behaviour in Task 8 Step 0.
6. ⚠ Dashboard's holding of `ApiState` — Task 9 Step 5 grep verifies; fallback to 0 sessions if not available.

**Tradeoffs locked in:**
- HTTP and stdio MCP share the handler core. HTTP behavior is unchanged unless `config.mcp.http_enabled = false` (then routes are not mounted at startup; toggle requires restart).
- Ticket write tools are off by default. Users opt in via `[mcp].expose_ticket_write_tools = true`.
- The descriptor extension is additive (`Option`, `skip_serializing_if`) so existing vscode-extension code keeps working until Phase 2 chooses to use the new field.
- Snippet delivery is "write to file + open in editor," reusing `EditFile` — no clipboard dependency.
- Stdio resource subscription is `listChanged: false` — clients re-list rather than subscribe. Simpler; revisit if a real client demands push.
