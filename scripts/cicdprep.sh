#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

# --- Colors & helpers ---

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

RUN_ALL=false
CONTINUE_ON_FAIL=false
FAILURES=()
PASSES=()
SKIPPED=()

usage() {
  echo "Usage: $(basename "$0") [OPTIONS]"
  echo ""
  echo "Run CI/CD checks locally before creating a PR."
  echo "Auto-detects changed files and runs only relevant workflow checks."
  echo ""
  echo "Options:"
  echo "  --all        Run all checks regardless of changed files"
  echo "  --continue   Don't stop on first failure; run everything and report"
  echo "  -h, --help   Show this help"
}

for arg in "$@"; do
  case "$arg" in
    --all)       RUN_ALL=true ;;
    --continue)  CONTINUE_ON_FAIL=true ;;
    -h|--help)   usage; exit 0 ;;
    *)           echo "Unknown option: $arg"; usage; exit 1 ;;
  esac
done

section() {
  echo ""
  echo -e "${CYAN}${BOLD}════════════════════════════════════════════════════════════${RESET}"
  echo -e "${CYAN}${BOLD}  $1${RESET}"
  echo -e "${CYAN}${BOLD}════════════════════════════════════════════════════════════${RESET}"
}

step() {
  echo -e "\n${BOLD}▸ $1${RESET}"
}

pass() {
  echo -e "  ${GREEN}✓ $1${RESET}"
  PASSES+=("$1")
}

fail() {
  echo -e "  ${RED}✗ $1${RESET}"
  FAILURES+=("$1")
  if [ "$CONTINUE_ON_FAIL" = false ]; then
    echo -e "\n${RED}${BOLD}FAILED: $1${RESET}"
    echo -e "${RED}Use --continue to run all checks despite failures.${RESET}"
    exit 1
  fi
}

skip() {
  echo -e "  ${YELLOW}⊘ $1 (skipped — no changes)${RESET}"
  SKIPPED+=("$1")
}

require_tool() {
  local tool="$1"
  local context="$2"
  if ! command -v "$tool" &>/dev/null; then
    echo -e "${RED}Missing required tool: ${BOLD}$tool${RESET}${RED} (needed for $context)${RESET}"
    echo "Install it and re-run."
    exit 1
  fi
}

run_step() {
  local label="$1"
  shift
  step "$label"
  if "$@"; then
    pass "$label"
  else
    fail "$label"
  fi
}

# Verify a bun project's lockfile is in sync with its package.json, exactly the
# way CI does (`bun install --frozen-lockfile`). This is the check that catches
# the "lockfile had changes, but lockfile is frozen" CI failure locally — it
# happens when package.json is edited but bun.lock isn't regenerated/committed.
check_bun_lockfile() {
  local dir="$1"
  step "Lockfile sync: $dir"
  if [ ! -f "$dir/bun.lock" ]; then
    skip "Lockfile sync: $dir (no bun.lock)"
    return
  fi
  if (cd "$dir" && bun install --frozen-lockfile) >/dev/null 2>&1; then
    pass "Lockfile sync: $dir"
  else
    echo -e "  ${YELLOW}lockfile out of sync — run: ${BOLD}(cd $dir && bun install)${RESET}${YELLOW} and commit bun.lock${RESET}"
    fail "Lockfile sync: $dir"
  fi
}

# --- Detect changed files ---

section "Detecting changes"

MAIN_BRANCH="main"
if ! git rev-parse --verify "$MAIN_BRANCH" &>/dev/null; then
  MAIN_BRANCH="origin/main"
fi

MERGE_BASE=$(git merge-base "$MAIN_BRANCH" HEAD 2>/dev/null || echo "")

if [ -z "$MERGE_BASE" ]; then
  echo -e "${YELLOW}Could not find merge base with $MAIN_BRANCH — running all checks.${RESET}"
  RUN_ALL=true
  CHANGED_FILES=""
