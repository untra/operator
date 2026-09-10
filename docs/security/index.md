---
title: "Security"
description: "Threat model and security architecture for Operator: trust boundaries, route classification, credential handling, and residual risks."
layout: doc
---

Operator launches AI coding agents against your source code, holds credentials for kanban and model providers, and exposes a REST API, a web dashboard, and an MCP server.

This page is the threat model for that surface: what Operator trusts, what it does not, and what remains your responsibility to control.

For the authentication mechanism itself — accounts, tokens, scopes, and recovery — see [Authentication](/security/authentication/).

## Trust boundaries

Operator sits inside four nested boundaries. Each one is enforced by a different mechanism, and each has a different failure mode.

| Boundary | Enforced by | What crossing it means |
|----------|-------------|------------------------|
| Host ↔ container | Container runtime, non-root UID/GID 10001, read-only root filesystem, dropped capabilities | A compromised process inside the container cannot write the host filesystem or escalate to root |
| Cluster ↔ pod | NetworkPolicy, no mounted ServiceAccount token, no RBAC | Operator cannot call the Kubernetes API, because it has no credential to call it with |
| Network ↔ Operator | Authentication, typed scopes, CORS, Host validation | Every request carries a principal and a scope, or it is rejected |
| Operator ↔ agent process | **Nothing.** See below. | An agent process runs as the same user, with the same filesystem access, as Operator itself |

### The agent process is inside the trust boundary

Operator and the agent processes it spawns share **one Unix identity and one filesystem**. The non-root user protects the host and the cluster from a
compromised agent. It does not protect *Operator's own state* from that agent.

An agent process can read and write:

- the workspace and every repository in it,
- `.tickets/` in full, including the queue, ticket bodies, and `state.json`,
- `config.toml`,
- the authentication database and the local session token,
- any environment variable Operator passed it, including provider API keys.

Local execution shares the Operator OS user. SSH and Coder targets can isolate the agent filesystem when their remote environments do not mount Operator state.

The practical consequence: **treat the agent tool you configure, and the model provider behind it, as trusted components.**

Operator's authentication protects the boundary between the network and Operator. It does not sandbox the agent.

## Route classification

Every HTTP route falls into exactly one of five classes. The mapping is held in a single table in the source and is verified by a test that walks the generated OpenAPI specification, so a new route cannot be added without being classified.

| Class | Requirement | Examples |
|-------|-------------|----------|
| Probe | Public | `/livez`, `/readyz` |
| Bootstrap / login | Public, rate-limited | Bootstrap status and submission, login, OAuth device-code and token endpoints |
| Read | `read` scope | Queue, agents, tickets, projects, issue types, collections, health, status |
| Write | `write` scope | Ticket creation and status changes, issue type and step edits, collection activation, kanban sync |
| Execute | `execute` scope | Ticket launch, step completion, model-provider probes, MCP tool calls, session focus |
| Admin | `admin` scope | Configuration read and write, delegator and model-server management, session and access-key administration |

Two deliberate choices in that table:

**Configuration is `admin`, not `write`.** The REST API exposes a deliberately narrow operational projection rather than the full internal configuration but changing launch behavior and resource limits still requires administrative
authority. 
Model servers, delegators, and execution targets have focused endpoints with their own response types.

**Health and status are not public.** They report the workspace directory name and a directory identifier. That is workspace identity, and it is exactly the sort of detail a public probe should not disclose — hence the separate, metadata-free `/livez` and `/readyz` endpoints for Kubernetes.

### The dashboard bundle is public

Operator's web dashboard is a single-page application using fragment-based
routing. Route names such as `#/config` live in the URL *fragment*, which
browsers never transmit to the server. The server therefore cannot distinguish
a request for the login screen from a request for any other screen: it serves
one HTML document and one JavaScript bundle for all of them.

Consequently the dashboard bundle is served publicly, and authorization is
enforced entirely at the API layer. An unauthenticated visitor can load the
shell; every data request returns `401`, and the client redirects to the login
screen.

**Residual risk:** the set of route names and the structure of the UI are
public. No workspace data, configuration, or credentials are in the bundle —
all of it arrives over authenticated API calls — but the shape of the
application is discoverable. This is accepted deliberately; the alternative is
a separately served login document, which is a larger change for a small
reduction in disclosure.

## Credential handling

### Credentials Operator holds

Operator stores **no third-party secret values** in `config.toml`. Configuration
holds the *name* of an environment variable, and the value is read from the
process environment. A configuration export therefore contains integration
topology, not credentials.

### Credentials Operator issues

The authentication database at `.tickets/operator/auth.sqlite3` is created owner-readable only and holds password verification data, token-signing material, sessions, and hashes of refresh tokens and access keys. It is identity-critical state and must be protected like a credential store.

