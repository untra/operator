#!/bin/sh
target=__OP_TARGET__
if [ -L "$target" ]; then
    exit 1
fi
if [ -f "$target/pid" ]; then
    read -r pid < "$target/pid"
    case "$pid" in ''|*[!0-9]*|0) exit 1;; esac
    if kill -0 "$pid" 2>/dev/null; then
        exit 1
    fi
fi
rm -rf -- "$target"