else
  CHANGED_FILES=$(git diff --name-only "$MERGE_BASE"...HEAD 2>/dev/null || "")
  UNSTAGED=$(git diff --name-only 2>/dev/null || "")
  STAGED=$(git diff --name-only --cached 2>/dev/null || "")
  CHANGED_FILES=$(echo -e "${CHANGED_FILES}\n${UNSTAGED}\n${STAGED}" | sort -u | grep -v '^$' || true)
fi

if [ "$RUN_ALL" = true ]; then
  echo -e "${YELLOW}Running ALL checks (--all or no merge base).${RESET}"
else
  FILE_COUNT=$(echo "$CHANGED_FILES" | grep -c '.' || echo 0)
  echo -e "Found ${BOLD}$FILE_COUNT${RESET} changed file(s) vs $MAIN_BRANCH."
  if [ "$FILE_COUNT" -eq 0 ]; then
    echo -e "${GREEN}No changes detected. Nothing to check.${RESET}"
    exit 0
  fi
fi

has_changes() {
  local pattern="$1"
  if [ "$RUN_ALL" = true ]; then
    return 0
  fi
  echo "$CHANGED_FILES" | grep -qE "$pattern"
}

# build.yaml triggers on everything EXCEPT docs-only or version-only changes
needs_operator() {
  if [ "$RUN_ALL" = true ]; then return 0; fi
  local non_ignored
  non_ignored=$(echo "$CHANGED_FILES" | grep -vE '^(docs/|\.github/workflows/docs\.yml$|VERSION$)' || true)
  [ -n "$non_ignored" ]
}

needs_opr8r()      { has_changes '^opr8r/'; }
needs_vscode()     { has_changes '^(vscode-extension/|icons/)'; }
needs_zed()        { has_changes '^zed-extension/'; }
needs_docs()       { has_changes '^(docs/|src/docs_gen/|src/taxonomy/taxonomy\.toml|src/templates/.*\.json)'; }

# A bun project needs a lockfile check whenever its package.json or bun.lock
# changed. Root project lives at the repo root; others under their dir.
needs_bun_root()       { has_changes '^(package\.json|bun\.lock)$'; }
needs_bun_ui()         { has_changes '^ui/(package\.json|bun\.lock)$'; }
needs_bun_backstage()  { has_changes '^backstage-server/(.*/)?(package\.json|bun\.lock)$'; }

# --- 0. Bun lockfiles ---
#
# Run this first, cheaply, across every bun project so a stale lockfile fails
# loudly and early instead of midway through a UI build. Mirrors CI's
# `bun install --frozen-lockfile`, and additionally covers the root and
# backstage-server lockfiles that CI does not currently enforce.

if needs_bun_root || needs_bun_ui || needs_bun_backstage; then
  section "Bun lockfiles"
  require_tool bun "bun lockfile sync"

  if needs_bun_root;      then check_bun_lockfile ".";                else skip "Lockfile sync: . (no changes)"; fi
  if needs_bun_ui;        then check_bun_lockfile "ui";               else skip "Lockfile sync: ui (no changes)"; fi
  if needs_bun_backstage; then check_bun_lockfile "backstage-server"; else skip "Lockfile sync: backstage-server (no changes)"; fi
else
  skip "Bun lockfiles"
fi

# --- 1. Operator (main crate) ---

if needs_operator; then
  section "Operator (main crate)"
  require_tool cargo "operator"
  require_tool bun "operator UI build"
  require_tool cargo-deny "operator dependency audit"

  step "UI build"
  (
    cd ui
    bun install --frozen-lockfile
    bun run build
    DIST_SIZE=$(du -sk dist/ | awk '{print $1 * 1024}')
    echo "  UI dist size: ${DIST_SIZE}B ($(echo "scale=1; $DIST_SIZE/1048576" | bc)MB)"
    if [ "$DIST_SIZE" -gt 5242880 ]; then
      echo "UI dist exceeds 5MB budget (${DIST_SIZE}B)" >&2
      exit 1
    fi
  ) && pass "UI build + size check" || fail "UI build + size check"

  run_step "cargo fmt" cargo fmt -- --check
  run_step "cargo clippy" cargo clippy --locked --all-targets --all-features -- -D warnings
  run_step "cargo test" cargo test --locked --all-features
  run_step "cargo deny" cargo deny --manifest-path Cargo.toml check
