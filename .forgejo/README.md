# Forgejo Actions runbook

`workflows/deploy-casa.yaml` builds `Dockerfile.local` with the dev license keyring and swaps the image on the
`operator` release at `operator.<devhost>`. `workflows/pr.yaml` is the required check on `main`: it builds the
same arm64 image without pushing it, then runs the `e2e/` Playwright suites against it (`scripts/ci/e2e.sh`).
Runner egress isn't restricted (jobs reach the internet through OrbStack's NAT), but everything they pull comes
from Forgejo mirrors anyway, for pinning and speed. These commands refresh the mirrors from a workstation.

| Workflow / job | Runner label | Reaches |
| --- | --- | --- |
| `deploy-casa` / `build` | `docker` | Forgejo registry and mirrors; `deb.debian.org`, `registry.npmjs.org`, `index.crates.io`, `static.crates.io` at build time |
| `deploy-casa` / `deploy` | `helmfile` | Forgejo registry, cluster API |
| `pr` / `arm64` | `docker` | as `build`, plus `webhook.operator.untra.casa` for the license suite |

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

Re-run whenever a `FROM` in `Dockerfile.local` or `e2e/Dockerfile` changes; the Playwright tag tracks
`@playwright/test` in `e2e/package.json`. Digest-pinned refs copy whole, so the pin still resolves; the rest copy arm64 only. `--jobs 1` keeps large layer uploads under the ingress timeout.

```sh
: "${MIRROR:?run the common shell setup first}"
crane auth login forgejo.<devhost> -u <registry-user> --password-stdin <<<"$FORGEJO_TOKEN"
sed -n 's/^FROM \${BASE_REGISTRY}\([^ ]*\).*/\1/p' Dockerfile.local e2e/Dockerfile | sort -u | while read -r ref; do
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

## E2E suites

`pr` runs both projects in `e2e/playwright.config.ts`: `workspace` (admin setup and onboarding on a fresh server)
and `license` (a test-mode key from the platform webhook's `GET /e2e/license`, applied to that server's
configuration). The license suite skips while `OPERATOR_E2E_LICENSE_TOKEN` is unset, so leave it unset until the
casa webhook serves the route. Its value is the casa webhook's `E2E_LICENSE_TOKEN`.

```sh
api -X PUT "$FORGEJO/api/v1/repos/$REPO/actions/secrets/OPERATOR_E2E_LICENSE_TOKEN" \
  -d "$(jq -cn --arg v "$E2E_LICENSE_TOKEN" '{data: $v}')"
api -X PUT "$FORGEJO/api/v1/repos/$REPO/actions/secrets/OPERATOR_E2E_CA_CERT" \
  -d "$(jq -cn --arg v "$(cat casa-root-ca.pem)" '{data: $v}')"   # only if the runner does not already trust the casa CA
```

Run the same thing locally with `make e2e`; pass the two `OPERATOR_E2E_*` variables to include the license suite.

## Required check on main

The status context is `<workflow name> / <job name> (<event>)`. Confirm it on the first PR's commit status
before requiring it, because a misspelled context blocks every merge.

```sh
api "$FORGEJO/api/v1/repos/$REPO/commits/<pr-head-sha>/statuses" | jq -r '.[].context'   # expect: pr / arm64 (pull_request)
api -X POST "$FORGEJO/api/v1/repos/$REPO/branch_protections" -d '{"rule_name":"main",
  "enable_status_check":true,"status_check_contexts":["pr / arm64 (pull_request)"]}'
# rule already exists:
api -X PATCH "$FORGEJO/api/v1/repos/$REPO/branch_protections/main" -d '{"enable_status_check":true,
  "status_check_contexts":["pr / arm64 (pull_request)"]}'
```

## Dispatch, status and rollback

```sh
api -X POST "$FORGEJO/api/v1/repos/$REPO/actions/workflows/deploy-casa.yaml/dispatches" -d '{"ref":"main"}'
api "$FORGEJO/api/v1/repos/$REPO/actions/runs?limit=1" | jq '.workflow_runs[0] | {id, status, html_url}'
helm history operator -n operator
helm rollback operator <revision> -n operator
```

Job logs are not readable through the API; open the run in the web UI.

Roll images through this workflow or `helm rollback`, not `kubectl set image`: `deploy` runs with `--force-conflicts`, so the next run overwrites out-of-band edits.

## Housekeeping

```sh
api "$FORGEJO/api/v1/repos/$REPO/actions/runners" | jq -r '.[] | "\(.name) \(.status) \(.labels | join(","))"'
api "$FORGEJO/api/v1/packages/untra-operator?type=container&q=operator" | jq -r '.[] | "\(.name):\(.version)"'
api -X DELETE "$FORGEJO/api/v1/packages/untra-operator/container/operator/<old-sha>"
docker buildx prune --filter until=168h --force   # on the docker runner host
```

`buildcache` is overwritten by every build; delete it to force a cold build.
