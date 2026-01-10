/**
 * Launch manager for Operator VS Code extension
 *
 * Orchestrates ticket launching and relaunching, coordinating between
 * terminal management, ticket parsing, and command building.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { TerminalManager } from './terminal-manager';
import { LaunchOptions, TicketInfo } from './types';
import { parseTicketMetadata, getCurrentSessionId } from './ticket-parser';
import { buildLaunchCommand, buildTerminalName } from './launch-command';

/**
 * Manages launching and relaunching tickets
 */
export class LaunchManager {
  private ticketsDir: string | undefined;

  constructor(private terminalManager: TerminalManager) {}

  /**
   * Set the tickets directory
   */
  setTicketsDir(dir: string | undefined): void {
    this.ticketsDir = dir;
  }

  /**
   * Launch a ticket with options
   */
  async launchTicket(ticket: TicketInfo, options: LaunchOptions): Promise<void> {
    // Parse ticket metadata
    const metadata = await parseTicketMetadata(ticket.filePath);
    if (!metadata) {
      throw new Error(`Could not parse ticket metadata: ${ticket.filePath}`);
    }

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

    // Get session ID for resume
    const sessionId = options.resumeSession ? getCurrentSessionId(metadata) : undefined;

    // Determine working directory
    const workingDir = metadata.worktreePath || this.getProjectDir(ticket);

    // Build the command
    const ticketRelPath = path.relative(workingDir, ticket.filePath);
    const command = buildLaunchCommand(ticketRelPath, metadata, options, sessionId);

    // Create terminal and send command
    await this.terminalManager.create({
      name: terminalName,
      workingDir,
    });

    await this.terminalManager.send(terminalName, command);
    await this.terminalManager.focus(terminalName);

    const resumeMsg = sessionId ? ' (resuming session)' : '';
    vscode.window.showInformationMessage(`Launched agent for ${ticket.id}${resumeMsg}`);
  }

  /**
   * Offer to relaunch when terminal not found
   */
  async offerRelaunch(ticket: TicketInfo): Promise<void> {
    const metadata = await parseTicketMetadata(ticket.filePath);
    const sessionId = metadata ? getCurrentSessionId(metadata) : undefined;

    const options: string[] = ['Launch Fresh'];
    if (sessionId) {
      options.push('Resume Session');
    }
    options.push('Cancel');

    const choice = await vscode.window.showWarningMessage(
      `Terminal for '${ticket.id}' not found`,
      ...options
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

  /**
   * Get project directory from ticket info
   */
  private getProjectDir(ticket: TicketInfo): string {
    // Default to parent of .tickets directory
    if (this.ticketsDir) {
      return path.dirname(this.ticketsDir);
    }

    // Fall back to ticket's parent directory
    return path.dirname(path.dirname(ticket.filePath));
  }
}
