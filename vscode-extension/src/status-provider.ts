/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Displays Operator connection status and session information.
 * Checks for vscode-session.json (webhook) and api-session.json (API).
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { SessionInfo } from './types';
import { discoverApiUrl, ApiSessionInfo } from './api-client';

/**
 * Webhook server connection status
 */
export interface WebhookStatus {
  running: boolean;
  version?: string;
  port?: number;
  workspace?: string;
  sessionFile?: string;
}

/**
 * Operator REST API connection status
 */
export interface ApiStatus {
  connected: boolean;
  version?: string;
  port?: number;
  url?: string;
}

/**
 * TreeDataProvider for status information
 */
export class StatusTreeProvider implements vscode.TreeDataProvider<StatusItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<
    StatusItem | undefined
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private webhookStatus: WebhookStatus = { running: false };
  private apiStatus: ApiStatus = { connected: false };
  private ticketsDir: string | undefined;

  async setTicketsDir(dir: string | undefined): Promise<void> {
    this.ticketsDir = dir;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    // Check webhook status (requires ticketsDir for session file)
    await this.checkWebhookStatus();

    // Always check API status, even without ticketsDir
    await this.checkApiStatus();

    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * Check webhook server status via session file
   * Requires ticketsDir since the webhook writes to .tickets/operator/vscode-session.json
   */
  private async checkWebhookStatus(): Promise<void> {
    if (!this.ticketsDir) {
      this.webhookStatus = { running: false };
      return;
    }

    const webhookSessionFile = path.join(this.ticketsDir, 'operator', 'vscode-session.json');
    try {
      const content = await fs.readFile(webhookSessionFile, 'utf-8');
      const session: SessionInfo = JSON.parse(content);

      this.webhookStatus = {
        running: true,
        version: session.version,
        port: session.port,
        workspace: session.workspace,
        sessionFile: webhookSessionFile,
      };
    } catch {
      this.webhookStatus = { running: false };
    }
  }

  /**
   * Check API status - tries session file first, then falls back to configured URL
   * Works even without ticketsDir by using the configured apiUrl
   */
  private async checkApiStatus(): Promise<void> {
    // Try session file first if ticketsDir exists
    if (this.ticketsDir) {
      const apiSessionFile = path.join(this.ticketsDir, 'operator', 'api-session.json');
      try {
        const content = await fs.readFile(apiSessionFile, 'utf-8');
        const session: ApiSessionInfo = JSON.parse(content);
        const apiUrl = `http://localhost:${session.port}`;

        if (await this.tryHealthCheck(apiUrl, session.version)) {
          return;
        }
      } catch {
        // Session file doesn't exist or is invalid, fall through to configured URL
      }
    }

    // Always try configured URL as fallback (works without ticketsDir)
    const apiUrl = await discoverApiUrl(this.ticketsDir);
    await this.tryHealthCheck(apiUrl);
  }

  /**
   * Attempt a health check against the given API URL
   * Returns true if successful, false otherwise
   */
  private async tryHealthCheck(apiUrl: string, sessionVersion?: string): Promise<boolean> {
    try {
      const response = await fetch(`${apiUrl}/api/v1/health`);
      if (response.ok) {
        const health = await response.json() as { version?: string };
        const port = new URL(apiUrl).port;
        this.apiStatus = {
          connected: true,
          version: health.version || sessionVersion,
          port: port ? parseInt(port, 10) : 7008,
          url: apiUrl,
        };
        return true;
      }
    } catch {
      // Health check failed
    }
    this.apiStatus = { connected: false };
    return false;
  }

  getTreeItem(element: StatusItem): vscode.TreeItem {
    return element;
  }

  getChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    // REST API status
    if (this.apiStatus.connected) {
      items.push(
        new StatusItem('API', this.apiStatus.url || '', 'pass', `Operator REST API at ${this.apiStatus.url}`)
      );
      if (this.apiStatus.version) {
        items.push(
          new StatusItem('API Version', this.apiStatus.version, 'versions')
        );
      }
      if (this.apiStatus.port) {
        items.push(
          new StatusItem('API Port', this.apiStatus.port.toString(), 'plug')
        );
      }
    } else {
      items.push(
        new StatusItem(
          'API',
          'Disconnected',
          'error',
          'Operator REST API not running. Use "Operator: Download Operator" command if not installed.'
        )
      );
    }

    // Webhook server status
    if (this.webhookStatus.running) {
      items.push(
        new StatusItem('Webhook', 'Running', 'pass', 'Local webhook server for terminal management')
      );
      if (this.webhookStatus.port) {
        items.push(
          new StatusItem('Webhook Port', this.webhookStatus.port.toString(), 'plug')
        );
      }
    } else {
      items.push(
        new StatusItem(
          'Webhook',
          'Stopped',
          'circle-slash',
          'Local webhook server not running'
        )
      );
    }

    // Tickets directory
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
