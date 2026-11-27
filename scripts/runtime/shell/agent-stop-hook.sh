#!/bin/bash
# Operator hook for __OP_TOOL__
signal_dir=__OP_SIGNAL_DIR__
session_id=$(jq -r '.session_id')
timestamp=$(date +%s)

mkdir -p "$signal_dir"
printf '{"event":"stop","timestamp":%s,"session_id":"%s"}\n' "$timestamp" "$session_id" \
    > "$signal_dir/$session_id.signal"
