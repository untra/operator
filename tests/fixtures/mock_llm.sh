#!/bin/bash
# Mock LLM script for integration testing
# Captures invocation details to a JSON file for test verification
#
# Environment variables:
#   MOCK_LLM_OUTPUT_DIR - Directory to write invocation files (default: /tmp/operator-test)
#
# Output: Creates invocation-{timestamp}.json with all captured data

set -e

OUTPUT_DIR="${MOCK_LLM_OUTPUT_DIR:-/tmp/operator-test}"
mkdir -p "$OUTPUT_DIR"

# Generate unique invocation ID using nanoseconds
INVOCATION_ID=$(date +%s%N 2>/dev/null || date +%s)
INVOCATION_FILE="$OUTPUT_DIR/invocation-$INVOCATION_ID.json"

# Capture all arguments as a JSON array
ARGS_JSON="["
FIRST=1
for arg in "$@"; do
    if [ $FIRST -eq 1 ]; then
        FIRST=0
    else
        ARGS_JSON="$ARGS_JSON,"
    fi
    # Escape special characters for JSON
    escaped_arg=$(printf '%s' "$arg" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/\\t/g; s/\n/\\n/g')
    ARGS_JSON="$ARGS_JSON\"$escaped_arg\""
done
ARGS_JSON="$ARGS_JSON]"

# Parse specific arguments we care about
SESSION_ID=""
MODEL=""
PROMPT_FILE=""
CONFIG_FLAGS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --session-id)
            SESSION_ID="$2"
            shift 2
            ;;
        --model)
            MODEL="$2"
            shift 2
            ;;
        --print-prompt-path)
            PROMPT_FILE="$2"
            shift 2
            ;;
        --permission-mode)
            CONFIG_FLAGS="${CONFIG_FLAGS}permission_mode=$2 "
            shift 2
            ;;
        --dangerously-skip-permissions)
            CONFIG_FLAGS="${CONFIG_FLAGS}yolo=true "
            shift
            ;;
        --allowedTools)
            CONFIG_FLAGS="${CONFIG_FLAGS}allowedTools=$2 "
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

# Read prompt file content if it exists
PROMPT_CONTENT=""
if [[ -n "$PROMPT_FILE" && -f "$PROMPT_FILE" ]]; then
    # Read and escape for JSON
    PROMPT_CONTENT=$(cat "$PROMPT_FILE" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/\\t/g' | awk '{printf "%s\\n", $0}' | sed 's/\\n$//')
fi

# Get current working directory
CWD=$(pwd)

# Get timestamp
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date +%Y-%m-%dT%H:%M:%SZ)

# Write invocation details as JSON
cat > "$INVOCATION_FILE" << EOF
{
    "timestamp": "$TIMESTAMP",
    "invocation_id": "$INVOCATION_ID",
    "command": "$0",
    "args": $ARGS_JSON,
    "session_id": "$SESSION_ID",
    "model": "$MODEL",
    "prompt_file": "$PROMPT_FILE",
    "prompt_content": "$PROMPT_CONTENT",
    "config_flags": "$CONFIG_FLAGS",
    "cwd": "$CWD"
}
EOF

# Log for debugging
echo "Mock LLM invoked"
echo "  Session ID: $SESSION_ID"
echo "  Model: $MODEL"
echo "  Prompt file: $PROMPT_FILE"
echo "  Output: $INVOCATION_FILE"

# Simulate a brief run and exit cleanly
sleep 1
exit 0
