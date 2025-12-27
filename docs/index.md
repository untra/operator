---
title: Operator!
description: "Operator! is a Rust TUI for orchestrating Claude Code agents across multi-project codebases with kanban-style ticket management."
layout: doc
---

Welcome friend! **Operator!** is a Rust TUI application for orchestrating Claude Code (and other llms) agents across multi-project codebases. It combines kanban-style ticket management with LLM tools and tmux terminals to create a powerful workflow for managing AI-assisted development.

## What is <span class="operator-brand">Operator!</span>?

<span class="operator-brand">Operator!</span> helps you:

- **Manage ticket queues** - Organize work items by priority (INV, FIX, FEAT, SPIKE)
- **Launch LLM agents** - Start Claude Code sessions with context from tickets
- **Track progress** - Monitor agent status and completion in real-time
- **Coordinate work** - Handle parallelism rules and project dependencies
- **Enforce Standards** - Define how work gets done, across your many software services
- **Catalog your Code** - Get the overview of your codebase, composed from project files

## Getting Started

1. Install <span class="operator-brand">Operator!</span> (downloads page)
2. Configure your projects (J - project tasks)
3. Create issuetypes to make new work (w - ui from portal)
3. Create tickets in `.tickets/queue/` (kanban connectors)
4. Launch agents and watch them work (measuring success)

## Quick Links

- [Kanban](/kanban/) - Understand the kanban workflow
- [Issue Types](/issue-types/) - Learn about INV, FIX, FEAT, and SPIKE
- [LLM Tools](/llm-tools/) - Configure LLM integration
- [Tickets](/tickets/) - Create and manage tickets
- [Agents](/agents/) - Agent lifecycle and modes
- [Tmux](/tmux/) - Terminal session management
