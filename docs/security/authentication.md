---
title: "Authentication"
description: "How Operator authenticates: the admin account, bootstrap, browser sessions, OAuth device flow, access keys, scopes, and recovery."
layout: doc
---

Operator's HTTP surface is authenticated **always**.

For trust boundaries and residual risks, see [Security](/security/).

## One human account

Operator has exactly one human account: **`admin`**. This is a deliberate constraint, not a limitation waiting to be lifted.
Operator orchestrates one operator's attention across their projects, and multi-user access control would imply an ownership model the ticket queue does not have.

Everything else that authenticates is a delegated, scoped credential:

| Identity | Kind | How it authenticates |
|----------|------|----------------------|
| `admin` | Human | Password, then a browser session or a device-flow token |
| IDE clients | Delegated | Local token when on the daemon's host; otherwise OAuth device authorization, all four scopes |
| Integrations | Delegated | Service access key exchanged for a short-lived token, only the scopes selected |
| Agent callbacks | Delegated | Single-purpose token minted at launch, pinned to one ticket and step |

Delegated credentials are not additional users. They act on the admin's authority, they carry only the scopes granted, and they can all be revoked individually.

These are distinct from the operating-system and Kubernetes identities, which are separate boundaries: the container runs as Unix user `operator` (UID/GID 10001) and the Kubernetes ServiceAccount is `operator` with no token and no RBAC. Neither has anything to do with logging in.

## Scopes

Four typed scopes, checked per route:

| Scope | Grants |
|-------|--------|
| `read` | Observing the queue, agents, tickets, projects, and issue types |
| `write` | Mutating tickets, issue types, steps, and collections |
| `execute` | Launching agents, completing steps, probing providers, calling MCP tools |
| `admin` | Configuration, delegators, model servers, and authentication administration |

Scopes are not hierarchical. A credential holding `write` does not implicitly hold `read`; it is granted both, or neither.
This makes an integration's grant legible in one glance rather than requiring you to reason about implication.

IDE clients receive all four, because an IDE client *is* the human admin working through a different surface. Integrations receive only what you select.

## Bootstrap

On first start the authentication database does not exist, and Operator is in
the `Uninitialized` state. Only the bootstrap and probe endpoints respond;
everything else returns `401`.

The state machine:

```
Uninitialized ──► AwaitingPassword ──► Complete
     │                   │                 │
     │                   │                 └─ normal operation; login required
     │                   └─ temporary password accepted, new password required
     └─ no admin exists; bootstrap endpoint open
```

Admin creation is **atomic**. The account row is constrained to a single identity.

### Bootstrap in a container

Supply a temporary password out of band and mount it as a read-only file. While the database is uninitialized, Operator reads it and requires the visitor to set a new admin password before anything else works.

### Bootstrap locally

