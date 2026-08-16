---
title: "Gitea"
description: "Configure Gitea and delegated Git credentials."
layout: doc
---

Operator uses **Tea 0.13 or newer with the `tea api` command** for Gitea PR operations.
Install Tea on Operator and on each execution target that will create PRs. Operator does not install client binaries.

```toml
[git]
provider = "gitea"

[git.gitea]
enabled = true
host = "https://gitea.kube.untra.casa"
token_env = "GITEA_TOKEN"
wip_prefix = "WIP: "
```

Set `GITEA_TOKEN` in Operator's environment to an existing account's PAT. The token needs repository access and permission to read the authenticated user. Create it at your instance's `/user/settings/applications` page. Private network hosts are supported; external CLI networking is outside Operator's Rust egress policy. Use deployment network controls to restrict destinations.

## Delegator configuration

Each existing named delegator may own its Git settings. Add these sections to its TOML entry, use delegator CRUD, or edit **Git settings** on the Model Providers page:

```toml
[delegators.git.identity]
name = "Operator agent {ticket_id}"
email = "agent-{ticket_id}@example.org"

[delegators.git.credentials]
repository_url = "https://gitea.kube.untra.casa/team/project.git"
username = "operator-agent"
token_env = "PROJECT_AGENT_TOKEN"

[[delegators.git.settings]]
key = "commit.gpgsign"
value = "false"
```

The environment variable holds the secret; the configuration, REST responses, and portable profiles contain only its name. Operator does not mint accounts or tokens. Use an account whose provider permissions match the intended repository scope.

The optional global `[git.identity]` supplies a default author and committer. A delegator's identity replaces that pair. Templates accept `{ticket_id}`, `{project}`, and `{ticket_type}`. With no identity configuration, ambient identity remains in effect.

HTTPS credentials require an HTTPS origin matching `repository_url`; SSH origins are rejected before provisioning. Git receives a repository-bound helper and process-local settings. Shared Git configuration and shared CLI logins are not changed.

Local, SSH, Coder, and Docker launches receive private runtime credentials. Remote credentials travel over SSH stdin before use; Docker receives a read-only runtime mount. Agent commands can use `git push` and `tea pulls create`; the session's Tea configuration uses the delegated account. Operator-side PR creation and monitoring use the captured Git context as well.

Runtime credentials are removed on normal exit and handled termination. Abrupt host failure can leave private credential files; provider tokens remain valid until their administrator revokes them. This is credential isolation between launches, not a sandbox against an agent running as the same OS user.

Draft creation prefixes the title with the configured WIP prefix. Configure this prefix to match the Gitea server. PR reads use the server's `draft` result. Comment association is reported as `NONE` where unavailable.

## Validation

The optional read-only live test uses `OPERATOR_GITPROVIDER_TEST_ENABLED=true`, `OPERATOR_GITPROVIDER_TEST_REPO_GITEA` (an HTTPS URL), `OPERATOR_GITPROVIDER_TEST_PR_GITEA`, and `GITEA_TOKEN`. Run `cargo test --test gitprovider_integration gitea_provider_live`.

Forgejo host detection and provider-independent Git settings are available. Forgejo PR operations and `fj` integration remain deferred.
