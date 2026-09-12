---
title: "Kubernetes"
description: "Run Operator in Kubernetes from the official OCI Helm chart, with persistent workspace state and an authenticated API."
layout: doc
---

<span class="badge alpha">Alpha</span>

Run [Operator](https://operator.untra.io) in a cluster from the official Helm chart.
The chart deploys a single-replica StatefulSet with a persistent workspace volume, a ClusterIP Service.

**Chart:** `oci://ghcr.io/untra/charts/operator` - **Image:** [`untra/operator`](https://hub.docker.com/r/untra/operator)

## What the chart does not contain

Stated up front, because it is the first thing worth knowing about running an agent orchestrator in the cluster:

- **No Docker socket** is mounted.
- **No Kubernetes controller.** Operator does not watch, create, or reconcile cluster resources.
- **No Role, RoleBinding, or ClusterRole** is created.
- The ServiceAccount sets `automountServiceAccountToken: false`, so the pod has no Kubernetes API credential at all.

Operator in a kubernetes cluster is an application with a volume and a port. It cannot reach the Kubernetes API, because it has no token and no client to use one with.

## Install

```bash
helm install operator oci://ghcr.io/untra/charts/operator \
  --namespace operator --create-namespace \
  --set publicUrl=https://operator.example.com
```

The chart's `appVersion` is the image tag. It is pinned to an exact release - the chart never deploys `latest`.

### Bootstrap the admin account

Operator's API is [always authenticated](/security/authentication/). Before installing, create the bootstrap Secret holding a temporary password:

```bash
kubectl -n operator create secret generic operator-bootstrap \
  --from-literal=password="$(openssl rand -base64 24)"
```

Then reference it:

```bash
helm install operator oci://ghcr.io/untra/charts/operator \
  --namespace operator --create-namespace \
  --set publicUrl=https://operator.example.com \
  --set bootstrap.existingSecret=operator-bootstrap
```

The Secret is mounted read-only and is read **only while the authentication database is uninitialized**.
Once you have set the admin password at `/setup`, it is ignored. Delete it afterward:

```bash
kubectl -n operator delete secret operator-bootstrap
```

> **Kubernetes Secrets are not encrypted in etcd by default.** Anyone who can
> read etcd, restore an etcd backup, or `get` Secrets in this namespace can read
> the bootstrap password. Enable
> [encryption at rest](https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/)
> for Secrets and restrict `get`/`list` on them before using this pattern. See
> [Security](/security/#secrets-are-not-encrypted-by-default).

## DNS and TLS

The chart does not manage certificates. Create the TLS Secret independently, or let cert-manager create it, then point the Ingress at it.

With cert-manager:

```yaml
ingress:
  enabled: true
  className: nginx
  host: operator.example.com
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    secretName: operator-tls
```

Without cert-manager, create the Secret from your own certificate:

```bash
kubectl -n operator create secret tls operator-tls \
  --cert=fullchain.pem --key=privkey.pem
```

Ingress is **disabled by default**, and no Gateway API resource is provided.
Until you enable it, Operator is reachable only in-cluster.

Set `publicUrl` to the externally reachable URL whenever you expose Operator. This is required.

> **Setting `publicUrl` also hardens outbound request validation.** A non-empty
> value switches model-server destination checks to the hardened policy, which
> rejects loopback and private ranges (`10.0.0.0/8`, `172.16.0.0/12`,
> `192.168.0.0/16`). If you point Operator at a self-hosted LLM inside the
> cluster or on the LAN, configuring that model server will fail once
> `publicUrl` is set, with an error that does not mention `publicUrl`. Only the
> model-server probe and save paths are affected; Git, kanban, webhook, and
> collection traffic is not.

## Persistence

The StatefulSet claims a **ReadWriteOnce** volume, `20Gi` by default, from the
cluster's default StorageClass:

```yaml
persistence:
  size: 50Gi
  storageClass: fast-ssd
```

The volume is mounted at `/op` and holds the workspace, repositories, `.tickets/`, and the authentication database. It is the only durable state - `$HOME` and `/tmp` are emptyDir mounts and are discarded on every restart.

Two things land here that are easy to overlook, both under `.tickets/operator/`: `ssh/` holds the per-workspace SSH config fragments for [Coder targets](#coder-targets), and `bin/` caches the `coder` CLI when Operator downloads one. Keeping them on the volume is why a pod restart does not re-download the CLI.

**Horizontal scaling is not supported.** Operator is a single-writer process
over a ReadWriteOnce volume with a local queue and a local SQLite database.
`replicas` is fixed at 1; raising it would mean two processes racing over one
volume, and the chart does not offer the option.

## NetworkPolicy

Optional and disabled by default. Enabling it is how you bound Operator's
egress - the code-level destination validation described in
[Security](/security/#server-side-request-forgery) is one control, and this is
the other.

```yaml
networkPolicy:
  enabled: true
  ingress:
    from:
      - namespaceSelector:
          matchLabels:
            kubernetes.io/metadata.name: ingress-nginx
  egress:
    allowDNS: true
    to:
      - ipBlock:
          cidr: 0.0.0.0/0
          except:
            - 169.254.169.254/32   # cloud metadata
            - 10.0.0.0/8
            - 172.16.0.0/12
            - 192.168.0.0/16
```

Operator needs egress to the model provider, kanban provider, and Git host. It does not need egress to the rest of the cluster - unless you use [Coder targets](#coder-targets), which need to reach the Coder deployment.

Note default: with `enabled: true` and an empty `egress.to`, the rendered policy permits DNS. An empty list is deny-all, not allow-all.

## Custom agent images

The base image ships `git`, `tmux`, `openssh-client`, `curl`, and `ca-certificates`, but **no agent CLI** - no `claude`, `codex`, or `gemini`, and no credentials for them.

```dockerfile
FROM untra/operator:0.2.7
USER root
RUN apt-get update && apt-get install -y --no-install-recommends nodejs npm \
 && npm install -g @anthropic-ai/claude-code \
 && rm -rf /var/lib/apt/lists/*
USER 10001
```

```yaml
image:
  repository: registry.example.com/operator-claude
  tag: "0.2.7"
```

Provide the agent's credentials as environment variables from a Secret:

```yaml
extraEnvFrom:
  - secretRef:
      name: operator-agent-credentials
```

Note that an agent process runs as the same user as Operator and can read
these. That is inherent to the current execution model - see
[the trust boundary discussion](/security/#the-agent-process-is-inside-the-trust-boundary).

## Coder targets

Operator can run agents in per-ticket [Coder](/getting-started/platforms/coder/#operator-targeting-coder)
workspaces instead of in its own pod. From a Kubernetes deployment that needs three things.

**1. Credentials, by name.** Operator reads the deployment URL and a user session token from environment variables. Put them in a Secret and reference it - the chart has no dedicated values for this:

```yaml
extraEnvFrom:
  - secretRef:
      name: operator-coder
```

```bash
kubectl -n operator create secret generic operator-coder   --from-literal=CODER_URL=https://coder.example.com   --from-literal=CODER_SESSION_TOKEN=<token>
```

Give Operator its own Coder service account. A session token can create, delete, and SSH into every workspace its user owns.

**2. Egress to Coder.** If `networkPolicy.enabled` is true, add the Coder namespace explicitly:

```yaml
networkPolicy:
  enabled: true
  egress:
    allowDNS: true
    to:
      - namespaceSelector:
          matchLabels:
            kubernetes.io/metadata.name: coder
```

Coder's own ingress NetworkPolicy has to admit Operator's namespace too.

```bash
kubectl -n operator exec operator-0 --   curl -sSf https://coder.example.com/api/v2/buildinfo
```

**3. Nothing else.** The image already ships `openssh-client`, and Operator downloads the `coder` CLI from the deployment on first use, caching it on the persistent volume at `.tickets/operator/bin/coder`. No custom image, no initContainer, and no relaxing of `readOnlyRootFilesystem` - the cache and the SSH fragments both live under `/op`.

## Security context

Applied by default; you should not need to change any of it:

```yaml
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001
  seccompProfile:
    type: RuntimeDefault
containerSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop: ["ALL"]
```

The root filesystem is read-only. Writable paths are the persistent volume at `/op`, plus emptyDir mounts for the home directory and `/tmp`, which tmux and the agent runtime need.

## Upgrades

```bash
helm upgrade operator oci://ghcr.io/untra/charts/operator --reuse-values
```

The StatefulSet uses `RollingUpdate`, but with one replica on a ReadWriteOnce volume the old pod must terminate before the new one attaches.

The authentication database migrates forward automatically on start.

## Backup and restore

Back up the persistent volume. It holds everything: workspace, tickets, state, and `auth.sqlite3`.

Treat the backup as sensitive - it contains the authentication database, which holds the token signing key.

To restore, pre-create the PersistentVolumeClaim the StatefulSet expects, backed by the snapshot, before installing the chart. A StatefulSet adopts an existing claim whose name matches its `volumeClaimTemplate`, which is `workspace-<release>-0`:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: workspace-operator-0
  namespace: operator
spec:
  accessModes: ["ReadWriteOnce"]
  resources:
    requests:
      storage: 20Gi
  dataSource:
    name: operator-snapshot
    kind: VolumeSnapshot
    apiGroup: snapshot.storage.k8s.io
```

Apply that, then `helm install` as normal. The requested size and StorageClass
must match the chart's `persistence` values, or the StatefulSet will reject the existing claim.

Restoring an **old** authentication database resurrects credentials revoked
after the snapshot. If the database is lost or stale, prefer re-bootstrapping:
delete `auth.sqlite3`, supply a fresh bootstrap Secret, and re-issue integration
access keys. That generates a new signing key and invalidates everything issued
previously, which is the safe direction. See [Backup and recovery](/security/#backup-and-recovery).

If you are locked out but the volume is intact, recover locally:

```bash
kubectl -n operator exec -it statefulset/operator -- operator auth reset-admin-password
```

There is no HTTP password-reset route by design.

## Troubleshooting

### The pod starts but every request returns 401

Expected before bootstrap. Visit `/setup` to set the admin password, or confirm
the bootstrap Secret is mounted if you expected it to be consumed.

### Probes fail while the API works

The chart uses **HTTP probes** against `/livez` (liveness) and `/readyz`
(readiness). Both are public and carry no workspace metadata. `/livez` answers as
soon as the server is serving; `/readyz` additionally checks that the
authentication database on the persistent volume is reachable, returning `503`
with `auth store unavailable` when it is not - so a pod stuck `NotReady` with a
healthy `/livez` points at the volume, not the process.

Do not repoint either probe at `/api/v1/health`: that endpoint reports workspace
identity and requires authentication, so an HTTP probe against it fails with `401`
even on a perfectly healthy pod.

### OAuth or MCP URLs point at the wrong host

`publicUrl` is unset or wrong. Operator generates the OAuth device-flow verification URIs and the MCP SSE transport URL from `publicUrl` rather than trusting the `Host` header.

### Agents fail to launch

The base image intentionally omits the agent CLI. Confirm the derived image provides an authenticated `claude`, `codex`, or `gemini` on `PATH`:

```bash
kubectl -n operator exec statefulset/operator -- sh -c 'command -v claude'
```
