---
title: Home
layout: doc
---

Welcome to the **Operator** documentation.

Operator is a Rust TUI application for orchestrating Claude Code agents across multi-project codebases. It combines kanban-style ticket management with LLM tools and tmux terminals to create a powerful workflow for managing AI-assisted development.

## What is Operator?

Operator helps you:

- **Manage ticket queues** - Organize work items by priority (INV, FIX, FEAT, SPIKE)
- **Launch LLM agents** - Start Claude Code sessions with context from tickets
- **Track progress** - Monitor agent status and completion in real-time
- **Coordinate work** - Handle parallelism rules and project dependencies

## Getting Started

1. Install Operator
2. Configure your projects
3. Create tickets in `.tickets/queue/`
4. Launch agents and watch them work

## Quick Links

- [Kanban](/kanban/) - Understand the kanban workflow
- [Issue Types](/issue-types/) - Learn about INV, FIX, FEAT, and SPIKE
- [LLM Tools](/llm-tools/) - Configure LLM integration
- [Tickets](/tickets/) - Create and manage tickets
- [Agents](/agents/) - Agent lifecycle and modes
- [Tmux](/tmux/) - Terminal session management
