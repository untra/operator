#!/usr/bin/env bash
# Regenerate the ts-rs TypeScript bindings and fail if the result differs from
# what is on disk (in CI, the committed checkout). A modified type changes its
# file's hash; a newly exported type adds a line to the "after" manifest.
# ts-rs never deletes files, so a removed export is undetectable by any
# before/after comparison.
#
# Called by .github/workflows/build.yaml (lint-test) and scripts/cicdprep.sh
# so CI and the local pre-flight can never disagree about freshness.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

manifest() { find bindings -type f -name '*.ts' -exec shasum {} + | sort; }

before="$(manifest)"
cargo test --locked export_bindings_
after="$(manifest)"

if [ "$before" != "$after" ]; then
  echo "bindings/ is out of date. Run 'make bindings' and commit the result." >&2
  diff <(echo "$before") <(echo "$after") >&2 || true
  exit 1
fi
