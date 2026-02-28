#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
fi

# Read current version from source of truth
CURRENT="$(tr -d '[:space:]' < VERSION)"
IFS='.' read -r major minor patch <<< "$CURRENT"
NEW="$major.$minor.$((patch + 1))"

echo "Bumping version: $CURRENT -> $NEW"

# Portable first-occurrence replacement using awk
# Replaces only the first line containing the old string
replace_first() {
  local file="$1" old="$2" new="$3"
  awk -v old="$old" -v new="$new" \
    '!done && index($0, old) { sub(old, new); done=1 } 1' \
    "$file" > "$file.tmp" && mv "$file.tmp" "$file"
}

# Text files: replace first occurrence of version string
TEXT_FILES=(
  "VERSION"
  "Cargo.toml"
  "opr8r/Cargo.toml"
  "vscode-extension/src/webhook-server.ts"
  "docs/_config.yml"
)

# JSON files: update .version via jq
JSON_FILES=(
  "vscode-extension/package.json"
  "backstage-server/package.json"
)

for f in "${TEXT_FILES[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "WARNING: $f not found, skipping"
    continue
  fi
  if $DRY_RUN; then
    echo "[dry-run] would update $f"
  else
    replace_first "$f" "$CURRENT" "$NEW"
    echo "Updated $f"
  fi
done

for f in "${JSON_FILES[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "WARNING: $f not found, skipping"
    continue
  fi
  if $DRY_RUN; then
    echo "[dry-run] would update $f"
  else
    jq --arg v "$NEW" '.version = $v' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    echo "Updated $f"
  fi
done

echo ""
echo "Done. Version is now $NEW"
