#!/bin/bash
set -e
ssh __OP_CONFIG____OP_ALIAS__ __OP_MKDIR__
ssh __OP_CONFIG____OP_ALIAS__ __OP_CAT_PROMPT__ < __OP_LOCAL_PROMPT__
ssh __OP_CONFIG____OP_ALIAS__ __OP_CAT_PAYLOAD__ < __OP_LOCAL_PAYLOAD__
env_file=__OP_ENV_FILE__
if [ -n "$env_file" ]; then
    ssh __OP_CONFIG____OP_ALIAS__ __OP_CAT_ENV__ < "$env_file"
fi
exec ssh -t __OP_CONFIG__-R __OP_TUNNEL__ -o ExitOnForwardFailure=yes __OP_ALIAS__ __OP_TMUX__
