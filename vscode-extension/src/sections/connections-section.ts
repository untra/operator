import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection, WebhookStatus, ApiStatus } from './types';
import { SessionInfo } from '../types';
import { discoverApiUrl, ApiSessionInfo } from '../api-client';
import { getOperatorPath, getOperatorVersion } from '../operator-binary';
import { isMcpServerRegistered } from '../mcp-connect';

export class ConnectionsSection implements StatusSection {
  readonly sectionId = 'connections';

  private webhookStatus: WebhookStatus = { running: false };
  private apiStatus: ApiStatus = { connected: false };
  private operatorVersion: string | undefined;
  private mcpRegistered: boolean = false;
  private wrapperType: string = 'vscode';

  get isApiConnected(): boolean {
    return this.apiStatus.connected;
  }

  isConfigured(): boolean {
    return this.apiStatus.connected || this.webhookStatus.running;
  }

  async check(ctx: SectionContext): Promise<void> {
    await Promise.allSettled([
      this.checkWebhookStatus(ctx),
      this.checkApiStatus(ctx),
      this.checkOperatorVersion(ctx),
      this.checkWrapperType(ctx),
    ]);
    this.mcpRegistered = isMcpServerRegistered();
  }

  private async checkWebhookStatus(ctx: SectionContext): Promise<void> {
    if (!ctx.ticketsDir) {
      this.webhookStatus = { running: false };
      return;
    }

    const webhookSessionFile = path.join(ctx.ticketsDir, 'operator', 'vscode-session.json');
    try {
      const content = await fs.readFile(webhookSessionFile, 'utf-8');
      const session = JSON.parse(content) as SessionInfo;

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

    // Fall back to live server state if file check missed it
    if (!this.webhookStatus.running && ctx.webhookServer?.isRunning()) {
      this.webhookStatus = {
        running: true,
        port: ctx.webhookServer.getPort(),
      };
    }
  }

  private async checkApiStatus(ctx: SectionContext): Promise<void> {
    if (ctx.ticketsDir) {
      const apiSessionFile = path.join(ctx.ticketsDir, 'operator', 'api-session.json');
      try {
        const content = await fs.readFile(apiSessionFile, 'utf-8');
        const session = JSON.parse(content) as ApiSessionInfo;
        const apiUrl = `http://localhost:${session.port}`;

        if (await this.tryHealthCheck(apiUrl, session.version)) {
          return;
        }
      } catch {
        // Fall through
      }
    }

    const apiUrl = await discoverApiUrl(ctx.ticketsDir);
    await this.tryHealthCheck(apiUrl);
  }

  private async checkOperatorVersion(ctx: SectionContext): Promise<void> {
    const operatorPath = await getOperatorPath(ctx.extensionContext);
    if (operatorPath) {
      this.operatorVersion = await getOperatorVersion(operatorPath) || undefined;
      return;
    }

    try {
      const response = await fetch('https://operator.untra.io/VERSION');
      if (response.ok) {
        this.operatorVersion = (await response.text()).trim() || undefined;
      }
    } catch {
      this.operatorVersion = undefined;
    }
  }

  private async checkWrapperType(ctx: SectionContext): Promise<void> {
    try {
      const config = await ctx.readConfigToml();
      const sessions = config.sessions as Record<string, unknown> | undefined;
      if (sessions?.wrapper && typeof sessions.wrapper === 'string') {
        this.wrapperType = sessions.wrapper;
      } else {
        this.wrapperType = 'vscode';
      }
    } catch {
      this.wrapperType = 'vscode';
    }
  }

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

  getTopLevelItem(ctx: SectionContext): StatusItem {
    return new StatusItem({
      label: 'Connections',
      description: ctx.configReady ? this.getConnectionsSummary() : 'Not Ready',
      icon: ctx.configReady ? this.getConnectionsIcon() : 'debug-configure',
      collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
      sectionId: this.sectionId,
      command: ctx.configReady ? undefined : (
        ctx.extensionContext.globalState.get<string>('operator.workingDirectory')
          ? { command: 'operator.runSetup', title: 'Run Operator Setup' }
          : { command: 'operator.selectWorkingDirectory', title: 'Select Working Directory' }
      ),
    });
  }

  getChildren(ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const configuredBoth = ctx.configReady;

    // 1. Session Wrapper
    const isVscodeWrapper = this.wrapperType === 'vscode';
    const wrapperItem = new StatusItem({
      label: 'Session Wrapper',
      description: isVscodeWrapper ? 'VS Code Terminal' : this.wrapperType,
      icon: isVscodeWrapper ? 'pass' : 'warning',
      tooltip: isVscodeWrapper
        ? 'Sessions route through the VS Code webhook to managed terminals'
        : `Sessions use ${this.wrapperType} — VS Code terminal integration unavailable`,
      sectionId: this.sectionId,
    });

    // 2. API Version
    let versionItem: StatusItem;
    if (this.apiStatus.connected && this.apiStatus.version) {
      const swaggerUrl = `http://localhost:${this.apiStatus.port || 7008}/swagger-ui`;
      versionItem = new StatusItem({
        label: 'Operator',
        description: 'Version ' + this.apiStatus.version,
        icon: 'versions',
        tooltip: 'Open Swagger UI',
        command: {
          command: 'vscode.open',
          title: 'Open Swagger UI',
          arguments: [vscode.Uri.parse(swaggerUrl)],
        },
        sectionId: this.sectionId,
      });
    } else {
      versionItem = new StatusItem({
        label: 'Operator Version',
        description: this.operatorVersion ? 'Version ' + this.operatorVersion : 'Not installed',
        icon: 'versions',
        tooltip: this.operatorVersion
          ? `Installed: ${this.operatorVersion} — click to update`
          : 'Click to download Operator',
        command: {
          command: 'operator.downloadOperator',
          title: 'Download Operator',
        },
        sectionId: this.sectionId,
      });
    }

    // 3. API Connection
    const apiItem = this.apiStatus.connected
      ? new StatusItem({
          label: 'API',
          description: this.apiStatus.url || 'Connected',
          icon: 'pass',
          tooltip: `Operator REST API at ${this.apiStatus.url}`,
          sectionId: this.sectionId,
        })
      : new StatusItem({
          label: 'API',
          description: configuredBoth ? 'Disconnected' : 'Not Ready',
          icon: 'error',
          tooltip: configuredBoth
            ? 'Click to start Operator API server'
            : 'Complete configuration first',
          command: configuredBoth ? {
            command: 'operator.startOperatorServer',
            title: 'Start Operator Server',
          } : undefined,
          sectionId: this.sectionId,
        });

    // 4. Webhook Connection
    const webhookItem = this.webhookStatus.running
      ? new StatusItem({
          label: 'Webhook',
          description: `Running${this.webhookStatus.port ? ` :${this.webhookStatus.port}` : ''}`,
          icon: 'pass',
          tooltip: `Webhook bridge: Operator API \u2192 VS Code terminals (port ${this.webhookStatus.port})`,
          sectionId: this.sectionId,
        })
      : new StatusItem({
          label: 'Webhook',
          description: configuredBoth ? `Stopped` : 'Not Ready',
          icon: 'circle-slash',
          tooltip: configuredBoth
            ? 'Click to start webhook server'
            : 'Complete configuration first',
          command: configuredBoth ? {
            command: 'operator.startWebhookServer',
            title: 'Start Webhook Server',
          } : undefined,
          sectionId: this.sectionId,
        });

    // 5. MCP Connection
    let mcpItem: StatusItem;
    if (this.mcpRegistered) {
      mcpItem = new StatusItem({
        label: 'MCP',
        description: 'Connected',
        icon: 'pass',
        tooltip: 'Operator MCP server is registered in workspace settings',
        command: this.apiStatus.connected ? {
          command: 'operator.connectMcpServer',
          title: 'Reconnect MCP Server',
        } : undefined,
        sectionId: this.sectionId,
      });
    } else if (this.apiStatus.connected) {
      mcpItem = new StatusItem({
        label: 'MCP',
        description: 'Connect',
        icon: 'plug',
        tooltip: 'Connect Operator as MCP server in VS Code',
        command: {
          command: 'operator.connectMcpServer',
          title: 'Connect MCP Server',
        },
        sectionId: this.sectionId,
      });
    } else {
      mcpItem = new StatusItem({
        label: 'MCP',
        description: 'API required',
        icon: 'circle-slash',
        tooltip: 'Start the Operator API to enable MCP connection',
        sectionId: this.sectionId,
      });
    }

    return [wrapperItem, versionItem, apiItem, webhookItem, mcpItem];
  }

  private getConnectionsSummary(): string {
    if (this.apiStatus.connected && this.webhookStatus.running) {
      return 'All connected';
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return 'Partial';
    }
    return 'Disconnected';
  }

  private getConnectionsIcon(): string {
    if (this.apiStatus.connected && this.webhookStatus.running) {
      return 'pass';
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return 'warning';
    }
    return 'error';
  }
}
