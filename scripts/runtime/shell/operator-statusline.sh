#!/usr/bin/env bash
# Operator status line for Claude Code sessions.
# Receives session JSON on stdin; reads OPERATOR_* env vars.
# Outputs two lines: Line 1 = cwd + git + UI link, Line 2 = operator context.

set -o pipefail

# ANSI color codes
RESET='\033[0m'
BOLD='\033[1m'
BLUE_BG='\033[44m'
GREEN_BG='\033[42m'
YELLOW_BG='\033[43m'
CYAN_BG='\033[46m'
MAGENTA_BG='\033[45m'
BLACK_FG='\033[30m'
WHITE_FG='\033[97m'

# Parse stdin JSON (Claude Code pipes session context)
if command -v jq >/dev/null 2>&1; then
  INPUT=$(cat)
  CWD=$(echo "$INPUT" | jq -r '.cwd // .workspace.current_dir // empty' 2>/dev/null)
  MODEL=$(echo "$INPUT" | jq -r '.model.display_name // .model // empty' 2>/dev/null)
  CTX_USED=$(echo "$INPUT" | jq -r '.context_window.used_percentage // empty' 2>/dev/null)
else
  # Consume stdin even without jq
  cat >/dev/null
  CWD=""
  MODEL=""
  CTX_USED=""
fi

# Fallbacks
CWD="${CWD:-$(pwd)}"

# Shorten home directory to ~
home="$HOME"
SHORT_CWD="${CWD/#$home/\~}"

# Git info (only if cwd is valid)
GIT_BRANCH=""
GIT_DIRTY=""
if [ -d "$CWD" ]; then
  GIT_BRANCH=$(git -C "$CWD" --no-optional-locks symbolic-ref --short HEAD 2>/dev/null)
  if [ -n "$GIT_BRANCH" ]; then
    GIT_STATUS=$(git -C "$CWD" --no-optional-locks status --porcelain 2>/dev/null)
    if [ -n "$GIT_STATUS" ]; then
      GIT_DIRTY=" ✚"
    fi
  fi
fi

# --- Line 1: cwd | git branch | View in UI ---
LINE1=""

# Directory segment
LINE1="${LINE1}$(printf "${BLUE_BG}${BLACK_FG}${BOLD} %s ${RESET}" "$SHORT_CWD")"

# Git segment
if [ -n "$GIT_BRANCH" ]; then
  if [ -n "$GIT_DIRTY" ]; then
    LINE1="${LINE1}$(printf "${YELLOW_BG}${BLACK_FG}${BOLD} ± %s%s ${RESET}" "$GIT_BRANCH" "$GIT_DIRTY")"
  else
    LINE1="${LINE1}$(printf "${GREEN_BG}${BLACK_FG}${BOLD} ± %s ${RESET}" "$GIT_BRANCH")"
  fi
fi

# View in UI link (OSC 8 hyperlink if OPERATOR_UI_URL is set)
if [ -n "$OPERATOR_UI_URL" ]; then
  LINE1="${LINE1}$(printf " \033]8;;%s\033\\${BOLD}View in UI${RESET}\033]8;;\033\\" "$OPERATOR_UI_URL")"
fi

# --- Line 2: [OPR8R] ticket | project | model | ctx:% ---
LINE2=""

# Operator badge
LINE2="${LINE2}$(printf "${MAGENTA_BG}${WHITE_FG}${BOLD} OPR8R ${RESET}")"

# Ticket ID
if [ -n "$OPERATOR_TICKET_ID" ]; then
  LINE2="${LINE2}$(printf " %s" "$OPERATOR_TICKET_ID")"
fi

# Project
if [ -n "$OPERATOR_PROJECT" ]; then
  LINE2="${LINE2}$(printf " | %s" "$OPERATOR_PROJECT")"
fi

# Model
if [ -n "$MODEL" ]; then
  LINE2="${LINE2}$(printf " ${CYAN_BG}${BLACK_FG} %s ${RESET}" "$MODEL")"
fi

# Context usage
if [ -n "$CTX_USED" ]; then
  CTX_INT=$(printf "%.0f" "$CTX_USED" 2>/dev/null || echo "$CTX_USED")
  LINE2="${LINE2}$(printf " ${WHITE_FG}ctx:%s%%${RESET}" "$CTX_INT")"
fi

printf "%b\n%b\n" "$LINE1" "$LINE2"
