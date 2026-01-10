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
| [tmux](/getting-started/sessions/tmux/) | Recommended | Terminal multiplexer, works headless |
| [VS Code Extension](/getting-started/sessions/vscode/) | Supported | Integrated terminals in VS Code |

## How It Works

Session managers provide:

- **Persistent terminals**: Agent sessions survive disconnects
- **Session switching**: Move between multiple agents
- **Activity tracking**: Monitor idle/running states
- **Integration**: Two-way communication with Operator TUI/API

## Choosing a Session Manager

**tmux** is recommended for most users, especially those running agents on remote servers or headless environments. It works anywhere with a terminal.

**VS Code Extension** is ideal for developers who prefer working within VS Code and want integrated ticket management alongside their editor.
