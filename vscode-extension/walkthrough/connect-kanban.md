# Connect Kanban Provider

Operator will sync tickets from [**Jira Cloud**](https://www.atlassian.com/software/jira) or [**Linear**](https://linear.app/features) to your local queue.

## Interactive Setup

Click **Configure Now** above to walk through connecting your kanban provider. The setup will:

1. Collect your credentials (domain, email, API token/key)
2. Validate them against the live API
3. Let you pick a project or team to sync
4. Write the config to `~/.config/operator/config.toml`
5. Set environment variables for the current session

## Manual Setup (Advanced)

If you prefer to configure manually, set these environment variables in your shell profile:

### [Jira Setup](https://operator.untra.io/getting-started/kanban/jira/)

```bash
export OPERATOR_JIRA_API_KEY="your-api-token"
```

Get an API token from [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens).

### [Linear Setup](https://operator.untra.io/getting-started/kanban/linear/)

```bash
export OPERATOR_LINEAR_API_KEY="lin_api_xxxxxxxxxxxxx"
```

Get an API key from [Linear API Settings](https://linear.app/settings/api).

## After Setup

This step is optional - you can also create tickets manually.
