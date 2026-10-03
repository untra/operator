# Forgejo Actions: deploy-casa runbook

`workflows/deploy-casa.yaml` builds `Dockerfile.local` with the dev license keyring and swaps the image on the
`operator` release at `operator.<devhost>`. Runners have no standing internet access: everything they pull
comes from Forgejo mirrors that these commands refresh, run from a workstation that does have it.

| Job | Runner label | Reaches |
| --- | --- | --- |
| `build` | `docker` | Forgejo registry; egress allowlist `deb.debian.org`, `registry.npmjs.org`, `index.crates.io`, `static.crates.io` |
| `deploy` | `helmfile` | Forgejo registry, cluster API; no internet |

Common shell setup:

```sh
FORGEJO=https://forgejo.<devhost>
REPO=untra-operator/operator
MIRROR=forgejo.<devhost>/untra-operator/mirror
CHARTS=forgejo.<devhost>/untra-operator/charts
FORGEJO_TOKEN=...   # scopes: write:package, write:repository, write:organization
api() { curl -fsS -H "Authorization: token $FORGEJO_TOKEN" -H 'Content-Type: application/json' "$@"; }
```

## License secrets

`OPERATOR_LICENSE_ROOT_KEYS` must carry the **dev** root only; the prod root belongs to GitHub `untra/operator`.

```sh
api -X PUT "$FORGEJO/api/v1/repos/$REPO/actions/secrets/OPERATOR_LICENSE_ROOT_KEYS" \
  -d "$(jq -cn --arg k "$(cat ~/untra-keys/keys/untra-dev-2026.pub)" '{data: ({"untra-dev-2026": $k} | tojson)}')"
api -X PUT "$FORGEJO/api/v1/repos/$REPO/actions/secrets/OPERATOR_LICENSE_ISSUER" -d '{"data":"operator-licensing"}'
api -X PUT "$FORGEJO/api/v1/repos/$REPO/actions/secrets/OPERATOR_PURCHASE_URL" -d '{"data":"https://app.operator.untra.casa/"}'
api "$FORGEJO/api/v1/repos/$REPO/actions/secrets" | jq -r '.[].name'   # names only; values are write-only
```

## Base image mirror

Re-run whenever a `FROM` in `Dockerfile.local` changes. Digest-pinned refs copy whole, so the pin still resolves; the rest copy arm64 only. `--jobs 1` keeps large layer uploads under the ingress timeout.

```sh
: "${MIRROR:?run the common shell setup first}"
crane auth login forgejo.<devhost> -u <registry-user> --password-stdin <<<"$FORGEJO_TOKEN"
sed -n 's/^FROM \${BASE_REGISTRY}\([^ ]*\).*/\1/p' Dockerfile.local | while read -r ref; do
  case $ref in
    *@*) crane copy --jobs 1 "$ref" "$MIRROR/${ref%@*}" ;;
    *) crane copy --jobs 1 --platform linux/arm64 "$ref" "$MIRROR/$ref" ;;
  esac
done
crane digest "$MIRROR/debian:trixie-slim"   # must equal the sha256 pinned in Dockerfile.local
```

## Chart mirror

Mirror every chart version that helmfile deploys; `deploy` upgrades at the release's current chart version.

```sh
: "${CHARTS:?run the common shell setup first}"
v=$(helm get metadata operator -n operator | awk '/^VERSION:/ {print $2}')
helm pull oci://ghcr.io/untra/charts/operator --version "$v" -d /tmp
helm registry login forgejo.<devhost> -u <registry-user> --password-stdin <<<"$FORGEJO_TOKEN"
helm push "/tmp/operator-$v.tgz" "oci://$CHARTS"
helm show chart "oci://$CHARTS/operator" --version "$v"
```

## Checkout action mirror

A pull mirror refreshes from the Forgejo server, so only the server needs egress to `code.forgejo.org`.
The `actions` org must be public so runners can fetch it without credentials.

```sh
api -X POST "$FORGEJO/api/v1/orgs" -d '{"username":"actions","visibility":"public"}'   # once
api -X POST "$FORGEJO/api/v1/repos/migrate" -d '{"clone_addr":"https://code.forgejo.org/actions/checkout",
  "repo_owner":"actions","repo_name":"checkout","mirror":true,"service":"git"}'          # once
api -X POST "$FORGEJO/api/v1/repos/actions/checkout/mirror-sync"                        # refresh
```

## Dispatch, status and rollback

```sh
api -X POST "$FORGEJO/api/v1/repos/$REPO/actions/workflows/deploy-casa.yaml/dispatches" -d '{"ref":"main"}'
api "$FORGEJO/api/v1/repos/$REPO/actions/runs?limit=1" | jq '.workflow_runs[0] | {id, status, html_url}'
helm history operator -n operator
helm rollback operator <revision> -n operator
```

Job logs are not readable through the API; open the run in the web UI.

## Housekeeping

```sh
api "$FORGEJO/api/v1/repos/$REPO/actions/runners" | jq -r '.[] | "\(.name) \(.status) \(.labels | join(","))"'
api "$FORGEJO/api/v1/packages/untra-operator?type=container&q=operator" | jq -r '.[] | "\(.name):\(.version)"'
api -X DELETE "$FORGEJO/api/v1/packages/untra-operator/container/operator/<old-sha>"
docker buildx prune --filter until=168h --force   # on the docker runner host
```

`buildcache` is overwritten by every build; delete it to force a cold build.
