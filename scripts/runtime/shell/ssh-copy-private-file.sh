#!/bin/sh
umask 077
set -C
target=__OP_TARGET__
cat > "$target" && chmod __OP_MODE__ "$target"
