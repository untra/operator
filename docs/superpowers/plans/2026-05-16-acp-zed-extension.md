# Plan: ACP Integration for the Operator Zed Extension

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Commit policy:** User handles all git commits manually. Where steps say "Commit", surface the diff to the user and let them run `git commit`. Do not commit automatically.

## Context

A sibling plan, `docs/superpowers/plans/2026-05-16-acp-agent.md`, wires the **operator binary** itself as an ACP agent (`operator acp` over stdio). This follow-up plan picks up where that one ends: integrate ACP into the **`zed-extension/`** package so Zed users can launch Operator from the agent panel — not just via slash commands.

Today the Zed extension (`zed-extension/src/lib.rs`, `extension.toml`) is a WASM-sandboxed slash-command bridge: 11 `/op-*` commands shell out to `curl` against the local REST API (`http://localhost:7008`). It does not register an ACP agent. The Zed agent panel surface is unused.

This plan adds ACP-agent registration to the existing extension so:
- Zed's agent panel shows **Operator** as a selectable agent alongside Claude / Codex / Gemini CLI
- Selecting Operator and opening a new thread spawns `operator acp` in the project root, wired to Zed via JSON-RPC stdio
- The existing `/op-*` slash commands stay — they cover different needs (status queries, queue inspection) and complement the agent thread

Workflow this enables: a developer in a Zed window for project X opens the agent panel, picks Operator, and chats. Operator (per the upstream ACP plan) creates a ticket from that chat, picks the next queued ticket if one matches, and delegates to Claude Code / Codex / Gemini under the hood — streaming the delegator's output back as `session/update` notifications visible inside Zed.

## Hard Dependency

This plan **assumes the operator-side ACP plan is complete and merged.** Specifically:
- `operator acp` subcommand exists and serves a working `Agent` impl over stdio
- `initialize`, `session/new`, and `session/prompt` roundtrip cleanly
- `tests/acp_integration.rs` is green

If `operator acp` doesn't exist yet, **execute the upstream plan first.** This plan adds Zed-side packaging on top.

## How Zed Discovers ACP Agents (Key Facts)

From `https://zed.dev/docs/extensions/agent-servers` and the user-config docs:

1. **Manifest registration:** A Zed extension declares ACP agents via `[agent_servers.<id>]` blocks in `extension.toml`. Each block has `name`, optional `icon`, optional `env`, plus per-platform `targets.<os>-<arch>` entries with `archive` (download URL), `cmd`, `args`, and recommended `sha256`.
2. **User override:** Users can override extension-provided agents (or add custom ones) under `agent_servers` in `settings.json`. The custom form is `{"type": "custom", "command": "...", "args": [...], "env": {...}}`. The registry form is `{"type": "registry", ...}` for curated entries.
3. **Lifecycle:** Zed spawns the configured command as a subprocess with `cwd` = the project root and pipes JSON-RPC over its stdio. No WASM API call is required from the extension code (`src/lib.rs`).
4. **Forwarded context:** Zed passes the project root as `cwd` in `session/new`, plus MCP server configurations, and forwards model/mode selection if the agent advertises support.

Consequence: the **majority of this plan is `extension.toml` + docs + a release pipeline** — `src/lib.rs` does not need ACP code, because ACP runs in the operator binary, not in the WASM sandbox.

## Critical Files

**Modify:**
- `zed-extension/extension.toml` — add `[agent_servers.operator]` block with platform targets
- `zed-extension/README.md` — document the agent panel flow alongside slash commands
- `zed-extension/TODO.md` — mark ACP agent panel as ✅ implemented; audit "not possible" entries against what ACP unblocks
- `bump-version.sh` — bump the extension's version when shipping
- `.github/workflows/*.yml` (or equivalent) — publish per-platform operator archives whose URLs are referenced from `extension.toml`

**Create:**
- `zed-extension/docs/acp-setup.md` — short walkthrough: install the extension, configure `agent_servers` in `settings.json` for dev, or use the bundled archive in release mode
- `zed-extension/tests/acp_smoke.sh` (or a CI step) — end-to-end smoke that builds operator, starts it under `operator acp`, sends `initialize`, asserts the JSON response
- `src/integrations/inventory.rs` (operator crate) — single source-of-truth list of operator capabilities exposed across surfaces
- `tests/surface_parity.rs` (operator crate) — enforces that every capability has both a slash-command and an ACP-tool entry point

