#!/usr/bin/env bash
# Render coder-module/run.sh the way main.tf's templatefile() does, then syntax-check and shellcheck the result.
# The rendered coder_script actually runs in a workspace, so a bash syntax error here is a broken module.
# templatefile() only escapes `${`, which makes over-escaped `$$(cmd)` a silent breakage.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUN_SH="$ROOT_DIR/coder-module/run.sh"
MAIN_TF="$ROOT_DIR/coder-module/main.tf"

# CI installs terraform; local dev machines may only have OpenTofu.
if command -v terraform &>/dev/null; then
  TF=terraform
elif command -v tofu &>/dev/null; then
  TF=tofu
else
  echo "Neither terraform nor tofu found; install one to run this check." >&2
  exit 1
fi

# Vars mirroring the templatefile() call in main.tf (values follow the
# variable defaults). Keys are parity-checked against main.tf below, so a
# newly added template var fails with a clear message instead of a render
# error deep inside CI.
VARS='
    VERSION         = "0.0.0",
    PORT            = 7008,
    INSTALL_PREFIX  = "/tmp/operator",
    LOG_PATH        = "/tmp/operator.log",
    CONFIG_TOML     = "",
    MAX_PARALLEL    = 2,
    SESSION_WRAPPER = "tmux",
    OFFLINE         = false,
    USE_CACHED      = false,
    AGENT_TEMPLATE  = "operator-agent",
    CODER_TOKEN_ENV = "CODER_SESSION_TOKEN",
    CALLBACK_URL    = "",
'

extract_keys() {
  awk -F= 'NF > 1 { gsub(/[ \t,]/, "", $1); if ($1 ~ /^[A-Z_]+$/) print $1 }' | sort
}

tf_keys="$(awk '/templatefile\(/ { f = 1; next } f && /\}\)/ { exit } f' "$MAIN_TF" | extract_keys)"
local_keys="$(extract_keys <<<"$VARS")"

if [ "$tf_keys" != "$local_keys" ]; then
  echo "Template var mismatch between coder-module/main.tf and $(basename "$0"):" >&2
  diff <(echo "$tf_keys") <(echo "$local_keys") | sed 's/^/  /' >&2 || true
  echo "Update the VARS map in this script to match main.tf's templatefile() call." >&2
  exit 1
fi

RENDER="$(mktemp -d)"
trap 'rm -rf "$RENDER"' EXIT

cat > "$RENDER/main.tf" <<EOF
output "s" {
  value = templatefile("$RUN_SH", {
$VARS
  })
}
EOF

"$TF" -chdir="$RENDER" init -input=false >/dev/null
"$TF" -chdir="$RENDER" apply -auto-approve -input=false >/dev/null
"$TF" -chdir="$RENDER" output -raw s > "$RENDER/rendered.sh"

bash -n "$RENDER/rendered.sh"
shellcheck -S error "$RENDER/rendered.sh"
echo "coder-module rendered startup script OK"
