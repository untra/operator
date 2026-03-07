#!/bin/bash
# Run zellij integration tests.
#
# Zellij launch tests require the ZELLIJ env var to be set, simulating
# running inside a zellij session. The tests use `require_in_zellij = false`
# in the config, so the launcher will proceed even without a real session,
# but zellij CLI commands are used for tab verification.
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

# Set environment variables for the tests
export OPERATOR_LAUNCH_TEST_ENABLED=true
export OPERATOR_ZELLIJ_TEST_ENABLED=true

# Simulate being inside zellij by setting the env var.
# The test config uses require_in_zellij = false, so the launcher
# checks this env var for session context but doesn't hard-fail.
export ZELLIJ="${ZELLIJ:-0}"
export ZELLIJ_SESSION_NAME="${ZELLIJ_SESSION_NAME:-operator-ci-test}"

echo "ZELLIJ=$ZELLIJ"
echo "ZELLIJ_SESSION_NAME=$ZELLIJ_SESSION_NAME"

# Run the tests
cargo test --test launch_integration_zellij -- --nocapture --test-threads=1
EXIT_CODE=$?

# Cleanup any leftover zellij sessions
zellij delete-all-sessions --yes --force 2>/dev/null || true

echo "=== Zellij Integration Tests Complete (exit: $EXIT_CODE) ==="
exit $EXIT_CODE
