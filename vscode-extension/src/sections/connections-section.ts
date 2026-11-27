import * as vscode from "vscode";
import { assessConnection } from "../connection-assessment";

import { StatusItem } from "../status-item";
import type { SectionContext, StatusSection, WebhookStatus, ApiStatus } from "./types";
import type { SectionId, SectionHealth } from "../generated";

import { resolveWorkingDirectory } from "../config-paths";
import { pendingApproval } from "../auth/sign-in";
import { SIGN_IN_COMMAND_TITLE } from "../auth/errors";
import {
  CHOOSE_CONNECTION_COMMAND,
  type ConnectionSelection,
  normalizeFailureMessage,
  currentConnection,
  serverOrigin,
  swaggerUrl,
} from "../connection";
import { headerCommand } from "../command-policy";
import { getOperatorPath, getOperatorVersion } from "../operator-binary";
import * as mcpConnect from "../mcp-connect";
import type { McpRegistrationPresence } from "../mcp-connect";

const CONNECTION_PRESENTATION = {
  Green: { description: "All connected", icon: "pass" },
  Yellow: { description: "Partial", icon: "warning" },
  Red: { description: "Disconnected", icon: "error" },
  Gray: { description: "Disconnected", icon: "error" },
};

const DEFAULT_WRAPPER = "vscode";
const NO_REGISTRATION: McpRegistrationPresence = {
  vscodeWorkspace: false,
  workspacePortable: false,
  copilotGlobal: false,
  cursor: false,
};

interface BinaryStatus {
  path?: string;
  version?: string;
  error?: string;
}

interface ConnectionResult {
  selection: ConnectionSelection;
  hasWorkingDirectory: boolean;
  host: mcpConnect.HostApp;
  webhookStatus: WebhookStatus;
  apiStatus: ApiStatus;
  binary: BinaryStatus;
  mcpPresence: McpRegistrationPresence;
  workspaceStdio?: boolean;
  wrapperType: string;
  webUiAvailable: boolean;
}

function initialResult(selection: ConnectionSelection): ConnectionResult {
  return {
    selection,
    hasWorkingDirectory: !!resolveWorkingDirectory(),
    host: mcpConnect.detectHostApp(),
    webhookStatus: { running: false },
    apiStatus: { connected: false },
    binary: {},
    mcpPresence: NO_REGISTRATION,
    wrapperType: DEFAULT_WRAPPER,
    webUiAvailable: false,
  };
}

