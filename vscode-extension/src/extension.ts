/**
 * Operator Terminals VS Code Extension
 *
 * Provides terminal integration for Operator multi-agent orchestration.
 * Creates and manages terminals with ticket-specific styling and
 * activity tracking via a local webhook server.
 *
 * The server writes a session file to .tickets/operator/vscode-session.json
 * for Operator to discover the connection details (port may be dynamic).
 */

import * as vscode from "vscode";
import * as path from "node:path";
import * as os from "node:os";
import { TerminalManager } from "./terminal-manager";
import { WebhookServer } from "./webhook-server";
import type { TicketItem } from "./ticket-provider";
import { TicketTreeProvider } from "./ticket-provider";
import { StatusTreeProvider, StatusItem } from "./status-provider";
import { LaunchManager } from "./launch-manager";
import { IssueTypeService } from "./issuetype-service";
import type { TicketInfo } from "./types";
import { OperatorApiClient, discoverApiUrl, discoverLocalApiUrl } from "./api-client";
import { AuthRequiredError } from "./auth/errors";
import {
  APPROVED_WORKSPACE_ORIGIN,
  CHOOSE_CONNECTION_COMMAND,
  CONNECTION_MODE_LOCAL,
  CONNECTION_MODE_REMOTE,
  DOWNLOADS_PAGE,
  PICK_LOCAL,
  PICK_REMOTE,
  currentConnection,
  normalizeFailureMessage,
  normalizeRemoteOrigin,
  resetConnection,
  resolveConnection,
  setConnectionOrigin,
} from "./connection";
import { SerialQueue } from "./serial-queue";
import {
  focusMissAction,
  guardRemoteCommand,
  launchWrapper,
  ticketOpenAction,
} from "./command-policy";
import { runConnectionPicker } from "./connection-picker";
import {
  handleConnect,
  openAgentTerminal,
  resumePendingConnect,
  type ConnectDeps,
} from "./agent-connect";
import {
  OperatorCredentials,
  clearCredentialProvider,
  setCredentialProvider,
} from "./auth/credentials";
import { TokenStore } from "./auth/token-store";
import { signIn, signOut } from "./auth/sign-in";
import { showLaunchOptionsDialog, showTicketPicker } from "./launch-dialog";
import { parseTicketMetadata, getCurrentSessionId } from "./ticket-parser";
import {
  getOperatorPath,
  getOperatorVersion,
  getExtensionVersion,
  isOperatorAvailable,
  downloadOperator,
} from "./operator-binary";
import {
  selectWorkingDirectory,
  checkKanbanConnection,
  configureJira,
  configureLinear,
  detectLlmTools,
  openWalkthrough,
  startKanbanOnboarding,
  updateWalkthroughContext,
  initializeTicketsDirectory,
} from "./walkthrough";
import { startGitOnboarding, onboardGitHub, onboardGitLab } from "./git-onboarding";
import { ConfigPanel } from "./config-panel";
import { openOperatorUi } from "./open-operator-ui";
import { CampaignTreeProvider, type CampaignNode } from "./campaign-provider";
import { routeOperatorUri } from "./uri-handler";
import {
  launchCampaignCommand,
  openCampaignCommand,
  pauseCampaignCommand,
  type CampaignCommandContext,
} from "./campaign-commands";
import { connectMcpServer } from "./mcp-connect";
import { configFileExists } from "./config-paths";
import { findParentTicketsDir, findTicketsDir, findOperatorServerDir } from "./tickets-dir";
import { addJiraProject, addLinearTeam } from "./kanban-onboarding";

// ---------------------------------------------------------------------------
// CommandContext interface
// ---------------------------------------------------------------------------

interface CommandContext {
  extensionContext: vscode.ExtensionContext;
  terminalManager: TerminalManager;
  webhookServer: WebhookServer;
  launchManager: LaunchManager;
  issueTypeService: IssueTypeService;
  statusProvider: StatusTreeProvider;
  statusTreeView: vscode.TreeView<StatusItem>;
  queueProvider: TicketTreeProvider;
  inProgressProvider: TicketTreeProvider;
  completedProvider: TicketTreeProvider;
  statusBarItem: vscode.StatusBarItem;
  createBarItem: vscode.StatusBarItem;
  outputChannel: vscode.OutputChannel;
  /** The Operator server this window is attached to (probe-first activation) */
  attachedServer?: { url: string; version?: string };
  reconnect: () => Promise<void>;
  listenerQueue: SerialQueue;
  getCurrentTicketsDir: () => string | undefined;
  setCurrentTicketsDir: (dir: string | undefined) => void;
  refreshAllProviders: () => Promise<void>;
  setTicketsDir: (dir: string | undefined) => Promise<void>;
}

// ---------------------------------------------------------------------------
// Module state
// ---------------------------------------------------------------------------

let currentTicketsDir: string | undefined;

// ---------------------------------------------------------------------------
// Launch commands
// ---------------------------------------------------------------------------

function isTicketFile(filePath: string): boolean {
  const normalized = filePath.replace(/\\/g, "/");
  return (
    (normalized.includes(".tickets/queue/") || normalized.includes(".tickets/in-progress/")) &&
    normalized.endsWith(".md")
  );
}

async function launchTicketCommand(ctx: CommandContext, treeItem?: TicketItem): Promise<void> {
  let ticket: TicketInfo | undefined;

  if (treeItem?.ticket) {
    ticket = treeItem.ticket;
  } else {
    const tickets = ctx.queueProvider.getTickets();
    if (tickets.length === 0) {
      void vscode.window.showInformationMessage("No tickets in queue");
      return;
    }
    ticket = await showTicketPicker(tickets);
  }

  if (!ticket) {
    return;
  }

  await ctx.launchManager.launchTicket(ticket, {
    delegator: null,
    model: "sonnet",
    yoloMode: false,
    resumeSession: false,
  });
}

async function launchTicketWithOptionsCommand(
  ctx: CommandContext,
  treeItem?: TicketItem,
): Promise<void> {
  let ticket: TicketInfo | undefined;

  if (treeItem?.ticket) {
    ticket = treeItem.ticket;
  } else {
    const tickets = ctx.queueProvider.getTickets();
    if (tickets.length === 0) {
      void vscode.window.showInformationMessage("No tickets in queue");
      return;
    }
    ticket = await showTicketPicker(tickets);
  }

  if (!ticket) {
    return;
  }

  const metadata = await parseTicketMetadata(ticket.filePath);
  const hasSession = metadata ? !!getCurrentSessionId(metadata) : false;

  const options = await showLaunchOptionsDialog(ticket, hasSession);
  if (!options) {
    return;
  }

  await ctx.launchManager.launchTicket(ticket, options);
}

async function relaunchTicketCommand(ctx: CommandContext, ticket: TicketInfo): Promise<void> {
  await ctx.launchManager.offerRelaunch(ticket);
}