else
  skip "Operator (main crate)"
fi

# --- 2. opr8r ---

if needs_opr8r; then
  section "opr8r"
  require_tool cargo "opr8r"
  require_tool cargo-deny "opr8r dependency audit"

  run_step "opr8r fmt" bash -c "cd opr8r && cargo fmt -- --check"
  run_step "opr8r clippy" bash -c "cd opr8r && cargo clippy --locked --all-targets --all-features -- -D warnings"
  run_step "opr8r test" bash -c "cd opr8r && cargo test --locked --all-features"
  run_step "opr8r cargo deny" cargo deny --manifest-path opr8r/Cargo.toml check
else
  skip "opr8r"
fi

# --- 3. vscode-extension ---

if needs_vscode; then
  section "vscode-extension"
  require_tool node "vscode-extension"
  require_tool npm "vscode-extension"

  step "Install dependencies"
  (cd vscode-extension && npm ci) && pass "vscode install" || fail "vscode install"

  run_step "vscode copy-types" bash -c "cd vscode-extension && npm run copy-types"
  run_step "vscode generate:icons" bash -c "cd vscode-extension && mkdir -p images/icons/dist && npm run generate:icons"
  run_step "vscode lint" bash -c "cd vscode-extension && npm run lint"
  run_step "vscode compile" bash -c "cd vscode-extension && npm run compile"
  run_step "vscode compile:webview" bash -c "cd vscode-extension && npm run compile:webview"
else
  skip "vscode-extension"
fi

# --- 4. zed-extension ---

if needs_zed; then
  section "zed-extension"
  require_tool cargo "zed-extension"
  require_tool cargo-deny "zed-extension dependency audit"

  if ! rustup target list --installed 2>/dev/null | grep -q wasm32-wasip1; then
    echo -e "${YELLOW}Installing wasm32-wasip1 target...${RESET}"
    rustup target add wasm32-wasip1
  fi

  run_step "zed fmt" bash -c "cd zed-extension && cargo fmt -- --check"
  run_step "zed clippy" bash -c "cd zed-extension && cargo clippy --locked --target wasm32-wasip1 -- -D warnings"
  run_step "zed build" bash -c "cd zed-extension && cargo build --locked --release --target wasm32-wasip1"
  run_step "zed cargo deny" cargo deny --manifest-path zed-extension/Cargo.toml check
else
  skip "zed-extension"
fi

# --- 5. docs ---

if needs_docs; then
  section "docs"
  require_tool cargo "docs generation"
  require_tool bundle "docs Jekyll build"

  run_step "docs generate" cargo run --locked -- docs
  step "Jekyll build"
  (cd docs && bundle install && bundle exec jekyll build) && pass "Jekyll build" || fail "Jekyll build"
else
  skip "docs"
fi

# --- Summary ---

section "Summary"

if [ ${#PASSES[@]} -gt 0 ]; then
  echo -e "\n${GREEN}${BOLD}Passed (${#PASSES[@]}):${RESET}"
  for p in "${PASSES[@]}"; do
    echo -e "  ${GREEN}✓${RESET} $p"
  done
fi

if [ ${#SKIPPED[@]} -gt 0 ]; then
  echo -e "\n${YELLOW}${BOLD}Skipped (${#SKIPPED[@]}):${RESET}"
  for s in "${SKIPPED[@]}"; do
    echo -e "  ${YELLOW}⊘${RESET} $s"
  done
fi

if [ ${#FAILURES[@]} -gt 0 ]; then
  echo -e "\n${RED}${BOLD}Failed (${#FAILURES[@]}):${RESET}"
  for f in "${FAILURES[@]}"; do
    echo -e "  ${RED}✗${RESET} $f"
  done
  echo ""
  echo -e "${RED}${BOLD}CI would fail. Fix the above issues before creating a PR.${RESET}"
  exit 1
fi

echo ""
echo -e "${GREEN}${BOLD}All checks passed. Ready to create a PR.${RESET}"
