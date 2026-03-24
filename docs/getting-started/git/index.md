---
title: "Supported Git Repositories"
description: "Git hosting integrations for Operator."
layout: doc
---

# Supported Git Repositories

Operator integrates with Git hosting platforms to manage branches and pull/merge requests.

## Prerequisites

All providers require:

| Requirement | Purpose | Verification |
|------------|---------|--------------|
| `git` | Version control operations | `git --version` |

## Available Integrations

| Platform | Status | CLI Tool | Notes |
|----------|--------|----------|-------|
| [GitHub](/getting-started/git/github/) | Supported | `gh` | Full PR integration |
| [GitLab](/getting-started/git/gitlab/) | Partial | `glab` | Detection and config ready; MR operations planned |

## Provider Auto-Detection

Operator detects your Git provider from the remote URL automatically. You can override this in config:

```toml
[git]
provider = "github"   # or "gitlab"
```

## Shared Git Configuration

These settings apply regardless of provider:

```toml
[git]
branch_format = "{type}/{ticket_id}"   # Branch naming pattern
use_worktrees = false                   # Per-ticket worktree isolation
```

**Branch format variables:** `{type}` is the ticket type prefix (e.g., `feature`, `fix`, `spike`, `investigation`), `{ticket_id}` is the ticket identifier.

**Worktrees:** When enabled, Operator creates isolated git worktrees per ticket, allowing parallel development without branch switching.

## How It Works

When an agent completes work on a ticket:

1. **Branch**: Creates a feature branch from main
2. **Commit**: Commits changes with ticket reference
3. **Push**: Pushes branch to remote
4. **PR/MR**: Opens a pull request or merge request for review

## Local Git

Even without platform integration, Operator manages local Git operations:

- Branch creation and checkout
- Commit formatting with ticket IDs
- Branch cleanup after completion
- Worktree management for parallel development

Local git operations require only the `git` binary—no provider CLI or tokens needed.

## Adding Provider Support

See the [Provider Support](/getting-started/git/provider-support/) guide for architecture details on implementing new providers.
