#!/bin/bash
# Operator tmux status bar script
# Reads state.json and outputs agent statistics
# Usage: tmux-status.sh /path/to/state.json

STATE_FILE="$1"

# Check if state file exists
if [ ! -f "$STATE_FILE" ]; then
    echo "?"
    exit 0
fi

# Use jq if available for accurate parsing
if command -v jq &> /dev/null; then
    # Count agents by status using jq
    RUNNING=$(jq '[.agents[] | select(.status == "running")] | length' "$STATE_FILE" 2>/dev/null || echo "0")
    AWAITING=$(jq '[.agents[] | select(.status == "awaiting_input")] | length' "$STATE_FILE" 2>/dev/null || echo "0")
    PAUSED=$(jq '.paused' "$STATE_FILE" 2>/dev/null || echo "false")
else
    # Fallback: simple grep-based counting (less accurate but works without jq)
    RUNNING=$(grep -c '"status": "running"' "$STATE_FILE" 2>/dev/null || echo "0")
    AWAITING=$(grep -c '"status": "awaiting_input"' "$STATE_FILE" 2>/dev/null || echo "0")
    PAUSED="false"
    grep -q '"paused": true' "$STATE_FILE" 2>/dev/null && PAUSED="true"
fi

# Build output string with tmux color codes (Operator theme - Terracotta background)
# Deep Pine #115566 = running/healthy
# Terracotta #E05D44 = warning/awaiting
# Cornflower #6688AA = muted
if [ "$PAUSED" = "true" ]; then
    echo "#[fg=#E05D44]PAUSED"
elif [ "$AWAITING" -gt 0 ]; then
    # Show running/awaiting in different colors
    echo "#[fg=#115566]$RUNNING#[fg=#6688AA]/#[fg=#E05D44]$AWAITING#[fg=#6688AA] agents"
elif [ "$RUNNING" -gt 0 ]; then
    echo "#[fg=#115566]$RUNNING#[fg=#6688AA] agents"
else
    echo "#[fg=#6688AA]0 agents"
fi
