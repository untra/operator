#!/usr/bin/env bash
# Run the e2e/ Playwright suites against an operator image in throwaway containers.
# Talks to the daemon only through the API (build, cp, start), never bind mounts, so it works under dind runners.
# The runner shares the server's network namespace: the Secure session cookie needs http://localhost.
#
# Usage: scripts/ci/e2e.sh <operator-image> [playwright args...]
# Env:   BASE_REGISTRY (mirror prefix for the Playwright image), E2E_ARTIFACTS (copy the HTML report here),
#        OPERATOR_E2E_ISSUER_URL, OPERATOR_E2E_LICENSE_TOKEN (license suite; skipped when unset),
#        OPERATOR_E2E_CA_CERT (PEM trusted for the issuer, e.g. the casa internal CA)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IMAGE="${1:?usage: e2e.sh <operator-image> [playwright args...]}"
shift

ID="${E2E_RUN_ID:-$$}"
SERVER="operator-e2e-$ID"
RUNNER="operator-e2e-runner-$ID"
RUNNER_IMAGE="operator-e2e-runner:$ID"
SCRATCH="$(mktemp -d)"

cleanup() {
  echo "::group::operator server logs"
  docker logs "$SERVER" 2>&1 || true
  echo "::endgroup::"
  docker rm -f "$RUNNER" "$SERVER" >/dev/null 2>&1 || true
  docker rmi "$RUNNER_IMAGE" >/dev/null 2>&1 || true
  rm -rf "$SCRATCH"
}
trap cleanup EXIT

docker build \
  --build-arg BASE_REGISTRY="${BASE_REGISTRY:-}" \
  --tag "$RUNNER_IMAGE" \
  "$ROOT_DIR/e2e"

od -An -tx1 -N16 /dev/urandom | tr -d ' \n' >"$SCRATCH/bootstrap"
chmod 0644 "$SCRATCH/bootstrap"

docker create --name "$SERVER" \
  --env OPERATOR_BOOTSTRAP_PASSWORD_FILE=/home/operator/bootstrap \
  "$IMAGE" api >/dev/null
docker cp "$SCRATCH/bootstrap" "$SERVER:/home/operator/bootstrap"
docker start "$SERVER" >/dev/null

ca_env=()
if [ -n "${OPERATOR_E2E_CA_CERT:-}" ]; then
  printf '%s\n' "$OPERATOR_E2E_CA_CERT" >"$SCRATCH/ca.pem"
  ca_env=(--env NODE_EXTRA_CA_CERTS=/tmp/issuer-ca.pem)
fi

docker create --name "$RUNNER" \
  --network "container:$SERVER" \
  --env CI=1 \
  --env OPERATOR_E2E_BASE_URL=http://localhost:7008 \
  --env OPERATOR_E2E_BOOTSTRAP_PASSWORD="$(cat "$SCRATCH/bootstrap")" \
  --env OPERATOR_E2E_ISSUER_URL="${OPERATOR_E2E_ISSUER_URL:-}" \
  --env OPERATOR_E2E_LICENSE_TOKEN="${OPERATOR_E2E_LICENSE_TOKEN:-}" \
  ${ca_env[@]+"${ca_env[@]}"} \
  "$RUNNER_IMAGE" "$@" >/dev/null
if [ -f "$SCRATCH/ca.pem" ]; then
  docker cp "$SCRATCH/ca.pem" "$RUNNER:/tmp/issuer-ca.pem"
fi

status=0
docker start --attach "$RUNNER" || status=$?

if [ -n "${E2E_ARTIFACTS:-}" ]; then
  mkdir -p "$E2E_ARTIFACTS"
  docker cp "$RUNNER:/e2e/playwright-report" "$E2E_ARTIFACTS/" 2>/dev/null || true
  docker cp "$RUNNER:/e2e/test-results" "$E2E_ARTIFACTS/" 2>/dev/null || true
fi

exit "$status"