async function launchTicketFromEditorCommand(ctx: CommandContext): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    void vscode.window.showWarningMessage("No active editor");
    return;
  }

  const filePath = editor.document.uri.fsPath;
  if (!isTicketFile(filePath)) {
    void vscode.window.showWarningMessage("Current file is not a ticket in .tickets/ directory");
    return;
  }

  const metadata = await parseTicketMetadata(filePath);
  if (!metadata?.id) {
    void vscode.window.showErrorMessage("Could not parse ticket ID from file");
    return;
  }

  const apiClient = new OperatorApiClient(await discoverApiUrl(ctx.getCurrentTicketsDir()));

  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  try {
    const response = await apiClient.launchTicket(metadata.id, {
      delegator: null,
      provider: null,
      wrapper: launchWrapper(currentConnection().selection.kind === "hosted"),
      model: "sonnet",
      model_server: null,
      yolo_mode: false,
      retry_reason: null,
      resume_session_id: null,
      target: null,
    });

    await ctx.launchManager.presentLaunchResponse(response);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
  }
}

async function launchTicketFromEditorWithOptionsCommand(ctx: CommandContext): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    void vscode.window.showWarningMessage("No active editor");
    return;
  }

  const filePath = editor.document.uri.fsPath;
  if (!isTicketFile(filePath)) {
    void vscode.window.showWarningMessage("Current file is not a ticket in .tickets/ directory");
    return;
  }

  const metadata = await parseTicketMetadata(filePath);
  if (!metadata?.id) {
    void vscode.window.showErrorMessage("Could not parse ticket ID from file");
    return;
  }

  const ticketType = ctx.issueTypeService.extractTypeFromId(metadata.id);
  const ticketStatus =
    metadata.status === "in-progress" || metadata.status === "completed"
      ? metadata.status
      : ("queue" as const);
  const ticketInfo: TicketInfo = {
    id: metadata.id,
    type: ticketType,
    title: "Ticket from editor",
    status: ticketStatus,
    filePath: filePath,
  };

  const hasSession = !!getCurrentSessionId(metadata);
  const options = await showLaunchOptionsDialog(ticketInfo, hasSession);
  if (!options) {
    return;
  }

  const apiClient = new OperatorApiClient(await discoverApiUrl(ctx.getCurrentTicketsDir()));

  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  try {
    const response = await apiClient.launchTicket(metadata.id, {
      delegator: options.delegator ?? null,
      provider: null,
      wrapper: launchWrapper(currentConnection().selection.kind === "hosted"),
      model: options.model,
      model_server: null,
      yolo_mode: options.yoloMode,
      retry_reason: null,
      resume_session_id: null,
      target: options.target ?? null,
    });

    await ctx.launchManager.presentLaunchResponse(response);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
  }
}

async function focusTicketTerminal(
  ctx: CommandContext,
  terminalName: string,
  ticket?: TicketInfo,
): Promise<void> {
  if (ctx.terminalManager.exists(terminalName)) {
    ctx.terminalManager.focus(terminalName);
    return;
  }
  if (focusMissAction(currentConnection().selection.kind === "hosted") === "open-queue") {
    await openOperatorUi(ctx.getCurrentTicketsDir(), "queue");
    return;
  }
  if (ticket) {
    await ctx.launchManager.offerRelaunch(ticket);
    return;
  }
  void vscode.window.showWarningMessage(`Terminal '${terminalName}' not found`);
}

function openTicketFile(filePath: string, ticketId?: string): void {
  const action = ticketOpenAction(currentConnection().selection.kind === "hosted", ticketId);
  if (action === "ui" && ticketId) {
    void openOperatorUi(currentTicketsDir, "tickets", ticketId);
    return;
  }
  if (action === "unavailable") {
    void vscode.window.showInformationMessage("This ticket lives on the Operator server.");
    return;
  }
  void vscode.workspace.openTextDocument(filePath).then((doc) => {
    return vscode.window.showTextDocument(doc);
  });
}

/** Open a ticket by id, asking the Operator server where its file lives. */
async function openTicketById(ticketId: string): Promise<void> {
  if (ticketOpenAction(currentConnection().selection.kind === "hosted", ticketId) === "ui") {
    await openOperatorUi(currentTicketsDir, "tickets", ticketId);
    return;
  }
  try {
    const client = new OperatorApiClient(await discoverApiUrl(currentTicketsDir));
    openTicketFile((await client.getTicket(ticketId)).filepath);
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showWarningMessage(`Operator: could not open ${ticketId}: ${msg}`);
  }
}

// ---------------------------------------------------------------------------
// Server commands
// ---------------------------------------------------------------------------

async function startWebhookServer(ctx: CommandContext): Promise<void> {
  if (await guardRemoteCommand("operator.startWebhookServer", ctx.getCurrentTicketsDir())) {
    return;
  }
  const { signal, selection } = currentConnection();
  await ctx.listenerQueue.run(async () => {
    if (!signal.aborted && selection.kind === "local") {
      await startLocalWebhookServer(ctx, signal);
    }
  });
}

async function startLocalWebhookServer(ctx: CommandContext, signal: AbortSignal): Promise<void> {
  const hasConfig = await configFileExists(ctx.getCurrentTicketsDir());
  if (!hasConfig) {
    showConfigMissingNotification();
    return;
  }

  if (ctx.webhookServer.isRunning()) {
    const ticketsDir = await findTicketsDir();
    if (ticketsDir && !signal.aborted) {
      await ctx.webhookServer.ensureSessionFile(ticketsDir);
    }
    void vscode.window.showInformationMessage(
      `Webhook connected on port ${ctx.webhookServer.getPort()}`,
    );
    return;
  }

  try {
    const ticketsDir = await findTicketsDir();
    if (signal.aborted) {
      return;
    }
    await ctx.webhookServer.start(ticketsDir);
    if (signal.aborted) {
      return;
    }

    const port = ctx.webhookServer.getPort();
    const configuredPort = ctx.webhookServer.getConfiguredPort();

    if (port !== configuredPort) {
      void vscode.window.showInformationMessage(
        `Operator webhook server started on port ${port} (configured port ${configuredPort} was in use)`,
      );
    } else {
      void vscode.window.showInformationMessage(`Operator webhook server started on port ${port}`);
    }

    updateStatusBar(ctx);
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to start webhook server: ${msg}`);
  }
}

function showStatus(ctx: CommandContext): void {
  const running = ctx.webhookServer.isRunning();
  const port = ctx.webhookServer.getPort();
  const configuredPort = ctx.webhookServer.getConfiguredPort();
  const terminals = ctx.terminalManager.list();

  let message: string;
  if (running) {
    if (port !== configuredPort) {
      message = `Operator server running on port ${port} (fallback from ${configuredPort})\nManaged terminals: ${terminals.length}`;
    } else {
      message = `Operator server running on port ${port}\nManaged terminals: ${terminals.length}`;
    }
  } else {
    message = "Operator server stopped";
  }

  void vscode.window.showInformationMessage(message);
}

function updateStatusBar(ctx: CommandContext): void {
  // Two distinct facts: which Operator server this window is attached to,
  // and whether the extension's own webhook listener is up.
  const webhook = ctx.webhookServer.isRunning()
    ? `webhook :${ctx.webhookServer.getPort()}`
    : "webhook off";
  if (ctx.attachedServer) {
    const coderWorkspace = process.env.CODER_WORKSPACE_NAME;
    const label = coderWorkspace
      ? `coder/${coderWorkspace}`
      : ctx.attachedServer.url.replace(/^https?:\/\//, "");
    ctx.statusBarItem.text = `$(server) Operator: ${label}`;
    ctx.statusBarItem.tooltip =
      `Attached to Operator ${ctx.attachedServer.version ? `v${ctx.attachedServer.version} ` : ""}` +
      `at ${ctx.attachedServer.url} · ${webhook}`;
    ctx.statusBarItem.backgroundColor = undefined;
  } else {
    const { selection } = currentConnection();
    ctx.statusBarItem.text =
      selection.kind === "hosted"
        ? `$(server) Operator: ${new URL(selection.origin).host}`
        : "$(server) Operator";
    ctx.statusBarItem.tooltip =
      selection.kind === "hosted"
        ? `Operator at ${selection.origin} is disconnected. Choose Connection to change servers.`
        : "Choose an Operator connection or open local setup.";

    ctx.statusBarItem.backgroundColor = new vscode.ThemeColor("statusBarItem.warningBackground");
  }
  ctx.statusBarItem.show();
}

// ---------------------------------------------------------------------------
// Queue commands
// ---------------------------------------------------------------------------

async function pauseQueueCommand(ctx: CommandContext): Promise<void> {
  const apiClient = new OperatorApiClient(currentConnection().origin);

  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  try {
    const result = await apiClient.pauseQueue();
    void vscode.window.showInformationMessage(result.message);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to pause queue: ${msg}`);
  }
}

