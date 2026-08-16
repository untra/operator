#!/usr/bin/env bash

BOLD='\033[0;1m'
RESET='\033[0m'

ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  PLATFORM="linux-x86_64" ;;
  aarch64) PLATFORM="linux-arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

OPERATOR_BIN="${INSTALL_PREFIX}/operator"

if [ "${USE_CACHED}" = "true" ] && [ -f "$OPERATOR_BIN" ]; then
  echo "Using cached operator binary"
elif [ "${OFFLINE}" = "true" ]; then
  if [ -f "$OPERATOR_BIN" ]; then
    echo "Using offline operator binary"
  else
    echo "No operator binary found in offline mode"
    exit 1
  fi
else
  printf "$${BOLD}Installing operator v${VERSION}...$${RESET}\n"

  if [ -n "$CODER_SCRIPT_BIN_DIR" ] && [ -e "$CODER_SCRIPT_BIN_DIR/operator" ]; then
    rm "$CODER_SCRIPT_BIN_DIR/operator"
  fi

  mkdir -p "${INSTALL_PREFIX}"
  RELEASE_URL="https://github.com/untra/operator/releases/download/v${VERSION}/operator-$PLATFORM"

  output=$(curl -fsSL "$RELEASE_URL" -o "$OPERATOR_BIN" 2>&1)
  if [ $? -ne 0 ]; then
    echo "Failed to download operator: $output"
    exit 1
  fi
  chmod +x "$OPERATOR_BIN"
  printf "Operator v${VERSION} installed to ${INSTALL_PREFIX}\n"
fi

if [ -n "$CODER_SCRIPT_BIN_DIR" ] && [ ! -e "$CODER_SCRIPT_BIN_DIR/operator" ]; then
  ln -s "$OPERATOR_BIN" "$CODER_SCRIPT_BIN_DIR/operator"
fi

# opr8r is the client half of the pair: agent sessions launched in this
# workspace call it to report step completion for multi-step ticket workflows.
# The operator server runs fine without it (single-step tickets are
# unaffected), and releases before v0.2.6 ship no opr8r asset, so a failure
# here warns instead of aborting workspace startup.
OPR8R_BIN="${INSTALL_PREFIX}/opr8r"

if [ "${USE_CACHED}" = "true" ] && [ -f "$OPR8R_BIN" ]; then
  echo "Using cached opr8r binary"
elif [ "${OFFLINE}" = "true" ]; then
  if [ -f "$OPR8R_BIN" ]; then
    echo "Using offline opr8r binary"
  else
    echo "No opr8r binary found in offline mode; multi-step workflows unavailable"
  fi
else
  printf "$${BOLD}Installing opr8r v${VERSION}...$${RESET}\n"

  if [ -n "$CODER_SCRIPT_BIN_DIR" ] && [ -e "$CODER_SCRIPT_BIN_DIR/opr8r" ]; then
    rm "$CODER_SCRIPT_BIN_DIR/opr8r"
  fi

  mkdir -p "${INSTALL_PREFIX}"
  OPR8R_URL="https://github.com/untra/operator/releases/download/v${VERSION}/opr8r-$PLATFORM"

  if output=$(curl -fsSL "$OPR8R_URL" -o "$OPR8R_BIN" 2>&1); then
    chmod +x "$OPR8R_BIN"
    printf "opr8r v${VERSION} installed to ${INSTALL_PREFIX}\n"
  else
    rm -f "$OPR8R_BIN"
    echo "Warning: failed to download opr8r: $output"
    echo "Multi-step ticket workflows will be unavailable in this workspace."
  fi
fi

if [ -n "$CODER_SCRIPT_BIN_DIR" ] && [ -x "$OPR8R_BIN" ] && [ ! -e "$CODER_SCRIPT_BIN_DIR/opr8r" ]; then
  ln -s "$OPR8R_BIN" "$CODER_SCRIPT_BIN_DIR/opr8r"
fi

mkdir -p .tickets/operator .tickets/queue

# Bind template values to shell variables so conditionals below are real runtime checks
config_toml="${CONFIG_TOML}"
agent_template="${AGENT_TEMPLATE}"
callback_url="${CALLBACK_URL}"

if [ -n "$config_toml" ]; then
  echo "$config_toml" > .tickets/operator/config.toml
else
  cat > .tickets/operator/config.toml <<CONF
[rest_api]
enabled = true
port = ${PORT}

[agents]
max_parallel = ${MAX_PARALLEL}

[sessions]
wrapper = "${SESSION_WRAPPER}"
CONF

  if [ -n "$agent_template" ]; then
    cat >> .tickets/operator/config.toml <<CONF

[[targets]]
name = "coder-agents"
kind = "coder"
template = "$agent_template"
token_env = "${CODER_TOKEN_ENV}"
CONF
    if [ -n "$callback_url" ]; then
      echo "callback_url = \"$callback_url\"" >> .tickets/operator/config.toml
    fi
  fi
fi

echo "Starting operator API server on port ${PORT}..."
"$OPERATOR_BIN" api --port "${PORT}" > "${LOG_PATH}" 2>&1 &

for i in $(seq 1 30); do
  if curl -s "http://localhost:${PORT}/api/v1/health" > /dev/null 2>&1; then
    echo "Operator is running on port ${PORT}"
    exit 0
  fi
  sleep 1
done

echo "Operator failed to start within 30 seconds. Check logs at ${LOG_PATH}"
exit 1
