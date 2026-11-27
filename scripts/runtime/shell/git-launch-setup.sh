#!/bin/bash
export OPERATOR_GIT_RUNTIME=__OP_RUNTIME__
trap 'rm -rf -- "$OPERATOR_GIT_RUNTIME"' EXIT
# shellcheck disable=SC1091
. "$OPERATOR_GIT_RUNTIME/env.sh" || exit 1
