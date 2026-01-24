#!/bin/bash
# Manual test: Run this to verify long commands work with real tmux
# This tests the stdin piping approach for set-buffer
#
# Usage: ./test_long_command.sh
#
# Expected output: Should show the echoed content and print SUCCESS

set -e

SESSION="op-test-long-cmd-$$"
BUFFER="test-buf-$$"

# Generate 4KB of content (exceeds typical CLI arg limits)
CONTENT=$(printf 'echo "%s"' "$(head -c 4000 /dev/zero | tr '\0' 'a')")

echo "=== Testing tmux set-buffer with stdin piping ==="
echo "Content length: ${#CONTENT} bytes"
echo "Session: $SESSION"
echo "Buffer: $BUFFER"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "=== Cleanup ==="
    tmux kill-session -t "$SESSION" 2>/dev/null || true
    tmux delete-buffer -b "$BUFFER" 2>/dev/null || true
    echo "Done"
}
trap cleanup EXIT

# Create session
echo "Creating tmux session..."
tmux new-session -d -s "$SESSION"

# Test: load buffer via stdin (the new approach)
# tmux load-buffer uses "-" to read from stdin, unlike set-buffer which takes CLI args
echo "Loading buffer via stdin (4KB content)..."
echo "$CONTENT" | tmux load-buffer -b "$BUFFER" -

# Verify buffer was set
echo "Verifying buffer content..."
BUFFER_SIZE=$(tmux show-buffer -b "$BUFFER" | wc -c | tr -d ' ')
echo "Buffer size: $BUFFER_SIZE bytes"

if [ "$BUFFER_SIZE" -lt 4000 ]; then
    echo "FAIL: Buffer content is too small (expected ~4000 bytes)"
    exit 1
fi

# Paste buffer to session
echo "Pasting buffer to session..."
tmux paste-buffer -b "$BUFFER" -t "$SESSION"

# Send Enter to execute
echo "Sending Enter key..."
tmux send-keys -t "$SESSION" Enter

# Wait for execution
sleep 1

# Capture output
echo ""
echo "=== Captured pane output (last 5 lines) ==="
tmux capture-pane -t "$SESSION" -p | tail -5

echo ""
echo "=== SUCCESS: Long command handling works correctly ==="
echo "The stdin piping approach successfully bypasses CLI argument limits."
