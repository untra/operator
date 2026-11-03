---
title: "Connections"
description: "Connect clients, editors, and services to a running Operator from the web interface."
layout: doc
---

Open **Connections** in Operator's web interface to reach a running Operator from other tools. The page has one card per connection, and each card's dot shows its last observed health:

| Dot | Meaning |
|-----|---------|
| Green circle | Reachable, or verified |
| Yellow triangle | Available but not yet verified |
| Red triangle | Failed or unreachable |
| Gray circle | Off, or cannot be observed from the browser |

| Card | Connects | Guide |
|------|----------|-------|
| Operator API | REST clients, Swagger, OpenAPI | [Operator API](/getting-started/connections/api/) |
| MCP | Remote MCP clients over SSE | [MCP](/getting-started/connections/mcp/) |
| ACP | Local editors over stdio | [ACP](/getting-started/connections/acp/) |
| Access Keys & Identities | Service keys and OIDC users | [Access Keys & Identities](/getting-started/connections/access-keys/) |

The Operator card also shows the version, the active profile, the control wrapper the operator runs inside, and when the status was observed.

## Behind a reverse proxy

Set `rest_api.public_url` to the externally reachable base URL so Operator advertises absolute endpoints. Without it, MCP advertises relative paths. Preserve SSE streaming through the proxy. The page reports route, authentication, protocol, and timeout failures separately.
