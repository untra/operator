/**
 * Ticket TreeDataProvider for Operator VS Code extension
 *
 * Displays tickets from .tickets directory in sidebar TreeViews.
 * Supports in-progress, queue, and completed ticket states.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { TerminalManager } from './terminal-manager';
import { TicketInfo, TicketType } from './types';

/**
 * TreeDataProvider for ticket lists
 */
export class TicketTreeProvider
  implements vscode.TreeDataProvider<TicketItem>
{
  private _onDidChangeTreeData = new vscode.EventEmitter<
    TicketItem | undefined
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private tickets: TicketInfo[] = [];
  private ticketsDir: string | undefined;

  constructor(
    private readonly status: 'in-progress' | 'queue' | 'completed',
    private readonly terminalManager?: TerminalManager
  ) {}

  async setTicketsDir(dir: string | undefined): Promise<void> {
    this.ticketsDir = dir;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    if (!this.ticketsDir) {
      this.tickets = [];
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    const subDir = path.join(this.ticketsDir, this.status);
    try {
      const files = await fs.readdir(subDir);
      const mdFiles = files.filter((f) => f.endsWith('.md'));

      this.tickets = await Promise.all(
        mdFiles.map(async (file) => {
          const filePath = path.join(subDir, file);
          const content = await fs.readFile(filePath, 'utf-8');
          return this.parseTicket(file, filePath, content);
        })
      );

      // Sort by ticket ID
      this.tickets.sort((a, b) => a.id.localeCompare(b.id));
    } catch {
      this.tickets = [];
    }

    this._onDidChangeTreeData.fire(undefined);
  }

  private parseTicket(
    filename: string,
    filePath: string,
    content: string
  ): TicketInfo {
    // Parse ticket ID and type from filename: FEAT-123-title.md or FEAT-123.md
    const match = filename.match(/^(FEAT|FIX|TASK|SPIKE|INV)-(\d+)/i);
    const type = (match?.[1]?.toUpperCase() || 'TASK') as TicketType;
    const id = match ? `${match[1].toUpperCase()}-${match[2]}` : filename.replace('.md', '');

    // Parse title from first heading or frontmatter
    const titleMatch =
      content.match(/^#\s+(.+)$/m) || content.match(/^title:\s*(.+)$/m);
    const title = titleMatch?.[1]?.trim() || id;

    return {
      id,
      title,
      type,
      status: this.status,
      filePath,
      terminalName: this.status === 'in-progress' ? `op-${id}` : undefined,
    };
  }

  getTreeItem(element: TicketItem): vscode.TreeItem {
    return element;
  }

  getChildren(): TicketItem[] {
    return this.tickets.map(
      (ticket) => new TicketItem(ticket, this.terminalManager)
    );
  }

  /**
   * Get all tickets (for launch command)
   */
  getTickets(): TicketInfo[] {
    return [...this.tickets];
  }
}

/**
 * TreeItem representing a single ticket
 */
export class TicketItem extends vscode.TreeItem {
  constructor(
    public readonly ticket: TicketInfo,
    private readonly terminalManager?: TerminalManager
  ) {
    super(ticket.title, vscode.TreeItemCollapsibleState.None);

    this.id = ticket.id;
    this.tooltip = `${ticket.id}: ${ticket.title}`;
    this.description = ticket.id;

    // Set icon based on ticket type
    this.iconPath = this.getIconForType(ticket.type);

    // Set context for menu commands
    this.contextValue = ticket.status;

    // Make in-progress items clickable to focus terminal (pass ticket for relaunch)
    if (ticket.status === 'in-progress' && ticket.terminalName) {
      this.command = {
        command: 'operator.focusTicket',
        title: 'Focus Terminal',
        arguments: [ticket.terminalName, ticket],
      };
    } else {
      // Queue and completed items open the file
      this.command = {
        command: 'operator.openTicket',
        title: 'Open Ticket',
        arguments: [ticket.filePath],
      };
    }
  }

  private getIconForType(type: TicketType): vscode.ThemeIcon {
    switch (type) {
      case 'FEAT':
        return new vscode.ThemeIcon(
          'sparkle',
          new vscode.ThemeColor('terminal.ansiCyan')
        );
      case 'FIX':
        return new vscode.ThemeIcon(
          'wrench',
          new vscode.ThemeColor('terminal.ansiRed')
        );
      case 'TASK':
        return new vscode.ThemeIcon(
          'tasklist',
          new vscode.ThemeColor('terminal.ansiGreen')
        );
      case 'SPIKE':
        return new vscode.ThemeIcon(
          'beaker',
          new vscode.ThemeColor('terminal.ansiMagenta')
        );
      case 'INV':
        return new vscode.ThemeIcon(
          'search',
          new vscode.ThemeColor('terminal.ansiYellow')
        );
      default:
        return new vscode.ThemeIcon('file');
    }
  }
}
