#!/bin/bash
set -euo pipefail

# Validates Apple notary + Developer ID signing credentials without notarizing
# anything, so an expired agreement/key/cert fails before release builds fan out.
# Usage: preflight-apple.sh
#
# Environment variables (same as notarize.sh / codesign.sh):
#   APPLE_NOTARY_KEY_BASE64       - base64-encoded .p8 API key
#   APPLE_NOTARY_KEY_ID           - API key ID
#   APPLE_NOTARY_ISSUER_ID        - API issuer UUID
#   APPLE_CERTIFICATE_P12_BASE64  - base64-encoded .p12 certificate
#   APPLE_CERTIFICATE_PASSWORD    - password for the .p12 file

readonly CERT_EXPIRY_WARN_DAYS=30
readonly SECONDS_PER_DAY=86400
readonly SIGNING_IDENTITY="Developer ID Application"
readonly PULL_REQUEST_EVENT="pull_request"

WORK_DIR="$(mktemp -d)"
SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/null}"
FAILED=0

trap 'rm -rf "$WORK_DIR"' EXIT

report() {
  echo "- $1 $2" >> "$SUMMARY"
}

fail() {
  echo "::error::$1: $2"
  report "❌" "$1: $2"
  FAILED=1
}

# Fork and Dependabot PRs receive no secrets; anything else must have them.
require_env() {
  local check="$1"
  shift
  local missing=()
  local name
  for name in "$@"; do
    [ -n "${!name:-}" ] || missing+=("$name")
  done
  [ ${#missing[@]} -eq 0 ] && return 0

  if [ "${GITHUB_EVENT_NAME:-}" = "$PULL_REQUEST_EVENT" ]; then
    echo "::warning::$check skipped: ${missing[*]} unavailable to this PR"
    report "⚠️" "$check skipped: secrets unavailable to this PR"
  else
    fail "$check" "${missing[*]} unset"
  fi
  return 1
}

check_notary() {
  local check="Notary API"
  require_env "$check" APPLE_NOTARY_KEY_BASE64 APPLE_NOTARY_KEY_ID APPLE_NOTARY_ISSUER_ID || return 0

  local key_path="$WORK_DIR/AuthKey_${APPLE_NOTARY_KEY_ID}.p8"
  echo "$APPLE_NOTARY_KEY_BASE64" | base64 --decode > "$key_path"

  # `history` is a read-only call to the same team-scoped API as `submit`, so a
  # missing agreement or revoked key 403s here exactly as it would on submit.
  local output
  if output=$(xcrun notarytool history \
    --key "$key_path" \
    --key-id "$APPLE_NOTARY_KEY_ID" \
    --issuer "$APPLE_NOTARY_ISSUER_ID" \
    --output-format json 2>&1); then
    echo "$check: credentials accepted"
    report "✅" "$check: credentials accepted"
  else
    echo "$output" >&2
    fail "$check" "$(echo "$output" | grep -m1 -i 'error' || echo 'request failed') — check developer.apple.com/account for unsigned agreements and that the API key is not revoked"
  fi
}

p12_to_pem() {
  openssl pkcs12 -in "$1" -nokeys -clcerts -passin env:APPLE_CERTIFICATE_PASSWORD 2>/dev/null \
    || openssl pkcs12 -legacy -in "$1" -nokeys -clcerts -passin env:APPLE_CERTIFICATE_PASSWORD 2>/dev/null
}

check_certificate() {
  local check="Signing certificate"
  require_env "$check" APPLE_CERTIFICATE_P12_BASE64 APPLE_CERTIFICATE_PASSWORD || return 0

  local p12_path="$WORK_DIR/cert.p12"
  local pem_path="$WORK_DIR/cert.pem"
  echo "$APPLE_CERTIFICATE_P12_BASE64" | base64 --decode > "$p12_path"

  if ! p12_to_pem "$p12_path" > "$pem_path" || [ ! -s "$pem_path" ]; then
    fail "$check" "could not decrypt .p12 (wrong APPLE_CERTIFICATE_PASSWORD or corrupt certificate)"
    return 0
  fi

  local subject expiry
  subject=$(openssl x509 -in "$pem_path" -noout -subject)
  expiry=$(openssl x509 -in "$pem_path" -noout -enddate | cut -d= -f2)

  if [[ "$subject" != *"$SIGNING_IDENTITY"* ]]; then
    fail "$check" "not a '$SIGNING_IDENTITY' certificate ($subject)"
  elif ! openssl x509 -in "$pem_path" -noout -checkend 0 > /dev/null; then
    fail "$check" "expired on $expiry"
  elif ! openssl x509 -in "$pem_path" -noout -checkend $((CERT_EXPIRY_WARN_DAYS * SECONDS_PER_DAY)) > /dev/null; then
    echo "::warning::$check expires on $expiry (within $CERT_EXPIRY_WARN_DAYS days)"
    report "⚠️" "$check: expires on $expiry"
  else
    echo "$check: valid until $expiry"
    report "✅" "$check: valid until $expiry"
  fi
}

echo "### Apple release credentials" >> "$SUMMARY"
check_notary
check_certificate
exit "$FAILED"
