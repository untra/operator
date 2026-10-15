---
title: "Authentik"
description: "Premium OIDC sign-in for Operator users and API keys."
layout: doc
---

Configure an OpenID Connect authorization-code client in Authentik. Enable PKCE (S256), register the exact public HTTPS callback URL `/api/v1/auth/oidc/callback`, and return the `auth_time` claim when `max_age=0` is requested. Discovery, JWKS, and token endpoints must share the issuer origin. Operator refuses redirects and loopback or metadata-service destinations.

In Operator **Connections → Operator identities**, add the issuer, client ID, callback URI, and optional path to a server-mounted client secret file. Keep that file readable only by Operator. A Premium license is required to add connections or expand permissions.

Users sign in from the login page. Their first sign-in creates a pending user identified by issuer and subject. Approve the user and select profile capabilities and projects in Connections. Approved users can create keys after a recent OIDC sign-in.

See [Authentication](/security/authentication/) for lifecycle, permission, revocation, and deployment constraints.
