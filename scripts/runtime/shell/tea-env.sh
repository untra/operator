#!/bin/sh
OPERATOR_TEA_BINARY=$(command -v tea || true)
export OPERATOR_TEA_BINARY
export PATH=__OP_BIN__:"$PATH"
