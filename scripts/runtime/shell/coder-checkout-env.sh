#!/bin/sh
git_runtime=__OP_GIT_RUNTIME__
# shellcheck disable=SC1091
. "$git_runtime/env.sh"
trap 'rm -rf -- "$git_runtime"' EXIT
