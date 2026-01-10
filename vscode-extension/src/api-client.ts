/**
 * Operator REST API client
 *
 * Provides methods to communicate with the Operator REST API
 * for launching tickets and checking health status.
 */

import * as vscode from 'vscode';

export interface LaunchTicketRequest {
  provider?: string;
  model?: string;
  yolo_mode?: boolean;
  wrapper?: string;
}

export interface LaunchTicketResponse {
  agent_id: string;
  ticket_id: string;
  working_directory: string;
  command: string;
  terminal_name: string;
  session_id: string;
  worktree_created: boolean;
  branch?: string;
}

export interface HealthResponse {
  status: string;
  version: string;
}

export interface ApiError {
  error: string;
  message: string;
}

/**
 * Client for the Operator REST API
 */
export class OperatorApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    const config = vscode.workspace.getConfiguration('operator');
    this.baseUrl = baseUrl || config.get('apiUrl', 'http://localhost:7008');
  }

  /**
   * Check if the Operator API is available
   */
  async health(): Promise<HealthResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/health`);
    if (!response.ok) {
      throw new Error('Operator API not available');
    }
    return (await response.json()) as HealthResponse;
  }

  /**
   * Launch a ticket via the Operator API
   *
   * Claims the ticket, sets up worktree if needed, and returns
   * the command to execute in a terminal.
   */
  async launchTicket(
    ticketId: string,
    options: LaunchTicketRequest
  ): Promise<LaunchTicketResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/tickets/${encodeURIComponent(ticketId)}/launch`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider: options.provider,
          model: options.model,
          yolo_mode: options.yolo_mode ?? false,
          wrapper: options.wrapper,
        }),
      }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as LaunchTicketResponse;
  }
}
