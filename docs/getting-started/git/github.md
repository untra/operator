---
title: "GitHub"
description: "Configure GitHub integration with Operator."
layout: doc
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

The `gh` CLI handles authentication and API operations.

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
```

### Windows

```powershell
winget install --id GitHub.cli
```

### Authenticate

```bash
gh auth login
```

Follow the prompts to authenticate via browser or token.

## Create Personal Access Token

1. Go to GitHub Settings > Developer settings > Personal access tokens
2. Click "Generate new token (classic)"
3. Select scopes: `repo`, `workflow`
4. Copy the token

## Configuration

Add GitHub to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[git.github]
enabled = true
token_env = "GITHUB_TOKEN"
owner = "your-username"  # or organization
repo = "your-repo"
```

Set your token:

```bash
export GITHUB_TOKEN="ghp_xxxxx"
```

## Pull Request Settings

Configure PR behavior:

```toml
[git.github.pr]
base_branch = "main"
draft = false
auto_merge = false
reviewers = ["teammate1", "teammate2"]
labels = ["automated", "ai-generated"]
```

## Branch Naming

Configure branch name format:

```toml
[git.github]
branch_format = "{type}/{ticket_id}-{slug}"
# Example: feat/PROJ-123-add-login
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

Test your token:

```bash
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/user
```

### Permission denied

Ensure your token has `repo` scope and you have push access.

### Rate limiting

GitHub has API rate limits. Check remaining quota:

```bash
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/rate_limit
```
