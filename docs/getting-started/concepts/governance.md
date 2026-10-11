---
title: "Governance"
description: "Named launch policies that limit which delegators, models and hosts an agent may use, and how many may run."
layout: doc
---

**Governance** is a named policy that decides whether a launch may happen. A delegator references a policy by name, and every launch through that delegator - from the TUI, web UI, VS Code, REST, MCP or CLI - is checked against it before anything is provisioned.

```toml
[[governance]]
name                  = "team-default"
allowed_llm_tools     = ["claude", "codex"]
allowed_model_servers = ["anthropic-api", "openai-api"]
allowed_targets       = ["local", "docker"]
max_parallel          = 3

[[delegators]]
name       = "claude-opus-auto"
llm_tool   = "claude"
model      = "opus"
governance = "team-default"
```

## Fields

Every constraint is optional. An absent constraint is unconstrained.

| Field | Description |
|-------|-------------|
| `name` | Unique policy name, referenced by `governance` on a delegator |
| `allowed_delegators` | Delegator names allowed to launch |
| `allowed_llm_tools` | LLM tools allowed to launch (`claude`, `codex`, `gemini`, ...) |
| `allowed_model_servers` | Model servers allowed, declared or implicit (`anthropic-api`, `ollama-local`, ...) |
| `allowed_targets` | Execution targets allowed (`local`, `docker`, or a `[[targets]]` name) |
| `max_parallel` | Maximum tickets running at once under this policy |
| `max_launches` | Maximum total launches, counted per campaign run |
| `max_retries` | Maximum relaunches of a failed ticket, counted per campaign run |
| `max_runtime_minutes` | Maximum wall-clock minutes per agent |
| `budget_usd` | Spend ceiling. Reserved: accepted, not yet enforced |
| `require_confirmation` | Require operator confirmation before each launch (default `false`) |

An empty list allows nothing: `allowed_targets = []` blocks every launch through that delegator.

## Layering

Policies stack. When more than one applies to a launch, they combine into one effective policy and the most restrictive rule wins:

- allow-lists intersect,
- caps take the lowest value,
- `require_confirmation` is on if any layer turns it on.

A delegator's own policy is always the innermost layer. Campaigns add their policy on top of it.

## When a launch is refused

A refused launch starts nothing: no ticket is claimed, no session or worktree is created. The REST API answers `403` with error code `governance_denied` and a message naming the policy and the rule that failed. MCP tools return the same `governance_denied: ...` text.

A delegator that names a policy that does not exist is refused the same way - an unknown name is never a silent pass.

## See also

- [Delegators](/delegators/) - where a policy is attached
- [Remote Targets](/getting-started/remote-targets/) - target names a policy can allow
