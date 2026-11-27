#!/bin/bash
printf '%s\n' "$$" > "$OPERATOR_GIT_RUNTIME/pid"
trap 'rm -rf -- "$OPERATOR_GIT_RUNTIME"' EXIT
( __OP_COMMAND__ ) <&0 &
operator_git_child=$!
trap 'kill -TERM "$operator_git_child" 2>/dev/null; wait "$operator_git_child"; exit 143' TERM
trap 'kill -INT "$operator_git_child" 2>/dev/null; wait "$operator_git_child"; exit 130' INT
trap 'kill -HUP "$operator_git_child" 2>/dev/null; wait "$operator_git_child"; exit 129' HUP
wait "$operator_git_child"
