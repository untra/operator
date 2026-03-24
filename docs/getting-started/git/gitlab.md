---
title: "GitLab"
description: "Configure GitLab integration with Operator."
layout: doc
published: true
---

# GitLab

Connect Operator to GitLab for repository management and merge requests.

## Prerequisites

| Requirement | Purpose | Verification |
|------------|---------|--------------|
| `git` | Version control | `git --version` |
| `glab` | GitLab CLI | `glab --version` |
| GitLab account | Repository access | - |
| Push access | Create branches/MRs | - |

## Install GitLab CLI

The `glab` CLI handles authentication and API operations. Operator uses `glab` directly rather than raw API calls.

### macOS

```bash
brew install glab
```

### Linux

```bash
# Debian/Ubuntu
curl -fsSL https://gitlab.com/gitlab-org/cli/-/releases/permalink/latest/downloads/glab_amd64.deb -o glab.deb
sudo dpkg -i glab.deb

# Fedora/RHEL
sudo dnf install glab
```

### Windows

```powershell
winget install --id GLab.GLab
```

## Authenticate

The `glab` CLI manages authentication, including OAuth flows and credential storage:

```bash
glab auth login
```

Follow the prompts to authenticate via browser or token. Verify with:

```bash
glab auth status
```

## Configuration

Add GitLab to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[git.gitlab]
enabled = true
token_env = "GITLAB_TOKEN"       # env var for token (used as fallback if glab CLI auth is unavailable)
host = "gitlab.com"              # or your self-hosted instance (e.g., "gitlab.example.com")
```

If `token_env` is set, export the token:

```bash
export GITLAB_TOKEN="glpat-xxxxx"
```

Merge request operations (create, monitor, review tracking) are not yet implemented. Provider detection, configuration, and `glab` CLI authentication work today.

### Provider Auto-Detection

Operator auto-detects GitLab from your git remote URL, including self-hosted instances (any URL containing `gitlab.`):

```bash
# All of these are auto-detected as GitLab:
git@gitlab.com:owner/repo.git
https://gitlab.com/owner/repo
https://gitlab.example.com/owner/repo
```

You can also set the provider explicitly:

```toml
[git]
provider = "gitlab"
```

### Shared Git Settings

Branch naming and worktree settings live under the shared `[git]` section (see [Supported Git Repositories](/getting-started/git/) for details):

```toml
[git]
branch_format = "{type}/{ticket_id}"
use_worktrees = false
```

## Troubleshooting

### Authentication errors

Check your auth status:

```bash
glab auth status
```

### Permission denied

Ensure you have push access to the repository and your `glab` session is authenticated.

### Self-hosted connectivity

If using a self-hosted GitLab instance, verify the `host` value in your config matches the instance hostname and that the instance is reachable:

```bash
glab auth status --hostname gitlab.example.com
```
