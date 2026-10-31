#!/bin/bash
set -euo pipefail

# Usage: notarize.sh <binary_path> [binary_path ...]
: "${1:?Usage: notarize.sh <binary_path> [binary_path ...]}"
NOTARY_DIR="$(mktemp -d)"
ZIP_PATH="$NOTARY_DIR/binaries.zip"
KEY_PATH="$NOTARY_DIR/AuthKey_${APPLE_NOTARY_KEY_ID}.p8"
RESULT_PATH="$NOTARY_DIR/result.json"

cleanup() {
  rm -rf "$NOTARY_DIR"
}
trap cleanup EXIT

mkdir "$NOTARY_DIR/binaries"
for binary in "$@"; do
  destination="$NOTARY_DIR/binaries/$(basename "$binary")"
  if [ -e "$destination" ]; then
    echo "Duplicate binary filename: $binary" >&2
    exit 1
  fi
  cp "$binary" "$destination"
done
echo "$APPLE_NOTARY_KEY_BASE64" | base64 --decode > "$KEY_PATH"
ditto -c -k --keepParent "$NOTARY_DIR/binaries" "$ZIP_PATH"

echo "Submitting $# binaries for notarization..."
if ! xcrun notarytool submit "$ZIP_PATH" \
  --key "$KEY_PATH" \
  --key-id "$APPLE_NOTARY_KEY_ID" \
  --issuer "$APPLE_NOTARY_ISSUER_ID" \
  --wait \
  --timeout 30m \
  --output-format json > "$RESULT_PATH"; then
  cat "$RESULT_PATH"
  exit 1
fi
cat "$RESULT_PATH"
if ! jq -e '.status == "Accepted"' "$RESULT_PATH" > /dev/null; then
  echo "Notarization was not accepted" >&2
  exit 1
fi

echo "Notarization complete"
