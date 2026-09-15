#!/usr/bin/env bash
# Rev every tracked manifest to a version, then verify each rewrite landed.
# Used by the release job in .github/workflows/build.yaml and for manual bumps.
# Keep the file list in sync with MANAGED in tests/version_parity.rs.
#
#   bump-version.sh              # patch bump of the VERSION file
#   bump-version.sh 1.2.3        # explicit version
#   bump-version.sh --dry-run    # report without writing
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
  shift
fi

CURRENT="$(tr -d '[:space:]' <VERSION)"
if [[ -n "${1:-}" ]]; then
  VERSION="$1"
else
  IFS='.' read -r major minor patch <<<"$CURRENT"
  VERSION="$major.$minor.$((patch + 1))"
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
  echo "usage: ${0##*/} [--dry-run] [<major.minor.patch>]" >&2
  exit 2
fi

echo "Bumping version: $CURRENT -> $VERSION"

# Replace the first line matching a regex. Anchored line rewrites keep an
# indented `version = ` in a dependency table from being mistaken for the
# package version.
replace_first() {
  local file="$1" re="$2" line="$3" tmp
  if $DRY_RUN; then
    echo "[dry-run] $file: $line"
    return
  fi
  tmp="$(mktemp)"
  awk -v re="$re" -v line="$line" \
    '!done && $0 ~ re { $0 = line; done = 1 } 1' "$file" >"$tmp"
  mv "$tmp" "$file"
}

set_json_version() {
  local file="$1" tmp
  if $DRY_RUN; then
    echo "[dry-run] $file: \"version\": \"$VERSION\""
    return
  fi
  tmp="$(mktemp)"
  jq --arg v "$VERSION" '.version = $v' "$file" >"$tmp"
  mv "$tmp" "$file"
}

if $DRY_RUN; then
  echo "[dry-run] VERSION: $VERSION"
else
  echo "$VERSION" >VERSION
fi

for f in Cargo.toml opr8r/Cargo.toml zed-extension/Cargo.toml zed-extension/extension.toml; do
  replace_first "$f" '^version = "' "version = \"$VERSION\""
done

replace_first docs/_config.yml '^version:' "version: $VERSION"
replace_first charts/operator/Chart.yaml '^version:' "version: $VERSION"
replace_first charts/operator/Chart.yaml '^appVersion:' "appVersion: \"$VERSION\""
replace_first vscode-extension/src/webhook-server.ts \
  '^const VERSION = ' "const VERSION = \"$VERSION\";"

for f in vscode-extension/package.json agnt-plugin/package.json agnt-plugin/manifest.json; do
  set_json_version "$f"
done

# Range-scoped so sibling variable defaults in the module are untouched.
if $DRY_RUN; then
  echo "[dry-run] coder-module/main.tf: install_version default = \"$VERSION\""
else
  tmp="$(mktemp)"
  sed "/variable \"install_version\"/,/^}/ s/^\(  default *= *\)\"[^\"]*\"/\1\"$VERSION\"/" \
    coder-module/main.tf >"$tmp"
  mv "$tmp" coder-module/main.tf
fi

if $DRY_RUN; then
  echo "[dry-run] cargo lockfiles"
  exit 0
fi

cargo update --workspace --quiet
(cd opr8r && cargo update --workspace --quiet)
(cd zed-extension && cargo update -p operator-zed --quiet)

# Fail loudly here rather than leaving a half-revved release commit.
failed=0
check() {
  grep -qF "$2" "$1" || {
    echo "bump-version: $1 does not carry $VERSION" >&2
    failed=1
  }
}
check VERSION "$VERSION"
for f in Cargo.toml opr8r/Cargo.toml zed-extension/Cargo.toml zed-extension/extension.toml; do
  check "$f" "version = \"$VERSION\""
done
check docs/_config.yml "version: $VERSION"
check charts/operator/Chart.yaml "version: $VERSION"
check charts/operator/Chart.yaml "appVersion: \"$VERSION\""
for f in vscode-extension/package.json agnt-plugin/package.json agnt-plugin/manifest.json; do
  check "$f" "\"version\": \"$VERSION\""
done
check vscode-extension/src/webhook-server.ts "const VERSION = \"$VERSION\";"
check coder-module/main.tf "default     = \"$VERSION\""

echo "Done. Version is now $VERSION"
exit "$failed"