Plaintext passwords, temporary bootstrap passwords, device codes, refresh tokens, and access keys are never written to disk or logs.

### Credentials Operator passes to agents

Launching an agent injects environment variables into the agent's process, including a short-lived callback credential and whatever provider keys the configured tool needs. Per the trust-boundary discussion above, the agent can read all of them.

## Server-side request forgery

Several Operator features fetch a URL that the caller or the configuration
controls:

- model-server reachability and model-listing probes, which attach the provider's API key to the request,
- outbound notification webhooks,
- hosted-collection fetches, where the fetched manifest itself supplies subsequent relative URLs,
- kanban provider hosts, where the host is a configuration key.

Untreated, the model-server probe is the sharpest of these: an authenticated
caller sets a base URL, triggers a probe, and Operator makes the request *with a
provider API key attached*. Redirects compound it — a permitted host can
redirect to a forbidden one.

Four controls apply together, and none is sufficient alone:

1. **Authentication and scopes** — probing requires `execute`; changing a
   model-server URL requires `admin`. An anonymous caller cannot reach either.
2. **Destination validation** — loopback, link-local, multicast, and cloud-metadata addresses are rejected unless explicitly allowed, and schemes and CIDR ranges are validated against configuration.
3. **Redirect re-validation** — every redirect hop is re-checked against the
   same policy, not just the initial URL.
4. **NetworkPolicy** — in Kubernetes, egress is restricted at the network
   layer, so a validation bug does not become cluster-internal access.

Control 2 is code, control 4 is cluster configuration, and **you must configure control 4 yourself**; the chart ships the template but leaves it disabled by default.

## Kubernetes exposure

The chart is deliberately minimal about what it can touch:

- **No Docker socket** is mounted.
- **No Kubernetes controller** ships in the image, and no Kubernetes client tooling is installed.
- **No Role, RoleBinding, or ClusterRole** is created.
- The ServiceAccount has `automountServiceAccountToken: false`, so no API token is present in the pod at all.

Operator running in your cluster cannot enumerate, create, or delete cluster resources, because it has neither the credential nor the tooling to try.

Ingress is disabled by default. Enabling it publishes an authenticated service, which is the intended posture — but it is your TLS certificate, your DNS name, and your decision.

### Secrets are not encrypted by default

Kubernetes Secrets are stored **unencrypted** in etcd unless you have enabled encryption at rest.
Anyone who can read etcd, take an etcd backup, or `get` Secrets in the namespace can read the bootstrap password you supply.

Before putting a bootstrap Secret in a cluster:

- enable [encryption at rest](https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/) for Secret resources.
- restrict `get`/`list` on Secrets in the namespace to the smallest possible set of principals.
- delete the bootstrap Secret once the admin password has been set.

See the [Kubernetes guide](/getting-started/platforms/kubernetes/) for the mechanics.

## Backup and recovery

`auth.sqlite3` is **identity-critical state**. It holds credential and signing state, so:

- Backing it up preserves every issued token's validity. Treat a backup with
  the same care as the live database.
- Losing it is not catastrophic but is disruptive: recovery means re-bootstrapping, which generates a new signing key and invalidates every
  session, refresh token, and access key. Integrations must be re-issued keys.
- Restoring an *old* copy resurrects credentials that were revoked after the backup was taken. Prefer re-bootstrapping over restoring a stale database.

The persistent volume also holds the workspace, the ticket queue, and
`state.json`. A backup that captures the volume captures all of it, including
the authentication database — so the volume snapshot inherits the same
sensitivity.

Password recovery is **local only**: `operator auth reset-admin-password` operates directly on the database.
It is never exposed as an HTTP route, so there is no network-reachable password-reset path to attack.

## Residual risks

Recorded deliberately, in rough order of significance.

1. **Local agents share Operator filesystem access.** SSH and Coder targets can place agents on separate filesystems, provided the target does not mount Operator state. Same-user processes can still read private Git runtime files: per-launch credentials prevent accidental identity mixing, not hostile same-user access.
2. **The dashboard bundle is public**, so UI structure and route names are discoverable. No data or credentials are exposed.
3. **A compromised authentication database yields the signing key**, allowing
   token forgery until the key is rotated by re-bootstrapping. File permissions and volume access control are the only barriers.
4. **The admin account is a single point of authority.** There is one human
   account by design; there is no separation of duties and no second approver.
5. **Integration access keys are bearer credentials.** Anyone holding one has
   its scopes until it expires or is revoked. Keys have mandatory expiry and last-use tracking so an unused key is visible, but there is no proof of possession.
6. **Egress is unrestricted unless you restrict it.** The destination
   validation above blocks the well-known dangerous targets, but Operator is designed to call third-party APIs; NetworkPolicy is what bounds that.
