---
title: "Prerequisites"
description: "System requirements and dependencies for running Operator."
layout: doc
---

# Prerequisites

Before installing Operator, ensure your system meets the following requirements.

## System Requirements

- **Operating System**: macOS 12.0+ (Monterey or later)
- **Architecture**: Apple Silicon (arm64) or Intel (x86_64)
- **Memory**: 4GB RAM minimum, 8GB recommended
- **Disk Space**: 100MB for Operator, plus space for project files

## Required Software

### tmux

Operator uses tmux for session management. Install via Homebrew:

```bash
brew install tmux
```

### Git

Version control is required for project management:

```bash
git --version  # Should be 2.30+
```

## Optional Dependencies

### Coding Agent

At least one AI coding agent should be installed:

- [Claude Code](/getting-started/agents/claude/) (recommended)
- [Codex](/getting-started/agents/codex/)
- [Gemini](/getting-started/agents/gemini/)

### Kanban Integration

For issue tracking integration, configure:

- [Jira Cloud](/getting-started/kanban/jira/)

## Next Steps

Once prerequisites are met, proceed to [Installation](/getting-started/installation/).
