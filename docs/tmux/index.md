---
title: Tmux Sessions
layout: doc
---

Operator uses tmux for terminal session management, providing a customized experience for managing multiple agent sessions.

## Custom Configuration

Operator generates a custom `.tmux.conf` for managed sessions with:

- Easy detach binding (Ctrl+a)
- Mouse support
- Increased scrollback
- Custom status bar with agent stats
- Operator color theme

## Color Theme

The tmux theme uses Operator's salmon-based palette:

| Element | Color | Hex |
|---------|-------|-----|
| Background | Salmon | `#cc6c55` |
| Primary text | Dark teal | `#114145` |
| Muted text | Darker salmon | `#8a4a3a` |
| Accent | Tan/cream | `#f4dbb7` |
| Warning | Red/coral | `#d46048` |

## Status Bar

The status bar shows:

```
op:SESSION | 1:window  | 3 agents | 14:30
```

### Agent Display

- `3 agents` - All agents running
- `2/1 agents` - 2 running, 1 awaiting input
- `PAUSED` - Queue is paused

## Key Bindings

| Binding | Action |
|---------|--------|
| `Ctrl+a` | Detach from session |
| `Ctrl+b` (prefix) | Standard tmux prefix |

## Configuration File

The generated config includes:

```bash
# Operator custom tmux configuration

# Easy detach: Ctrl+a (no prefix needed!)
bind-key -n C-a detach-client

# Increase scrollback buffer
set -g history-limit 10000

# Status bar colors (Operator theme)
set -g status-style "bg=#cc6c55,fg=#114145"

# Left: session name
set -g status-left "#[fg=#f4dbb7,bold] op:#S #[fg=#8a4a3a]| "

# Right: operator stats + time
set -g status-right "#[fg=#8a4a3a]| #(status-script) | %H:%M"
```

## Status Script

The status script reads `state.json` and outputs:

```bash
#!/bin/bash
STATE_FILE="$1"

if command -v jq &> /dev/null; then
    RUNNING=$(jq '[.agents[] | select(.status == "running")] | length' "$STATE_FILE")
    AWAITING=$(jq '[.agents[] | select(.status == "awaiting_input")] | length' "$STATE_FILE")
fi

if [ "$AWAITING" -gt 0 ]; then
    echo "$RUNNING/$AWAITING agents"
else
    echo "$RUNNING agents"
fi
```

## Best Practices

1. **Use Ctrl+a to detach** - Quick detach without prefix
2. **Check status bar** - Monitor agent count at a glance
3. **Scroll history** - Use mouse or vi keys in copy mode
4. **Multiple windows** - Organize by project or task
