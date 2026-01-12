/**
 * Launch manager for Operator VS Code extension
 *
 * Orchestrates ticket launching and relaunching, coordinating between
 * terminal management, ticket parsing, and command building.
 *
 * Prefers launching via the Operator REST API when available, falling
 * back to local command building when the API is unavailable.
 */

import * as vscode from 'vscode';
import { TerminalManager } from './terminal-manager';
import { LaunchOptions, TicketInfo } from './types';
import { parseTicketMetadata, getCurrentSessionId } from './ticket-parser';
import {
  OperatorApiClient,
  discoverApiUrl,
  LaunchTicketResponse,
} from './api-client';

  /**
 * Build terminal name from ticket ID
 *
 * Sanitizes the ticket ID to be valid for terminal names,
 * matching the Rust sanitize_session_name behavior.
 */
  function buildTerminalName(ticketId: string): string {
    // Sanitize for terminal name (same as Rust sanitize_session_name)
    const sanitized = ticketId.replace(/[^a-zA-Z0-9_-]/g, '-');
    return `op-${sanitized}`;
  }

/**
 * Manages launching and relaunching tickets
 */
export class LaunchManager {
  private ticketsDir: string | undefined;
  private apiClient: OperatorApiClient | undefined;
  private outputChannel: vscode.OutputChannel | undefined;

  constructor(private terminalManager: TerminalManager) { }

  /**
   * Set the tickets directory
   */
  setTicketsDir(dir: string | undefined): void {
    this.ticketsDir = dir;
    // Reset API client when tickets dir changes
    this.apiClient = undefined;
  }

  /**
   * Set the output channel for logging
   */
  setOutputChannel(channel: vscode.OutputChannel): void {
    this.outputChannel = channel;
  }

  /**
   * Initialize or refresh the API client
   */
  private async ensureApiClient(): Promise<OperatorApiClient> {
    if (!this.apiClient) {
      const apiUrl = await discoverApiUrl(this.ticketsDir);
      this.apiClient = new OperatorApiClient(apiUrl);
      this.log(`Initialized API client with URL: ${apiUrl}`);
    }
    return this.apiClient;
  }

  /**
   * Log a message to the output channel
   */
  private log(message: string): void {
    if (this.outputChannel) {
      this.outputChannel.appendLine(`[LaunchManager] ${message}`);
    }
  }

  /**
   * Launch a ticket with options
   *
   * Attempts to launch via the Operator API first. If the API is unavailable
   * or returns an error, falls back to building the command locally.
   */
  async launchTicket(ticket: TicketInfo, options: LaunchOptions): Promise<void> {
    const terminalName = buildTerminalName(ticket.id);

    // Check if terminal already exists
    if (this.terminalManager.exists(terminalName)) {
      const choice = await vscode.window.showWarningMessage(
        `Terminal '${terminalName}' already exists`,
        'Focus Existing',
        'Kill and Relaunch'
      );

      if (choice === 'Focus Existing') {
        await this.terminalManager.focus(terminalName);
        return;
      } else if (choice === 'Kill and Relaunch') {
        await this.terminalManager.kill(terminalName);
      } else {
        return; // Cancelled
      }
    }

    // Try API launch
    try {
      await this.launchViaApi(ticket, options);
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Unknown error';
      this.log(`API launch failed: ${msg}`);
      vscode.window.showErrorMessage(`Failed to launch ticket: ${msg}`);
    }
  }

  /**
   * Launch a ticket via the Operator REST API
   */
  private async launchViaApi(
    ticket: TicketInfo,
    options: LaunchOptions
  ): Promise<void> {
    const apiClient = await this.ensureApiClient();

    // Check API health first
    try {
      await apiClient.health();
    } catch {
      throw new Error('Operator API not available');
    }

    this.log(`Launching ticket ${ticket.id} via API`);

    const response: LaunchTicketResponse = await apiClient.launchTicket(
      ticket.id,
      {
        provider: null,
        model: options.model,
        yolo_mode: options.yoloMode,
        wrapper: 'vscode',
        retry_reason: null,
        resume_session_id: null,
      }
    );

    this.log(
      `API response: terminal=${response.terminal_name}, ` +
      `workdir=${response.working_directory}, ` +
      `worktree=${response.worktree_created}`
    );

    // Create terminal with API response
    await this.terminalManager.create({
      name: response.terminal_name,
      workingDir: response.working_directory,
    });

    await this.terminalManager.send(response.terminal_name, response.command);
    await this.terminalManager.focus(response.terminal_name);

    const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
    const branchMsg = response.branch ? ` on branch ${response.branch}` : '';
    vscode.window.showInformationMessage(
      `Launched agent for ${ticket.id}${worktreeMsg}${branchMsg}`
    );
  }

  /**
   * Offer to relaunch when terminal not found
   */
  async offerRelaunch(ticket: TicketInfo): Promise<void> {
    const metadata = await parseTicketMetadata(ticket.filePath);
    const sessionId = metadata ? getCurrentSessionId(metadata) : undefined;

    const choices: string[] = ['Launch Fresh'];
    if (sessionId) {
      choices.push('Resume Session');
    }
    choices.push('Cancel');

    const choice = await vscode.window.showWarningMessage(
      `Terminal for '${ticket.id}' not found`,
      ...choices
    );

    if (choice === 'Launch Fresh') {
      await this.launchTicket(ticket, {
        model: 'sonnet',
        yoloMode: false,
        resumeSession: false,
      });
    } else if (choice === 'Resume Session') {
      await this.launchTicket(ticket, {
        model: 'sonnet',
        yoloMode: false,
        resumeSession: true,
      });
    }
  }
}
