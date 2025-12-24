---
title: LLM Tools
layout: doc
---

Operator integrates with LLM tools like Claude Code to power AI-assisted development.

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

Operator launches Claude Code with project context:

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

Operator tracks agent status:

- **Running** - Agent is actively working
- **Awaiting Input** - Agent needs human response
- **Completed** - Work is finished
- **Failed** - An error occurred

## Configuration

Configure LLM tool settings in your Operator config:

```toml
[llm]
tool = "claude-code"
max_concurrent = 4

[llm.claude]
path = "/Applications/Claude.app"
```

## Best Practices

1. **Clear tickets** - Write detailed ticket descriptions
2. **Project context** - Maintain good CLAUDE.md files
3. **Monitor paired work** - Stay engaged with INV/SPIKE agents
4. **Review autonomous work** - Check completed FEAT/FIX work
