---
title: "Supported Coding Agents"
description: "AI coding agents compatible with Operator."
layout: doc
---

Operator orchestrates AI coding agents to work on tickets from your kanban board. The following agents are currently supported:

## Available Agents

| Agent | Status | Notes |
|-------|--------|-------|
| [Claude](/getting-started/agents/claude/) | Recommended | Full feature support |
| [Codex](/getting-started/agents/codex/) | Supported | OpenAI's coding model |
| [Gemini CLI](/getting-started/agents/gemini-cli/) | Experimental | Google's AI assistant |

## Agent Capabilities

Each agent can:

- Read and understand ticket requirements
- Browse project codebases
- Write and modify code
- Run tests and builds
- Create pull requests

## Choosing an Agent

**Claude** is recommended for most users due to its strong code understanding and generation capabilities. See individual agent pages for setup instructions and specific features.

## Agent Lifecycle

Operator tracks every agent it launches through these states:

```
Created -> Running -> Completed
              |
              v
        Awaiting Input
```

| State | Description |
|-------|-------------|
| **Created** | Agent initialized, not yet started |
| **Running** | Actively working on a ticket |
| **Awaiting Input** | Needs a human response |
| **Completed** | Work finished successfully |
| **Failed** | An error occurred |

## Autonomous and Paired Modes

Every issue type declares a `mode`, and that decides how much of your attention
its tickets need:

- **Autonomous** — launch and monitor. Minimal intervention, and several can run
  in parallel across different projects.
- **Paired** — active human participation, with back-and-forth discussion. One at
  a time, because they compete for the same operator: you.

Mode is a property of the issue type, not of the agent, so a collection decides
which of its work types are hands-off. See [Workflows](/workflows/).

## Sessions

Agent sessions persist under `.operator/`:

```
.operator/
├── state.json
├── sessions/
│   ├── agent-123.json
│   └── agent-456.json
└── history.json
```

Session files record ticket information, start and end times, status history, and
output logs. Operator can also detect completion from files an agent produces —
see [Artifact Detection](/artifact-detection/).

## Best Practices

1. **Monitor paired agents** — stay engaged with paired work
2. **Review autonomous work** — check completed tickets
3. **Handle failures promptly** — address failed agents quickly
4. **Balance load** — don't overload with too many agents
