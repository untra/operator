---
title: LLM Tools
description: "Configure Claude Code and other LLM tools for AI-powered agent integration with Operator!."
layout: doc
---

<span class="operator-brand">Operator!</span> integrates with LLM tools like Claude Code to power AI-assisted development.

## Supported Tools

### Claude Code

The primary LLM tool supported by Operator. 

### OpenAI Codex

### Google Gemini

## Custom Tool Configs

Beyond the builtin tools (claude, gemini, codex), any LLM CLI can be added at runtime by dropping a JSON config into the user tool-config directory - no rebuild required:

- Linux: `~/.config/operator/tools/<tool_name>.json`
- macOS: `~/Library/Application Support/operator/tools/<tool_name>.json`

Configs are loaded fresh on every startup. A user config whose `tool_name`
matches a builtin **fully replaces** that builtin (no field-by-field merge).
Malformed files are skipped with a logged warning. Runtime-loaded tools work
everywhere the builtins do, including remote (SSH) launches, where the tool's
presence on the remote host is verified by a `command -v` preflight.

> **Security note:** `command_template` is arbitrary shell executed at launch.
> <span class="operator-brand">Operator!</span> only loads tool configs from the
> user-global config directory - never from repository-local paths - so a
> cloned repo cannot inject a tool config.

The full config format is documented in the schema reference on this site
(source of truth: `src/llm/tools/tool_config.schema.json`).

## Detection Modes

By default a tool is detected only when `which <tool_name>` succeeds. The
optional `detection` object overrides this:

```json
{
  "detection": { "mode": "always", "health_command": "your-tool ping" }
}
```

- `mode: "which"` (default) - gate detection on the binary being in PATH
- `mode: "always"` - skip the PATH lookup and use `tool_name` verbatim as the
  invocation path; for tools not installed locally, e.g. only present on a
  remote SSH host
- `health_command` - health check run at every startup

Health is **earned, never assumed**, and re-verified on every startup:

| Mode | No `health_command` | With `health_command` |
|------|---------------------|-----------------------|
| `which` | Healthy - the PATH lookup proves the binary is present | Healthy if still on PATH **and** the command passes |
| `always` | **Unhealthy** - nothing is locally verifiable | Healthy if the command passes |

An unhealthy tool stays listed among the detected tools, but launching a local agent with it fails until it is healthy again.

Remote (SSH) launches are unaffected - they are gated by their own `command -v` preflight on the remote host. An `always`-mode tool should therefore define a `health_command` that proves reachability.

## Integration Points

### Launching Agents

<span class="operator-brand">Operator!</span> launches Claude Code with project context:

```bash
# macOS launch command
open -a "Claude" --args --project "/path/to/project"
```

### Initial Prompts

Tickets provide context to agents through:

1. **Ticket content** - The markdown ticket file
2. **Project CLAUDE.md** - Project-specific instructions
3. **Clipboard injection** - Initial prompt via paste simulation

### Monitoring

<span class="operator-brand">Operator!</span> tracks agent status:

- **Running** - Agent is actively working
- **Awaiting Input** - Agent needs human response
- **Completed** - Work is finished
- **Failed** - An error occurred

## Configuration

Configure LLM tool settings in your <span class="operator-brand">Operator!</span> config:

```toml
[llm]
tool = "claude-code"
max_concurrent = 4

[llm.claude]
path = "/Applications/Claude.app"
```

## Known Limitations

### JSON Schema for Structured Output (Temporarily Disabled)

The `jsonSchema` and `jsonSchemaFile` step properties are currently disabled. These properties configure the `--json-schema` flag for Claude Code to enable structured output validation.

**Issue**: Even when writing schemas to files (rather than passing inline JSON), the command line length can exceed OS limits when combined with other flags.

**Workaround**: Until this is resolved, use Claude Code's native structured output capabilities without the `--json-schema` flag, or validate outputs manually in subsequent steps.

**Tracking**: See `JSON_SCHEMA_ENABLED` constant in `src/agents/launcher/llm_command.rs`.

## Best Practices

1. **Clear tickets** - Write detailed ticket descriptions
2. **Project context** - Maintain good CLAUDE.md files
3. **Monitor paired work** - Stay engaged with INV/SPIKE agents
4. **Review autonomous work** - Check completed FEAT/FIX work
