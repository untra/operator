# Connect Kanban Provider

Operator will sync tickets from [**Jira Cloud**](https://www.atlassian.com/software/jira) or [**Linear**](https://linear.app/features) to your local queue.

## [Jira Setup](https://operator.untra.io/getting-started/kanban/jira/)

Set these environment variables in your shell profile:

```bash
export OPERATOR_JIRA_API_KEY="your-api-token"
export OPERATOR_JIRA_EMAIL="your-email@example.com"
export OPERATOR_JIRA_URL="https://your-domain.atlassian.net"
```

Get an API token from [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens).

## [Linear Setup](https://operator.untra.io/getting-started/kanban/linear/)

Set this environment variable:

```bash
export OPERATOR_LINEAR_API_KEY="lin_api_xxxxxxxxxxxxx"
```

Get an API key from [Linear API Settings](https://linear.app/settings/api).

## After Setup

1. Add the variables to `~/.zshrc` or `~/.bashrc`
2. Restart VS Code
3. Click **Check Connection** to verify

This step is optional - you can also create tickets manually.
