#!/bin/sh
if [ -n "${OPERATOR_GIT_RUNTIME:-}" ]; then
    # shellcheck disable=SC1091
    . "$OPERATOR_GIT_RUNTIME/env.sh"
fi
__OP_COMMAND__
