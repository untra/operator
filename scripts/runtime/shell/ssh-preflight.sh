#!/bin/sh
# Exit status placeholders are rendered as numeric constants.
# shellcheck disable=SC2242
command -v tmux >/dev/null || exit __OP_NO_TMUX__
command -v __OP_TOOL__ >/dev/null || exit __OP_NO_TOOL__
test -d __OP_WORKDIR__ || exit __OP_NO_WORKDIR__
command -v git >/dev/null || exit __OP_NO_GIT__
__OP_PROVIDER_CHECK__
