---
title: "GitHub"
description: "Configure GitHub integration with Operator."
layout: doc
published: true
---

# GitHub

Connect Operator to GitHub for repository management and pull requests.

## Prerequisites

| Requirement | Purpose | Verification |
|------------|---------|--------------|
| `git` | Version control | `git --version` |
| `gh` | GitHub CLI | `gh --version` |
| GitHub account | Repository access | - |
| Push access | Create branches/PRs | - |

## Install GitHub CLI

The `gh` CLI handles authentication and API operations. Operator uses `gh` directly rather than raw API calls.

### macOS

```bash
brew install gh
```

### Linux

```bash
# Debian/Ubuntu
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh

# Fedora/RHEL
sudo dnf install gh
```

### Windows

```powershell
winget install --id GitHub.cli
```

## Authenticate

The `gh` CLI manages authentication, including OAuth flows and credential storage:

```bash
gh auth login
```

Follow the prompts to authenticate via browser or token. Verify with:

```bash
gh auth status
```

## Configuration

Add GitHub to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[git.github]
enabled = true
token_env = "GITHUB_TOKEN"   # env var for token (used as fallback if gh CLI auth is unavailable)
```

If `token_env` is set, export the token:

```bash
export GITHUB_TOKEN="ghp_xxxxx"
```

### Provider Auto-Detection

Operator auto-detects GitHub from your git remote URL. You can also set it explicitly:

```toml
[git]
provider = "github"
```

### Shared Git Settings

Branch naming and worktree settings live under the shared `[git]` section (see [Supported Git Repositories](/getting-started/git/) for details):

```toml
[git]
branch_format = "{type}/{ticket_id}"
use_worktrees = false
```

## Commit Messages

Operator formats commits with ticket references:

```
feat(auth): add login form

Implements user authentication UI.

Ticket: PROJ-123
```

## Troubleshooting

### Authentication errors

Check your auth status:

```bash
gh auth status
```

### Permission denied

Ensure you have push access to the repository and your `gh` session is authenticated.

### Rate limiting

Check remaining API quota:

```bash
gh api rate_limit
```
