"use strict";
/**
 * Launch command builder for Claude CLI
 *
 * Constructs the claude CLI command with appropriate flags
 * based on launch options and ticket metadata.
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildLaunchCommand = buildLaunchCommand;
exports.buildTerminalName = buildTerminalName;
exports.getDefaultLaunchOptions = getDefaultLaunchOptions;
/**
 * Build Claude CLI command for launching a ticket
 */
function buildLaunchCommand(ticketPath, metadata, options, sessionId) {
    const parts = ['claude'];
    // Model selection
    parts.push('--model', options.model);
    // Resume from existing session
    if (options.resumeSession && sessionId) {
        parts.push('--resume', sessionId);
    }
    // YOLO mode (auto-accept all prompts)
    if (options.yoloMode) {
        parts.push('--dangerously-skip-permissions');
    }
    // Prompt: read the ticket file
    parts.push('--print', `"Read and work on the ticket at ${ticketPath}"`);
    return parts.join(' ');
}
/**
 * Build terminal name from ticket ID
 *
 * Sanitizes the ticket ID to be valid for terminal names,
 * matching the Rust sanitize_session_name behavior.
 */
function buildTerminalName(ticketId) {
    // Sanitize for terminal name (same as Rust sanitize_session_name)
    const sanitized = ticketId.replace(/[^a-zA-Z0-9_-]/g, '-');
    return `op-${sanitized}`;
}
/**
 * Default launch options
 */
function getDefaultLaunchOptions() {
    return {
        model: 'sonnet',
        yoloMode: false,
        resumeSession: false,
    };
}
//# sourceMappingURL=launch-command.js.map