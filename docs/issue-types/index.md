---
title: Issue Types
layout: doc
---

Operator supports four issue types, each with different priority levels and execution modes.

## Priority Order

1. **INV** (Investigation) - Highest priority
2. **FIX** - Bug fixes
3. **FEAT** - Features
4. **SPIKE** - Research (lowest priority)

## Issue Type Details

### INV - Investigation

**Priority:** 1 (highest)
**Mode:** Paired

Investigation tickets are for diagnosing failures, understanding bugs, or exploring issues. They require human interaction and are worked on with operator pairing.

```
INV-001-project-investigate-login-failure.md
```

### FIX - Bug Fix

**Priority:** 2
**Mode:** Autonomous

Bug fixes are addressed after investigations. Agents can work autonomously once the problem is understood.

```
FIX-042-project-fix-null-pointer-exception.md
```

### FEAT - Feature

**Priority:** 3
**Mode:** Autonomous

New features are implemented after critical bugs are addressed. Agents can work autonomously with clear requirements.

```
FEAT-123-project-add-dark-mode.md
```

### SPIKE - Research

**Priority:** 4 (lowest)
**Mode:** Paired

Spikes are for research, exploration, and proof-of-concept work. They require human interaction and discussion.

```
SPIKE-007-project-evaluate-new-framework.md
```

## Agent Modes

### Autonomous Mode (FEAT, FIX)

- Launch and monitor progress
- Minimal human intervention
- Can run multiple agents in parallel

### Paired Mode (SPIKE, INV)

- Requires active human participation
- Tracks "awaiting input" states
- One paired agent at a time per operator
