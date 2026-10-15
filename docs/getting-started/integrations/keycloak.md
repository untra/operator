---
title: "Keycloak"
description: "Premium OIDC sign-in for Operator users and API keys."
layout: doc
---

Configure an OpenID Connect authorization-code client in Keycloak. Enable PKCE (S256), register the exact public HTTPS callback URL `/api/v1/auth/oidc/callback`, and return the `auth_time` claim when `max_age=0` is requested. Discovery, JWKS, and token endpoints must share the issuer origin. Operator refuses redirects and loopback or metadata-service destinations.

In Operator **Connections → Operator identities**, add the issuer, client ID, callback URI, and optional path to a server-mounted client secret file. Keep that file readable only by Operator. A Premium license is required to add connections or expand permissions.

Users sign in from the login page. Their first sign-in creates a pending user identified by issuer and subject. Approve the user and select profile capabilities and projects in Connections. Approved users can create keys after a recent OIDC sign-in.

## MCP clients

Remote MCP clients such as AGNT authenticate with a Keycloak access token instead of an Operator key. Operator publishes `/.well-known/oauth-protected-resource` when `[rest_api].public_url` is set and this connection is enabled.

- **Dynamic client registration.** MCP clients register themselves. In the realm, allow anonymous registration from the client's host under **Clients → Client registration → Anonymous access policies → Trusted Hosts**.
- **Audience.** Keycloak ignores the OAuth `resource` parameter. Add an **Audience** mapper to a default client scope with *Included Custom Audience* set to the Operator public URL, for example `https://operator.example.com`. Tokens without that audience are rejected.
- **Signing.** Operator accepts RS256 and ES256 tokens and verifies them against the realm JWKS.
- **Users.** Only users who have signed in to Operator and been approved are accepted. A token never provisions a user.
- **Lifetime.** An MCP stream closes when its token expires. A short *Access Token Lifespan* makes clients reconnect more often.

See [Authentication](/security/authentication/) for lifecycle, permission, revocation, and deployment constraints.
