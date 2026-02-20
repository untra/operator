#!/bin/bash
set -euo pipefail

# Notarizes a signed macOS binary via App Store Connect API key.
# Usage: notarize.sh <binary_path>
#
# Required environment variables:
#   APPLE_NOTARY_KEY_BASE64  - base64-encoded .p8 API key
#   APPLE_NOTARY_KEY_ID      - API key ID
#   APPLE_NOTARY_ISSUER_ID   - API issuer UUID

BINARY_PATH="${1:?Usage: notarize.sh <binary_path>}"
ZIP_PATH="${BINARY_PATH}.zip"
KEY_DIR="/tmp/notary-keys-$$"
KEY_PATH="$KEY_DIR/AuthKey_${APPLE_NOTARY_KEY_ID}.p8"

cleanup() {
  rm -f "$ZIP_PATH"
  rm -rf "$KEY_DIR"
}
trap cleanup EXIT

# Decode API key
mkdir -p "$KEY_DIR"
echo "$APPLE_NOTARY_KEY_BASE64" | base64 --decode > "$KEY_PATH"

# Zip the signed binary (notarytool requires .zip, .dmg, or .pkg)
ditto -c -k --keepParent "$BINARY_PATH" "$ZIP_PATH"

echo "Submitting $BINARY_PATH for notarization..."
xcrun notarytool submit "$ZIP_PATH" \
  --key "$KEY_PATH" \
  --key-id "$APPLE_NOTARY_KEY_ID" \
  --issuer "$APPLE_NOTARY_ISSUER_ID" \
  --wait \
  --timeout 30m

echo "Notarization complete: $BINARY_PATH"
# Note: xcrun stapler staple does NOT work on standalone Mach-O binaries.
# macOS checks the notarization ticket online via Gatekeeper.
