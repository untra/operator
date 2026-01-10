/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Displays Operator connection status and session information.
 * Checks for vscode-session.json to determine if Operator is running.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { SessionInfo } from './types';

/**
 * Operator connection status
 */
export interface OperatorStatus {
  running: boolean;
  version?: string;
  port?: number;
  workspace?: string;
  sessionFile?: string;
}

/**
 * TreeDataProvider for status information
 */
export class StatusTreeProvider implements vscode.TreeDataProvider<StatusItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<
    StatusItem | undefined
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private status: OperatorStatus = { running: false };
  private ticketsDir: string | undefined;

  async setTicketsDir(dir: string | undefined): Promise<void> {
    this.ticketsDir = dir;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    if (!this.ticketsDir) {
      this.status = { running: false };
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    // Check for vscode-session.json in .tickets/operator/
    const sessionFile = path.join(this.ticketsDir, 'operator', 'vscode-session.json');
    try {
      const content = await fs.readFile(sessionFile, 'utf-8');
      const session: SessionInfo = JSON.parse(content);

      // Session file exists - server is running
      this.status = {
        running: true,
        version: session.version,
        port: session.port,
        workspace: session.workspace,
        sessionFile,
      };
    } catch {
      this.status = { running: false };
    }

    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: StatusItem): vscode.TreeItem {
    return element;
  }

  getChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    if (this.status.running) {
      items.push(
        new StatusItem('Status', 'Connected', 'pass', 'Webhook server is running')
      );
      if (this.status.version) {
        items.push(
          new StatusItem('Version', this.status.version, 'versions')
        );
      }
      if (this.status.port) {
        items.push(
          new StatusItem('Port', this.status.port.toString(), 'plug')
        );
      }
    } else {
      items.push(
        new StatusItem(
          'Status',
          'Disconnected',
          'error',
          'Webhook server not running'
        )
      );
    }

    if (this.ticketsDir) {
      items.push(
        new StatusItem('Tickets', path.basename(this.ticketsDir), 'folder')
      );
    } else {
      items.push(
        new StatusItem('Tickets', 'Not found', 'folder', 'No .tickets directory found')
      );
    }

    return items;
  }
}

/**
 * TreeItem for status display
 */
class StatusItem extends vscode.TreeItem {
  constructor(
    label: string,
    value: string,
    icon: string,
    tooltip?: string
  ) {
    super(label, vscode.TreeItemCollapsibleState.None);
    this.description = value;
    this.tooltip = tooltip || `${label}: ${value}`;
    this.iconPath = new vscode.ThemeIcon(icon);
  }
}