async function resumeQueueCommand(ctx: CommandContext): Promise<void> {
  const apiClient = new OperatorApiClient(currentConnection().origin);

  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  try {
    const result = await apiClient.resumeQueue();
    void vscode.window.showInformationMessage(result.message);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to resume queue: ${msg}`);
  }
}

// ---------------------------------------------------------------------------
// Review commands
// ---------------------------------------------------------------------------

async function showAwaitingAgentPicker(apiClient: OperatorApiClient): Promise<string | undefined> {
  try {
    const data = await apiClient.listActiveAgents();

    const awaitingAgents = data.agents.filter((a) => a.status === "awaiting_input");

    if (awaitingAgents.length === 0) {
      void vscode.window.showInformationMessage("No agents awaiting review");
      return undefined;
    }

    const items = awaitingAgents.map((a) => ({
      label: a.ticket_id,
      description: a.project,
      detail: `Agent: ${a.id}`,
      agentId: a.id,
    }));

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: "Select agent to review",
    });

    return selected?.agentId;
  } catch {
    void vscode.window.showErrorMessage("Failed to fetch agents");
    return undefined;
  }
}

async function approveReviewCommand(ctx: CommandContext, agentId: string): Promise<void> {
  const apiClient = new OperatorApiClient(currentConnection().origin);
  let selectedAgentId: string | undefined = agentId;
  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  if (!agentId) {
    selectedAgentId = await showAwaitingAgentPicker(apiClient);
    if (!selectedAgentId) {
      return;
    }
  }

  try {
    const result = await apiClient.approveReview(selectedAgentId);
    void vscode.window.showInformationMessage(result.message);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to approve review: ${msg}`);
  }
}

async function rejectReviewCommand(ctx: CommandContext, agentId: string): Promise<void> {
  const apiClient = new OperatorApiClient(currentConnection().origin);
  let selectedAgentId: string | undefined = agentId;
  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  if (!agentId) {
    selectedAgentId = await showAwaitingAgentPicker(apiClient);
    if (!selectedAgentId) {
      return;
    }
  }

  const reason = await vscode.window.showInputBox({
    prompt: "Enter rejection reason",
    placeHolder: "Why is this being rejected?",
    validateInput: (value) => {
      if (!value || value.trim().length === 0) {
        return "Rejection reason is required";
      }
      return null;
    },
  });

  if (!reason) {
    return;
  }

  try {
    const result = await apiClient.rejectReview(selectedAgentId, reason);
    void vscode.window.showInformationMessage(result.message);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to reject review: ${msg}`);
  }
}

// ---------------------------------------------------------------------------
// Kanban commands
// ---------------------------------------------------------------------------

async function syncKanbanCommand(ctx: CommandContext): Promise<void> {
  const apiClient = new OperatorApiClient(currentConnection().origin);

  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  try {
    const result = await apiClient.syncKanban();
    const message = `Synced: ${result.created.length} created, ${result.skipped.length} skipped`;
    if (result.errors.length > 0) {
      void vscode.window.showWarningMessage(`${message}, ${result.errors.length} errors`);
    } else {
      void vscode.window.showInformationMessage(message);
    }
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to sync kanban: ${msg}`);
  }
}

async function syncKanbanCollectionCommand(ctx: CommandContext, item: StatusItem): Promise<void> {
  const provider = item.provider;
  const projectKey = item.projectKey;

  if (!provider || !projectKey) {
    void vscode.window.showWarningMessage("No collection selected for sync.");
    return;
  }

  const apiClient = new OperatorApiClient(currentConnection().origin);

  try {
    await apiClient.currentSession();
  } catch {
    void vscode.window.showErrorMessage("Operator API not running. Start operator first.");
    return;
  }

  try {
    const result = await apiClient.syncKanbanCollection(provider, projectKey);
    const createdList = result.created.length > 0 ? ` (${result.created.join(", ")})` : "";
    const message = `Synced ${projectKey}: ${result.created.length} created${createdList}, ${result.skipped.length} skipped`;
    if (result.errors.length > 0) {
      void vscode.window.showWarningMessage(`${message}, ${result.errors.length} errors`);
    } else {
      void vscode.window.showInformationMessage(message);
    }
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : "Unknown error";
    void vscode.window.showErrorMessage(`Failed to sync collection: ${msg}`);
  }
}

async function addJiraProjectCommand(ctx: CommandContext, workspaceKey: string): Promise<void> {
  await addJiraProject(ctx.extensionContext, workspaceKey);
  await ctx.refreshAllProviders();
}

async function addLinearTeamCommand(ctx: CommandContext, workspaceKey: string): Promise<void> {
  await addLinearTeam(ctx.extensionContext, workspaceKey);
  await ctx.refreshAllProviders();
}

// ---------------------------------------------------------------------------
// Setup commands
// ---------------------------------------------------------------------------

function showConfigMissingNotification(): void {
  void vscode.window
    .showInformationMessage(
      "Could not find Operator! configuration file for this repository workspace. Run the setup walkthrough to create it and get started.",
      "Open Setup",
    )
    .then((choice) => {
      if (choice === "Open Setup") {
        return vscode.commands.executeCommand(
          "workbench.action.openWalkthrough",
          "untra.operator-terminals#operator-setup",
          true,
        );
      }
      return undefined;
    });
}

