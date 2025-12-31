---
title: "GitLab"
description: "Configure GitLab integration with Operator (Coming Soon)."
layout: doc
published: false
---

# GitLab

GitLab integration is planned for a future release.

## Planned Features

When available, GitLab support will include:

- Merge request creation and monitoring
- Review status tracking
- CI/CD pipeline status integration
- Branch management

## Expected Prerequisites

GitLab integration will require:

| Requirement | Purpose |
|------------|---------|
| GitLab account | Repository access |
| Personal Access Token or Project Token | API authentication |
| `glab` CLI (optional) | Alternative to direct API |

### Token Scopes (Planned)

- `api` - Full API access
- `read_repository` - Read repository contents
- `write_repository` - Push changes

## Planned Configuration

```toml
# ~/.config/operator/config.toml

[git]
provider = "gitlab"

[git.gitlab]
enabled = true
token_env = "GITLAB_TOKEN"
host = "gitlab.com"  # Or self-hosted instance
project_id = "namespace/project"
```

### Merge Request Settings (Planned)

```toml
[git.gitlab.mr]
target_branch = "main"
draft = false
remove_source_branch = true
squash = false
reviewers = ["username1", "username2"]
labels = ["automated", "ai-generated"]
```

## CLI Alternative

GitLab's official CLI (`glab`) may be used as an alternative:

```bash
# Install glab
brew install glab

# Authenticate
glab auth login
```

## Contributing

Interested in helping implement GitLab support? See the [Provider Support](/getting-started/git/provider-support/) guide for architecture details.
