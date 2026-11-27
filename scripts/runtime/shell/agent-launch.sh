#!/bin/bash
if [ -f "$0.env" ]; then
    set -a
    # shellcheck disable=SC1090
    . "$0.env"
    set +a
    rm -f "$0.env"
fi
__OP_SETUP__
cd __OP_PROJECT__ || exit 1
__OP_RUN__
