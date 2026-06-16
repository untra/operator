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

import * as vscode from 'vscode';
import * as path from 'path';
import * as os from 'os';
import { TerminalManager } from './terminal-manager';
import { WebhookServer } from './webhook-server';
import { TicketTreeProvider, TicketItem } from './ticket-provider';
import { StatusTreeProvider, StatusItem } from './status-provider';
import { LaunchManager } from './launch-manager';
import { IssueTypeService } from './issuetype-service';
import { TicketInfo } from './types';
import { OperatorApiClient, discoverApiUrl } from './api-client';
import { showLaunchOptionsDialog, showTicketPicker } from './launch-dialog';
import { parseTicketMetadata, getCurrentSessionId } from './ticket-parser';
import {
  getOperatorPath,
  getOperatorVersion,
  getExtensionVersion,
  isOperatorAvailable,
  downloadOperator,
} from './operator-binary';
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
} from './walkthrough';
import { startGitOnboarding, onboardGitHub, onboardGitLab } from './git-onboarding';
import { ConfigPanel } from './config-panel';
import { openOperatorUi } from './open-operator-ui';
import { connectMcpServer } from './mcp-connect';
import { configFileExists } from './config-paths';
import { findParentTicketsDir, findTicketsDir, findOperatorServerDir } from './tickets-dir';
import { addJiraProject, addLinearTeam } from './kanban-onboarding';

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
  const normalized = filePath.replace(/\\/g, '/');
  return (
    (normalized.includes('.tickets/queue/') ||
      normalized.includes('.tickets/in-progress/')) &&
    normalized.endsWith('.md')
  );
}

async function launchTicketCommand(
  ctx: CommandContext,
  treeItem?: TicketItem
): Promise<void> {
  let ticket: TicketInfo | undefined;

  if (treeItem?.ticket) {
    ticket = treeItem.ticket;
  } else {
    const tickets = ctx.queueProvider.getTickets();
    if (tickets.length === 0) {
      void vscode.window.showInformationMessage('No tickets in queue');
      return;
    }
    ticket = await showTicketPicker(tickets);
  }

  if (!ticket) {
    return;
  }

  await ctx.launchManager.launchTicket(ticket, {
    delegator: null,
    model: 'sonnet',
    yoloMode: false,
    resumeSession: false,
  });
}

async function launchTicketWithOptionsCommand(
  ctx: CommandContext,
  treeItem?: TicketItem
): Promise<void> {
  let ticket: TicketInfo | undefined;

  if (treeItem?.ticket) {
    ticket = treeItem.ticket;
  } else {
    const tickets = ctx.queueProvider.getTickets();
    if (tickets.length === 0) {
      void vscode.window.showInformationMessage('No tickets in queue');
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

async function relaunchTicketCommand(
  ctx: CommandContext,
  ticket: TicketInfo
): Promise<void> {
  await ctx.launchManager.offerRelaunch(ticket);
}

async function launchTicketFromEditorCommand(
  ctx: CommandContext
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    void vscode.window.showWarningMessage('No active editor');
    return;
  }

  const filePath = editor.document.uri.fsPath;
  if (!isTicketFile(filePath)) {
    void vscode.window.showWarningMessage(
      'Current file is not a ticket in .tickets/ directory'
    );
    return;
  }

  const metadata = await parseTicketMetadata(filePath);
  if (!metadata?.id) {
    void vscode.window.showErrorMessage('Could not parse ticket ID from file');
    return;
  }

  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const response = await apiClient.launchTicket(metadata.id, {
      delegator: null,
      provider: null,
      wrapper: 'vscode',
      model: 'sonnet',
      model_server: null,
      yolo_mode: false,
      retry_reason: null,
      resume_session_id: null,
    });

    ctx.terminalManager.create({
      name: response.terminal_name,
      workingDir: response.working_directory,
    });
    ctx.terminalManager.send(response.terminal_name, response.command);
    ctx.terminalManager.focus(response.terminal_name);

    const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
    void vscode.window.showInformationMessage(
      `Launched agent for ${response.ticket_id}${worktreeMsg}`
    );

    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
  }
}

async function launchTicketFromEditorWithOptionsCommand(
  ctx: CommandContext
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    void vscode.window.showWarningMessage('No active editor');
    return;
  }

  const filePath = editor.document.uri.fsPath;
  if (!isTicketFile(filePath)) {
    void vscode.window.showWarningMessage(
      'Current file is not a ticket in .tickets/ directory'
    );
    return;
  }

  const metadata = await parseTicketMetadata(filePath);
  if (!metadata?.id) {
    void vscode.window.showErrorMessage('Could not parse ticket ID from file');
    return;
  }

  const ticketType = ctx.issueTypeService.extractTypeFromId(metadata.id);
  const ticketStatus = (metadata.status === 'in-progress' || metadata.status === 'completed')
    ? metadata.status
    : 'queue' as const;
  const ticketInfo: TicketInfo = {
    id: metadata.id,
    type: ticketType,
    title: 'Ticket from editor',
    status: ticketStatus,
    filePath: filePath,
  };

  const hasSession = !!getCurrentSessionId(metadata);
  const options = await showLaunchOptionsDialog(ticketInfo, hasSession);
  if (!options) {
    return;
  }

  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const response = await apiClient.launchTicket(metadata.id, {
      delegator: options.delegator ?? null,
      provider: null,
      wrapper: 'vscode',
      model: options.model,
      model_server: null,
      yolo_mode: options.yoloMode,
      retry_reason: null,
      resume_session_id: null,
    });

    ctx.terminalManager.create({
      name: response.terminal_name,
      workingDir: response.working_directory,
    });
    ctx.terminalManager.send(response.terminal_name, response.command);
    ctx.terminalManager.focus(response.terminal_name);

    const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
    void vscode.window.showInformationMessage(
      `Launched agent for ${response.ticket_id}${worktreeMsg}`
    );

    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
  }
}

