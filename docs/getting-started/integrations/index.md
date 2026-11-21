---
title: "Automation Platform Integrations"
description: "Connect Operator! to agent OS and automation platforms like AGNT.gg."
layout: doc
---

Operator is not just a standalone TUI - it exposes its ticket orchestration over
a **REST API** and a **stdio MCP server**, so external *automation platforms* and
*agent operating systems* can drive it, and Operator can hand work out to them.

This is a distinct integration category from your [kanban
provider](/getting-started/kanban/), [coding agent](/getting-started/agents/),
[git host](/getting-started/git/), or [session
wrapper](/getting-started/sessions/). Those are the building blocks Operator
*uses*; automation platforms are peers that *compose with* Operator.

## The two directions

Operator connects to an automation platform in two complementary directions:

| Direction | What it means | How |
|-----------|---------------|-----|
| **Operator → platform** | Export an Operator issuetype/ticket as the platform's native workflow | `operator workflow export --format <fmt>` |
| **Platform → Operator** | The platform calls Operator to create tickets, launch agents, poll the queue | a platform plugin, or Operator's MCP server |

## The portable substrate

Because the "platform → Operator" direction rides on **REST + MCP** - both
portable, widely supported substrates - exposing them once connects Operator to
the *whole category*, not just one tool. Platforms in this space include
[AGNT.gg](https://agnt.gg), n8n, Activepieces, Windmill, Dify, Flowise,
Langflow, OpenAI AgentKit, and Zapier/Make.

Operator already ships:

- a **REST API** (`operator api`, default port `7008`) with endpoints to create
  tickets, launch agents, read queue status, export workflows, and raise alerts;
- a **stdio MCP server** (`operator mcp`) exposing ~18 orchestration tools, which
  any MCP-capable client (Claude Code, Cursor, Zed, and MCP-consuming automation
  platforms) can spawn.

## Supported integrations

- **[AGNT.gg](/getting-started/integrations/agnt/)** - interactive agents use Operator MCP against the Operator that holds the tickets; visual workflows use the `operator-plugin` nodes; `operator workflow export --format agnt` emits a runnable scaffold of a ticket's shape, not a lossless copy of Operator's sessions, gates, and ticket state.

> Write/launch tools mutate your repositories. Only connect platforms you trust,
> and gate Operator's MCP write tools with `[mcp].expose_ticket_write_tools`.

## Connect from the web interface

Open **Connections** in Operator's web interface to copy API addresses, open
Swagger, configure remote MCP clients, and manage service access keys.
Swagger and OpenAPI links use the active API host, including HTTPS and any
configured path prefix. They require an authenticated browser.

The MCP card exposes the existing SSE transport. Choose the Claude Code CLI
recipe for user-scope registration, or merge the JSON recipe into your Claude
user configuration. Install `curl` and `jq` and provide `OPERATOR_ACCESS_KEY` in
the client's environment. The generated header helper exchanges that key at
`/api/v1/auth/token` for a short-lived bearer token on each connection. The raw
key is never itself a bearer token. Project-scoped helpers may filter credential
environment variables; use the displayed user-scope recipe.

Administrators can create a key on this page with `read` and `execute` scopes.
Additional scopes require explicit selection. Copy the secret when it is created;
it cannot be retrieved later. To rotate it, create a replacement, update the
client, verify the connection, and revoke the old key. Already issued bearer
tokens remain valid until their expiry.

**Test MCP connection** checks the active browser API route by opening SSE, initializing MCP, and listing tools. Leave the optional key field blank to use the browser session, or supply an access key to test its token exchange and permissions. Results show their timestamp and authentication method. Availability, a successful handshake, and an initialized client session are separate observations; creating a key does not establish a connection.

For a reverse proxy, set `rest_api.public_url` to the externally reachable base URL when advertising an absolute endpoint. Otherwise MCP advertises relative paths. Preserve SSE streaming through the proxy. The page reports route, authentication, protocol, and timeout failures independently.

The ACP card generates local editor configuration for Zed, JetBrains, Emacs, or Kiro. Install Operator on the editor's machine and choose its local executable and profile.
