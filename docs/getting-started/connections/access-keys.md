---
title: "Access Keys & Identities"
description: "Create, rotate, and revoke Operator API keys, and approve OIDC users."
layout: doc
---

The **Access Keys & Identities** card manages the credentials that clients and people use to reach Operator.

## Operator API keys

Administrators can create a key with `read` and `execute` scopes. Additional scopes must be selected explicitly. MCP clients need `mcp:connect` plus the tool capabilities and projects they may call.

Copy the secret when it's created, because it can't be retrieved later. To rotate a key, create a replacement, update the client, verify the connection, and then revoke the old key. Bearer tokens that were already issued stay valid until they expire.

## Operator identities

Add OIDC connections so users can sign in. A new user is created on first sign-in, waits for approval, and starts with no permissions. Approve the user, then select their profile capabilities and projects. Provider guides: [Authentik](/getting-started/integrations/authentik/), [Keycloak](/getting-started/integrations/keycloak/).

See [Authentication](/security/authentication/) for scopes, the token lifecycle, revocation, and deployment constraints.
