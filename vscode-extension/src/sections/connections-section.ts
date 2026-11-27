import * as vscode from "vscode";
import { assessConnection } from "../connection-assessment";

import { StatusItem } from "../status-item";
import type { SectionContext, StatusSection, WebhookStatus, ApiStatus } from "./types";
import type { SectionId, SectionHealth } from "../generated";

import { resolveWorkingDirectory } from "../config-paths";
import { SIGN_IN_COMMAND_TITLE } from "../auth/errors";
import {
  CHOOSE_CONNECTION_COMMAND,
  normalizeFailureMessage,
  currentConnection,
  serverOrigin,
  swaggerUrl,
} from "../connection";
import { headerCommand } from "../command-policy";
import { getOperatorPath, getOperatorVersion } from "../operator-binary";
import * as mcpConnect from "../mcp-connect";
import type { McpRegistrationPresence } from "../mcp-connect";

export class ConnectionsSection implements StatusSection {
  readonly sectionId: SectionId = "connections";
  readonly prerequisites: SectionId[] = [];

  private webhookStatus: WebhookStatus = { running: false };
  private apiStatus: ApiStatus = { connected: false };
  private binaryError: string | undefined;
  private operatorPath: string | undefined;
  private operatorVersion: string | undefined;

  private mcpPresence: McpRegistrationPresence = {
    vscodeWorkspace: false,
    workspacePortable: false,
    copilotGlobal: false,
    cursor: false,
  };
  private workspaceStdio: boolean | undefined;
  private wrapperType = "vscode";
  private webUiAvailable = false;
  private remote = false;

  get isApiConnected(): boolean {
    return this.apiStatus.connected;
  }

  isConfigured(): boolean {
    return this.apiStatus.connected || this.webhookStatus.running;
  }

  health(): SectionHealth {
    const api = this.apiStatus.connected;
    const wh = this.webhookStatus.running;
    if (api && (this.remote || wh)) {
      return "Green";
    }
    if (api || wh) {
      return "Yellow";
    }
    return currentConnection().selection.kind === "undecided" ? "Gray" : "Red";
  }

  async check(ctx: SectionContext): Promise<void> {
    const { selection } = currentConnection();
    this.remote = selection.kind === "hosted";
    if (selection.kind === "undecided" || selection.kind === "invalid") {
      return;
    }
    if (selection.kind === "local") {
      try {
        await this.checkOperatorVersion(ctx);
      } catch (error) {
        this.binaryError = error instanceof Error ? error.message : String(error);
      }
      this.checkWebhookStatus(ctx);
      await this.checkWrapperType(ctx);
    } else {
      this.webhookStatus = { running: false };
    }
    await this.checkApiStatus(ctx);
    if (this.apiStatus.connected && this.apiStatus.url) {
      try {
        const descriptor = await mcpConnect.fetchMcpDescriptor(this.apiStatus.url);
        this.mcpPresence = await mcpConnect.inspectMcpRegistration(
          this.remote ? { ...descriptor, stdio: undefined } : descriptor,
        );
      } catch {
        // MCP is optional.
      }
    }
    this.workspaceStdio = mcpConnect.isVscodeWorkspaceStdio();
  }

  private checkWebhookStatus(ctx: SectionContext): void {
    this.webhookStatus = ctx.webhookServer?.isRunning()
      ? { running: true, port: ctx.webhookServer.getPort() }
      : { running: false };
  }

  private async checkApiStatus(ctx: SectionContext): Promise<void> {
    const { origin: apiUrl, assessment, selection } = currentConnection();
    if (!apiUrl) {
      this.apiStatus = { connected: false };
      this.webUiAvailable = false;
      return;
    }
    this.apiStatus =
      assessment ??
      (await assessConnection(selection, apiUrl, ctx.ticketsDir, ctx.extensionContext));
    if (this.apiStatus.connected) {
      await this.checkWebUi(apiUrl);
    }
  }

  private async checkOperatorVersion(ctx: SectionContext): Promise<void> {
    const operatorPath = await getOperatorPath(ctx.extensionContext);
    this.operatorPath = operatorPath;
    if (operatorPath) {
      this.operatorVersion = (await getOperatorVersion(operatorPath)) || undefined;
      return;
    }

    this.operatorVersion = undefined;
  }

  private async checkWrapperType(ctx: SectionContext): Promise<void> {
    try {
      const config = await ctx.readConfigToml();
      const sessions = config.sessions as Record<string, unknown> | undefined;
      if (sessions?.wrapper && typeof sessions.wrapper === "string") {
        this.wrapperType = sessions.wrapper;
      } else {
        this.wrapperType = "vscode";
      }
    } catch {
      this.wrapperType = "vscode";
    }
  }

  private async checkWebUi(apiUrl: string): Promise<void> {
    try {
      const res = await fetch(apiUrl, { signal: currentConnection().signal });
      this.webUiAvailable = res.ok && (res.headers.get("content-type") || "").includes("text/html");
    } catch {
      this.webUiAvailable = false;
    }
  }

