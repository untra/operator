/**
 * Launch command builder for Claude CLI
 *
 * Constructs the claude CLI command with appropriate flags
 * based on launch options and ticket metadata.
 */

import { LaunchOptions, TicketMetadata } from './types';

/**
 * Build Claude CLI command for launching a ticket
 */
export function buildLaunchCommand(
  ticketPath: string,
  metadata: TicketMetadata,
  options: LaunchOptions,
  sessionId?: string
): string {
  const parts: string[] = ['claude'];

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
export function buildTerminalName(ticketId: string): string {
  // Sanitize for terminal name (same as Rust sanitize_session_name)
  const sanitized = ticketId.replace(/[^a-zA-Z0-9_-]/g, '-');
  return `op-${sanitized}`;
}

/**
 * Default launch options
 */
export function getDefaultLaunchOptions(): LaunchOptions {
  return {
    model: 'sonnet',
    yoloMode: false,
    resumeSession: false,
  };
}
