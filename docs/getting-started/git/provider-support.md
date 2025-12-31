---
title: "Provider Support"
description: "Architecture guide for adding new Git provider integrations."
layout: doc
---

# Provider Support

This guide explains how Operator integrates with Git hosting providers and how to add support for new providers.

## Architecture Overview

Operator uses a trait-based architecture for Git provider support:

```
┌─────────────────────────────────────────┐
│            PrService trait              │
│  (get_pr, is_ready_to_merge, etc.)     │
├─────────────────────────────────────────┤
│  GitHubService  │  NewProviderService  │
├─────────────────────────────────────────┤
│     GhCli       │    ProviderCli       │
│   (gh binary)   │   (cli binary)       │
└─────────────────────────────────────────┘
```

## Implementation Approaches

### CLI-Based (Recommended)

Uses the provider's official CLI tool:

| Provider | CLI Tool | Install |
|----------|----------|---------|
| GitHub | `gh` | `brew install gh` |

**Advantages:**
- Built-in authentication management
- OAuth flows handled by CLI
- Credentials stored securely in system keychain
- Consistent behavior with official tooling

**Disadvantages:**
- Requires external binary installation
- May have version compatibility concerns

### API-Based

Direct REST/GraphQL API calls:

**Advantages:**
- No external dependencies
- Fine-grained control
- Works in restricted environments

**Disadvantages:**
- Manual token management
- Must implement OAuth flows
- API versioning complexity

## Core Traits

### PrService

Provider-agnostic interface for PR/MR operations:

```rust
#[async_trait]
pub trait PrService: Send + Sync {
    /// Get PR/MR information
    async fn get_pr(&self, repo: &RepoInfo, number: i64)
        -> Result<PullRequestInfo>;

    /// Check if PR/MR is ready to merge
    async fn is_ready_to_merge(&self, repo: &RepoInfo, number: i64)
        -> Result<bool>;

    /// Get review/approval state
    async fn get_review_state(&self, repo: &RepoInfo, number: i64)
        -> Result<ReviewState>;

    /// Create a new PR/MR
    async fn create_pr(&self, repo: &RepoInfo, request: &CreatePrRequest)
        -> Result<PullRequestInfo, CreatePrError>;
}
```

### RepoProvider

For status tracking and CI integration:

```rust
#[async_trait]
pub trait RepoProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_configured(&self) -> bool;

    async fn get_pr_status(&self, repo: &str, number: u64)
        -> Result<PrStatus, ApiError>;
    async fn get_check_runs(&self, repo: &str, ref_sha: &str)
        -> Result<Vec<CheckStatus>, ApiError>;
    async fn test_connection(&self) -> Result<bool, ApiError>;
}
```

## Adding a New Provider

### 1. Create CLI Wrapper (if CLI-based)

```rust
// src/api/newprovider_cli.rs
pub struct NewProviderCli;

impl NewProviderCli {
    pub async fn is_installed() -> bool { ... }
    pub async fn check_auth() -> Result<bool> { ... }
    pub async fn create_pr(...) -> Result<PullRequestInfo> { ... }
}
```

### 2. Implement PrService

```rust
// src/api/newprovider_service.rs
pub struct NewProviderService {
    cli: NewProviderCli,
    // or api_client for API-based
}

#[async_trait]
impl PrService for NewProviderService {
    async fn get_pr(...) -> Result<PullRequestInfo> { ... }
    // ... other methods
}
```

### 3. Implement RepoProvider

```rust
// src/api/providers/repo/newprovider.rs
pub struct NewProviderProvider { ... }

#[async_trait]
impl RepoProvider for NewProviderProvider { ... }
```

### 4. Add Configuration

```rust
// src/config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct NewProviderConfig {
    pub enabled: bool,
    pub token_env: String,
    pub host: Option<String>,
}
```

### 5. Register Provider

```rust
// src/api/providers/mod.rs
pub fn create_pr_service(config: &Config) -> Box<dyn PrService> {
    match config.git.provider {
        GitProvider::GitHub => Box::new(GitHubService::new()),
        GitProvider::NewProvider => Box::new(NewProviderService::new()),
        // ...
    }
}
```

### 6. Add Tests

```rust
// tests/providers/newprovider_test.rs
#[tokio::test]
async fn test_newprovider_create_pr() { ... }
```

## Terminology Mapping

Different providers use different terminology for similar concepts:

| Concept | GitHub | Other Providers |
|---------|--------|-----------------|
| Code Review Request | Pull Request | Merge Request, Pull Request |
| CI Status | Checks | Pipelines, Builds |
| CI Automation | Actions | CI/CD, Pipelines |
| Approval | Review | Approval, Review |

## Provider Detection

Operator auto-detects the provider from git remote URLs:

```rust
pub fn detect_provider(remote_url: &str) -> Option<GitProvider> {
    if remote_url.contains("github.com") {
        Some(GitProvider::GitHub)
    } else {
        // Future providers can be detected here
        None
    }
}
```

## Shared vs Provider-Specific Config

### Shared Configuration

```toml
[git]
provider = "github"  # Auto-detected if not specified
branch_format = "{type}/{ticket_id}-{slug}"
```

### Provider-Specific

```toml
[git.github]
token_env = "GITHUB_TOKEN"
```

## Testing Guidelines

1. **Unit tests**: Mock CLI output / API responses
2. **Integration tests**: Use test repositories (opt-in, requires tokens)
3. **Mock responses**: Store in `tests/fixtures/providers/`

```rust
#[test]
fn test_parse_pr_response() {
    let json = include_str!("../fixtures/providers/github_pr.json");
    let pr: PullRequestInfo = serde_json::from_str(json).unwrap();
    assert_eq!(pr.state, PrState::Open);
}
```
