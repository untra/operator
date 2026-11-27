#!/bin/sh
if [ -z "${OPERATOR_TEA_BINARY:-}" ]; then
    echo "operator: tea not found on PATH; install it on this machine to use Gitea or Forgejo" >&2
    exit 127
fi
XDG_CONFIG_HOME=__OP_RUNTIME__ exec "$OPERATOR_TEA_BINARY" "$@"
