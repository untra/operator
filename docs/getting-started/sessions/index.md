---
title: "Supported Session Management"
description: "Session wrapper tools for managing AI agent terminals."
layout: doc
---

# Supported Session Management

Operator supports multiple session management backends for running AI coding agents in persistent, manageable terminal sessions.

## Available Options

| Option | Status | Notes |
|--------|--------|-------|
| [VS Code Extension](/getting-started/sessions/vscode/) | Recommended (Preferred) | Integrated terminals in VS Code, works on all platforms |
| [tmux](/getting-started/sessions/tmux/) | Supported | Terminal multiplexer, ideal for headless/server environments |

## How It Works

Session managers provide:

- **Persistent terminals**: Agent sessions survive disconnects
- **Session switching**: Move between multiple agents
- **Activity tracking**: Monitor idle/running states
- **Integration**: Two-way communication with Operator TUI/API

## Choosing a Session Manager

**VS Code Extension** is the recommended choice for most users. It provides an integrated experience with ticket management, color-coded terminals, and works seamlessly on macOS, Linux, and Windows without additional setup.

**tmux** remains an excellent choice for headless/server environments, SSH sessions, and users who prefer terminal-based workflows. It's particularly useful for remote servers where VS Code may not be available.

## Feature Parity: Core Operations

All session management tools support the following Core Operations:

| Operation | TUI | VS Code Extension | REST API |
|-----------|-----|-------------------|----------|
| Sync Kanban Collections | `S` | `Operator: Sync Kanban Collections` | `POST /api/v1/queue/sync` |
| Pause Queue Processing | `P` | `Operator: Pause Queue Processing` | `POST /api/v1/queue/pause` |
| Resume Queue Processing | `R` | `Operator: Resume Queue Processing` | `POST /api/v1/queue/resume` |
| Approve Review | `Y` | `Operator: Approve Review` | `POST /api/v1/agents/{id}/approve` |
| Reject Review | `X` | `Operator: Reject Review` | `POST /api/v1/agents/{id}/reject` |

### Core Operations Explained

1. **Sync Kanban Collections** - Fetch issues from external kanban providers (Jira, Linear) and create local tickets in the queue

2. **Pause Queue Processing** - Temporarily stop automatic agent launches while maintaining queue state

3. **Resume Queue Processing** - Continue automatic agent launches after pausing

4. **Approve Review** - Approve an agent's pending plan or visual review to continue workflow

5. **Reject Review** - Reject a review with feedback, triggering the agent to re-do the work
