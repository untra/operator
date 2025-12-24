//! Tmux configuration file generation for operator-managed sessions.
//!
//! Generates a custom `.tmux.conf` and status bar script for operator-managed
//! tmux sessions, providing a customized experience with operator stats display.

use std::path::Path;

/// Generate the custom tmux configuration content.
///
/// The generated config includes:
/// - Easy detach binding: Ctrl+a (no prefix needed)
/// - Mouse support and increased scrollback
/// - Custom status bar with operator stats
/// - Catppuccin-style dark theme
pub fn generate_tmux_conf(status_script_path: &Path, state_path: &Path) -> String {
    let script_path = status_script_path.display();
    let state_file = state_path.join("state.json");
    let state_file_path = state_file.display();

    format!(
        r##"# Operator custom tmux configuration
# This config is used only for operator-managed sessions
# Feel free to customize this file to your preferences

# ============================================
# COLOR PALETTE (Operator theme - Refined)
# ============================================
# Terracotta:  #E05D44 (primary, background)
# Deep Pine:   #115566 (primary text, healthy/running)
# Cornflower:  #6688AA (muted text, separators)
# Cream:       #F2EAC9 (accent, session name)
# (warning uses Terracotta)

# ============================================
# KEY BINDINGS
# ============================================

# Easy detach: Ctrl+a (no prefix needed!)
bind-key -n C-a detach-client

# ============================================
# GENERAL SETTINGS
# ============================================

# Increase scrollback buffer
set -g history-limit 10000

# Start windows and panes at 1, not 0
set -g base-index 1
setw -g pane-base-index 1

# Faster key repetition
set -s escape-time 0

# ============================================
# STATUS BAR
# ============================================

set -g status on
set -g status-interval 5
set -g status-position bottom

# Status bar colors (Operator theme - Terracotta background)
set -g status-style "bg=#E05D44,fg=#115566"

# Left: session name
set -g status-left "#[fg=#F2EAC9,bold] op:#S #[fg=#6688AA]| "
set -g status-left-length 30

# Middle: window status
setw -g window-status-format "#[fg=#6688AA]#I:#W"
setw -g window-status-current-format "#[fg=#115566,bold]#I:#W"
setw -g window-status-separator "  "

# Right: operator stats from script + time
set -g status-right "#[fg=#6688AA]| #('{script_path}' '{state_file_path}') #[fg=#6688AA]| %H:%M"
set -g status-right-length 50

# ============================================
# PANE BORDERS
# ============================================

set -g pane-border-style "fg=#6688AA"
set -g pane-active-border-style "fg=#F2EAC9"

# ============================================
# COPY/PASTE (macOS)
# ============================================

# Copy mode uses vi keys
setw -g mode-keys vi

# Copy to system clipboard on mouse selection
bind-key -T copy-mode-vi MouseDragEnd1Pane send-keys -X copy-pipe-and-cancel "pbcopy"

# Also support y key for yank in copy mode
bind-key -T copy-mode-vi y send-keys -X copy-pipe-and-cancel "pbcopy"

# ============================================
# TERMINAL
# ============================================

# Better color support
set -g default-terminal "screen-256color"
set -as terminal-features ",xterm-256color:RGB"

# Enable focus events (for vim/editor integration)
set -g focus-events on

# ============================================
# ACTIVITY MONITORING
# ============================================

# Monitor activity in other windows
setw -g monitor-activity on
set -g visual-activity off

# ============================================
# MESSAGE STYLE
# ============================================

set -g message-style "bg=#E05D44,fg=#115566"
set -g message-command-style "bg=#E05D44,fg=#115566"
"##
    )
}

