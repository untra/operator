/**
 * Shared TypeScript interfaces for Operator VS Code extension
 */

/**
 * Ticket type identifier
 */
export type TicketType = 'FEAT' | 'FIX' | 'TASK' | 'SPIKE' | 'INV';

/**
 * Ticket status in the workflow
 */
export type TicketStatus = 'in-progress' | 'queue' | 'completed';

/**
 * Information about a ticket from .tickets directory
 */
export interface TicketInfo {
  /** Ticket ID (e.g., "FEAT-123") */
  id: string;
  /** Ticket title from markdown heading */
  title: string;
  /** Ticket type */
  type: TicketType;
  /** Current status */
  status: TicketStatus;
  /** Path to the ticket markdown file */
  filePath: string;
  /** Terminal name if in-progress (e.g., "op-FEAT-123") */
  terminalName?: string;
}

/**
 * Options for creating a new terminal
 */
export interface TerminalCreateOptions {
  /** Terminal name (e.g., "op-FEAT-123") */
  name: string;
  /** Working directory for the terminal */
  workingDir?: string;
  /** Environment variables to set */
  env?: Record<string, string>;
}

/**
 * Terminal activity state
 */
export type ActivityState = 'idle' | 'running' | 'unknown';

/**
 * State of a managed terminal
 */
export interface TerminalState {
  /** Terminal name */
  name: string;
  /** Process ID if available */
  pid?: number;
  /** Current activity state */
  activity: ActivityState;
  /** Timestamp when terminal was created */
  createdAt: number;
}

/**
 * Health check response
 */
export interface HealthResponse {
  status: 'ok';
  version: string;
  port: number;
}

/**
 * Generic success response
 */
export interface SuccessResponse {
  success: true;
  name?: string;
}

/**
 * Terminal exists response
 */
export interface ExistsResponse {
  exists: boolean;
}

/**
 * Activity query response
 */
export interface ActivityResponse {
  activity: ActivityState;
}

/**
 * Terminal list response
 */
export interface ListResponse {
  terminals: TerminalState[];
}

/**
 * Error response
 */
export interface ErrorResponse {
  error: string;
}

/**
 * Send command request body
 */
export interface SendCommandRequest {
  command: string;
}

/**
 * Session info written to .tickets/operator/vscode-session.json
 * Used by Operator to discover the extension's webhook server
 */
export interface SessionInfo {
  /** Wrapper type identifier */
  wrapper: 'vscode';
  /** Actual port the webhook server is listening on */
  port: number;
  /** Process ID of VS Code */
  pid: number;
  /** Extension version */
  version: string;
  /** ISO timestamp when server started */
  started_at: string;
  /** Workspace folder path */
  workspace: string;
}

/**
 * Model options for Claude CLI
 */
export type ModelOption = 'sonnet' | 'opus' | 'haiku';

/**
 * Launch options for starting an agent on a ticket
 */
export interface LaunchOptions {
  /** Model to use (sonnet, opus, haiku) */
  model: ModelOption;
  /** YOLO mode - auto-accept all prompts */
  yoloMode: boolean;
  /** Resume from existing session (uses session_id from ticket) */
  resumeSession: boolean;
}

/**
 * Parsed ticket metadata from YAML frontmatter
 */
export interface TicketMetadata {
  /** Ticket ID */
  id: string;
  /** Current status */
  status: string;
  /** Current step name */
  step: string;
  /** Priority level */
  priority: string;
  /** Project name */
  project: string;
  /** Session UUIDs by step name */
  sessions?: Record<string, string>;
  /** Git worktree path if using per-ticket worktrees */
  worktree_path?: string;
  /** Git branch name */
  branch?: string;
}
