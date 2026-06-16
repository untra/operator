---
title: "Automation Platform Integrations"
description: "Connect Operator! to agent OS and automation platforms like AGNT.gg."
layout: doc
---

# Automation Platform Integrations

Operator is not just a standalone TUI — it exposes its ticket orchestration over
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

Because the "platform → Operator" direction rides on **REST + MCP** — both
portable, widely supported substrates — exposing them once connects Operator to
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

- **[AGNT.gg](/getting-started/integrations/agnt/)** — export Operator workflows
  as AGNT graphs, and drive Operator from AGNT workflows via the `operator-plugin`.

> Write/launch tools mutate your repositories. Only connect platforms you trust,
> and gate Operator's MCP write tools with `[mcp].expose_ticket_write_tools`.
