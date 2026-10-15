---
title: "Authentication"
description: "How Operator authenticates: the admin account, bootstrap, browser sessions, OAuth device flow, access keys, scopes, and recovery."
layout: doc
---

Operator's HTTP surface is authenticated **always**.

For trust boundaries and residual risks, see [Security](/security/).

## Operator identities

The local `admin` account provides bootstrap, configuration, and recovery. Keycloak and Authentik provide additional operator identities through Premium OIDC connections. Users are identified by the immutable `(issuer, subject)` pair. The first sign-in creates a pending account with no permissions; an administrator must approve it and assign profile capabilities and projects.

API keys belong to an approved OIDC user and exactly one profile. Only a browser session authenticated with OIDC within five minutes may create or rotate them. Local passwords, device tokens, callback tokens, and API keys cannot issue API keys. Identity management also requires a browser session. Cookie mutations require an allowed origin and the session CSRF header.

OIDC uses authorization code, S256 PKCE, state, nonce, issuer/audience/signature checks, and recent `auth_time`. The public provider listing, login redirect, and callback are intentionally unauthenticated. Pending login state is bounded, expires after five minutes, and is process-local: use session affinity when routing to multiple processes. Metadata and token HTTP requests require HTTPS, pin resolved addresses, refuse redirects, cap response size and time, and reject loopback and metadata destinations. IdP-side disablement alone does not revoke an existing Operator session; disable the user or connection in Operator.

With `[rest_api].public_url` set and an enabled OIDC connection, Operator is an OAuth protected resource (RFC 9728). The public `GET /.well-known/oauth-protected-resource` returns the public URL and the enabled issuers. API 401 challenges name it in `resource_metadata`. A bearer that is not an Operator token is verified as a provider access token. The checks are: `iss` matches an enabled connection, the signature (RS256/ES256) matches that issuer's JWKS, `aud` contains the public URL, `exp` has not passed, and `(issuer, subject)` is an existing approved user. The token carries that user's profile grants. It never provisions a user. JWKS is fetched under the same egress rules as login metadata, and is refreshed at most once a minute on an unknown key. An MCP stream opened with such a token closes when the token expires, or when the user's grants change or the user is disabled.

## Scopes

Legacy local and device credentials retain four typed scopes, checked per route:

| Scope | Grants |
|-------|--------|
| `read` | Observing the queue, agents, tickets, projects, and issue types |
| `write` | Mutating tickets, issue types, steps, and collections |
| `execute` | Launching agents, completing steps, probing providers, calling MCP tools |
| `admin` | Configuration, delegators, model servers, and authentication administration |

Scopes are not hierarchical. A credential holding `write` does not implicitly hold `read`; it is granted both, or neither.
This makes an integration's grant legible in one glance rather than requiring you to reason about implication.

IDE clients receive all four, because an IDE client *is* the human admin working through a different surface. User-owned API keys use the fine-grained capabilities described below.

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

The dashboard holds its CSRF token in memory only, so a page reload leaves it without one. It fetches a token lazily, on the first mutation that needs it, and concurrent mutations share that single fetch. If a mutation is rejected with `csrf_failed`, the dashboard fetches a fresh token and retries **once**; a second rejection surfaces as an error.

Issuing a token rotates it. The server keeps exactly **one previous token** valid alongside the current one, so a reload in one tab does not break mutations from another tab still holding the old token. A second rotation retires it. Revoking or logging out a session invalidates both tokens at once.

Logging out deletes the session server-side. The cookie becoming invalid is a consequence, not the mechanism, so a copied cookie dies with the session.

## Access tokens and refresh tokens

Non-browser clients use bearer tokens.

**Access tokens** are signed bearer tokens with a **15-minute** lifetime. Clients must use the returned expiry and scopes rather than depending on token internals.

Device-flow access tokens are not revocable individually, so their blast radius is bounded by expiry. Key-derived access tokens additionally require an active key and owner on every request.

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

## Operator API keys

Create and manage your keys in **Connections → Operator API keys**, also available in **Security**. The key secret is shown once and remains only in the current component's memory. Copy it to your client's secret store. It is not recoverable from Operator.

