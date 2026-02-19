#!/bin/bash
set -euo pipefail

# Signs a macOS binary with a Developer ID Application certificate.
# Usage: codesign.sh <binary_path>
#
# Required environment variables:
#   APPLE_CERTIFICATE_P12_BASE64  - base64-encoded .p12 certificate
#   APPLE_CERTIFICATE_PASSWORD    - password for the .p12 file

BINARY_PATH="${1:?Usage: codesign.sh <binary_path>}"

# Unique keychain per invocation (PID avoids collisions)
KEYCHAIN_NAME="signing-$$.keychain-db"
KEYCHAIN_PASSWORD="$(openssl rand -hex 24)"
CERT_PATH="/tmp/codesign-$$.p12"

cleanup() {
  security delete-keychain "$KEYCHAIN_NAME" 2>/dev/null || true
  rm -f "$CERT_PATH"
}
trap cleanup EXIT

# Decode certificate
echo "$APPLE_CERTIFICATE_P12_BASE64" | base64 --decode > "$CERT_PATH"

# Create temporary keychain
security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_NAME"
security set-keychain-settings -lut 21600 "$KEYCHAIN_NAME"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_NAME"

# Add to search list (preserve existing keychains)
ORIGINAL_KEYCHAINS=$(security list-keychains -d user | tr -d '"' | tr '\n' ' ')
security list-keychains -d user -s "$KEYCHAIN_NAME" $ORIGINAL_KEYCHAINS

# Import certificate
echo "Importing certificate ($(stat -f%z "$CERT_PATH") bytes)"
security import "$CERT_PATH" \
  -P "$APPLE_CERTIFICATE_PASSWORD" \
  -A -t cert -f pkcs12 \
  -k "$KEYCHAIN_NAME"

# Allow codesign to access key without UI prompt
security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_NAME"

# Find the Developer ID Application identity
IDENTITY=$(security find-identity -v -p codesigning "$KEYCHAIN_NAME" \
  | grep "Developer ID Application" | head -1 | awk -F'"' '{print $2}')

if [ -z "$IDENTITY" ]; then
  echo "ERROR: No 'Developer ID Application' identity found in keychain" >&2
  security find-identity -v -p codesigning "$KEYCHAIN_NAME"
  exit 1
fi

echo "Signing $BINARY_PATH with identity: $IDENTITY"
codesign --force --options runtime --timestamp \
  --sign "$IDENTITY" \
  --keychain "$KEYCHAIN_NAME" \
  "$BINARY_PATH"

# Verify signature
codesign --verify --verbose=2 "$BINARY_PATH"
echo "Successfully signed: $BINARY_PATH"