async function updateOperatorContext(ctx: CommandContext): Promise<void> {
  const { selection, signal } = currentConnection();
  const operatorAvailable =
    selection.kind === "local" && (await isOperatorAvailable(ctx.extensionContext));
  if (signal.aborted) {
    return;
  }
  await vscode.commands.executeCommand(
    "setContext",
    "operator.operatorAvailable",
    operatorAvailable,
  );

  const ticketsParentFound = ctx.getCurrentTicketsDir() !== undefined;
  await vscode.commands.executeCommand(
    "setContext",
    "operator.ticketsParentFound",
    ticketsParentFound,
  );

  await updateWalkthroughContext(ctx.extensionContext);
}

async function runSetupCommand(ctx: CommandContext): Promise<void> {
  if (await guardRemoteCommand("operator.runSetup", ctx.getCurrentTicketsDir())) {
    return;
  }

  const workingDir = ctx.extensionContext.globalState.get<string>("operator.workingDirectory");
  if (!workingDir) {
    await vscode.commands.executeCommand("operator.selectWorkingDirectory");
    return;
  }

  const choice = await vscode.window.showInformationMessage(
    `Run operator setup in ${workingDir.replace(os.homedir(), "~")}?`,
    "Yes",
    "Cancel",
  );

  if (choice !== "Yes") {
    return;
  }

  const operatorPath = await getOperatorPath(ctx.extensionContext);
  const success = await initializeTicketsDirectory(workingDir, operatorPath ?? undefined);

  if (success) {
    const ticketsDir = path.join(workingDir, ".tickets");
    ctx.setCurrentTicketsDir(ticketsDir);
    await ctx.setTicketsDir(ticketsDir);

    const watcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(ticketsDir, "**/*.md"),
    );
    watcher.onDidChange(() => {
      void ctx.refreshAllProviders();
    });
    watcher.onDidCreate(() => {
      void ctx.refreshAllProviders();
    });
    watcher.onDidDelete(() => {
      void ctx.refreshAllProviders();
    });
    ctx.extensionContext.subscriptions.push(watcher);

    await updateOperatorContext(ctx);
    void vscode.window.showInformationMessage("Operator setup completed successfully.");
  } else {
    void vscode.window.showErrorMessage("Failed to run operator setup.");
  }
}

function readConnectionInputs(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("operator");
  const folder = vscode.workspace.workspaceFolders?.[0];
  const folderUrl =
    folder &&
    vscode.workspace.getConfiguration("operator", folder.uri).inspect<string>("apiUrl")
      ?.workspaceFolderValue;
  const url = config.inspect<string>("apiUrl");
  return {
    userMode: config.inspect<string>("connectionMode")?.globalValue,
    userUrl: url?.globalValue,
    suggestion: folderUrl ?? url?.workspaceValue,
    approvedOrigin: context.workspaceState.get<string>(APPROVED_WORKSPACE_ORIGIN),
  };
}

async function downloadOperatorCommand(ctx: CommandContext): Promise<boolean> {
  const { signal } = currentConnection();
  if (await guardRemoteCommand("operator.downloadOperator", ctx.getCurrentTicketsDir())) {
    return false;
  }

  const existingPath = await getOperatorPath(ctx.extensionContext);
  if (existingPath) {
    const version = await getOperatorVersion(existingPath);
    const choice = await vscode.window.showInformationMessage(
      `Operator ${version ?? "unknown version"} is already installed at ${existingPath}`,
      "Reinstall/Update",
      "Open Downloads Page",
      "Cancel",
    );

    if (choice === "Open Downloads Page") {
      void vscode.env.openExternal(vscode.Uri.parse(DOWNLOADS_PAGE));
      return false;
    }
    if (choice !== "Reinstall/Update") {
      return false;
    }
  }

  if (signal.aborted) {
    return false;
  }
  try {
    const downloadedPath = await downloadOperator(ctx.extensionContext);
    const version = await getOperatorVersion(downloadedPath);

    void vscode.window.showInformationMessage(
      `Operator ${version ?? getExtensionVersion()} downloaded successfully to ${downloadedPath}`,
    );

    await updateOperatorContext(ctx);
    return !signal.aborted;
  } catch (error) {
    const msg = error instanceof Error ? error.message : "Unknown error";

    const choice = await vscode.window.showErrorMessage(
      `Failed to download Operator: ${msg}`,
      "Open Downloads Page",
      "Cancel",
    );

    if (choice === "Open Downloads Page") {
      void vscode.env.openExternal(vscode.Uri.parse(DOWNLOADS_PAGE));
    }
  }
  return false;
}

async function startOperatorServerCommand(ctx: CommandContext, automatic = false): Promise<void> {
  const { selection, signal, origin } = currentConnection();
  if (selection.kind !== "local") {
    await guardRemoteCommand("operator.startOperatorServer", ctx.getCurrentTicketsDir());
    return;
  }

  const hasConfig = await configFileExists(ctx.getCurrentTicketsDir());
  if (!hasConfig) {
    showConfigMissingNotification();
    return;
  }

  const operatorPath = await getOperatorPath(ctx.extensionContext);

  if (signal.aborted) {
    return;
  }
  if (!operatorPath) {
    void vscode.window
      .showErrorMessage("Operator binary not found", "Download Operator", "Cancel")
      .then(async (choice) => {
        if (
          choice === "Download Operator" &&
          !signal.aborted &&
          (await downloadOperatorCommand(ctx))
        ) {
          return startOperatorServerCommand(ctx);
        }
        return undefined;
      });
    return;
  }

  const serverDir = await findOperatorServerDir();
  if (!serverDir) {
    void vscode.window.showErrorMessage("No workspace folder found.");
    return;
  }

  const apiClient = new OperatorApiClient(origin);
  try {
    const health = await apiClient.health();
    const localVersion = await getOperatorVersion(operatorPath);
    if (localVersion && health.version && health.version !== localVersion) {
      // The port is held by a different operator version - adopting it could
      // mix incompatible client/server APIs. Warn instead of silently starting.
      void vscode.window.showWarningMessage(
        `A different Operator version (v${health.version}) is already running on this port; ` +
          `this binary is v${localVersion}. Stop the other instance or set a different ` +
          `port (operator.apiUrl) before starting your own server.`,
      );
      return;
    }
    void vscode.window.showInformationMessage(
      health.version
        ? `Operator is already running (v${health.version})`
        : "Operator is already running",
    );
    return;
  } catch {
    // Not running, proceed to start
  }

  if (signal.aborted) {
    return;
  }
  const terminalName = "Operator API";

  if (ctx.terminalManager.exists(terminalName)) {
    if (!automatic) {
      ctx.terminalManager.focus(terminalName);
    }
    return;
  }

  ctx.terminalManager.create({
    name: terminalName,
    workingDir: serverDir,
  });

  ctx.terminalManager.send(terminalName, `"${operatorPath}" api`);
  if (!automatic) {
    ctx.terminalManager.focus(terminalName);
  }

  void vscode.window.showInformationMessage(`Starting Operator API server in ${serverDir}...`);

  setTimeout(() => {
    if (!signal.aborted) {
      void ctx.reconnect();
    }
  }, 2000);
}

