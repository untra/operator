---
title: "MCP"
description: "Register a remote MCP client against Operator's SSE transport and verify the handshake."
layout: doc
---

The **MCP** card shows Operator's SSE endpoint and how many remote clients are connected.

## Transports

- **SSE (remote)** is served by the API. If the card says *HTTP transport off*, set `mcp.http_enabled` in your Operator configuration and restart the API.
- **stdio (local)** is spawned by a local client as `operator mcp`. A hosted page can't see local stdio processes.

## Configure a client

Open **Configure MCP client** and pick a recipe:

| Recipe | Use |
|--------|-----|
| Claude Code CLI | `claude mcp add-json --scope user` registration |
| Claude Code JSON | Merge into your user configuration (`~/.claude.json`) |
| Generic SSE client | Endpoint, token exchange, and header for any SSE client |

The Claude Code recipes need `curl` and `jq`, and `OPERATOR_ACCESS_KEY` set in the client's environment. A header helper exchanges that key at `/api/v1/auth/token` for a short-lived bearer token each time the client connects. The raw key never works as a bearer token itself. Use user scope: project-scoped helpers may filter credential environment variables.

## Test the connection

**Test MCP connection** uses this browser's Operator address. It opens SSE, initializes MCP, and lists tools without calling any of them.

- Leave the access key field blank to test with the browser session.
- Enter an access key to test its token exchange and permissions. The key needs `mcp:connect`, plus the individual MCP tool capabilities and projects you want it to be able to call.

A passing result is a single past observation, not a continuous check. Availability, a successful handshake, and an initialized client session are separate facts, and creating a key doesn't establish a connection. A public address configured separately (see [`rest_api.public_url`](/getting-started/connections/#behind-a-reverse-proxy)) still has to be verified from the client itself. If the handshake times out, check that your proxy streams SSE.

Administrators can expand **Observed clients** to see connected remote clients and when each was last active.

> Write and launch tools change your repositories. Connect only clients you trust, and gate the write tools with `[mcp].expose_ticket_write_tools`.