async function focusTicketTerminal(
  ctx: CommandContext,
  terminalName: string,
  ticket?: TicketInfo
): Promise<void> {
  if (ctx.terminalManager.exists(terminalName)) {
    ctx.terminalManager.focus(terminalName);
  } else if (ticket) {
    await ctx.launchManager.offerRelaunch(ticket);
  } else {
    void vscode.window.showWarningMessage(`Terminal '${terminalName}' not found`);
  }
}

function openTicketFile(filePath: string): void {
  void vscode.workspace.openTextDocument(filePath).then((doc) => {
    void vscode.window.showTextDocument(doc);
  });
}

// ---------------------------------------------------------------------------
// Server commands
// ---------------------------------------------------------------------------

async function startServer(ctx: CommandContext): Promise<void> {
  const hasConfig = await configFileExists();
  if (!hasConfig) {
    showConfigMissingNotification();
    return;
  }

  if (ctx.webhookServer.isRunning()) {
    const ticketsDir = await findTicketsDir();
    if (ticketsDir) {
      await ctx.webhookServer.ensureSessionFile(ticketsDir);
    }
    void vscode.window.showInformationMessage(
      `Webhook connected on port ${ctx.webhookServer.getPort()}`
    );
    await ctx.refreshAllProviders();
    return;
  }

  try {
    const ticketsDir = await findTicketsDir();
    await ctx.webhookServer.start(ticketsDir);

    const port = ctx.webhookServer.getPort();
    const configuredPort = ctx.webhookServer.getConfiguredPort();

    if (port !== configuredPort) {
      void vscode.window.showInformationMessage(
        `Operator webhook server started on port ${port} (configured port ${configuredPort} was in use)`
      );
    } else {
      void vscode.window.showInformationMessage(
        `Operator webhook server started on port ${port}`
      );
    }

    updateStatusBar(ctx);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
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
    message = 'Operator server stopped';
  }

  void vscode.window.showInformationMessage(message);
}

function updateStatusBar(ctx: CommandContext): void {
  if (ctx.webhookServer.isRunning()) {
    const port = ctx.webhookServer.getPort();
    ctx.statusBarItem.text = `$(terminal) Operator :${port}`;
    ctx.statusBarItem.tooltip = `Operator webhook server running on port ${port}`;
    ctx.statusBarItem.backgroundColor = undefined;
  } else {
    ctx.statusBarItem.text = '$(terminal) Operator (off)';
    ctx.statusBarItem.tooltip = 'Operator webhook server stopped';
    ctx.statusBarItem.backgroundColor = new vscode.ThemeColor(
      'statusBarItem.warningBackground'
    );
  }
  ctx.statusBarItem.show();
}

