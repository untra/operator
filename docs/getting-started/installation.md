---
title: "Installation"
description: "Download and install Operator on your system."
layout: doc
---

# Installation

This guide covers installing Operator on macOS.

## Download

Get the latest release from the [Downloads](/downloads/) page or use the command line:

```bash
# Apple Silicon (M1/M2/M3)
curl -L https://github.com/untra/operator/releases/latest/download/operator-macos-arm64 -o operator

# Intel Mac
curl -L https://github.com/untra/operator/releases/latest/download/operator-macos-x86_64 -o operator
```

## Install

Move the binary to your PATH:

```bash
chmod +x operator
sudo mv operator /usr/local/bin/
```

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

This creates `~/.config/operator/config.toml` with default settings.

## Next Steps

Configure your integrations:

- [Set up a Coding Agent](/getting-started/agents/claude/)
- [Connect your Kanban Provider](/getting-started/kanban/jira/)
- [Link your Git Repository](/getting-started/git/github/)