A local run needs none of this. See [local access](#local-access) below.

The first-run setup wizard offers a **Web UI password** step as a shortcut. It is optional and skippable: the terminal and the CLI authenticate without it, and it exists only so the web dashboard is reachable. Skipping it leaves the deployment uninitialized. The step is hidden once an admin account exists.

## Browser sessions

Logging in through the dashboard creates an **opaque server-side session**. The cookie carries a random identifier; all session state lives in the database, so a cookie is not a token and cannot be replayed anywhere else.

The cookie is named `__Host-operator_session` and is set `Secure`, `HttpOnly`, `SameSite=Strict`, `Path=/`. The `__Host-` prefix is enforced by the browser: it refuses the cookie unless it is `Secure`, has no `Domain` attribute, and has `Path=/`. That makes it impossible for a sibling subdomain to set or overwrite the session cookie.

Because a cookie is sent automatically, cookie-authenticated **mutations** additionally require a CSRF token and a matching `Origin`.

Logging out deletes the session server-side. The cookie becoming invalid is a consequence, not the mechanism, so a copied cookie dies with the session.

## Access tokens and refresh tokens

Non-browser clients use bearer tokens.

**Access tokens** are signed bearer tokens with a **15-minute** lifetime. Clients must use the returned expiry and scopes rather than depending on token internals.

Access tokens are not revocable individually, so their blast radius is bounded by expiry rather than by revocation.

**Refresh tokens** are opaque, rotating, and stored only as hashes. Each has a **30-day idle** lifetime and a **90-day absolute** lifetime. Using it resets the idle clock, but the absolute deadline is fixed at issuance, so a continuously refreshed session still requires re-authentication quarterly.

Refresh tokens rotate on every use: redeeming one issues a replacement and retires the original. If a *retired* token is presented again, that means two
parties hold the same token - the legitimate client and a thief. Operator cannot tell which is which, so it **revokes the entire token family**. Both are
logged out, and the admin re-authenticates. This is deliberate: a noisy failure is preferable.

## OAuth device authorization

IDE clients and other public clients, which cannot keep a client secret, use the
OAuth device authorization flow when they cannot use the [local token](#local-access):

1. The client requests a device code and receives a user code and a verification URL.
2. The client opens the browser to that URL; the human approves in an authenticated session.
3. The client polls the token endpoint, honoring the returned interval, until approval completes.

The client never handles the password, and the approval happens in a context
where the human can see what is being authorized.

**VS Code stores refresh credentials in `SecretStorage` only** - never in settings, workspace files, logs, or webview state. Settings sync to other machines and workspace files land in Git; neither is an acceptable home for a credential.

## Service access keys

Integrations authenticate with access keys, which are exchanged at the token endpoint for a short-lived access token. The key itself is never a bearer credential for the API - it buys a token, and the token does the work.

Access keys have:

- **selected scopes**, only what you grant,
- **mandatory expiry** - there is no non-expiring key,
- **hash-only storage** - the secret is displayed exactly once, at creation, and
  cannot be retrieved afterward,
- **revocation** and **last-use tracking**, so a key that stopped being used is
  visible and can be retired.

If a key is lost, create a new one and revoke the old. There is no recovery path, by design.

## Agent callback credentials

When Operator launches an agent, it mints a **single-purpose token** pinned to that ticket, step, and session, and injects it into the agent's environment.The agent's wrapper presents it when reporting step completion.

Its lifetime is tied to the step rather than the standard 15 minutes, because a step may legitimately run for hours and a callback that expires mid-work would strand the agent. The narrow claims are what bound it: the token completes one step of one ticket and is useful for nothing else.

Per the [threat model](/security/#the-agent-process-is-inside-the-trust-boundary), the agent can read this token - as it can read everything else in its environment.

## Rate limiting

Bootstrap, login, device-code creation, device-code polling, and token exchange are all rate-limited with persisted backoff,
so restarting the process does not reset an attacker's budget.

Backoff **never becomes a permanent lockout**. A permanent lockout on a single-account system is a denial-of-service vector against the only human who
can fix it: an attacker who can guess wrong repeatedly could otherwise lock the admin out of their own deployment. Delay grows; the door does not lock.

## Local access

A local `operator` run - the TUI, the CLI, and the agent wrapper talking to loopback - requires no login and no bootstrap.

The VS Code extension host runs as the same user, so when it talks to a loopback daemon it reads the same file and needs no sign-in either; this holds under Remote-SSH too, where the extension host runs on the remote machine. The daemon advertises the state directory in its session file so a non-default `paths.state` is still found. The token is only ever presented to a loopback address.

When Operator binds loopback, it issues itself a local admin credential and writes it to the state directory with **owner-only permissions**. Only the user
account running Operator can read it, which is the same trust boundary a local login would establish.

This is not an authentication bypass. The credential is a real one, checked the same way as any other; it is simply issued automatically to a caller who has already proven, through file ownership, that they are the user who started the process.

## Recovery

Run `operator auth reset-admin-password` only **locally**, against the database file.
It sets a new admin password and revokes every session, refresh-token family, issued token record, and access key.

The dashboard's password-change form also revokes all credentials.

In Kubernetes that means `kubectl exec`, which is itself an audited, RBAC-gated action.

## Audit records

Operator records security-relevant authentication activity without secret
material. In particular, alert on refresh-token reuse: it means a retired token
was presented again and the affected token family was revoked.