**Do NOT modify:**
- `zed-extension/src/lib.rs` — slash commands stay as-is. ACP runs out-of-process in the operator binary, not in the extension WASM.

## Tasks

### Task 1: Confirm operator-side ACP is functional

- [ ] **Step 1:** Run `cargo run -- acp < /tmp/init.json` from operator root with a hand-rolled JSON-RPC `initialize` request. Assert it produces a valid `InitializeResponse` containing `agentCapabilities` with `loadSession: false` (per upstream plan v1).
- [ ] **Step 2:** Run `cargo test --test acp_integration` and confirm green.
- [ ] **Step 3:** If either fails, **stop** — the upstream plan is the blocker; finish that first.

---

### Task 2: Add `[agent_servers.operator]` to `extension.toml`

**Files:**
- Modify: `zed-extension/extension.toml`

- [ ] **Step 1:** Add the agent_servers block after `[slash_commands]`:

```toml
[agent_servers.operator]
name = "Operator"
icon = "https://operator.untra.io/icon.png"

[agent_servers.operator.targets.darwin-aarch64]
archive = "https://github.com/untra/operator/releases/download/v{VERSION}/operator-darwin-aarch64.tar.gz"
cmd = "./operator"
args = ["acp"]
sha256 = "{SHA256}"

[agent_servers.operator.targets.darwin-x86_64]
archive = "https://github.com/untra/operator/releases/download/v{VERSION}/operator-darwin-x86_64.tar.gz"
cmd = "./operator"
args = ["acp"]
sha256 = "{SHA256}"

[agent_servers.operator.targets.linux-x86_64]
archive = "https://github.com/untra/operator/releases/download/v{VERSION}/operator-linux-x86_64.tar.gz"
cmd = "./operator"
args = ["acp"]
sha256 = "{SHA256}"

[agent_servers.operator.targets.linux-aarch64]
archive = "https://github.com/untra/operator/releases/download/v{VERSION}/operator-linux-aarch64.tar.gz"
cmd = "./operator"
args = ["acp"]
sha256 = "{SHA256}"
```

- [ ] **Step 2:** Pin `{VERSION}` to the operator version that first contains `operator acp`. Bake `{SHA256}` per target at release time via `bump-version.sh` (or accept manual updates in Task 3).
- [ ] **Step 3:** Confirm `extension.toml` parses by building the extension:
  ```bash
  cd zed-extension && cargo build --release --target wasm32-wasip1
  ```
- [ ] **Step 4:** Stop for commit review.

---

### Task 3: Update the release pipeline to ship operator archives

The `archive` URLs in Task 2 must resolve to real artifacts. Inventory `.github/workflows/` and `bump-version.sh` first to see what exists; add the missing pieces.

**Files:**
- Modify: `.github/workflows/*.yml` (release workflow)
- Modify: `bump-version.sh`

- [ ] **Step 1:** Add a CI step that, on a tagged release, produces `operator-{os}-{arch}.tar.gz` for the four target tuples in Task 2. Each archive contains the `operator` binary at the archive root (so `cmd = "./operator"` resolves).
- [ ] **Step 2:** Add a CI step that computes each archive's `sha256` and rewrites `zed-extension/extension.toml` with the real `{SHA256}` and `{VERSION}` values before publishing the extension.
- [ ] **Step 3:** Verify by tagging a pre-release and downloading one archive locally:
  ```bash
  tar -tzf operator-darwin-aarch64.tar.gz | head
  ```
  Expected: `operator` appears at the top level.
- [ ] **Step 4:** Stop for commit review.

---

### Task 4: Document the dev-mode override

Most operator developers will not consume the archive — they'll point Zed at their local debug build. Document this clearly so the extension is usable before the release pipeline is finished.

**Files:**
- Create: `zed-extension/docs/acp-setup.md`

- [ ] **Step 1:** Write the setup doc, including:

  ````markdown
  # Operator ACP Setup

  ## Dev mode (local binary)

  Add to `~/.config/zed/settings.json` (or per-project `.zed/settings.json`):

  ```jsonc
  {
    "agent_servers": {
      "operator": {
        "type": "custom",
        "command": "/Users/you/Documents/gbqr-us/operator/target/debug/operator",
        "args": ["acp"],
        "env": {
          "RUST_LOG": "operator=debug"
        }
      }
    }
  }
  ```

  This override takes precedence over the extension-provided `[agent_servers.operator]` block, so you can run an unreleased build of operator without rebuilding the extension.

  ## Verify it works

  1. Open Zed's agent panel
  2. Pick **Operator** from the agent selector
  3. Open a new thread
  4. Type `hello` — you should see streamed output

  ## Release mode

  Install the extension from the Zed extension registry. Zed fetches the matching `operator-{os}-{arch}.tar.gz` archive automatically; no `settings.json` changes needed.

  ## Known issues

  (Populated as Task 7 surfaces them.)
  ````