async function revealTicketsDirCommand(ctx: CommandContext): Promise<void> {
  if (await guardRemoteCommand("operator.revealTicketsDir", ctx.getCurrentTicketsDir())) {
    return;
  }

  const dir = ctx.getCurrentTicketsDir();
  if (!dir) {
    void vscode.window.showWarningMessage("No .tickets directory found.");
    return;
  }

  const uri = vscode.Uri.file(dir);
  await vscode.commands.executeCommand("revealFileInOS", uri);
}

// ---------------------------------------------------------------------------
// Create commands
// ---------------------------------------------------------------------------

async function showCreateMenu(ctx: CommandContext): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      {
        label: "$(rocket) New Delegator",
        detail: "delegator",
        description: "Create a tool+model pairing for autonomous launches",
      },
      {
        label: "$(list-tree) New Issue Type",
        detail: "issuetype",
        description: "Define a custom issue type with steps",
      },
      {
        label: "$(project) New Managed Project",
        detail: "project",
        description: "Assess and register a project",
      },
    ],
    {
      title: "Create New",
      placeHolder: "What would you like to create?",
    },
  );

  if (!choice) {
    return;
  }

  switch (choice.detail) {
    case "delegator":
      if (!(await guardRemoteCommand("operator.openCreateDelegator", ctx.getCurrentTicketsDir()))) {
        void openCreateDelegator(ctx);
      }
      break;
    case "issuetype":
      // Issue types are managed in the hosted Operator UI.
      await openOperatorUi(ctx.getCurrentTicketsDir(), "issuetypes");
      break;
    case "project":
      // Projects are browsed/assessed in the hosted Operator UI.
      await openOperatorUi(ctx.getCurrentTicketsDir(), "projects");
      break;
  }
}

async function openCreateDelegator(
  ctx: CommandContext,
  tool?: string,
  model?: string,
): Promise<void> {
  if (await guardRemoteCommand("operator.openCreateDelegator", ctx.getCurrentTicketsDir())) {
    return;
  }
  // Delegators bind a model provider to an llm tool, so they live in the Model
  // Providers section (distinct from the Coding Agents / llm-tools section).
  ConfigPanel.createOrShow(ctx.extensionContext.extensionUri);
  ConfigPanel.navigateTo("section-model-providers", {
    action: "createDelegator",
    tool,
    model,
  });
}

// ---------------------------------------------------------------------------
// Extension activation
// ---------------------------------------------------------------------------

function warn(message: string): void {
  void vscode.window.showWarningMessage(message);
}

