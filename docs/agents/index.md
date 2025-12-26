---
title: Agents
layout: doc
---

Agents are LLM-powered workers that execute tickets. <span class="operator-brand">Operator!</span> manages their lifecycle and coordinates their work.

## Agent Lifecycle

```
Created -> Running -> Completed
              |
              v
        Awaiting Input
```

### States

| State | Description |
|-------|-------------|
| **Created** | Agent initialized, not yet started |
| **Running** | Actively working on ticket |
| **Awaiting Input** | Needs human response |
| **Completed** | Work finished successfully |
| **Failed** | Error occurred |

## Agent Modes

### Autonomous Mode

Used for: **FEAT**, **FIX**

- Launch and monitor
- Minimal intervention
- Can run in parallel

### Paired Mode

Used for: **INV**, **SPIKE**

- Active human participation
- Back-and-forth discussion
- One at a time per operator

## Parallelism

<span class="operator-brand">Operator!</span> enforces parallelism rules:

```
Max agents = min(configured_max, cpu_cores - reserved)
```

### Rules

1. **Different projects** - Autonomous agents can run in parallel
2. **Same project** - Sequential only (avoid conflicts)
3. **Paired agents** - One at a time

## Tracking

<span class="operator-brand">Operator!</span> tracks agents in real-time:

```json
{
  "agents": [
    {
      "id": "agent-123",
      "ticket": "FEAT-042",
      "project": "backend",
      "status": "running",
      "started_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

## Sessions

Agent sessions persist in `.operator/sessions/`:

```
.operator/
├── state.json
├── sessions/
│   ├── agent-123.json
│   └── agent-456.json
└── history.json
```

Session files contain:
- Ticket information
- Start/end times
- Status history
- Output logs

## Best Practices

1. **Monitor paired agents** - Stay engaged with INV/SPIKE
2. **Review autonomous work** - Check completed FEAT/FIX
3. **Handle failures promptly** - Address failed agents quickly
4. **Balance load** - Don't overload with too many agents