  getTopLevelItem(ctx: SectionContext): StatusItem {
    return new StatusItem({
      label: "Connections",
      description: ctx.configReady ? this.getConnectionsSummary() : "Not Ready",
      icon: ctx.configReady ? this.getConnectionsIcon() : "debug-configure",
      collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
      sectionId: this.sectionId,
      command: ["undecided", "invalid"].includes(currentConnection().selection.kind)
        ? { command: CHOOSE_CONNECTION_COMMAND, title: "Choose Connection" }
        : ctx.configReady
          ? undefined
          : headerCommand(this.remote, !!resolveWorkingDirectory()),
      health: this.health(),
    });
  }

  getChildren(ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const { selection } = currentConnection();
    const choose = new StatusItem({
      label: "Choose Connection",
      icon: "plug",
      description:
        selection.kind === "invalid" ? normalizeFailureMessage(selection.reason) : undefined,
      command: { command: CHOOSE_CONNECTION_COMMAND, title: "Choose Connection" },
      sectionId: this.sectionId,
    });
    if (selection.kind === "undecided" || selection.kind === "invalid") {
      return [choose];
    }
    const configuredBoth = ctx.configReady;

    // 1. Session Wrapper
    const isVscodeWrapper = this.wrapperType === "vscode";
    const wrapperItem = new StatusItem({
      label: "Session Wrapper",
      description: isVscodeWrapper ? "VS Code Terminal" : this.wrapperType,
      icon: isVscodeWrapper ? "pass" : "terminal",
      tooltip: isVscodeWrapper
        ? "Sessions route through the VS Code webhook to managed terminals"
        : `CLI launches use ${this.wrapperType}. Extension launches use VS Code terminals.`,
      sectionId: this.sectionId,
    });

    // 2. API Version
    let versionItem: StatusItem;
    const origin = this.apiStatus.url ? serverOrigin(this.apiStatus.url) : undefined;
    const docs = this.apiStatus.url ? swaggerUrl(this.apiStatus.url) : undefined;
    if (this.apiStatus.connected && this.apiStatus.version && docs) {
      versionItem = new StatusItem({
        label: "Operator",
        description: `Version ${this.apiStatus.version}`,
        icon: "versions",
        tooltip: "Open Swagger UI",
        command: {
          command: "vscode.open",
          title: "Open Swagger UI",
          arguments: [vscode.Uri.parse(docs)],
        },
        sectionId: this.sectionId,
      });
    } else if (this.remote) {
      versionItem = new StatusItem({
        label: "Operator",
        description: origin ?? "Remote",
        icon: "cloud",
        tooltip: "This window uses a remote Operator and does not download a binary.",
        command: {
          command: CHOOSE_CONNECTION_COMMAND,
          title: "Choose Connection",
        },
        sectionId: this.sectionId,
      });
    } else {
      versionItem = new StatusItem({
        label: "Operator Version",
        description: this.binaryError
          ? "Check operator.operatorPath"
          : this.operatorVersion
            ? `Version ${this.operatorVersion}`
            : "Not installed",
        icon: "versions",
        tooltip:
          this.binaryError ??
          (this.operatorVersion
            ? `Installed: ${this.operatorPath} (v${this.operatorVersion}) - click to update`
            : "Click to download Operator"),
        command: this.binaryError
          ? {
              command: "workbench.action.openSettings",
              title: "Configure Operator executable",
              arguments: ["operator.operatorPath"],
            }
          : {
              command: "operator.downloadOperator",
              title: "Download Operator",
            },
        sectionId: this.sectionId,
      });
    }

    // 3. API Connection
    let apiItem: StatusItem;
    if (this.apiStatus.connected) {
      apiItem = new StatusItem({
        label: "API",
        description: this.apiStatus.url ? this.apiStatus.url : "Connected",
        icon: "pass",
        tooltip: this.apiStatus.directoryName
          ? `Operator REST API at ${this.apiStatus.url} (serves '${this.apiStatus.directoryName}')`
          : `Operator REST API at ${this.apiStatus.url}`,
        sectionId: this.sectionId,
      });
    } else if (this.apiStatus.mismatch?.kind === "auth") {
      apiItem = new StatusItem({
        label: "API",
        description: "Sign-in required",
        icon: "key",
        tooltip:
          `The Operator API ${this.apiStatus.mismatch.detail}. ` +
          `Run "${SIGN_IN_COMMAND_TITLE}" to authorize this editor.`,
        command: { command: "operator.signIn", title: SIGN_IN_COMMAND_TITLE },
        sectionId: this.sectionId,
      });
    } else if (this.apiStatus.mismatch) {
      // An API answered on the port but is not adoptable as ours.
      const mismatch = this.apiStatus.mismatch;
      apiItem = new StatusItem({
        label: "API",
        description: mismatch.kind === "version" ? "Version mismatch" : "Different project",
        icon: "warning",
        tooltip:
          `An Operator API on this port ${mismatch.detail}. ` +
          "Stop the other instance or configure a different port (operator.apiUrl), then start your own server.",
        command: {
          command: CHOOSE_CONNECTION_COMMAND,
          title: "Choose Connection",
        },
        sectionId: this.sectionId,
      });
    } else if (this.remote) {
      apiItem = new StatusItem({
        label: "API",
        description: this.apiStatus.url ? "Can't reach" : "Not configured",
        icon: "error",
        tooltip: this.apiStatus.url
          ? `Can't reach Operator at ${this.apiStatus.url}.`
          : "Run Operator: Choose Connection and enter the server's https address.",
        command: { command: "operator.retryConnection", title: "Retry connection" },
        sectionId: this.sectionId,
      });
    } else {
      apiItem = new StatusItem({
        label: "API",
        description: configuredBoth ? "Disconnected" : "Not Ready",
        icon: "error",
        tooltip: configuredBoth
          ? "Click to start Operator API server"
          : "Complete configuration first",
        command: configuredBoth
          ? {
              command: "operator.startOperatorServer",
              title: "Start Operator Server",
            }
          : undefined,
        sectionId: this.sectionId,
      });
    }

    // 4. Web UI
    const webUiItem =
      this.webUiAvailable && origin
        ? new StatusItem({
            label: "Web UI",
            description: origin,
            icon: "pass",
            tooltip: "Click to open the embedded web UI in browser",
            command: {
              command: "vscode.open",
              title: "Open Web UI",
              arguments: [vscode.Uri.parse(origin)],
            },
            sectionId: this.sectionId,
          })
        : new StatusItem({
            label: "Web UI",
            description: this.apiStatus.connected ? "Not available" : "API required",
            icon: "circle-slash",
            tooltip: this.apiStatus.connected
              ? "The embedded web UI requires the binary to be built with --features embed-ui"
              : "Start the Operator API to enable the web UI",
            sectionId: this.sectionId,
          });

    // 5. Webhook Connection
    const webhookItem = this.webhookStatus.running
      ? new StatusItem({
          label: "Webhook",
          description: `Running${this.webhookStatus.port ? ` :${this.webhookStatus.port}` : ""}`,
          icon: "pass",
          tooltip: `Webhook bridge: Operator API \u2192 VS Code terminals (port ${this.webhookStatus.port})`,
          sectionId: this.sectionId,
        })
      : new StatusItem({
          label: "Webhook",
          description: configuredBoth ? `Stopped` : "Not Ready",
          icon: "circle-slash",
          tooltip: configuredBoth
            ? "Click to start webhook server"
            : "Complete configuration first",
          command: configuredBoth
            ? {
                command: "operator.startWebhookServer",
                title: "Start Webhook Server",
              }
            : undefined,
          sectionId: this.sectionId,
        });

    // 5. MCP Connection
    const mcpStatus = mcpConnect.mcpStatusDescription(
      this.mcpPresence,
      mcpConnect.detectHostApp(),
      this.workspaceStdio,
    );
    let mcpItem: StatusItem;
    const mcpTooltip = mcpStatus.tooltip;
    if (mcpStatus.connected) {
      mcpItem = new StatusItem({
        label: "MCP",
        description: `Registered · ${mcpStatus.description}`,
        icon: "pass",
        tooltip: mcpTooltip,
        command: this.apiStatus.connected
          ? {
              command: "operator.connectMcpServer",
              title: "Reconnect MCP Server",
            }
          : undefined,
        sectionId: this.sectionId,
      });
    } else if (this.apiStatus.connected) {
      mcpItem = new StatusItem({
        label: "MCP",
        description: mcpStatus.description,
        icon: "plug",
        tooltip: mcpTooltip,
        command: {
          command: "operator.connectMcpServer",
          title: "Connect MCP Server",
        },
        sectionId: this.sectionId,
      });
    } else {
      mcpItem = new StatusItem({
        label: "MCP",
        description: "API required",
        icon: "circle-slash",
        tooltip: "Start the Operator API to enable MCP connection",
        sectionId: this.sectionId,
      });
    }

    return this.remote
      ? [choose, versionItem, apiItem, webUiItem, mcpItem]
      : [choose, wrapperItem, versionItem, apiItem, webUiItem, webhookItem, mcpItem];
  }

  private getConnectionsSummary(): string {
    if (this.apiStatus.connected && (this.remote || this.webhookStatus.running)) {
      return "All connected";
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return "Partial";
    }
    return "Disconnected";
  }

  private getConnectionsIcon(): string {
    if (this.apiStatus.connected && (this.remote || this.webhookStatus.running)) {
      return "pass";
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return "warning";
    }
    return "error";
  }
}
