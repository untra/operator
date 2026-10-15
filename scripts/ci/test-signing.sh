#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_DIR="$(mktemp -d)"
trap 'rm -rf "$TEST_DIR"' EXIT
mkdir -p "$TEST_DIR/tmp" "$TEST_DIR/inputs"
export TMPDIR="$TEST_DIR/tmp"
export TRACE="$TEST_DIR/trace"
export APPLE_CERTIFICATE_P12_BASE64="dGVzdA=="
export APPLE_CERTIFICATE_PASSWORD="test"
export APPLE_NOTARY_KEY_BASE64="dGVzdA=="
export APPLE_NOTARY_KEY_ID="test"
export APPLE_NOTARY_ISSUER_ID="test"
export NOTARY_STATUS="Accepted"
export NOTARY_EXIT=0
export SIGN_FAILURE="false"

openssl() {
  echo test-keychain-password
}
export -f openssl
security() {
  echo "security $*" >> "$TRACE"
  case "$1" in
    find-identity) echo '1) ABC "Developer ID Application: Test"' ;;
    list-keychains) if [ "$#" -eq 1 ]; then echo '"login.keychain-db"'; fi ;;
  esac
}
export -f security
codesign() {
  echo "codesign $*" >> "$TRACE"
  test "$SIGN_FAILURE" = false
}
export -f codesign
ditto() {
  set -euo pipefail
  echo "ditto $*" >> "$TRACE"
  local source_dir="${*: -2:1}"
  find "$source_dir" -type f -exec basename {} \; | sort > "$TRACE.files"
  touch "${@: -1}"
}
export -f ditto
xcrun() {
  echo "xcrun $*" >> "$TRACE"
  printf '{"id":"test-id","status":"%s"}\n' "$NOTARY_STATUS"
  return "$NOTARY_EXIT"
}
export -f xcrun
touch "$TEST_DIR/inputs/first binary" "$TEST_DIR/inputs/second"

for count in 1 2; do
  binaries=("$TEST_DIR/inputs/first binary")
  if [ "$count" -eq 2 ]; then binaries+=("$TEST_DIR/inputs/second"); fi
  : > "$TRACE"
  bash "$ROOT/scripts/codesign.sh" "${binaries[@]}"
  test "$(grep -c '^security import ' "$TRACE")" -eq 1
  test "$(grep -c '^codesign --force ' "$TRACE")" -eq "$count"
  test "$(grep -c '^codesign --verify ' "$TRACE")" -eq "$count"
  grep -q '^security delete-keychain ' "$TRACE"
  : > "$TRACE"
  bash "$ROOT/scripts/notarize.sh" "${binaries[@]}"
  test "$(grep -c '^xcrun notarytool submit ' "$TRACE")" -eq 1
  test "$(wc -l < "$TRACE.files" | tr -d ' ')" -eq "$count"
  test -z "$(ls -A "$TMPDIR")"
done

export SIGN_FAILURE=true
: > "$TRACE"
if bash "$ROOT/scripts/codesign.sh" "$TEST_DIR/inputs/second"; then
  echo "Signing failure was ignored" >&2
  exit 1
fi
grep -q '^security delete-keychain ' "$TRACE"
export SIGN_FAILURE=false
export NOTARY_STATUS=Invalid
if bash "$ROOT/scripts/notarize.sh" "$TEST_DIR/inputs/second"; then
  echo "Rejected notarization was ignored" >&2
  exit 1
fi
test -z "$(ls -A "$TMPDIR")"
export NOTARY_STATUS=Accepted
export NOTARY_EXIT=1
if bash "$ROOT/scripts/notarize.sh" "$TEST_DIR/inputs/second"; then
  echo "Notary transport failure was ignored" >&2
  exit 1
fi
test -z "$(ls -A "$TMPDIR")"
echo "Signing script tests passed"
