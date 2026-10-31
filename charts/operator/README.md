# Operator Helm chart

This chart installs Operator as a single-replica StatefulSet with a persistent workspace volume and authenticated HTTP API.

```bash
kubectl create namespace operator
kubectl -n operator create secret generic operator-bootstrap \
  --from-literal=password="$(openssl rand -base64 24)"
helm install operator oci://ghcr.io/untra/charts/operator \
  --namespace operator \
  --set bootstrap.existingSecret=operator-bootstrap
kubectl -n operator port-forward service/operator 7008:7008
```

Open `http://127.0.0.1:7008/setup`, set the admin password, and complete the workspace wizard. Delete the temporary bootstrap Secret afterward. The wizard writes `/op/.tickets/operator/config.toml`; the chart does not project or override that file. Back up the workspace PVC.

## Common values

| Value | Default | Purpose |
| --- | --- | --- |
| `image.repository` | `untra/operator` | Container image repository |
| `image.tag` | chart `appVersion` | Exact image version |
| `publicUrl` | empty | External URL used in generated links |
| `persistence.size` | `20Gi` | Workspace PVC size |
| `extraEnv` / `extraEnvFrom` | `[]` | Additional environment configuration |
| `extraVolumes` / `extraVolumeMounts` | `[]` | Additional Secrets, ConfigMaps, and volumes |
| `terminationGracePeriodSeconds` | `90` | Kubernetes termination budget |
| `shutdownDrainSeconds` | `60` | Time allowed for agents to finish |
| `shutdownCleanupSeconds` | `15` | Time reserved for cleanup and persistence |

`terminationGracePeriodSeconds` must be greater than the sum of the two shutdown intervals. A custom `lifecycle` hook consumes the same Kubernetes grace period.

## Upgrading

Charts up to 0.2.14 put version labels on the StatefulSet's immutable `volumeClaimTemplates`, so Kubernetes rejects an upgrade from them to any other chart version. Orphan-delete the StatefulSet once before that upgrade; the pod and the workspace PVC stay in place and the new StatefulSet adopts them:

```bash
kubectl -n operator delete statefulset operator --cascade=orphan
helm upgrade operator oci://ghcr.io/untra/charts/operator --namespace operator --reuse-values
```

## Custom trust and SSH material

Mount a CA bundle that contains both public and internal roots, then configure each client that needs it:

```yaml
extraEnv:
  - name: SSL_CERT_FILE
    value: /etc/operator-ca/ca-bundle.crt
  - name: GIT_SSL_CAINFO
    value: /etc/operator-ca/ca-bundle.crt
extraVolumes:
  - name: operator-ca
    configMap:
      name: operator-ca
extraVolumeMounts:
  - name: operator-ca
    mountPath: /etc/operator-ca
    readOnly: true
```

Verify `operator`, `curl`, and `git` independently because they may use different TLS implementations. Merely mounting a certificate does not add it to the system trust store.

For Git over SSH, mount a Secret containing a private key and pinned `known_hosts`, readable by UID/GID 10001, then set `GIT_SSH_COMMAND` to use both files with `StrictHostKeyChecking=yes`. Do not disable host-key verification.

The chart deliberately does not create RBAC or mount a service-account token. See the Kubernetes guide at https://operator.untra.io/getting-started/platforms/kubernetes/ for ingress, NetworkPolicy, Coder integration, backup, and restore details.

## Telemetry

No collector is bundled and no destination is configured by default. To export
Operator's own signals, configure `telemetry.export.endpoint`. To additionally
accept agent telemetry, set `telemetry.enabled=true`. Enable
`telemetry.enableRedaction=true` to pseudonymize personal attributes and remove
content. Secrets are protected in both modes.

```yaml
telemetry:
  enabled: true
  enableRedaction: true
  export:
    endpoint: http://collector.observability.svc:4318
    headers:
      existingSecret: operator-otlp
      key: headers
  collectorEgress:
    enabled: true
    peers:
      - namespaceSelector:
          matchLabels:
            kubernetes.io/metadata.name: observability
        podSelector:
          matchLabels:
            app: collector
    port: 4318
networkPolicy:
  enabled: true
```

The Secret value is an OTLP header list such as `Authorization=Bearer%20...`.
The chart renders standard `OTEL_*` export variables, JSON logging, and Downward
API resource attributes for the pod and namespace. It reuses the REST service
port and ingress. Collector egress is effective only when NetworkPolicy is
enabled; its peers must be explicit. Existing broad egress rules remain additive.

Use `extraEnv` for per-signal endpoints or `OTEL_SDK_DISABLED=true`. Avoid duplicate
names with chart-managed telemetry variables. `OTEL_SDK_DISABLED` disables
Operator self-export; agent ingest remains controlled independently. The runtime
export configuration is immutable until the process restarts.

### Private model servers

Keep `publicUrl` set. Opt a LAN model host into application egress with
`egress.allowPrivateCidrs: ["192.168.1.50/32"]`. The chart exports a comma-separated `OPERATOR_EGRESS__ALLOW_PRIVATE_CIDRS`.
When NetworkPolicy is enabled, add a rule under `networkPolicy.egress.additionalRules` with the same `ipBlock` and TCP port `1234`. Existing `egress.to` rules allow all ports to their destinations. Routing and the host firewall must permit traffic too.

See the [LM Studio walkthrough](https://operator.untra.io/getting-started/model-servers/lmstudio/) for Windows setup, both CLI delegators, and acceptance checks.
