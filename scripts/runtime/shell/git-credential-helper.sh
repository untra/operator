#!/bin/sh
[ "$1" = get ] || exit 0
protocol='' host='' path=''
while IFS='=' read -r key value; do
    case "$key" in protocol) protocol=$value;; host) host=$value;; path) path=$value;; esac
done
[ "$protocol" = https ] && [ "$host" = __OP_HOST__ ] || exit 0
case "$path" in __OP_REPO__|__OP_REPO_GIT__) cat "$(dirname "$0")/credential";; esac
