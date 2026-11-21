import * as vscode from "vscode";
import * as path from "node:path";
import * as fs from "node:fs/promises";
import { StatusItem } from "../status-item";
import type { SectionContext, StatusSection, WebhookStatus, ApiStatus } from "./types";
import type { SectionId, SectionHealth } from "../generated";
import type { SessionInfo } from "../types";
import type { ApiSessionInfo } from "../api-client";
import { discoverApiUrl, AuthRequiredError, OperatorApiClient } from "../api-client";
import { SIGN_IN_COMMAND_TITLE } from "../auth/errors";
import {
  CHOOSE_CONNECTION_COMMAND,
  localAdoptMismatch,
  readConnectionSettings,
  serverOrigin,
  swaggerUrl,
  usingRemoteServer,
} from "../connection";
import { headerCommand } from "../command-policy";
import { getOperatorPath, getOperatorVersion } from "../operator-binary";
import * as mcpConnect from "../mcp-connect";
import type { McpRegistrationPresence } from "../mcp-connect";

export class ConnectionsSection implements StatusSection {
  readonly sectionId: SectionId = "connections";
  readonly prerequisites: SectionId[] = ["config"];

  private webhookStatus: WebhookStatus = { running: false };
  private apiStatus: ApiStatus = { connected: false };
  private operatorVersion: string | undefined;
  private localDirectoryName: string | undefined;
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
    if (api && wh) {
      return "Green";
    }
    if (api || wh) {
      return "Yellow";
    }
    return "Red";
  }

  async check(ctx: SectionContext): Promise<void> {
    // Top-level name of this workspace (parent of the .tickets dir), used to
    // confirm an already-running API serves THIS project before adopting it.
    this.localDirectoryName = ctx.ticketsDir
      ? path.basename(path.dirname(ctx.ticketsDir))
      : undefined;
    this.remote = usingRemoteServer(
      readConnectionSettings(vscode.workspace.getConfiguration("operator")),
    );
    // A local binary version is the adopt gate for a server on this machine.
    // A remote server is identified by sign-in, not by that binary.
    if (!this.remote) {
      await this.checkOperatorVersion(ctx);
    } else {
      this.operatorVersion = undefined;
    }
    await Promise.allSettled([
      this.checkWebhookStatus(ctx),
      this.checkApiStatus(ctx),
      this.checkWrapperType(ctx),
    ]);
    this.mcpPresence = await mcpConnect.inspectMcpRegistration();
    this.workspaceStdio = mcpConnect.isVscodeWorkspaceStdio();
  }

  private async checkWebhookStatus(ctx: SectionContext): Promise<void> {
    if (!ctx.ticketsDir) {
      this.webhookStatus = { running: false };
      return;
    }

    const webhookSessionFile = path.join(ctx.ticketsDir, "operator", "vscode-session.json");
    try {
      const content = await fs.readFile(webhookSessionFile, "utf-8");
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
    if (!this.remote && ctx.ticketsDir) {
      const apiSessionFile = path.join(ctx.ticketsDir, "operator", "api-session.json");
      try {
        const content = await fs.readFile(apiSessionFile, "utf-8");
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
    if (!apiUrl) {
      this.apiStatus = { connected: false };
      this.webUiAvailable = false;
      return;
    }
    await this.tryHealthCheck(apiUrl);
  }

  private async checkOperatorVersion(ctx: SectionContext): Promise<void> {
    const operatorPath = await getOperatorPath(ctx.extensionContext);
    if (operatorPath) {
      this.operatorVersion = (await getOperatorVersion(operatorPath)) || undefined;
      return;
    }

    try {
      const response = await fetch("https://operator.untra.io/VERSION");
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
      if (sessions?.wrapper && typeof sessions.wrapper === "string") {
        this.wrapperType = sessions.wrapper;
      } else {
        this.wrapperType = "vscode";
      }
    } catch {
      this.wrapperType = "vscode";
    }
  }

  private async tryHealthCheck(apiUrl: string, sessionVersion?: string): Promise<boolean> {
    const client = new OperatorApiClient(apiUrl);
    const portStr = new URL(apiUrl).port;
    const port = portStr ? Number.parseInt(portStr, 10) : 7008;
    try {
      if (await client.isReachable()) {
        const health = await client.health();
        const found = health.version;
        const mismatch = localAdoptMismatch({
          remote: this.remote,
          expectedVersion: this.operatorVersion,
          foundVersion: found,
          localDirectoryName: this.localDirectoryName,
          foundDirectoryName: health.directory_name,
        });
        if (mismatch) {
          this.apiStatus = {
            connected: false,
            port,
            url: apiUrl,
            mismatch,
          };
          this.webUiAvailable = false;
          return false;
        }

        this.apiStatus = {
          connected: true,
          version: found || sessionVersion,
          port,
          url: apiUrl,
          directoryName: health.directory_name,
        };
        await this.checkWebUi(apiUrl);
        return true;
      }
    } catch (err) {
      if (err instanceof AuthRequiredError) {
        // Something is listening and it is Operator, but it will not talk to us yet.
        this.apiStatus = {
          connected: false,
          port,
          url: apiUrl,
          mismatch: { kind: "auth", detail: `at ${apiUrl} requires sign-in` },
        };
        this.webUiAvailable = false;
        return false;
      }
    }
    this.apiStatus = { connected: false, url: apiUrl };
    this.webUiAvailable = false;
    return false;
  }

  private async checkWebUi(apiUrl: string): Promise<void> {
    try {
      const res = await fetch(apiUrl);
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
      command: ctx.configReady
        ? undefined
        : headerCommand(
            this.remote,
            !!ctx.extensionContext.globalState.get<string>("operator.workingDirectory"),
          ),
      health: this.health(),
    });
  }

  getChildren(ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const configuredBoth = ctx.configReady;

    // 1. Session Wrapper
    const isVscodeWrapper = this.wrapperType === "vscode";
    const wrapperItem = new StatusItem({
      label: "Session Wrapper",
      description: isVscodeWrapper ? "VS Code Terminal" : this.wrapperType,
      icon: isVscodeWrapper ? "pass" : "warning",
      tooltip: isVscodeWrapper
        ? "Sessions route through the VS Code webhook to managed terminals"
        : `Sessions use ${this.wrapperType} - VS Code terminal integration unavailable`,
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
        description: this.operatorVersion ? `Version ${this.operatorVersion}` : "Not installed",
        icon: "versions",
        tooltip: this.operatorVersion
          ? `Installed: ${this.operatorVersion} - click to update`
          : "Click to download Operator",
        command: {
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
          command: "operator.startOperatorServer",
          title: "Start Operator Server",
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
        command: {
          command: CHOOSE_CONNECTION_COMMAND,
          title: "Choose Connection",
        },
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
    const webhookItem = this.remote
      ? new StatusItem({
          label: "Webhook",
          description: "Local only",
          icon: "circle-slash",
          tooltip: "VS Code terminals need a local Operator.",
          sectionId: this.sectionId,
        })
      : this.webhookStatus.running
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
    const mcpTooltip = this.remote
      ? "Remote Operator is connected over SSE. Stdio entries are not written for a remote server."
      : mcpStatus.tooltip;
    if (mcpStatus.connected) {
      mcpItem = new StatusItem({
        label: "MCP",
        description: mcpStatus.description,
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

    return [wrapperItem, versionItem, apiItem, webUiItem, webhookItem, mcpItem];
  }

  private getConnectionsSummary(): string {
    if (this.apiStatus.connected && this.webhookStatus.running) {
      return "All connected";
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return "Partial";
    }
    return "Disconnected";
  }

  private getConnectionsIcon(): string {
    if (this.apiStatus.connected && this.webhookStatus.running) {
      return "pass";
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return "warning";
    }
    return "error";
  }
}