- [ ] **Step 2:** Cross-link from `zed-extension/README.md` and the operator-side `docs/cli/index.md` ACP section.
- [ ] **Step 3:** Stop for commit review.

---

### Task 5: Rewrite README + TODO to reflect dual-surface

`zed-extension/README.md` and `zed-extension/TODO.md` currently document only the slash-command surface and list many features as "Not Possible in Zed." ACP unblocks several. Update them honestly.

**Files:**
- Modify: `zed-extension/README.md`
- Modify: `zed-extension/TODO.md`

- [ ] **Step 1:** In `README.md`, add a top-level "Two ways to use the extension" section:
  1. **Slash commands** — existing, REST-backed status queries surfaced in the AI assistant
  2. **Agent panel** — new, ACP-backed full sessions inside Zed
- [ ] **Step 2:** State explicitly that the two surfaces are intentionally parallel — they cover the same operator concepts (queue, tickets, agents, kanban) but from different entry points. Task 6 enforces this with tests.
- [ ] **Step 3:** In `TODO.md`, audit each "Not Possible in Zed" row honestly:
  - Sidebar Views → still N/A (ACP doesn't help here)
  - Webhook Server → still N/A
  - **Terminal Management** → N/A in extension, but ACP sessions provide an in-IDE chat surface
  - **Status Bar** → still N/A
  - **File System Watching** → still N/A in WASM, but the ACP path lets the agent see filesystem state via `fs/read_text_file` requests routed to Zed
- [ ] **Step 4:** Be specific about what ACP does and doesn't add. Don't oversell.
- [ ] **Step 5:** Stop for commit review.

---

### Task 6: Structural-parity tests between slash-command and ACP surfaces

The two surfaces (slash commands, ACP threads) must stay in structural sync: adding a new operator capability shouldn't expose it on only one side. Drive both surfaces from a single shared inventory of operator capabilities and let CI enforce the contract.

**Files:**
- Create: `src/integrations/inventory.rs` (operator crate)
- Create: `tests/surface_parity.rs` (operator crate)
- Create: `zed-extension/tests/acp_smoke.sh`

- [ ] **Step 1: Add `src/integrations/inventory.rs`** (operator-side, not WASM) enumerating user-facing operator capabilities. One entry per:
  ```rust
  pub struct Capability {
      pub id: &'static str,                  // e.g. "queue.list"
      pub description: &'static str,
      pub rest_endpoint: Option<&'static str>, // path matched against OpenAPI
      pub slash_command_id: Option<&'static str>, // e.g. "op-queue"
      pub acp_tool_id: Option<&'static str>, // e.g. "operator__queue_list"
  }

  pub const INVENTORY: &[Capability] = &[ /* ... */ ];
  ```
  Use the existing OpenAPI generation output as the source-of-truth for `rest_endpoint` values.

- [ ] **Step 2: Add `tests/surface_parity.rs`** asserting:
  1. Every `slash_command_id` in the inventory corresponds to a registered slash command in `zed-extension/extension.toml` (parse the TOML, check the `[slash_commands]` table).
  2. Every `acp_tool_id` is exposed by the operator ACP agent. The exact mechanism depends on what the upstream ACP plan ships — if v1 only delegates to Claude/Codex/Gemini, the ACP surface may expose operator-specific tools through the co-shipped MCP server (covered by the MCP plan at `2026-05-16-mcp-stdio-and-tickets.md`).
  3. Every entry has BOTH a `slash_command_id` AND an `acp_tool_id`, OR an explicit allow-list reason in a separate `tests/fixtures/surface_exceptions.toml`. The default is parity; deviations require an explicit rationale.

- [ ] **Step 3: Add `zed-extension/tests/acp_smoke.sh`** for runtime smoke (separate from parity):
  ```bash
  #!/usr/bin/env bash
  set -euo pipefail
  cd "$(dirname "$0")/../.."
  cargo build --bin operator
  printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{}}}\n' \
    | ./target/debug/operator acp \
    | head -1 \
    | jq -e '.result.agentCapabilities'
  ```

- [ ] **Step 4: Wire both into CI.** Parity runs as `cargo test --test surface_parity`. Smoke runs as a shell job. Any failure blocks releases.

- [ ] **Step 5: Document the contract in `zed-extension/README.md`:** "Adding a new operator capability requires registering it in `src/integrations/inventory.rs`. CI will fail if the new entry lacks either a slash command or an ACP tool (without an explicit exception entry)."

- [ ] **Step 6:** Run full validation:
  ```bash
  cargo fmt && cargo clippy -- -D warnings && cargo test
  bash zed-extension/tests/acp_smoke.sh
  ```
  Expected: green.

- [ ] **Step 7:** Stop for commit review.

---

### Task 7: User-facing verification in Zed

Before declaring the integration shipped, manually verify in the real editor — CI cannot prove this.

- [ ] **Step 1:** Install the extension as dev:
  ```bash
  cd zed-extension && cargo build --release --target wasm32-wasip1
  mkdir -p ~/.local/share/zed/extensions/installed/operator-dev/
  cp extension.toml ~/.local/share/zed/extensions/installed/operator-dev/
  cp target/wasm32-wasip1/release/operator_zed.wasm ~/.local/share/zed/extensions/installed/operator-dev/extension.wasm
  ```
- [ ] **Step 2:** Apply the `settings.json` override from Task 4.
- [ ] **Step 3:** Open a Zed project that has `.tickets/` (the operator repo itself works).
- [ ] **Step 4:** Open agent panel → confirm **Operator** appears in the agent selector.
- [ ] **Step 5:** Open a thread → confirm the prompt arrives and streams a response from the configured delegator.
- [ ] **Step 6:** Cancel a thread mid-stream → confirm the delegator process exits (per upstream plan's `session/cancel` task — may be a v1.1 follow-up).
- [ ] **Step 7:** Document any rough edges in `zed-extension/docs/acp-setup.md` under "Known issues."
- [ ] **Step 8:** Stop for user to commit.

---

## Verification

End-to-end acceptance passes when:
1. `cargo build --release --target wasm32-wasip1` from `zed-extension/` produces a valid WASM artifact.
2. `bash zed-extension/tests/acp_smoke.sh` exits 0.
3. `cargo test --test surface_parity` exits 0.
4. In Zed with the dev override: opening the agent panel → Operator → new thread → typing `hello` → streamed text returns. (Human verification — primary gate.)
5. The four release archive URLs in `extension.toml` resolve to real artifacts whose SHA256 matches.

## Self-Review

**Spec coverage:**
- Operator-side ACP confirmation (Task 1) — covered
- Extension manifest (Task 2) — covered
- Release pipeline (Task 3) — covered
- Dev-mode override docs (Task 4) — covered
- README + TODO updates (Task 5) — covered
- Structural-parity tests + smoke (Task 6) — covered
- Manual Zed verification (Task 7) — covered

**Open assumptions:**
- The operator-side ACP plan ships first. This plan is gated on `operator acp` working — without it there's nothing for Zed to connect to.
- Operator's release pipeline can produce per-target archives. If today's pipeline only produces a single platform, Task 3 expands.
- The Zed `[agent_servers.<id>]` manifest schema is stable. Zed documents it publicly, but the schema is newer than the slash-command API and may shift.

**Explicit non-goals (v1):**
- No JetBrains, Emacs, Kiro, etc. integration. The operator-side plan generates config snippets (Task 8 there) covering those — they don't need a per-editor extension because they read user config directly.
- No deprecation of slash commands. They serve different workflows.
- No sidebar / status bar / file watcher work — ACP doesn't unblock these in Zed's current extension API.
- No per-ticket "Open in Operator agent panel" deep-link from slash commands. Conceivable but out of scope here.

**Resolved decisions:**
1. Release archives are in scope for v1 (Task 3 ships per-platform tarballs + SHA256 + `extension.toml` pinning).
2. Slash commands stay as a parallel surface. Task 6 enforces structural parity between the two surfaces with tests so they cannot drift silently.
3. Plan file lives at `docs/superpowers/plans/2026-05-16-acp-zed-extension.md` (sibling of the operator-side ACP plan).
