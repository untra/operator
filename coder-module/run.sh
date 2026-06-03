#!/usr/bin/env bash

BOLD='\033[0;1m'
RESET='\033[0m'

ARCH=$$(uname -m)
case "$$ARCH" in
  x86_64)  PLATFORM="linux-x86_64" ;;
  aarch64) PLATFORM="linux-arm64" ;;
  *)
    echo "Unsupported architecture: $$ARCH"
    exit 1
    ;;
esac

OPERATOR_BIN="${INSTALL_PREFIX}/operator"

if [ "${USE_CACHED}" = "true" ] && [ -f "$$OPERATOR_BIN" ]; then
  echo "Using cached operator binary"
elif [ "${OFFLINE}" = "true" ]; then
  if [ -f "$$OPERATOR_BIN" ]; then
    echo "Using offline operator binary"
  else
    echo "No operator binary found in offline mode"
    exit 1
  fi
else
  printf "$${BOLD}Installing operator v${VERSION}...$${RESET}\n"

  if [ -n "$$CODER_SCRIPT_BIN_DIR" ] && [ -e "$$CODER_SCRIPT_BIN_DIR/operator" ]; then
    rm "$$CODER_SCRIPT_BIN_DIR/operator"
  fi

  mkdir -p "${INSTALL_PREFIX}"
  RELEASE_URL="https://github.com/untra/operator/releases/download/v${VERSION}/operator-$$PLATFORM"

  output=$$(curl -fsSL "$$RELEASE_URL" -o "$$OPERATOR_BIN" 2>&1)
  if [ $$? -ne 0 ]; then
    echo "Failed to download operator: $$output"
    exit 1
  fi
  chmod +x "$$OPERATOR_BIN"
  printf "Operator v${VERSION} installed to ${INSTALL_PREFIX}\n"
fi

if [ -n "$$CODER_SCRIPT_BIN_DIR" ] && [ ! -e "$$CODER_SCRIPT_BIN_DIR/operator" ]; then
  ln -s "$$OPERATOR_BIN" "$$CODER_SCRIPT_BIN_DIR/operator"
fi

mkdir -p .tickets/operator .tickets/queue

if [ -n "${CONFIG_TOML}" ]; then
  echo "${CONFIG_TOML}" > .tickets/operator/config.toml
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
fi

echo "Starting operator API server on port ${PORT}..."
"$$OPERATOR_BIN" api --port "${PORT}" > "${LOG_PATH}" 2>&1 &

for i in $$(seq 1 30); do
  if curl -s "http://localhost:${PORT}/api/v1/health" > /dev/null 2>&1; then
    echo "Operator is running on port ${PORT}"
    exit 0
  fi
  sleep 1
done

echo "Operator failed to start within 30 seconds. Check logs at ${LOG_PATH}"
exit 1
