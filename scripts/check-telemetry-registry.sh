#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

registry_dir="$(mktemp -d)"
trap 'rm -rf "$registry_dir"' EXIT
cp -R telemetry/model/. "$registry_dir/"
cp telemetry/registry_manifest.yaml "$registry_dir/"
"${OPERATOR_WEAVER:-weaver}" registry check --registry "$registry_dir"