Keys contain 32 OS-random bytes plus a version and public UUID prefix. The database stores SHA-256 of the complete key; list responses contain only metadata and the public prefix. Fast hashing is appropriate for uniformly random 256-bit secrets and avoids password-KDF overhead on exchange. Protect the database, backups, signing key, and OIDC client secret file with host-level controls.

Each key has explicit capabilities, selected projects, an immutable profile ID, and a separate flag permitting selected instance operations. No projects means no project access. Capabilities are stored as normalized rows, not wildcard strings. REST authorization uses an explicit method/path registry; MCP authorization checks the individual tool. Missing rules deny access. Object reads, counts, mutation targets, and batch items also require project access. The effective grant is always the intersection of the key's grant and its owner's current grant. An administrator's key has no administrator bypass. Keys cannot manage credentials, approve devices, or ingest OTLP. Agent telemetry tokens retain their separate ingest audience.

Exchange the key at `/api/v1/auth/token` with `grant_type: "operator:access-key"`. The returned JWT carries owner, key, and profile provenance; it has no refresh token and expires within 15 minutes or at key expiry, whichever comes first. Every key-derived request checks authoritative database state. This costs a database read per request but eliminates stale authorization caches and propagates revocation across processes on the next authorization check. Already executing operations may finish. MCP sessions bind the originating credential, check before delivery, and close revoked idle streams within the 15-second keepalive interval.

Expiry is required: the UI defaults to 90 days and the maximum is 365 days. Rotation atomically inserts a replacement, revokes the old key, and records an audit event; only one concurrent rotation succeeds, and permissions cannot expand. For overlap, create a separate replacement, switch clients, then revoke the original. Expired Premium leaves existing keys working and permits revocation, but blocks new keys, rotation, new connections, and permission expansion.

Last-used timestamps record successful credential authentication, including requests later denied authorization. Internal MCP validity checks do not count as use. Timestamps are approximate. Observations coalesce by credential with a maximum timestamp, flush every 60 seconds, and never move the stored value backwards. The buffer is bounded at 10,000 credentials; failed flushes merge observations back subject to that bound. Shutdown makes a best-effort flush. This avoids a write per request; crashes and capacity pressure can lose usage observations, which are never used for authorization.

Token exchange is limited to 60 requests per minute per connection IP. Other bearer-authenticated API requests are limited to 1,200 per peer before credential validation; key requests to 600 per minute per key and 1,200 per owner. Forwarded headers are not trusted. Runtime buckets are bounded and shared across profiles in one process; separate processes have separate budgets. Key issuance additionally has a database-persisted limit of 20 per owner per hour. Rate limits trade burst capacity for resistance to guessing and resource exhaustion; configure an edge limit for deployments requiring a global quota across replicas.

Lifecycle audit records commit with credential mutations and identify the actor, owner, key, and profile. Creation and rotation also record the granted permissions. Exchange records identify the key and its owner. Telemetry export follows commit and may lose an export on process failure; the database is authoritative. Raw `opk_` secrets are scrubbed even when optional content redaction is disabled. HTTP spans omit query strings so OIDC callback codes do not enter request traces. Secret-bearing responses use `Cache-Control: no-store`.

If a creation or rotation response is lost, refresh the key list and revoke the new credential before trying again. Operator never retrieves or replays a secret. Rotation requires confirmation and can only retain or reduce permissions. Restoring an owner's permissions may restore an unexpired key's original authority; revoke the key for permanent withdrawal.

Credential rejection counts are aggregated without user or key labels. With metrics export enabled, `operator.auth.authentication.denied`, `operator.auth.authorization.denied`, and `operator.auth.rate_limited` count rejected attempts; `operator.auth.usage.dropped` counts usage-buffer losses and `operator.auth.usage.flush_failed` counts failed database batches. These process-lifetime counters add atomic increments without database writes. Telemetry export failures remain separate from authoritative audit records: `operator.telemetry.dropped` reports forwarding drops, and the SDK's `otel.sdk.processor.log.processed` metric reports batch-processor losses, including `queue_full`. Export outages can delay or lose the metrics themselves.

The identity migration preserves the telemetry installation key, revokes legacy ownerless API keys, and refuses incompatible draft schemas. Legacy API JWTs without credential version/provenance must be replaced; back up the authentication database before upgrading. Scope checks do not sandbox launched code: grant execution and configuration capabilities only to trusted operators.

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