/// Generate the status bar shell script content.
///
/// The script reads state.json and outputs agent statistics:
/// - "3 agents" when all are running
/// - "2/1 agents" when some are awaiting input (running/awaiting)
/// - "PAUSED" when the queue is paused
pub fn generate_status_script() -> String {
    r##"#!/bin/bash
# Operator tmux status bar script
# Reads state.json and outputs agent statistics
# Usage: tmux-status.sh /path/to/state.json

STATE_FILE="$1"

# Check if state file exists
if [ ! -f "$STATE_FILE" ]; then
    echo "?"
    exit 0
fi

# Use jq if available for accurate parsing
if command -v jq &> /dev/null; then
    # Count agents by status using jq
    RUNNING=$(jq '[.agents[] | select(.status == "running")] | length' "$STATE_FILE" 2>/dev/null || echo "0")
    AWAITING=$(jq '[.agents[] | select(.status == "awaiting_input")] | length' "$STATE_FILE" 2>/dev/null || echo "0")
    PAUSED=$(jq '.paused' "$STATE_FILE" 2>/dev/null || echo "false")
else
    # Fallback: simple grep-based counting (less accurate but works without jq)
    RUNNING=$(grep -c '"status": "running"' "$STATE_FILE" 2>/dev/null || echo "0")
    AWAITING=$(grep -c '"status": "awaiting_input"' "$STATE_FILE" 2>/dev/null || echo "0")
    PAUSED="false"
    grep -q '"paused": true' "$STATE_FILE" 2>/dev/null && PAUSED="true"
fi

# Build output string with tmux color codes (Operator theme - Terracotta background)
# Deep Pine #115566 = running/healthy
# Terracotta #E05D44 = warning/awaiting
# Cornflower #6688AA = muted
if [ "$PAUSED" = "true" ]; then
    echo "#[fg=#E05D44]PAUSED"
elif [ "$AWAITING" -gt 0 ]; then
    # Show running/awaiting in different colors
    echo "#[fg=#115566]$RUNNING#[fg=#6688AA]/#[fg=#E05D44]$AWAITING#[fg=#6688AA] agents"
elif [ "$RUNNING" -gt 0 ]; then
    echo "#[fg=#115566]$RUNNING#[fg=#6688AA] agents"
else
    echo "#[fg=#6688AA]0 agents"
fi
"##
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_tmux_conf_contains_key_binding() {
        let script_path = PathBuf::from("/tmp/status.sh");
        let state_path = PathBuf::from("/tmp");

        let conf = generate_tmux_conf(&script_path, &state_path);

        // Should contain the Ctrl+a binding
        assert!(conf.contains("bind-key -n C-a detach-client"));
    }

    #[test]
    fn test_generate_tmux_conf_contains_status_bar() {
        let script_path = PathBuf::from("/tmp/status.sh");
        let state_path = PathBuf::from("/tmp");

        let conf = generate_tmux_conf(&script_path, &state_path);

        // Should contain status bar configuration
        assert!(conf.contains("status-right"));
        assert!(conf.contains("/tmp/status.sh"));
        assert!(conf.contains("/tmp/state.json"));
    }

    #[test]
    fn test_generate_tmux_conf_contains_mouse() {
        let script_path = PathBuf::from("/tmp/status.sh");
        let state_path = PathBuf::from("/tmp");

        let conf = generate_tmux_conf(&script_path, &state_path);

        assert!(conf.contains("mouse on"));
    }

    #[test]
    fn test_generate_status_script_is_bash() {
        let script = generate_status_script();

        assert!(script.starts_with("#!/bin/bash"));
    }

    #[test]
    fn test_generate_status_script_uses_jq() {
        let script = generate_status_script();

        // Should prefer jq for parsing
        assert!(script.contains("command -v jq"));
        assert!(script.contains("jq"));
    }

    #[test]
    fn test_generate_status_script_has_fallback() {
        let script = generate_status_script();

        // Should have grep fallback
        assert!(script.contains("grep"));
    }

    #[test]
    fn test_generate_status_script_handles_paused() {
        let script = generate_status_script();

        assert!(script.contains("PAUSED"));
    }
}
