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
