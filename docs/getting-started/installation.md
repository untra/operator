---
title: "Installation"
description: "Download and install Operator on your system."
layout: doc
---

# Installation

This guide covers installing Operator on macOS, Linux, and Windows.

## VS Code Extension (Recommended)

The **VS Code Extension** is the recommended way to get started with Operator.
<a href="https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals" target="_blank" class="button">Install from VS Code Marketplace</a>

Works on **macOS**, **Linux**, and **Windows** - no additional setup required.

For detailed setup instructions, see the [VS Code Extension documentation](/getting-started/sessions/vscode/).

---

## CLI Installation (Alternative)

For headless servers, CI/CD pipelines, or advanced workflows, install the CLI binary for your platform.

### macOS

```bash
# Apple Silicon (M1/M2/M3)
curl -L https://github.com/untra/operator/releases/latest/download/operator-macos-arm64 -o operator
chmod +x operator
sudo mv operator /usr/local/bin/
```

### Linux

```bash
# ARM64
curl -L https://github.com/untra/operator/releases/latest/download/operator-linux-arm64 -o operator

# x86_64
curl -L https://github.com/untra/operator/releases/latest/download/operator-linux-x86_64 -o operator

chmod +x operator
sudo mv operator /usr/local/bin/
```

### Windows (PowerShell)

```powershell
Invoke-WebRequest -Uri "https://github.com/untra/operator/releases/latest/download/operator-windows-x86_64.exe" -OutFile "operator.exe"
# Add to PATH or move to desired location
```

For checksums and all available downloads, see the [Downloads](/downloads/) page.

## Verify Installation

Confirm Operator is installed correctly:

```bash
operator --version
```

## Initial Configuration

Create a configuration file:

```bash
operator init
```

This creates the configuration file with default settings:
- **macOS/Linux**: `~/.config/operator/config.toml`
- **Windows**: `%APPDATA%\operator\config.toml`

## Next Steps

Configure your integrations:

- [Set up a Session Manager](/getting-started/sessions/)
- [Set up a Coding Agent](/getting-started/agents/claude/)
- [Connect your Kanban Provider](/getting-started/kanban/jira/)
- [Link your Git Repository](/getting-started/git/github/)