// ---------------------------------------------------------------------------
// Queue commands
// ---------------------------------------------------------------------------

async function pauseQueueCommand(ctx: CommandContext): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.pauseQueue();
    void vscode.window.showInformationMessage(result.message);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to pause queue: ${msg}`);
  }
}

async function resumeQueueCommand(ctx: CommandContext): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.resumeQueue();
    void vscode.window.showInformationMessage(result.message);
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to resume queue: ${msg}`);
  }
}

// ---------------------------------------------------------------------------
// Review commands
// ---------------------------------------------------------------------------

async function showAwaitingAgentPicker(
  _apiClient: OperatorApiClient
): Promise<string | undefined> {
  try {
    const response = await fetch(
      `${vscode.workspace.getConfiguration('operator').get('apiUrl', 'http://localhost:7008')}/api/v1/agents/active`
    );
    if (!response.ok) {
      void vscode.window.showErrorMessage('Failed to fetch active agents');
      return undefined;
    }
    const data = (await response.json()) as {
      agents: Array<{
        id: string;
        ticket_id: string;
        project: string;
        status: string;
      }>;
    };

    const awaitingAgents = data.agents.filter(
      (a) => a.status === 'awaiting_input'
    );

    if (awaitingAgents.length === 0) {
      void vscode.window.showInformationMessage('No agents awaiting review');
      return undefined;
    }

    const items = awaitingAgents.map((a) => ({
      label: a.ticket_id,
      description: a.project,
      detail: `Agent: ${a.id}`,
      agentId: a.id,
    }));

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select agent to review',
    });

    return selected?.agentId;
  } catch {
    void vscode.window.showErrorMessage('Failed to fetch agents');
    return undefined;
  }
}

async function approveReviewCommand(
  ctx: CommandContext,
  agentId: string
): Promise<void> {
  const apiClient = new OperatorApiClient();
  let selectedAgentId: string | undefined = agentId;
  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
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
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to approve review: ${msg}`);
  }
}

async function rejectReviewCommand(
  ctx: CommandContext,
  agentId: string
): Promise<void> {
  const apiClient = new OperatorApiClient();
  let selectedAgentId: string | undefined = agentId;
  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  if (!agentId) {
    selectedAgentId = await showAwaitingAgentPicker(apiClient);
    if (!selectedAgentId) {
      return;
    }
  }

  const reason = await vscode.window.showInputBox({
    prompt: 'Enter rejection reason',
    placeHolder: 'Why is this being rejected?',
    validateInput: (value) => {
      if (!value || value.trim().length === 0) {
        return 'Rejection reason is required';
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
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to reject review: ${msg}`);
  }
}

// ---------------------------------------------------------------------------
// Kanban commands
// ---------------------------------------------------------------------------

async function syncKanbanCommand(ctx: CommandContext): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.syncKanban();
    const message = `Synced: ${result.created.length} created, ${result.skipped.length} skipped`;
    if (result.errors.length > 0) {
      void vscode.window.showWarningMessage(
        `${message}, ${result.errors.length} errors`
      );
    } else {
      void vscode.window.showInformationMessage(message);
    }
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to sync kanban: ${msg}`);
  }
}

async function syncKanbanCollectionCommand(
  ctx: CommandContext,
  item: StatusItem
): Promise<void> {
  const provider = item.provider;
  const projectKey = item.projectKey;

  if (!provider || !projectKey) {
    void vscode.window.showWarningMessage('No collection selected for sync.');
    return;
  }

  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    void vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.syncKanbanCollection(provider, projectKey);
    const createdList = result.created.length > 0
      ? ` (${result.created.join(', ')})`
      : '';
    const message = `Synced ${projectKey}: ${result.created.length} created${createdList}, ${result.skipped.length} skipped`;
    if (result.errors.length > 0) {
      void vscode.window.showWarningMessage(`${message}, ${result.errors.length} errors`);
    } else {
      void vscode.window.showInformationMessage(message);
    }
    await ctx.refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to sync collection: ${msg}`);
  }
}

async function addJiraProjectCommand(
  ctx: CommandContext,
  workspaceKey: string
): Promise<void> {
  await addJiraProject(ctx.extensionContext, workspaceKey);
  await ctx.refreshAllProviders();
}

