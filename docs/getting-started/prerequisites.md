---
title: "Prerequisites"
description: "System requirements and dependencies for running Operator."
layout: doc
---

# Prerequisites

Before installing Operator, ensure your system meets the following requirements.

## System Requirements

### Supported Platforms

| Platform | Versions | Architectures |
|----------|----------|---------------|
| macOS | 12.0+ (Monterey or later) | Apple Silicon (arm64), Intel (x86_64) |
| Linux | Modern distributions | arm64, x86_64 |
| Windows | 10+ | x86_64 |

### Hardware Requirements

- **Memory**: 4GB RAM minimum, 8GB recommended
- **Disk Space**: 100MB for Operator, plus space for project files

## Required Software

### Git

Version control is required for project management. Verify installation:

```bash
git --version  # Should be 2.30+
```

**Installation by platform:**

| Platform | Installation |
|----------|--------------|
| macOS | `brew install git` or install Xcode Command Line Tools |
| Linux | Use package manager (`apt install git`, `dnf install git`, etc.) |
| Windows | Download [Git for Windows](https://git-scm.com/download/win) |

### Session Manager

Operator requires a session manager for launching and managing coding agents:

| Platform | Recommended | Alternative |
|----------|-------------|-------------|
| macOS | [VS Code Extension](/getting-started/vscode-extension/) | tmux |
| Linux | [VS Code Extension](/getting-started/vscode-extension/) | tmux |
| Windows | [VS Code Extension](/getting-started/vscode-extension/) (required) | N/A |

#### VS Code Extension

The VS Code Extension provides the best experience across all platforms and is **required on Windows**. See [VS Code Extension Setup](/getting-started/vscode-extension/) for installation instructions.

#### tmux (macOS/Linux only)

For terminal-based workflows on macOS and Linux, tmux can be used as an alternative session manager:

**macOS:**
```bash
brew install tmux
```

**Linux:**
```bash
# Debian/Ubuntu
apt install tmux

# Fedora/RHEL
dnf install tmux
```

> **Note**: tmux is not available on Windows. Windows users must use the VS Code Extension.

## Windows-Specific Notes

Windows support has some limitations:

- **Session Manager**: VS Code Extension is required (tmux not available)
- **Backstage Server**: Not supported on Windows
- **Notifications**: Native notifications are planned; currently logs only

## Optional Dependencies

### Coding Agent

At least one AI coding agent should be installed:

- [Claude Code](/getting-started/agents/claude/) (recommended)
- [Codex](/getting-started/agents/codex/)
- [Gemini](/getting-started/agents/gemini/)

### Backstage Server (macOS/Linux only)

For centralized project management across multiple repositories, see [Backstage Server Setup](/getting-started/backstage-server/). Note: Not supported on Windows.

### Kanban Integration

For issue tracking integration, configure:

- [Jira Cloud](/getting-started/kanban/jira/)

## Next Steps

Once prerequisites are met, proceed to [Installation](/getting-started/installation/).
