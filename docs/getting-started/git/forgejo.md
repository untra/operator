---
title: "Forgejo"
description: "Configure Forgejo Git and pull-request operations."
layout: doc
---

Forgejo Git support is **Alpha**. Operator uses Tea 0.13 or newer with `tea api` for pull-request creation, reads, reviews, and comments. Install Tea on Operator and every execution target that creates pull requests. Forgejo has its own provider identity and credentials; the compatible Git transport is shared with Gitea.

```toml
[git]
provider = "forgejo"

[git.forgejo]
enabled = true
host = "https://codeberg.org"
token_env = "FORGEJO_TOKEN"
wip_prefix = "WIP: "
```

For a self-hosted deployment, set `host` to its base URL. The default is Codeberg. Create a personal access token at your instance's `/user/settings/applications` page and set `FORGEJO_TOKEN` in Operator's environment.

Use `read:user` for authentication, `write:repository` for pull requests, and `write:issue` for issue comments. The account also needs repository write permission. Read-only PR monitoring can use `read:repository` and `read:issue`. Forgejo's specific-repository tokens do not allow `read:user`, so they cannot satisfy Operator's authenticated-user onboarding probe. Choose the appropriate public/private access restriction with these scopes. See [Forgejo token scopes](https://forgejo.org/docs/latest/user/authentication/token-scope/).

Delegated Git identity and repository-bound HTTPS credentials follow the [Gitea Git setup](/getting-started/git/gitea/). They use the configured Forgejo host and token; Operator does not require `fj` or modify shared Tea logins. WIP title prefixes must match the server configuration.

A missing Tea binary/API command is an installation error; HTTP 401 indicates invalid credentials, while 403 indicates insufficient scope or repository permission. No token value is saved in TOML or returned by configuration endpoints.

[Forgejo Projects](/getting-started/kanban/forgejo/) is a separate Proto integration. Git support does not imply project-board API support.