async function addLinearTeamCommand(
  ctx: CommandContext,
  workspaceKey: string
): Promise<void> {
  await addLinearTeam(ctx.extensionContext, workspaceKey);
  await ctx.refreshAllProviders();
}

// ---------------------------------------------------------------------------
// Setup commands
// ---------------------------------------------------------------------------

function showConfigMissingNotification(): void {
  void vscode.window.showInformationMessage(
    'Could not find Operator! configuration file for this repository workspace. Run the setup walkthrough to create it and get started.',
    'Open Setup'
  ).then((choice) => {
    if (choice === 'Open Setup') {
      void vscode.commands.executeCommand(
        'workbench.action.openWalkthrough',
        'untra.operator-terminals#operator-setup',
        true
      );
    }
  });
}

async function updateOperatorContext(ctx: CommandContext): Promise<void> {
  const operatorAvailable = await isOperatorAvailable(ctx.extensionContext);
  await vscode.commands.executeCommand(
    'setContext',
    'operator.operatorAvailable',
    operatorAvailable
  );

  const ticketsParentFound = ctx.getCurrentTicketsDir() !== undefined;
  await vscode.commands.executeCommand(
    'setContext',
    'operator.ticketsParentFound',
    ticketsParentFound
  );

  await updateWalkthroughContext(ctx.extensionContext);
}

async function runSetupCommand(ctx: CommandContext): Promise<void> {
  const workingDir = ctx.extensionContext.globalState.get<string>('operator.workingDirectory');
  if (!workingDir) {
    await vscode.commands.executeCommand('operator.selectWorkingDirectory');
    return;
  }

  const choice = await vscode.window.showInformationMessage(
    `Run operator setup in ${workingDir.replace(os.homedir(), '~')}?`,
    'Yes',
    'Cancel'
  );

  if (choice !== 'Yes') {
    return;
  }

  const operatorPath = await getOperatorPath(ctx.extensionContext);
  const success = await initializeTicketsDirectory(workingDir, operatorPath ?? undefined);

  if (success) {
    const ticketsDir = path.join(workingDir, '.tickets');
    ctx.setCurrentTicketsDir(ticketsDir);
    await ctx.setTicketsDir(ticketsDir);

    const watcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(ticketsDir, '**/*.md')
    );
    watcher.onDidChange(() => void ctx.refreshAllProviders());
    watcher.onDidCreate(() => void ctx.refreshAllProviders());
    watcher.onDidDelete(() => void ctx.refreshAllProviders());
    ctx.extensionContext.subscriptions.push(watcher);

    await updateOperatorContext(ctx);
    void vscode.window.showInformationMessage('Operator setup completed successfully.');
  } else {
    void vscode.window.showErrorMessage('Failed to run operator setup.');
  }
}

async function downloadOperatorCommand(ctx: CommandContext): Promise<void> {
  const existingPath = await getOperatorPath(ctx.extensionContext);
  if (existingPath) {
    const version = await getOperatorVersion(existingPath);
    const choice = await vscode.window.showInformationMessage(
      `Operator ${version ?? 'unknown version'} is already installed at ${existingPath}`,
      'Reinstall/Update',
      'Open Downloads Page',
      'Cancel'
    );

    if (choice === 'Open Downloads Page') {
      void vscode.env.openExternal(
        vscode.Uri.parse('https://operator.untra.io/downloads/')
      );
      return;
    } else if (choice !== 'Reinstall/Update') {
      return;
    }
  }

  try {
    const downloadedPath = await downloadOperator(ctx.extensionContext);
    const version = await getOperatorVersion(downloadedPath);

    void vscode.window.showInformationMessage(
      `Operator ${version ?? getExtensionVersion()} downloaded successfully to ${downloadedPath}`
    );

    await updateOperatorContext(ctx);
    await ctx.refreshAllProviders();
  } catch (error) {
    const msg = error instanceof Error ? error.message : 'Unknown error';

    const choice = await vscode.window.showErrorMessage(
      `Failed to download Operator: ${msg}`,
      'Open Downloads Page',
      'Cancel'
    );

    if (choice === 'Open Downloads Page') {
      void vscode.env.openExternal(
        vscode.Uri.parse('https://operator.untra.io/downloads/')
      );
    }
  }
}

