---
title: LLM Tools
description: "Configure Claude Code and other LLM tools for AI-powered agent integration with Operator!."
layout: doc
---

<span class="operator-brand">Operator!</span> integrates with LLM tools like Claude Code to power AI-assisted development.

## Supported Tools

### Claude Code

The primary LLM tool supported by Operator. Claude Code is a CLI tool that provides:

- Code generation and editing
- Bug analysis and fixes
- Test writing
- Documentation
- Refactoring

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
