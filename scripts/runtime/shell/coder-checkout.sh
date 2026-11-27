#!/bin/sh
set -e
if [ -d __OP_DIR__/.git ] && [ "$(git -C __OP_DIR__ remote get-url origin)" = __OP_URL__ ]; then
    git -C __OP_DIR__ fetch origin
else
    rm -rf __OP_DIR__
    git clone __OP_URL__ __OP_DIR__
fi
git -C __OP_DIR__ checkout -B __OP_BRANCH__