async function startOperatorServerCommand(ctx: CommandContext): Promise<void> {
  const hasConfig = await configFileExists();
  if (!hasConfig) {
    showConfigMissingNotification();
    return;
  }

  const operatorPath = await getOperatorPath(ctx.extensionContext);

  if (!operatorPath) {
    const choice = await vscode.window.showErrorMessage(
      'Operator binary not found',
      'Download Operator',
      'Cancel'
    );

    if (choice === 'Download Operator') {
      await downloadOperatorCommand(ctx);
    }
    return;
  }

  const serverDir = await findOperatorServerDir();
  if (!serverDir) {
    void vscode.window.showErrorMessage('No workspace folder found.');
    return;
  }

  const apiClient = new OperatorApiClient();
  try {
    const health = await apiClient.health();
    const localVersion = await getOperatorVersion(operatorPath);
    if (localVersion && health.version && health.version !== localVersion) {
      // The port is held by a different operator version — adopting it could
      // mix incompatible client/server APIs. Warn instead of silently starting.
      void vscode.window.showWarningMessage(
        `A different Operator version (v${health.version}) is already running on this port; ` +
          `this binary is v${localVersion}. Stop the other instance or set a different ` +
          `port (operator.apiUrl) before starting your own server.`
      );
      return;
    }
    void vscode.window.showInformationMessage(
      health.version ? `Operator is already running (v${health.version})` : 'Operator is already running'
    );
    return;
  } catch {
    // Not running, proceed to start
  }

  const terminalName = 'Operator API';

  if (ctx.terminalManager.exists(terminalName)) {
    ctx.terminalManager.focus(terminalName);
    return;
  }

  ctx.terminalManager.create({
    name: terminalName,
    workingDir: serverDir,
  });

  ctx.terminalManager.send(terminalName, `"${operatorPath}" api`);
  ctx.terminalManager.focus(terminalName);

  void vscode.window.showInformationMessage(
    `Starting Operator API server in ${serverDir}...`
  );

  setTimeout(() => {
    void ctx.refreshAllProviders();
  }, 2000);
}

async function revealTicketsDirCommand(ctx: CommandContext): Promise<void> {
  const dir = ctx.getCurrentTicketsDir();
  if (!dir) {
    void vscode.window.showWarningMessage('No .tickets directory found.');
    return;
  }

  const uri = vscode.Uri.file(dir);
  await vscode.commands.executeCommand('revealFileInOS', uri);
}

// ---------------------------------------------------------------------------
// Create commands
// ---------------------------------------------------------------------------

async function showCreateMenu(ctx: CommandContext): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      { label: '$(rocket) New Delegator', detail: 'delegator', description: 'Create a tool+model pairing for autonomous launches' },
      { label: '$(list-tree) New Issue Type', detail: 'issuetype', description: 'Define a custom issue type with steps' },
      { label: '$(project) New Managed Project', detail: 'project', description: 'Assess and register a project' },
    ],
    {
      title: 'Create New',
      placeHolder: 'What would you like to create?',
    }
  );

  if (!choice) { return; }

  switch (choice.detail) {
    case 'delegator':
      openCreateDelegator(ctx);
      break;
    case 'issuetype':
      // Issue types are managed in the hosted Operator UI.
      await openOperatorUi(ctx.getCurrentTicketsDir(), 'issuetypes');
      break;
    case 'project':
      // Projects are browsed/assessed in the hosted Operator UI.
      await openOperatorUi(ctx.getCurrentTicketsDir(), 'projects');
      break;
  }
}

function openCreateDelegator(ctx: CommandContext, tool?: string, model?: string): void {
  // Delegators bind a model provider to an llm tool, so they live in the Model
  // Providers section (distinct from the Coding Agents / llm-tools section).
  ConfigPanel.createOrShow(ctx.extensionContext.extensionUri);
  ConfigPanel.navigateTo('section-model-providers', {
    action: 'createDelegator',
    tool,
    model,
  });
}

// ---------------------------------------------------------------------------
// Extension activation
// ---------------------------------------------------------------------------

