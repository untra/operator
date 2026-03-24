#!/bin/bash
# Run zellij integration tests inside a real zellij session.
#
# Zellij launch tests require a running zellij server because the operator
# binary calls `zellij action new-tab`, `zellij action write-chars`, etc.
# We use `script -qefc` to allocate a pseudo-TTY (required by zellij on
# headless CI runners) and `zellij run --close-on-exit` to execute the
# tests inside a real session.
#
# MOCK_LLM_OUTPUT_DIR is exported BEFORE starting zellij so the server
# process inherits it, and every new tab the operator creates will too.
#
# Usage:
#   scripts/ci/run-zellij-integration.sh
#
# Prerequisites:
#   - zellij binary installed and on PATH
#   - Rust toolchain available
#   - operator binary built (target/release/operator or target/debug/operator)
set -euo pipefail

echo "=== Zellij Integration Tests ==="
echo "Zellij version: $(zellij --version)"

# Fixed shared output dir for mock LLM invocations.
# Exported before zellij starts so the server (and all tabs) inherit it.
export MOCK_LLM_OUTPUT_DIR="/tmp/operator-zellij-mock-output"
mkdir -p "$MOCK_LLM_OUTPUT_DIR"

# 1. Start a zellij session in the background (with PTY via script).
#    `zellij run` requires an already-running session to create a pane in,
#    so we must start the session first.
script -qfc "zellij --session operator-ci-test" /dev/null &
ZELLIJ_BG_PID=$!

# 2. Wait for session to be ready
echo "Waiting for zellij session..."
for i in $(seq 1 15); do
  if zellij list-sessions 2>/dev/null | grep -q operator-ci-test; then
    echo "Session ready after ${i}s"
    break
  fi
  if [ "$i" -eq 15 ]; then
    echo "ERROR: Zellij session did not start within 15s"
    kill $ZELLIJ_BG_PID 2>/dev/null || true
    exit 1
  fi
  sleep 1
done

# 3. Run tests inside the real session.
#    - `script -qefc` provides a pseudo-TTY for headless CI
#    - `-e` propagates the child exit code
#    - `zellij run --close-on-exit` returns the pane's exit code
script -qefc "zellij --session operator-ci-test run --close-on-exit -- bash -c '
  export OPERATOR_LAUNCH_TEST_ENABLED=true
  export OPERATOR_ZELLIJ_TEST_ENABLED=true
  cargo test --test launch_integration_zellij -- --nocapture --test-threads=1
'" /dev/null
EXIT_CODE=$?

# 4. Cleanup
kill $ZELLIJ_BG_PID 2>/dev/null || true
zellij delete-all-sessions --yes --force 2>/dev/null || true
rm -rf "$MOCK_LLM_OUTPUT_DIR"

echo "=== Zellij Integration Tests Complete (exit: $EXIT_CODE) ==="
exit $EXIT_CODE