let stopTerminalListener: (() => Promise<void>) | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  // Create output channel for logging
  const outputChannel = vscode.window.createOutputChannel("Operator");
  context.subscriptions.push(outputChannel);
  outputChannel.appendLine("[Operator] Activation started");

  // Every daemon request resolves its credential through this one provider.
  const tokenStore = new TokenStore(context.secrets);
  const credentials = new OperatorCredentials(tokenStore);
  setCredentialProvider(credentials);
  context.subscriptions.push({ dispose: clearCredentialProvider });

  // Initialize issue type service (constructor is safe - no network calls)
  const issueTypeService = new IssueTypeService(outputChannel);
  const statusProvider = new StatusTreeProvider(context);
  const inProgressProvider = new TicketTreeProvider("in-progress", issueTypeService);
  const queueProvider = new TicketTreeProvider("queue", issueTypeService);
  const completedProvider = new TicketTreeProvider("completed", issueTypeService);
  const campaignProvider = new CampaignTreeProvider();

  const statusTreeView = vscode.window.createTreeView("operator-status", {
    treeDataProvider: statusProvider,
  });
  context.subscriptions.push(
    statusTreeView,
    vscode.window.registerTreeDataProvider("operator-in-progress", inProgressProvider),
    vscode.window.registerTreeDataProvider("operator-queue", queueProvider),
    vscode.window.registerTreeDataProvider("operator-completed", completedProvider),
    vscode.window.registerTreeDataProvider("operator-campaigns", campaignProvider),
  );

  // Synchronous object construction - these constructors do no I/O
  const terminalManager = new TerminalManager();
  terminalManager.setIssueTypeService(issueTypeService);
  inProgressProvider.setTerminalManager(terminalManager);

  const connectDeps: ConnectDeps = {
    state: context.globalState,
    connection: currentConnection,
    issue: (agentId) => new OperatorApiClient(currentConnection().origin).issueAttach(agentId),
    redeem: (agentId, attach) =>
      new OperatorApiClient(currentConnection().origin).redeemAttach(agentId, attach),
    openTerminal: (terminal) => openAgentTerminal(terminalManager, terminal, warn),
  };

  context.subscriptions.push(
    vscode.window.registerUriHandler({
      handleUri(uri: vscode.Uri) {
        routeOperatorUri(uri, {
          focusSession: (name) => {
            if (currentConnection().selection.kind !== "local") {
              return;
            }
            if (terminalManager.exists(name)) {
              terminalManager.focus(name);
            } else {
              void vscode.window.showWarningMessage(
                `Operator: no terminal named '${name}' to focus`,
              );
            }
          },
          openTicket: (id) => {
            void openTicketById(id);
          },
          connect: (request) => {
            handleConnect(request, connectDeps).catch((err: unknown) => {
              warn(`Operator: could not connect to agent: ${String(err)}`);
            });
          },
          warn,
        });
      },
    }),
  );

  const webhookServer = new WebhookServer(terminalManager);
  const launchManager = new LaunchManager(terminalManager);

  statusProvider.setWebhookServer(webhookServer);

  // Create status bar items
  const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBarItem.command = "operator.showStatus";
  context.subscriptions.push(statusBarItem);

  const createBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 99);
  createBarItem.text = "$(add) New";
  createBarItem.tooltip = "Create new delegator, issue type, or project";
  createBarItem.command = "operator.showCreateMenu";
  createBarItem.show();
  context.subscriptions.push(createBarItem);

  // Build shared command context
  const ctx: CommandContext = {
    extensionContext: context,
    terminalManager,
    webhookServer,
    launchManager,
    issueTypeService,
    statusProvider,
    statusTreeView,
    queueProvider,
    inProgressProvider,
    completedProvider,
    statusBarItem,
    reconnect: () => reconcileConnection(false),
    listenerQueue: new SerialQueue(),
    createBarItem,
    outputChannel,
    getCurrentTicketsDir: () => currentTicketsDir,
    setCurrentTicketsDir: (dir) => {
      currentTicketsDir = dir;
    },
    refreshAllProviders: async () => {
      await statusProvider.refresh();
      await inProgressProvider.refresh();
      await queueProvider.refresh();
      await completedProvider.refresh();
      await campaignProvider.refresh();
    },
    setTicketsDir: async (dir) => {
      credentials.setTicketsDir(dir);
      await statusProvider.setTicketsDir(dir);
      await inProgressProvider.setTicketsDir(dir);
      await queueProvider.setTicketsDir(dir);
      await completedProvider.setTicketsDir(dir);
      launchManager.setTicketsDir(dir);
    },
  };

  const transitions = new SerialQueue();
  let initialized = false;
  let savingChoice = false;
  let watcher: vscode.FileSystemWatcher | undefined;
  let sessionWatcher: vscode.FileSystemWatcher | undefined;

  function clearAttachedData(): void {
    ctx.attachedServer = undefined;
    currentTicketsDir = undefined;
    credentials.setTicketsDir(undefined);
    launchManager.setTicketsDir(undefined);
    issueTypeService.setBaseUrl("");
    statusProvider.clearConnection();
    queueProvider.clearConnection();
    inProgressProvider.clearConnection();
    completedProvider.clearConnection();
    campaignProvider.setApiClient(undefined);
    ConfigPanel.close();
    watcher?.dispose();
    watcher = undefined;
    sessionWatcher?.dispose();
    sessionWatcher = undefined;
    updateStatusBar(ctx);
  }

  async function signInToCurrent(): Promise<void> {
    const { origin, signal } = currentConnection();
    const outcome = await signIn(origin, credentials, tokenStore, signal);
    if (!signal.aborted && outcome?.status === "approved") {
      await ctx.reconnect();
    }
  }

  async function reconcileConnection(allowStartup: boolean): Promise<void> {
    const inputs = readConnectionInputs(context);
    const next = resolveConnection(inputs);
    const previous = currentConnection().selection;
    const changed = !initialized || JSON.stringify(previous) !== JSON.stringify(next);
    initialized = true;
    if (changed) {
      resetConnection(next);
      clearAttachedData();
    }
    const { signal } = currentConnection();
    await transitions.run(async () => {
      if (changed) {
        await ctx.listenerQueue.run(() => ctx.webhookServer.stop());
      }
      if (signal.aborted) {
        return;
      }
      const suggestion = normalizeRemoteOrigin(inputs.suggestion ?? "");
      if (
        inputs.approvedOrigin &&
        (!suggestion.ok || suggestion.origin !== inputs.approvedOrigin)
      ) {
        await context.workspaceState.update(APPROVED_WORKSPACE_ORIGIN, undefined);
      }
      if (signal.aborted) {
        return;
      }
      if (next.kind === "undecided" || next.kind === "invalid") {
        await updateOperatorContext(ctx);
        if (next.kind === "invalid") {
          void vscode.window
            .showErrorMessage(normalizeFailureMessage(next.reason), "Open Operator Settings")
            .then((choice) => {
              if (choice && !signal.aborted) {
                return vscode.commands.executeCommand(
                  "workbench.action.openSettings",
                  "operator.apiUrl",
                );
              }
              return undefined;
            });
        } else if (changed) {
          void openWalkthrough();
        }
        return;
      }

      if (next.kind === "local") {
        const dir = await findParentTicketsDir();
        if (signal.aborted) {
          return;
        }
        if (changed || dir !== currentTicketsDir) {
          watcher?.dispose();
          watcher = undefined;
          sessionWatcher?.dispose();
          sessionWatcher = undefined;
          currentTicketsDir = dir;
          await ctx.setTicketsDir(dir);
          if (signal.aborted) {
            return;
          }
          if (dir) {
            watcher = vscode.workspace.createFileSystemWatcher(
              new vscode.RelativePattern(dir, "**/*.md"),
            );
            const refresh = () => {
              if (currentTicketsDir === dir) {
                void ctx.refreshAllProviders();
              }
            };
            watcher.onDidChange(refresh);
            watcher.onDidCreate(refresh);
            watcher.onDidDelete(refresh);
            sessionWatcher = vscode.workspace.createFileSystemWatcher(
              new vscode.RelativePattern(dir, "operator/api-session.json"),
            );
            const reconnect = () => {
              if (currentTicketsDir === dir) {
                void ctx.reconnect();
              }
            };
            sessionWatcher.onDidCreate(reconnect);
            sessionWatcher.onDidChange(reconnect);
            sessionWatcher.onDidDelete(reconnect);
          }
        }
      }

      const url = next.origin ?? (await discoverLocalApiUrl(currentTicketsDir));
      if (signal.aborted) {
        return;
      }
      const localOriginChanged = !!currentConnection().origin && currentConnection().origin !== url;
      if (localOriginChanged) {
        resetConnection(next);
        ctx.attachedServer = undefined;
        ticketApiClients(undefined);
        issueTypeService.setBaseUrl("");
        statusProvider.clearConnection();
        queueProvider.clearConnection();
        inProgressProvider.clearConnection();
        completedProvider.clearConnection();
        ConfigPanel.close();
        const replacement = currentConnection().signal;
        await ctx.listenerQueue.run(() => ctx.webhookServer.stop());
        if (replacement.aborted) {
          return;
        }
        await ctx.setTicketsDir(currentTicketsDir);
        if (replacement.aborted) {
          return;
        }
      }
      setConnectionOrigin(url);
      const active = currentConnection().signal;
      const client = new OperatorApiClient(url);
      let authRequired = false;
      let reachable = await client.isReachable();
      if (active.aborted) {
        return;
      }
      ctx.attachedServer = undefined;
      if (reachable) {
        try {
          let version: string | undefined;
          if (next.kind === "hosted") {
            await client.currentSession();
          } else {
            version = (await client.health()).version;
          }
          if (active.aborted) {
            return;
          }
          ctx.attachedServer = { url, version };
        } catch (error) {
          authRequired = error instanceof AuthRequiredError;
          reachable = authRequired;
        }
      }
      if (active.aborted) {
        return;
      }

      ticketApiClients(ctx.attachedServer ? client : undefined);
      issueTypeService.setBaseUrl(ctx.attachedServer ? url : "");
      if (next.kind === "local" && (allowStartup || localOriginChanged)) {
        const hasConfig = await configFileExists(ctx.getCurrentTicketsDir());
        if (hasConfig && !active.aborted) {
          await ctx.listenerQueue.run(async () => {
            if (!active.aborted) {
              await startLocalWebhookServer(ctx, active);
            }
          });
        }
        if (
          allowStartup &&
          !reachable &&
          !active.aborted &&
          vscode.workspace.getConfiguration("operator").get("autoStart", true)
        ) {
          await startOperatorServerCommand(ctx, true);
        }
      }
      if (active.aborted) {
        return;
      }
      updateStatusBar(ctx);
      await issueTypeService.refresh();
      if (active.aborted) {
        return;
      }
      await Promise.all([ctx.refreshAllProviders(), updateOperatorContext(ctx)]);
      if (active.aborted) {
        return;
      }
      if (ctx.attachedServer) {
        await resumePendingConnect(connectDeps);
      }
      if (authRequired) {
        void vscode.window
          .showInformationMessage(`Operator at ${url} requires sign-in.`, "Sign In")
          .then((choice) => (choice && !active.aborted ? signInToCurrent() : undefined));
      } else if (!reachable && next.kind === "hosted") {
        void vscode.window
          .showWarningMessage(`Can't reach Operator at ${url}.`, "Retry", "Choose Connection")
          .then((choice) => {
            if (active.aborted) {
              return undefined;
            }
            if (choice === "Retry") {
              return ctx.reconnect();
            }
            if (choice === "Choose Connection") {
              return vscode.commands.executeCommand(CHOOSE_CONNECTION_COMMAND);
            }
            return undefined;
          });
      }
    });
  }

  function ticketApiClients(client: OperatorApiClient | undefined): void {
    queueProvider.setApiClient(client);
    inProgressProvider.setApiClient(client);
    completedProvider.setApiClient(client);
    campaignProvider.setApiClient(client);
  }

  async function chooseConnection(
    preferred?: typeof PICK_LOCAL | typeof PICK_REMOTE,
  ): Promise<void> {
    await runConnectionPicker(
      {
        ...readConnectionInputs(context),
        saveLocal: async () => {
          savingChoice = true;
          const config = vscode.workspace.getConfiguration("operator");
          try {
            await context.workspaceState.update(APPROVED_WORKSPACE_ORIGIN, undefined);
            const userUrl = readConnectionInputs(context).userUrl;
            if (userUrl && resolveConnection({ userUrl }).kind !== "local") {
              await config.update("apiUrl", undefined, vscode.ConfigurationTarget.Global);
            }
            await config.update(
              "connectionMode",
              CONNECTION_MODE_LOCAL,
              vscode.ConfigurationTarget.Global,
            );
          } finally {
            savingChoice = false;
          }
          await reconcileConnection(true);
        },
        saveRemote: async (origin, workspace) => {
          savingChoice = true;
          try {
            if (workspace) {
              await context.workspaceState.update(APPROVED_WORKSPACE_ORIGIN, origin);
            } else {
              const config = vscode.workspace.getConfiguration("operator");
              await context.workspaceState.update(APPROVED_WORKSPACE_ORIGIN, undefined);
              await config.update("apiUrl", origin, vscode.ConfigurationTarget.Global);
              await config.update(
                "connectionMode",
                CONNECTION_MODE_REMOTE,
                vscode.ConfigurationTarget.Global,
              );
            }
          } finally {
            savingChoice = false;
          }
          await reconcileConnection(false);
        },
      },
      preferred,
    );
  }

  const campaignCtx: CampaignCommandContext = {
    getCurrentTicketsDir: () => currentTicketsDir,
    refreshCampaigns: () => campaignProvider.refresh(),
  };

  // Register all commands BEFORE any async work - ensures commands are
  // always available even if network/API initialization fails.
  context.subscriptions.push(
    vscode.commands.registerCommand("operator.refreshCampaigns", () => campaignProvider.refresh()),
    vscode.commands.registerCommand("operator.launchCampaign", (node?: CampaignNode) =>
      launchCampaignCommand(campaignCtx, node),
    ),
    vscode.commands.registerCommand("operator.pauseCampaign", (node?: CampaignNode) =>
      pauseCampaignCommand(campaignCtx, node),
    ),
    vscode.commands.registerCommand("operator.openCampaign", (node?: CampaignNode) =>
      openCampaignCommand(campaignCtx, node),
    ),
    vscode.commands.registerCommand("operator.showStatus", () => showStatus(ctx)),
    vscode.commands.registerCommand("operator.refreshTickets", () => ctx.reconnect()),
    vscode.commands.registerCommand("operator.focusTicket", (name: string, ticket?: TicketInfo) =>
      focusTicketTerminal(ctx, name, ticket),
    ),
    vscode.commands.registerCommand("operator.openTicket", openTicketFile),
    vscode.commands.registerCommand("operator.launchTicket", (treeItem?: TicketItem) =>
      launchTicketCommand(ctx, treeItem),
    ),
    vscode.commands.registerCommand("operator.launchTicketWithOptions", (treeItem?: TicketItem) =>
      launchTicketWithOptionsCommand(ctx, treeItem),
    ),
    vscode.commands.registerCommand("operator.relaunchTicket", (ticket: TicketInfo) =>
      relaunchTicketCommand(ctx, ticket),
    ),
    vscode.commands.registerCommand("operator.launchTicketFromEditor", () =>
      launchTicketFromEditorCommand(ctx),
    ),
    vscode.commands.registerCommand("operator.launchTicketFromEditorWithOptions", () =>
      launchTicketFromEditorWithOptionsCommand(ctx),
    ),
    vscode.commands.registerCommand("operator.downloadOperator", () =>
      downloadOperatorCommand(ctx),
    ),
    vscode.commands.registerCommand("operator.attachAgent", async (agentId?: string) => {
      let agent = agentId;
      if (!agent) {
        const { origin, signal } = currentConnection();
        if (!origin) {
          throw new Error("Choose an Operator connection first.");
        }
        const list = await new OperatorApiClient(origin).listActiveAgents();
        signal.throwIfAborted();
        const picked = await vscode.window.showQuickPick(
          list.agents.map((a) => ({ label: a.ticket_id, description: a.project, id: a.id })),
          { title: "Attach to agent terminal" },
        );
        signal.throwIfAborted();
        agent = picked?.id;
      }
      if (agent) {
        await handleConnect({ agent }, connectDeps);
      }
    }),
    vscode.commands.registerCommand(CHOOSE_CONNECTION_COMMAND, () => chooseConnection()),
    vscode.commands.registerCommand("operator.chooseLocalConnection", () =>
      chooseConnection(PICK_LOCAL),
    ),
    vscode.commands.registerCommand("operator.chooseHostedConnection", () =>
      chooseConnection(PICK_REMOTE),
    ),
    vscode.commands.registerCommand("operator.pauseQueue", () => pauseQueueCommand(ctx)),
    vscode.commands.registerCommand("operator.resumeQueue", () => resumeQueueCommand(ctx)),
    vscode.commands.registerCommand("operator.syncKanban", () => syncKanbanCommand(ctx)),
    vscode.commands.registerCommand("operator.approveReview", (agentId: string) =>
      approveReviewCommand(ctx, agentId),
    ),
    vscode.commands.registerCommand("operator.rejectReview", (agentId: string) =>
      rejectReviewCommand(ctx, agentId),
    ),
    vscode.commands.registerCommand("operator.startOperatorServer", () =>
      startOperatorServerCommand(ctx),
    ),
    vscode.commands.registerCommand("operator.selectWorkingDirectory", async () => {
      if (await guardRemoteCommand("operator.selectWorkingDirectory", currentTicketsDir)) {
        return;
      }
      const operatorPath = await getOperatorPath(ctx.extensionContext);
      await selectWorkingDirectory(ctx.extensionContext, operatorPath ?? undefined);
      await reconcileConnection(true);
    }),
    vscode.commands.registerCommand("operator.runSetup", () => runSetupCommand(ctx)),
    vscode.commands.registerCommand("operator.checkKanbanConnection", () =>
      checkKanbanConnection(ctx.extensionContext),
    ),
    vscode.commands.registerCommand("operator.configureJira", () =>
      configureJira(ctx.extensionContext),
    ),
    vscode.commands.registerCommand("operator.configureLinear", () =>
      configureLinear(ctx.extensionContext),
    ),
    vscode.commands.registerCommand("operator.startKanbanOnboarding", async () => {
      if (!(await guardRemoteCommand("operator.startKanbanOnboarding", currentTicketsDir))) {
        await startKanbanOnboarding(ctx.extensionContext);
      }
    }),
    vscode.commands.registerCommand("operator.startGitOnboarding", () =>
      startGitOnboarding().then(() => ctx.refreshAllProviders()),
    ),
    vscode.commands.registerCommand("operator.configureGitHub", () =>
      onboardGitHub().then(() => ctx.refreshAllProviders()),
    ),
    vscode.commands.registerCommand("operator.configureGitLab", () =>
      onboardGitLab().then(() => ctx.refreshAllProviders()),
    ),
    vscode.commands.registerCommand("operator.showCreateMenu", () => showCreateMenu(ctx)),
    vscode.commands.registerCommand(
      "operator.openCreateDelegator",
      (tool?: string, model?: string) => {
        void openCreateDelegator(ctx, tool, model);
      },
    ),
    vscode.commands.registerCommand("operator.detectLlmTools", async () => {
      if (!(await guardRemoteCommand("operator.detectLlmTools", currentTicketsDir))) {
        await detectLlmTools(ctx.extensionContext, getOperatorPath);
      }
    }),
    vscode.commands.registerCommand(
      "operator.setDefaultLlm",
      async (tool?: string, model?: string) => {
        if (!tool || !model) {
          return;
        }
        try {
          const apiUrl = await discoverApiUrl(ctx.getCurrentTicketsDir());
          await new OperatorApiClient(apiUrl).setDefaultLlm({ tool, model });
          void vscode.window.showInformationMessage(`Default LLM set to ${tool}:${model}`);
          void ctx.refreshAllProviders();
        } catch (err) {
          const msg = err instanceof Error ? err.message : "Operator API not available";
          void vscode.window.showErrorMessage(`Failed to set default LLM: ${msg}`);
        }
      },
    ),
    vscode.commands.registerCommand("operator.signIn", signInToCurrent),
    vscode.commands.registerCommand("operator.signOut", async () => {
      const { origin, signal } = currentConnection();
      if (origin) {
        await signOut(origin, tokenStore);
        if (!signal.aborted) {
          await ctx.reconnect();
        }
      }
    }),
    vscode.commands.registerCommand("operator.openWalkthrough", openWalkthrough),
    vscode.commands.registerCommand("operator.openSettings", async () => {
      if (await guardRemoteCommand("operator.openSettings", ctx.getCurrentTicketsDir())) {
        return;
      }
      ConfigPanel.createOrShow(ctx.extensionContext.extensionUri);
    }),
    // Link out to the daemon-hosted Operator UI (Simple Browser) for the
    // operational surfaces the extension webview no longer reimplements.
    vscode.commands.registerCommand("operator.openUi", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "dashboard"),
    ),
    vscode.commands.registerCommand("operator.openIssueTypes", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "issuetypes"),
    ),
    vscode.commands.registerCommand("operator.openProjects", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "projects"),
    ),
    vscode.commands.registerCommand("operator.openKanban", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "kanban"),
    ),
    vscode.commands.registerCommand("operator.openQueue", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "queue"),
    ),
    vscode.commands.registerCommand("operator.openWorkflows", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "workflows"),
    ),
    vscode.commands.registerCommand("operator.openRemoteTargets", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "remote-targets"),
    ),
    vscode.commands.registerCommand("operator.openLicense", () =>
      openOperatorUi(ctx.getCurrentTicketsDir(), "license"),
    ),
    vscode.commands.registerCommand("operator.syncKanbanCollection", (item: StatusItem) =>
      syncKanbanCollectionCommand(ctx, item),
    ),
    vscode.commands.registerCommand("operator.addJiraProject", (workspaceKey: string) =>
      addJiraProjectCommand(ctx, workspaceKey),
    ),
    vscode.commands.registerCommand("operator.addLinearTeam", (workspaceKey: string) =>
      addLinearTeamCommand(ctx, workspaceKey),
    ),
    vscode.commands.registerCommand("operator.revealTicketsDir", () =>
      revealTicketsDirCommand(ctx),
    ),
    vscode.commands.registerCommand("operator.startWebhookServer", () => startWebhookServer(ctx)),
    vscode.commands.registerCommand("operator.connectMcpServer", () =>
      connectMcpServer(ctx.getCurrentTicketsDir()),
    ),
    // ABXY navigation commands for status panel - registered last but still before async init
    vscode.commands.registerCommand("operator.statusSpecialAction", () => {
      const selected = ctx.statusTreeView?.selection?.[0];
      if (selected instanceof StatusItem && selected.specialCommand) {
        const args = (selected.specialCommand.arguments ?? []) as unknown[];
        void vscode.commands.executeCommand(selected.specialCommand.command, ...args);
      }
    }),
    vscode.commands.registerCommand("operator.statusRefreshAction", () => {
      const selected = ctx.statusTreeView?.selection?.[0];
      if (selected instanceof StatusItem && selected.refreshCommand) {
        const args = (selected.refreshCommand.arguments ?? []) as unknown[];
        void vscode.commands.executeCommand(selected.refreshCommand.command, ...args);
      }
    }),
    vscode.commands.registerCommand("operator.statusBackAction", () => {
      void vscode.commands.executeCommand("list.collapse");
    }),
  );

  outputChannel.appendLine("[Operator] Command registration complete");
  stopTerminalListener = () => ctx.listenerQueue.run(() => webhookServer.stop());

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        !savingChoice &&
        (event.affectsConfiguration("operator.apiUrl") ||
          event.affectsConfiguration("operator.connectionMode"))
      ) {
        void reconcileConnection(true).catch(warn);
      }
    }),
    {
      dispose: () => {
        resetConnection({ kind: "undecided" });
        watcher?.dispose();
        sessionWatcher?.dispose();
      },
    },
  );
  try {
    const configWorkingDir = vscode.workspace
      .getConfiguration("operator")
      .get<string>("workingDirectory");
    if (configWorkingDir && !context.globalState.get("operator.workingDirectory")) {
      await context.globalState.update("operator.workingDirectory", configWorkingDir);
    }
    await reconcileConnection(true);
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    outputChannel.appendLine(`[Operator] Activation error: ${msg}`);
    if (err instanceof Error && err.stack) {
      outputChannel.appendLine(err.stack);
    }
    void vscode.window.showErrorMessage(`Operator extension failed to fully activate: ${msg}`);
  }
}

export async function deactivate(): Promise<void> {
  await stopTerminalListener?.();
  stopTerminalListener = undefined;
}