export async function activate(
  context: vscode.ExtensionContext
): Promise<void> {
  // Create output channel for logging
  const outputChannel = vscode.window.createOutputChannel('Operator');
  context.subscriptions.push(outputChannel);
  outputChannel.appendLine('[Operator] Activation started');

  // Initialize issue type service (constructor is safe — no network calls)
  const issueTypeService = new IssueTypeService(outputChannel);

  // Register tree view providers IMMEDIATELY so VS Code never shows
  // "no data provider registered" — they start empty and populate async.
  const statusProvider = new StatusTreeProvider(context);
  const inProgressProvider = new TicketTreeProvider('in-progress', issueTypeService);
  const queueProvider = new TicketTreeProvider('queue', issueTypeService);
  const completedProvider = new TicketTreeProvider('completed', issueTypeService);

  const statusTreeView = vscode.window.createTreeView('operator-status', {
    treeDataProvider: statusProvider,
  });
  context.subscriptions.push(
    statusTreeView,
    vscode.window.registerTreeDataProvider('operator-in-progress', inProgressProvider),
    vscode.window.registerTreeDataProvider('operator-queue', queueProvider),
    vscode.window.registerTreeDataProvider('operator-completed', completedProvider)
  );

  // Synchronous object construction — these constructors do no I/O
  const terminalManager = new TerminalManager();
  terminalManager.setIssueTypeService(issueTypeService);
  inProgressProvider.setTerminalManager(terminalManager);

  // Handle deep-links from the Operator web UI / control plane:
  //   vscode://untra.operator-terminals/focus-session?name=<terminal>
  // focuses the agent's terminal tab by name. The web UI's launch panel emits
  // this link after launching a ticket when the operator control wrapper is VS
  // Code, so the user can jump straight to the running agent's terminal.
  context.subscriptions.push(
    vscode.window.registerUriHandler({
      handleUri(uri: vscode.Uri) {
        if (uri.path !== '/focus-session') {
          return;
        }
        const name = new URLSearchParams(uri.query).get('name');
        if (!name) {
          void vscode.window.showWarningMessage(
            'Operator: focus-session link is missing a terminal name'
          );
          return;
        }
        if (terminalManager.exists(name)) {
          terminalManager.focus(name);
        } else {
          void vscode.window.showWarningMessage(
            `Operator: no terminal named '${name}' to focus`
          );
        }
      },
    })
  );

  const webhookServer = new WebhookServer(terminalManager);
  const launchManager = new LaunchManager(terminalManager);

  statusProvider.setWebhookServer(webhookServer);

  // Create status bar items
  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBarItem.command = 'operator.showStatus';
  context.subscriptions.push(statusBarItem);

  const createBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    99
  );
  createBarItem.text = '$(add) New';
  createBarItem.tooltip = 'Create new delegator, issue type, or project';
  createBarItem.command = 'operator.showCreateMenu';
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
    createBarItem,
    outputChannel,
    getCurrentTicketsDir: () => currentTicketsDir,
    setCurrentTicketsDir: (dir) => { currentTicketsDir = dir; },
    refreshAllProviders: async () => {
      await statusProvider.refresh();
      await inProgressProvider.refresh();
      await queueProvider.refresh();
      await completedProvider.refresh();
    },
    setTicketsDir: async (dir) => {
      await statusProvider.setTicketsDir(dir);
      await inProgressProvider.setTicketsDir(dir);
      await queueProvider.setTicketsDir(dir);
      await completedProvider.setTicketsDir(dir);
      launchManager.setTicketsDir(dir);
    },
  };

  // Register all commands BEFORE any async work — ensures commands are
  // always available even if network/API initialization fails.
  context.subscriptions.push(
    vscode.commands.registerCommand('operator.showStatus', () => showStatus(ctx)),
    vscode.commands.registerCommand('operator.refreshTickets', () => ctx.refreshAllProviders()),
    vscode.commands.registerCommand('operator.focusTicket',
      (name: string, ticket?: TicketInfo) => focusTicketTerminal(ctx, name, ticket)),
    vscode.commands.registerCommand('operator.openTicket', openTicketFile),
    vscode.commands.registerCommand('operator.launchTicket',
      (treeItem?: TicketItem) => launchTicketCommand(ctx, treeItem)),
    vscode.commands.registerCommand('operator.launchTicketWithOptions',
      (treeItem?: TicketItem) => launchTicketWithOptionsCommand(ctx, treeItem)),
    vscode.commands.registerCommand('operator.relaunchTicket',
      (ticket: TicketInfo) => relaunchTicketCommand(ctx, ticket)),
    vscode.commands.registerCommand('operator.launchTicketFromEditor',
      () => launchTicketFromEditorCommand(ctx)),
    vscode.commands.registerCommand('operator.launchTicketFromEditorWithOptions',
      () => launchTicketFromEditorWithOptionsCommand(ctx)),
    vscode.commands.registerCommand('operator.downloadOperator',
      () => downloadOperatorCommand(ctx)),
    vscode.commands.registerCommand('operator.pauseQueue',
      () => pauseQueueCommand(ctx)),
    vscode.commands.registerCommand('operator.resumeQueue',
      () => resumeQueueCommand(ctx)),
    vscode.commands.registerCommand('operator.syncKanban',
      () => syncKanbanCommand(ctx)),
    vscode.commands.registerCommand('operator.approveReview',
      (agentId: string) => approveReviewCommand(ctx, agentId)),
    vscode.commands.registerCommand('operator.rejectReview',
      (agentId: string) => rejectReviewCommand(ctx, agentId)),
    vscode.commands.registerCommand('operator.startOperatorServer',
      () => startOperatorServerCommand(ctx)),
    vscode.commands.registerCommand('operator.selectWorkingDirectory',
      async () => {
        const operatorPath = await getOperatorPath(ctx.extensionContext);
        await selectWorkingDirectory(ctx.extensionContext, operatorPath ?? undefined);
      }),
    vscode.commands.registerCommand('operator.runSetup',
      () => runSetupCommand(ctx)),
    vscode.commands.registerCommand('operator.checkKanbanConnection',
      () => checkKanbanConnection(ctx.extensionContext)),
    vscode.commands.registerCommand('operator.configureJira',
      () => configureJira(ctx.extensionContext)),
    vscode.commands.registerCommand('operator.configureLinear',
      () => configureLinear(ctx.extensionContext)),
    vscode.commands.registerCommand('operator.startKanbanOnboarding',
      () => startKanbanOnboarding(ctx.extensionContext)),
    vscode.commands.registerCommand('operator.startGitOnboarding',
      () => startGitOnboarding().then(() => ctx.refreshAllProviders())),
    vscode.commands.registerCommand('operator.configureGitHub',
      () => onboardGitHub().then(() => ctx.refreshAllProviders())),
    vscode.commands.registerCommand('operator.configureGitLab',
      () => onboardGitLab().then(() => ctx.refreshAllProviders())),
    vscode.commands.registerCommand('operator.showCreateMenu',
      () => showCreateMenu(ctx)),
    vscode.commands.registerCommand('operator.openCreateDelegator',
      (tool?: string, model?: string) => openCreateDelegator(ctx, tool, model)),
    vscode.commands.registerCommand('operator.detectLlmTools',
      () => detectLlmTools(ctx.extensionContext, getOperatorPath)),
    vscode.commands.registerCommand('operator.setDefaultLlm',
      async (tool?: string, model?: string) => {
        if (!tool || !model) { return; }
        try {
          const apiUrl = await discoverApiUrl(ctx.getCurrentTicketsDir());
          const resp = await fetch(`${apiUrl}/api/v1/llm-tools/default`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ tool, model }),
          });
          if (resp.ok) {
            void vscode.window.showInformationMessage(`Default LLM set to ${tool}:${model}`);
            void ctx.refreshAllProviders();
          } else {
            void vscode.window.showErrorMessage('Failed to set default LLM');
          }
        } catch {
          void vscode.window.showErrorMessage('Operator API not available');
        }
      }),
    vscode.commands.registerCommand('operator.openWalkthrough', openWalkthrough),
    vscode.commands.registerCommand('operator.openSettings',
      () => ConfigPanel.createOrShow(ctx.extensionContext.extensionUri)),
    // Link out to the daemon-hosted Operator UI (Simple Browser) for the
    // operational surfaces the extension webview no longer reimplements.
    vscode.commands.registerCommand('operator.openUi',
      () => openOperatorUi(ctx.getCurrentTicketsDir(), 'dashboard')),
    vscode.commands.registerCommand('operator.openIssueTypes',
      () => openOperatorUi(ctx.getCurrentTicketsDir(), 'issuetypes')),
    vscode.commands.registerCommand('operator.openProjects',
      () => openOperatorUi(ctx.getCurrentTicketsDir(), 'projects')),
    vscode.commands.registerCommand('operator.openKanban',
      () => openOperatorUi(ctx.getCurrentTicketsDir(), 'kanban')),
    vscode.commands.registerCommand('operator.openQueue',
      () => openOperatorUi(ctx.getCurrentTicketsDir(), 'queue')),
    vscode.commands.registerCommand('operator.syncKanbanCollection',
      (item: StatusItem) => syncKanbanCollectionCommand(ctx, item)),
    vscode.commands.registerCommand('operator.addJiraProject',
      (workspaceKey: string) => addJiraProjectCommand(ctx, workspaceKey)),
    vscode.commands.registerCommand('operator.addLinearTeam',
      (workspaceKey: string) => addLinearTeamCommand(ctx, workspaceKey)),
    vscode.commands.registerCommand('operator.revealTicketsDir',
      () => revealTicketsDirCommand(ctx)),
    vscode.commands.registerCommand('operator.startWebhookServer',
      () => startServer(ctx)),
    vscode.commands.registerCommand('operator.connectMcpServer',
      () => connectMcpServer(ctx.getCurrentTicketsDir())),
    // ABXY navigation commands for status panel — registered last but still before async init
    vscode.commands.registerCommand('operator.statusSpecialAction', () => {
      const selected = ctx.statusTreeView?.selection?.[0];
      if (selected instanceof StatusItem && selected.specialCommand) {
        const args = (selected.specialCommand.arguments ?? []) as unknown[];
        void vscode.commands.executeCommand(
          selected.specialCommand.command,
          ...args
        );
      }
    }),
    vscode.commands.registerCommand('operator.statusRefreshAction', () => {
      const selected = ctx.statusTreeView?.selection?.[0];
      if (selected instanceof StatusItem && selected.refreshCommand) {
        const args = (selected.refreshCommand.arguments ?? []) as unknown[];
        void vscode.commands.executeCommand(
          selected.refreshCommand.command,
          ...args
        );
      }
    }),
    vscode.commands.registerCommand('operator.statusBackAction', () => {
      void vscode.commands.executeCommand('list.collapse');
    }),
  );

  outputChannel.appendLine('[Operator] Command registration complete');

  // Async initialization — failures here are recoverable; commands still work.
  try {
    await issueTypeService.refresh();

    // Find tickets directory (check parent first, then workspace)
    currentTicketsDir = await findParentTicketsDir();
    await ctx.setTicketsDir(currentTicketsDir);

    // Set up file watcher if tickets directory exists
    if (currentTicketsDir) {
      const watcher = vscode.workspace.createFileSystemWatcher(
        new vscode.RelativePattern(currentTicketsDir, '**/*.md')
      );
      watcher.onDidChange(() => void ctx.refreshAllProviders());
      watcher.onDidCreate(() => void ctx.refreshAllProviders());
      watcher.onDidDelete(() => void ctx.refreshAllProviders());
      context.subscriptions.push(watcher);
    }

    // Auto-start if configured and config.toml exists
    const autoStart = vscode.workspace
      .getConfiguration('operator')
      .get('autoStart', true);
    if (autoStart) {
      const hasConfig = await configFileExists();
      if (hasConfig) {
        await startServer(ctx);
      } else {
        showConfigMissingNotification();
      }
    }

    updateStatusBar(ctx);

    // Set initial context for command visibility
    await updateOperatorContext(ctx);

    // Restore working directory from persistent VS Code settings if globalState is empty
    const configWorkingDir = vscode.workspace.getConfiguration('operator').get<string>('workingDirectory');
    if (configWorkingDir && !context.globalState.get('operator.workingDirectory')) {
      await context.globalState.update('operator.workingDirectory', configWorkingDir);
    }

    // Auto-open walkthrough for new users with no working directory
    const workingDirectory = context.globalState.get<string>('operator.workingDirectory');
    if (!workingDirectory) {
      void vscode.commands.executeCommand(
        'workbench.action.openWalkthrough',
        'untra.operator-terminals#operator-setup',
        false
      );
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    outputChannel.appendLine(`[Operator] Activation error: ${msg}`);
    if (err instanceof Error && err.stack) {
      outputChannel.appendLine(err.stack);
    }
    void vscode.window.showErrorMessage(`Operator extension failed to fully activate: ${msg}`);
  }
}

/**
 * Extension deactivation
 */
export function deactivate(): void {
  // Cleanup handled by disposables registered in context.subscriptions
}