async function checkBinary(ctx: SectionContext): Promise<BinaryStatus> {
  try {
    const path = await getOperatorPath(ctx.extensionContext);
    return path ? { path, version: await getOperatorVersion(path) } : {};
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

function checkWebhook(server: SectionContext["webhookServer"]): WebhookStatus {
  return server?.isRunning() ? { running: true, port: server.getPort() } : { running: false };
}

async function checkWrapperType(ctx: SectionContext): Promise<string> {
  try {
    const config = await ctx.readConfigToml();
    const sessions = config.sessions as Record<string, unknown> | undefined;
    return typeof sessions?.wrapper === "string" && sessions.wrapper
      ? sessions.wrapper
      : DEFAULT_WRAPPER;
  } catch {
    return DEFAULT_WRAPPER;
  }
}

async function checkWebUi(url: string, signal: AbortSignal): Promise<boolean> {
  try {
    const res = await fetch(url, { signal });
    return res.ok && (res.headers.get("content-type") || "").includes("text/html");
  } catch {
    return false;
  }
}

async function checkMcp(api: ApiStatus, remote: boolean): Promise<McpRegistrationPresence> {
  if (!api.connected || !api.url) {
    return NO_REGISTRATION;
  }
  try {
    const descriptor = await mcpConnect.fetchMcpDescriptor(api.url);
    return await mcpConnect.inspectMcpRegistration(
      remote ? { ...descriptor, stdio: undefined } : descriptor,
    );
  } catch {
    return NO_REGISTRATION;
  }
}

async function checkConnection(
  { selection, origin, assessment, signal }: ReturnType<typeof currentConnection>,
  ctx: SectionContext,
): Promise<ConnectionResult> {
  const result = initialResult(selection);
  if (selection.kind === "undecided" || selection.kind === "invalid") {
    return result;
  }
  if (selection.kind === "local") {
    result.binary = await checkBinary(ctx);
    result.webhookStatus = checkWebhook(ctx.webhookServer);
    result.wrapperType = await checkWrapperType(ctx);
  }
  if (origin) {
    result.apiStatus =
      assessment ??
      (await assessConnection(selection, origin, ctx.ticketsDir, ctx.extensionContext));
    result.webUiAvailable = result.apiStatus.connected && (await checkWebUi(origin, signal));
  }
  result.mcpPresence = await checkMcp(result.apiStatus, selection.kind === "hosted");
  result.workspaceStdio = mcpConnect.isVscodeWorkspaceStdio();
  return result;
}

export class ConnectionsSection implements StatusSection {
  readonly sectionId: SectionId = "connections";
  readonly prerequisites: SectionId[] = [];
  private result = initialResult(currentConnection().selection);

  get isApiConnected(): boolean {
    return this.result.apiStatus.connected;
  }

  isConfigured(): boolean {
    return this.isApiConnected || this.result.webhookStatus.running;
  }

  health(): SectionHealth {
    const { selection, webhookStatus } = this.result;
    if (this.isApiConnected && (selection.kind === "hosted" || webhookStatus.running)) {
      return "Green";
    }
    if (this.isConfigured()) {
      return "Yellow";
    }
    return selection.kind === "undecided" ? "Gray" : "Red";
  }

  async check(ctx: SectionContext): Promise<void> {
    this.result = await checkConnection(currentConnection(), ctx);
  }

  getTopLevelItem(ctx: SectionContext): StatusItem {
    const { selection, hasWorkingDirectory } = this.result;
    const health = this.health();
    return new StatusItem({
      label: "Connections",
      ...(ctx.configReady
        ? CONNECTION_PRESENTATION[health]
        : { description: "Not Ready", icon: "debug-configure" }),
      collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
      sectionId: this.sectionId,
      command: ["undecided", "invalid"].includes(selection.kind)
        ? { command: CHOOSE_CONNECTION_COMMAND, title: "Choose Connection" }
        : ctx.configReady
          ? undefined
          : headerCommand(selection.kind === "hosted", hasWorkingDirectory),
      health,
    });
  }

  getChildren(ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const {
      selection,
      wrapperType,
      apiStatus,
      binary,
      webUiAvailable,
      webhookStatus,
      host,
      mcpPresence,
      workspaceStdio,
    } = this.result;
    const remote = selection.kind === "hosted";
    const choose = new StatusItem({
      label:
        selection.kind === "undecided" ? "Connect Operator to This Workspace" : "Choose Connection",
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
    const isVscodeWrapper = wrapperType === DEFAULT_WRAPPER;
    const wrapperItem = new StatusItem({
      label: "Session Wrapper",
      description: isVscodeWrapper ? "VS Code Terminal" : wrapperType,
      icon: isVscodeWrapper ? "pass" : "terminal",
      tooltip: isVscodeWrapper
        ? "Sessions route through the VS Code webhook to managed terminals"
        : `CLI launches use ${wrapperType}. Extension launches use VS Code terminals.`,
      sectionId: this.sectionId,
    });

    // 2. API Version
    let versionItem: StatusItem;
    const origin = apiStatus.url ? serverOrigin(apiStatus.url) : undefined;
    const docs = apiStatus.url ? swaggerUrl(apiStatus.url) : undefined;
    if (apiStatus.connected && apiStatus.version && docs) {
      versionItem = new StatusItem({
        label: "Operator",
        description: `Version ${apiStatus.version}`,
        icon: "versions",
        tooltip: "Open Swagger UI",
        command: {
          command: "vscode.open",
          title: "Open Swagger UI",
          arguments: [vscode.Uri.parse(docs)],
        },
        sectionId: this.sectionId,
      });
    } else if (remote) {
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
        description: binary.error
          ? "Check operator.operatorPath"
          : binary.version
            ? `Version ${binary.version}`
            : "Not installed",
        icon: "versions",
        tooltip:
          binary.error ??
          (binary.version
            ? `Installed: ${binary.path} (v${binary.version}) - click to update`
            : "Click to download Operator"),
        command: binary.error
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
    if (apiStatus.connected) {
      apiItem = new StatusItem({
        label: "API",
        description: remote
          ? [apiStatus.subject, apiStatus.url].filter(Boolean).join(" · ")
          : `Connected locally · ${ctx.ticketsDir ?? apiStatus.url}`,
        icon: "pass",
        tooltip: apiStatus.directoryName
          ? `Operator REST API at ${apiStatus.url} (serves '${apiStatus.directoryName}')`
          : `Operator REST API at ${apiStatus.url}`,
        sectionId: this.sectionId,
      });
    } else if (apiStatus.mismatch?.kind === "auth") {
      apiItem = new StatusItem({
        label: "API",
        description: remote ? `Sign in to ${apiStatus.url}` : "Retry Local Connection",
        icon: "key",
        tooltip: !remote
          ? `Check the daemon registration in ${ctx.ticketsDir ?? "the selected configuration"} and retry.`
          : `The Operator API ${apiStatus.mismatch.detail}. ` +
            `Run "${SIGN_IN_COMMAND_TITLE}" to authorize this editor.`,
        command: remote
          ? { command: "operator.signIn", title: SIGN_IN_COMMAND_TITLE }
          : { command: "operator.retryConnection", title: "Retry Local Connection" },
        sectionId: this.sectionId,
      });
    } else if (apiStatus.mismatch?.kind === "forbidden") {
      apiItem = new StatusItem({
        label: "API",
        description: "Access required",
        tooltip: apiStatus.mismatch.detail,
        icon: "lock",
        sectionId: this.sectionId,
      });
    } else if (apiStatus.mismatch) {
      // An API answered on the port but is not adoptable as ours.
      const mismatch = apiStatus.mismatch;
      apiItem = new StatusItem({
        label: "API",
        description: mismatch.kind === "version" ? "Version mismatch" : "Different configuration",
        icon: "warning",
        tooltip: `An Operator API on this port ${mismatch.detail}. ${
          mismatch.kind === "version"
            ? "Check the configured Operator executable and server version."
            : "Choose the connection for this workspace or select the correct Operator configuration."
        }`,
        command: {
          command: CHOOSE_CONNECTION_COMMAND,
          title: "Choose Connection",
        },
        sectionId: this.sectionId,
      });
    } else if (remote) {
      apiItem = new StatusItem({
        label: "API",
        description: apiStatus.url ? "Can't reach" : "Not configured",
        icon: "error",
        tooltip: apiStatus.url
          ? `Can't reach Operator at ${apiStatus.url}.`
          : "Run Operator: Choose Connection and enter the server's https address.",
        command: { command: "operator.retryConnection", title: "Retry connection" },
        sectionId: this.sectionId,
      });
    } else {
      apiItem = new StatusItem({
        label: "API",
        description: configuredBoth ? "Start Local Operator" : "Not Ready",
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
      webUiAvailable && origin
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
            description: apiStatus.connected ? "Not available" : "API required",
            icon: "circle-slash",
            tooltip: apiStatus.connected
              ? "The embedded web UI requires the binary to be built with --features embed-ui"
              : "Start the Operator API to enable the web UI",
            sectionId: this.sectionId,
          });

    // 5. Webhook Connection
    const webhookItem = webhookStatus.running
      ? new StatusItem({
          label: "Webhook",
          description: `Running${webhookStatus.port ? ` :${webhookStatus.port}` : ""}`,
          icon: "pass",
          tooltip: `Webhook bridge: Operator API \u2192 VS Code terminals (port ${webhookStatus.port})`,
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
    const mcpStatus = mcpConnect.mcpStatusDescription(mcpPresence, host, workspaceStdio);
    let mcpItem: StatusItem;
    const mcpTooltip = mcpStatus.tooltip;
    if (mcpStatus.connected) {
      mcpItem = new StatusItem({
        label: "MCP",
        description: `Registered · ${mcpStatus.description}`,
        icon: "pass",
        tooltip: mcpTooltip,
        command: apiStatus.connected
          ? {
              command: "operator.connectMcpServer",
              title: "Reconnect MCP Server",
            }
          : undefined,
        sectionId: this.sectionId,
      });
    } else if (apiStatus.connected) {
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

    const action = (label: string, command: string, description?: string) =>
      new StatusItem({
        label,
        icon: "plug",
        description,
        sectionId: this.sectionId,
        command: { command, title: label },
      });
    const items = remote
      ? [choose, versionItem, apiItem, webUiItem, mcpItem]
      : [choose, wrapperItem, versionItem, apiItem, webUiItem, webhookItem, mcpItem];
    if (!remote && apiStatus.mismatch?.kind === "auth") {
      items.push(action("Choose Configuration", "operator.selectWorkingDirectory"));
    }
    const approval = pendingApproval();
    if (approval) {
      items.unshift(
        action("Open Approval Page", "operator.signIn", approval.address),
        action("Copy Code", "operator.copyApprovalCode", approval.code),
        action("Cancel Sign-in", "operator.cancelApproval"),
      );
    }
    items.push(action("Disconnect This Workspace", "operator.disconnectWorkspace"));
    return items;
  }
}
