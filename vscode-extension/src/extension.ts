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
import * as fs from 'fs/promises';
import { TerminalManager } from './terminal-manager';
import { WebhookServer } from './webhook-server';
import { TicketTreeProvider, TicketItem } from './ticket-provider';
import { StatusTreeProvider, StatusItem } from './status-provider';
import { LaunchManager } from './launch-manager';
import { showLaunchOptionsDialog, showTicketPicker } from './launch-dialog';
import { parseTicketMetadata, getCurrentSessionId } from './ticket-parser';
import { TicketInfo } from './types';
import { OperatorApiClient } from './api-client';
import { IssueTypeService } from './issuetype-service';
import {
  downloadOperator,
  getOperatorPath,
  isOperatorAvailable,
  getOperatorVersion,
  getExtensionVersion,
} from './operator-binary';
import {
  updateWalkthroughContext,
  selectWorkingDirectory,
  checkKanbanConnection,
  configureJira,
  configureLinear,
  detectLlmTools,
  openWalkthrough,
  startKanbanOnboarding,
} from './walkthrough';
import { addJiraProject, addLinearTeam } from './kanban-onboarding';
import { ConfigPanel } from './config-panel';
import { configFileExists, getResolvedConfigPath } from './config-paths';

/**
 * Show a notification when config.toml is missing, with a button to open the walkthrough.
 */
function showConfigMissingNotification(): void {
  // Fire notification without awaiting to prevent blocking activation
  void vscode.window.showInformationMessage(
    `Operator! could not find configuration ${getResolvedConfigPath() || 'config.toml'}. Run the setup walkthrough to create it and get started`,
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

let terminalManager: TerminalManager;
let webhookServer: WebhookServer;
let statusBarItem: vscode.StatusBarItem;
let launchManager: LaunchManager;
let issueTypeService: IssueTypeService;

// TreeView providers
let statusProvider: StatusTreeProvider;
let inProgressProvider: TicketTreeProvider;
let queueProvider: TicketTreeProvider;
let completedProvider: TicketTreeProvider;

// Current tickets directory
let currentTicketsDir: string | undefined;

// Output channel for logging
let outputChannel: vscode.OutputChannel;

// Extension context for use in commands
let extensionContext: vscode.ExtensionContext;

/**
 * Extension activation
 */
export async function activate(
  context: vscode.ExtensionContext
): Promise<void> {
  // Store context for use in commands
  extensionContext = context;

  // Create output channel for logging
  outputChannel = vscode.window.createOutputChannel('Operator');
  context.subscriptions.push(outputChannel);

  // Initialize issue type service (fetches types from API)
  issueTypeService = new IssueTypeService(outputChannel);
  await issueTypeService.refresh();

  terminalManager = new TerminalManager();
  terminalManager.setIssueTypeService(issueTypeService);
  webhookServer = new WebhookServer(terminalManager);
  launchManager = new LaunchManager(terminalManager);

  // Create status bar item
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBarItem.command = 'operator.showStatus';
  context.subscriptions.push(statusBarItem);

  // Create TreeView providers (with issue type service)
  statusProvider = new StatusTreeProvider(context);
  inProgressProvider = new TicketTreeProvider('in-progress', issueTypeService, terminalManager);
  queueProvider = new TicketTreeProvider('queue', issueTypeService);
  completedProvider = new TicketTreeProvider('completed', issueTypeService);

  // Register TreeViews
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('operator-status', statusProvider),
    vscode.window.registerTreeDataProvider(
      'operator-in-progress',
      inProgressProvider
    ),
    vscode.window.registerTreeDataProvider('operator-queue', queueProvider),
    vscode.window.registerTreeDataProvider(
      'operator-completed',
      completedProvider
    )
  );

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('operator.showStatus', showStatus),
    vscode.commands.registerCommand('operator.refreshTickets', refreshAllProviders),
    vscode.commands.registerCommand('operator.focusTicket', focusTicketTerminal),
    vscode.commands.registerCommand('operator.openTicket', openTicketFile),
    vscode.commands.registerCommand('operator.launchTicket', launchTicketCommand),
    vscode.commands.registerCommand(
      'operator.launchTicketWithOptions',
      launchTicketWithOptionsCommand
    ),
    vscode.commands.registerCommand('operator.relaunchTicket', relaunchTicketCommand),
    vscode.commands.registerCommand(
      'operator.launchTicketFromEditor',
      launchTicketFromEditorCommand
    ),
    vscode.commands.registerCommand(
      'operator.launchTicketFromEditorWithOptions',
      launchTicketFromEditorWithOptionsCommand
    ),
    vscode.commands.registerCommand(
      'operator.downloadOperator',
      downloadOperatorCommand
    ),
    vscode.commands.registerCommand('operator.pauseQueue', pauseQueueCommand),
    vscode.commands.registerCommand('operator.resumeQueue', resumeQueueCommand),
    vscode.commands.registerCommand('operator.syncKanban', syncKanbanCommand),
    vscode.commands.registerCommand(
      'operator.approveReview',
      approveReviewCommand
    ),
    vscode.commands.registerCommand(
      'operator.rejectReview',
      rejectReviewCommand
    ),
    vscode.commands.registerCommand(
      'operator.startOperatorServer',
      startOperatorServerCommand
    ),
    vscode.commands.registerCommand(
      'operator.selectWorkingDirectory',
      async () => {
        const operatorPath = await getOperatorPath(extensionContext);
        await selectWorkingDirectory(extensionContext, operatorPath ?? undefined);
      }
    ),
    vscode.commands.registerCommand(
      'operator.checkKanbanConnection',
      () => checkKanbanConnection(extensionContext)
    ),
    vscode.commands.registerCommand(
      'operator.configureJira',
      () => configureJira(extensionContext)
    ),
    vscode.commands.registerCommand(
      'operator.configureLinear',
      () => configureLinear(extensionContext)
    ),
    vscode.commands.registerCommand(
      'operator.startKanbanOnboarding',
      () => startKanbanOnboarding(extensionContext)
    ),
    vscode.commands.registerCommand(
      'operator.detectLlmTools',
      () => detectLlmTools(extensionContext, getOperatorPath)
    ),
    vscode.commands.registerCommand('operator.openWalkthrough', openWalkthrough),
    vscode.commands.registerCommand('operator.openSettings', () =>
      ConfigPanel.createOrShow(context.extensionUri)
    ),
    vscode.commands.registerCommand(
      'operator.syncKanbanCollection',
      syncKanbanCollectionCommand
    ),
    vscode.commands.registerCommand(
      'operator.addJiraProject',
      (workspaceKey?: string) => addJiraProjectCommand(workspaceKey)
    ),
    vscode.commands.registerCommand(
      'operator.addLinearTeam',
      (workspaceKey?: string) => addLinearTeamCommand(workspaceKey)
    ),
    vscode.commands.registerCommand(
      'operator.revealTicketsDir',
      revealTicketsDirCommand
    )
  );

  // Find tickets directory (check parent first, then workspace)
  currentTicketsDir = await findParentTicketsDir();
  await setTicketsDir(currentTicketsDir);

  // Set up file watcher if tickets directory exists
  if (currentTicketsDir) {
    const watcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(currentTicketsDir, '**/*.md')
    );
    watcher.onDidChange(() => refreshAllProviders());
    watcher.onDidCreate(() => refreshAllProviders());
    watcher.onDidDelete(() => refreshAllProviders());
    context.subscriptions.push(watcher);
  }

  // Auto-start if configured and config.toml exists
  const autoStart = vscode.workspace
    .getConfiguration('operator')
    .get('autoStart', true);
  if (autoStart) {
    const hasConfig = await configFileExists();
    if (hasConfig) {
      await startServer();
    } else {
      showConfigMissingNotification();
    }
  }

  updateStatusBar();

  // Set initial context for command visibility
  await updateOperatorContext();

  // Restore working directory from persistent VS Code settings if globalState is empty
  const configWorkingDir = vscode.workspace.getConfiguration('operator').get<string>('workingDirectory');
  if (configWorkingDir && !context.globalState.get('operator.workingDirectory')) {
    await context.globalState.update('operator.workingDirectory', configWorkingDir);
  }

  // Auto-open walkthrough for new users with no working directory
  const workingDirectory = context.globalState.get<string>('operator.workingDirectory');
  if (!workingDirectory) {
    vscode.commands.executeCommand(
      'workbench.action.openWalkthrough',
      'untra.operator-terminals#operator-setup',
      false
    );
  }
}

/**
 * Find .tickets directory - check parent directory first, then workspace
 */
async function findParentTicketsDir(): Promise<string | undefined> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  // First check parent directory for .tickets (monorepo setup)
  const parentDir = path.dirname(workspaceFolder.uri.fsPath);
  const parentTicketsPath = path.join(parentDir, '.tickets');

  try {
    await fs.access(parentTicketsPath);
    return parentTicketsPath;
  } catch {
    // Parent doesn't have .tickets, check workspace
  }

  // Fall back to configured tickets directory in workspace
  const configuredDir = vscode.workspace
    .getConfiguration('operator')
    .get<string>('ticketsDir', '.tickets');

  const ticketsPath = path.isAbsolute(configuredDir)
    ? configuredDir
    : path.join(workspaceFolder.uri.fsPath, configuredDir);

  try {
    await fs.access(ticketsPath);
    return ticketsPath;
  } catch {
    // .tickets directory doesn't exist yet - create it
    try {
      await fs.mkdir(ticketsPath, { recursive: true });
      return ticketsPath;
    } catch {
      return undefined;
    }
  }
}

/**
 * Find the .tickets directory for webhook session file.
 * Walks up from workspace to find existing .tickets, or creates in parent (org level).
 */
async function findTicketsDir(): Promise<string | undefined> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  const configuredDir = vscode.workspace
    .getConfiguration('operator')
    .get<string>('ticketsDir', '.tickets');

  // If absolute path configured, use it directly
  if (path.isAbsolute(configuredDir)) {
    try {
      await fs.mkdir(configuredDir, { recursive: true });
      return configuredDir;
    } catch {
      return undefined;
    }
  }

  // Walk up from workspace to find existing .tickets directory
  let currentDir = workspaceFolder.uri.fsPath;
  const root = path.parse(currentDir).root;

  while (currentDir !== root) {
    const ticketsPath = path.join(currentDir, configuredDir);
    try {
      await fs.access(ticketsPath);
      return ticketsPath; // Found existing .tickets
    } catch {
      // Not found, try parent
      currentDir = path.dirname(currentDir);
    }
  }

  // Not found - create in parent of workspace (org level)
  const parentDir = path.dirname(workspaceFolder.uri.fsPath);
  if (parentDir === workspaceFolder.uri.fsPath) {
    // Workspace is at filesystem root
    return undefined;
  }

  const ticketsPath = path.join(parentDir, configuredDir);
  try {
    await fs.mkdir(ticketsPath, { recursive: true });
    return ticketsPath;
  } catch {
    return undefined;
  }
}

/**
 * Find the directory to run the operator server in.
 * Prefers parent directory if it has .tickets/operator/, otherwise uses workspace.
 */
async function findOperatorServerDir(): Promise<string | undefined> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  const workspaceDir = workspaceFolder.uri.fsPath;
  const parentDir = path.dirname(workspaceDir);

  // Check if parent has .tickets/operator/ (initialized operator setup)
  const parentOperatorPath = path.join(parentDir, '.tickets', 'operator');
  try {
    await fs.access(parentOperatorPath);
    return parentDir; // Parent has initialized operator
  } catch {
    // Parent doesn't have .tickets/operator
  }

  // Fall back to workspace directory
  return workspaceDir;
}

/**
 * Set tickets directory for all providers
 */
async function setTicketsDir(dir: string | undefined): Promise<void> {
  await statusProvider.setTicketsDir(dir);
  await inProgressProvider.setTicketsDir(dir);
  await queueProvider.setTicketsDir(dir);
  await completedProvider.setTicketsDir(dir);
  launchManager.setTicketsDir(dir);
}

/**
 * Refresh all TreeView providers
 */
async function refreshAllProviders(): Promise<void> {
  await statusProvider.refresh();
  await inProgressProvider.refresh();
  await queueProvider.refresh();
  await completedProvider.refresh();
}

/**
 * Focus a terminal by name, or offer relaunch if not found
 */
async function focusTicketTerminal(
  terminalName: string,
  ticket?: TicketInfo
): Promise<void> {
  if (terminalManager.exists(terminalName)) {
    await terminalManager.focus(terminalName);
  } else if (ticket) {
    await launchManager.offerRelaunch(ticket);
  } else {
    vscode.window.showWarningMessage(`Terminal '${terminalName}' not found`);
  }
}

/**
 * Open a ticket file
 */
function openTicketFile(filePath: string): void {
  vscode.workspace.openTextDocument(filePath).then((doc) => {
    vscode.window.showTextDocument(doc);
  });
}

/**
 * Start the webhook server
 */
async function startServer(): Promise<void> {
  // Require config.toml before starting webhook server
  const hasConfig = await configFileExists();
  if (!hasConfig) {
    showConfigMissingNotification();
    return;
  }

  if (webhookServer.isRunning()) {
    vscode.window.showInformationMessage(
      'Operator webhook server already running'
    );
    return;
  }

  try {
    // Find tickets directory for session file
    const ticketsDir = await findTicketsDir();

    // Start server with optional session file registration
    await webhookServer.start(ticketsDir);

    const port = webhookServer.getPort();
    const configuredPort = webhookServer.getConfiguredPort();

    if (port !== configuredPort) {
      vscode.window.showInformationMessage(
        `Operator webhook server started on port ${port} (configured port ${configuredPort} was in use)`
      );
    } else {
      vscode.window.showInformationMessage(
        `Operator webhook server started on port ${port}`
      );
    }

    updateStatusBar();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to start webhook server: ${msg}`);
  }
}

/**
 * Show server status
 */
function showStatus(): void {
  const running = webhookServer.isRunning();
  const port = webhookServer.getPort();
  const configuredPort = webhookServer.getConfiguredPort();
  const terminals = terminalManager.list();

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

  vscode.window.showInformationMessage(message);
}

/**
 * Update status bar appearance
 */
function updateStatusBar(): void {
  if (webhookServer.isRunning()) {
    const port = webhookServer.getPort();
    statusBarItem.text = `$(terminal) Operator :${port}`;
    statusBarItem.tooltip = `Operator webhook server running on port ${port}`;
    statusBarItem.backgroundColor = undefined;
  } else {
    statusBarItem.text = '$(terminal) Operator (off)';
    statusBarItem.tooltip = 'Operator webhook server stopped';
    statusBarItem.backgroundColor = new vscode.ThemeColor(
      'statusBarItem.warningBackground'
    );
  }
  statusBarItem.show();
}

/**
 * Command: Launch ticket (quick, uses defaults)
 *
 * When invoked from inline button on tree item, the TicketItem is passed.
 * When invoked from command palette, shows a ticket picker.
 */
async function launchTicketCommand(treeItem?: TicketItem): Promise<void> {
  let ticket: TicketInfo | undefined;

  // If called from inline button, treeItem contains the ticket
  if (treeItem?.ticket) {
    ticket = treeItem.ticket;
  } else {
    // Called from command palette - show picker
    const tickets = queueProvider.getTickets();
    if (tickets.length === 0) {
      vscode.window.showInformationMessage('No tickets in queue');
      return;
    }
    ticket = await showTicketPicker(tickets);
  }

  if (!ticket) {
    return;
  }

  await launchManager.launchTicket(ticket, {
    model: 'sonnet',
    yoloMode: false,
    resumeSession: false,
  });
}

/**
 * Command: Launch ticket with options dialog
 *
 * When invoked from inline button on tree item, the TicketItem is passed.
 * When invoked from command palette, shows a ticket picker.
 */
async function launchTicketWithOptionsCommand(
  treeItem?: TicketItem
): Promise<void> {
  let ticket: TicketInfo | undefined;

  // If called from inline button, treeItem contains the ticket
  if (treeItem?.ticket) {
    ticket = treeItem.ticket;
  } else {
    // Called from command palette - show picker
    const tickets = queueProvider.getTickets();
    if (tickets.length === 0) {
      vscode.window.showInformationMessage('No tickets in queue');
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

  await launchManager.launchTicket(ticket, options);
}

/**
 * Command: Relaunch in-progress ticket
 */
async function relaunchTicketCommand(ticket: TicketInfo): Promise<void> {
  await launchManager.offerRelaunch(ticket);
}

/**
 * Check if a file path is a ticket file in the .tickets directory
 */
function isTicketFile(filePath: string): boolean {
  const normalized = filePath.replace(/\\/g, '/');
  return (
    (normalized.includes('.tickets/queue/') ||
      normalized.includes('.tickets/in-progress/')) &&
    normalized.endsWith('.md')
  );
}

/**
 * Command: Launch ticket from the active editor
 *
 * Uses the Operator API to properly claim the ticket and track state.
 */
async function launchTicketFromEditorCommand(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showWarningMessage('No active editor');
    return;
  }

  const filePath = editor.document.uri.fsPath;
  if (!isTicketFile(filePath)) {
    vscode.window.showWarningMessage(
      'Current file is not a ticket in .tickets/ directory'
    );
    return;
  }

  const metadata = await parseTicketMetadata(filePath);
  if (!metadata?.id) {
    vscode.window.showErrorMessage('Could not parse ticket ID from file');
    return;
  }

  const apiClient = new OperatorApiClient();

  // Check if Operator API is running
  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  // Launch via Operator API
  try {
    const response = await apiClient.launchTicket(metadata.id, {
      provider: null,
      wrapper: 'vscode',
      model: 'sonnet',
      yolo_mode: false,
      retry_reason: null,
      resume_session_id: null,
    });

    // Create terminal and execute command
    await terminalManager.create({
      name: response.terminal_name,
      workingDir: response.working_directory,
    });
    await terminalManager.send(response.terminal_name, response.command);
    await terminalManager.focus(response.terminal_name);

    const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
    vscode.window.showInformationMessage(
      `Launched agent for ${response.ticket_id}${worktreeMsg}`
    );

    // Refresh ticket providers to reflect the change
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
  }
}

/**
 * Command: Launch ticket from editor with options dialog
 */
async function launchTicketFromEditorWithOptionsCommand(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showWarningMessage('No active editor');
    return;
  }

  const filePath = editor.document.uri.fsPath;
  if (!isTicketFile(filePath)) {
    vscode.window.showWarningMessage(
      'Current file is not a ticket in .tickets/ directory'
    );
    return;
  }

  const metadata = await parseTicketMetadata(filePath);
  if (!metadata?.id) {
    vscode.window.showErrorMessage('Could not parse ticket ID from file');
    return;
  }

  // Create a minimal TicketInfo for the dialog
  const ticketType = issueTypeService.extractTypeFromId(metadata.id);
  const ticketStatus = (metadata.status === 'in-progress' || metadata.status === 'completed')
    ? metadata.status as 'in-progress' | 'completed'
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

  // Check if Operator API is running
  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  // Launch via Operator API
  try {
    const response = await apiClient.launchTicket(metadata.id, {
      provider: null,
      wrapper: 'vscode',
      model: options.model,
      yolo_mode: options.yoloMode,
      retry_reason: null,
      resume_session_id: null,
    });

    // Create terminal and execute command
    await terminalManager.create({
      name: response.terminal_name,
      workingDir: response.working_directory,
    });
    await terminalManager.send(response.terminal_name, response.command);
    await terminalManager.focus(response.terminal_name);

    const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
    vscode.window.showInformationMessage(
      `Launched agent for ${response.ticket_id}${worktreeMsg}`
    );

    // Refresh ticket providers to reflect the change
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
  }
}

/**
 * Update context variables for command visibility
 */
async function updateOperatorContext(): Promise<void> {
  const operatorAvailable = await isOperatorAvailable(extensionContext);
  await vscode.commands.executeCommand(
    'setContext',
    'operator.operatorAvailable',
    operatorAvailable
  );

  // Check if parent directory has .tickets/
  const ticketsParentFound = currentTicketsDir !== undefined;
  await vscode.commands.executeCommand(
    'setContext',
    'operator.ticketsParentFound',
    ticketsParentFound
  );

  // Update walkthrough context keys
  await updateWalkthroughContext(extensionContext);
}

/**
 * Command: Download Operator binary
 */
async function downloadOperatorCommand(): Promise<void> {
  // Check if already installed
  const existingPath = await getOperatorPath(extensionContext);
  if (existingPath) {
    const version = await getOperatorVersion(existingPath);
    const choice = await vscode.window.showInformationMessage(
      `Operator ${version ?? 'unknown version'} is already installed at ${existingPath}`,
      'Reinstall/Update',
      'Open Downloads Page',
      'Cancel'
    );

    if (choice === 'Open Downloads Page') {
      vscode.env.openExternal(
        vscode.Uri.parse('https://operator.untra.io/downloads/')
      );
      return;
    } else if (choice !== 'Reinstall/Update') {
      return;
    }
  }

  try {
    const downloadedPath = await downloadOperator(extensionContext);
    const version = await getOperatorVersion(downloadedPath);

    vscode.window.showInformationMessage(
      `Operator ${version ?? getExtensionVersion()} downloaded successfully to ${downloadedPath}`
    );

    // Update context for command visibility
    await updateOperatorContext();

    // Refresh status provider
    await refreshAllProviders();
  } catch (error) {
    const msg = error instanceof Error ? error.message : 'Unknown error';

    // Offer to open downloads page on failure
    const choice = await vscode.window.showErrorMessage(
      `Failed to download Operator: ${msg}`,
      'Open Downloads Page',
      'Cancel'
    );

    if (choice === 'Open Downloads Page') {
      vscode.env.openExternal(
        vscode.Uri.parse('https://operator.untra.io/downloads/')
      );
    }
  }
}

/**
 * Command: Start Operator API server
 */
async function startOperatorServerCommand(): Promise<void> {
  // Ensure config.toml exists before starting the server
  const hasConfig = await configFileExists();
  if (!hasConfig) {
    showConfigMissingNotification();
    return;
  }

  const operatorPath = await getOperatorPath(extensionContext);

  if (!operatorPath) {
    const choice = await vscode.window.showErrorMessage(
      'Operator binary not found',
      'Download Operator',
      'Cancel'
    );

    if (choice === 'Download Operator') {
      await downloadOperatorCommand();
    }
    return;
  }

  // Find the directory to run the operator server in
  const serverDir = await findOperatorServerDir();
  if (!serverDir) {
    vscode.window.showErrorMessage('No workspace folder found.');
    return;
  }

  // Check if Operator is already running
  const apiClient = new OperatorApiClient();
  try {
    await apiClient.health();
    vscode.window.showInformationMessage('Operator is already running');
    return;
  } catch {
    // Not running, proceed to start
  }

  // Create terminal and run operator api
  const terminalName = 'Operator API';

  if (terminalManager.exists(terminalName)) {
    await terminalManager.focus(terminalName);
    return;
  }

  await terminalManager.create({
    name: terminalName,
    workingDir: serverDir,
  });

  await terminalManager.send(terminalName, `"${operatorPath}" api`);
  await terminalManager.focus(terminalName);

  vscode.window.showInformationMessage(
    `Starting Operator API server in ${serverDir}...`
  );

  // Wait a moment and refresh providers to pick up the new status
  setTimeout(async () => {
    await refreshAllProviders();
  }, 2000);
}

/**
 * Command: Pause queue processing
 */
async function pauseQueueCommand(): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.pauseQueue();
    vscode.window.showInformationMessage(result.message);
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to pause queue: ${msg}`);
  }
}

/**
 * Command: Resume queue processing
 */
async function resumeQueueCommand(): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.resumeQueue();
    vscode.window.showInformationMessage(result.message);
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to resume queue: ${msg}`);
  }
}

/**
 * Command: Sync kanban collections
 */
async function syncKanbanCommand(): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  try {
    const result = await apiClient.syncKanban();
    const message = `Synced: ${result.created.length} created, ${result.skipped.length} skipped`;
    if (result.errors.length > 0) {
      vscode.window.showWarningMessage(
        `${message}, ${result.errors.length} errors`
      );
    } else {
      vscode.window.showInformationMessage(message);
    }
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to sync kanban: ${msg}`);
  }
}

/**
 * Command: Approve agent review
 */
async function approveReviewCommand(agentId?: string): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  // If no agent ID provided, show picker for awaiting agents
  if (!agentId) {
    agentId = await showAwaitingAgentPicker(apiClient);
    if (!agentId) {
      return;
    }
  }

  try {
    const result = await apiClient.approveReview(agentId);
    vscode.window.showInformationMessage(result.message);
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to approve review: ${msg}`);
  }
}

/**
 * Command: Reject agent review
 */
async function rejectReviewCommand(agentId?: string): Promise<void> {
  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
      'Operator API not running. Start operator first.'
    );
    return;
  }

  // If no agent ID provided, show picker for awaiting agents
  if (!agentId) {
    agentId = await showAwaitingAgentPicker(apiClient);
    if (!agentId) {
      return;
    }
  }

  // Ask for rejection reason
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
    const result = await apiClient.rejectReview(agentId, reason);
    vscode.window.showInformationMessage(result.message);
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to reject review: ${msg}`);
  }
}

/**
 * Helper: Show picker for agents awaiting review
 */
async function showAwaitingAgentPicker(
  _apiClient: OperatorApiClient
): Promise<string | undefined> {
  // Fetch active agents from Operator API
  try {
    const response = await fetch(
      `${vscode.workspace.getConfiguration('operator').get('apiUrl', 'http://localhost:7008')}/api/v1/agents/active`
    );
    if (!response.ok) {
      vscode.window.showErrorMessage('Failed to fetch active agents');
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
      vscode.window.showInformationMessage('No agents awaiting review');
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
  } catch (err) {
    vscode.window.showErrorMessage('Failed to fetch agents');
    return undefined;
  }
}

/**
 * Command: Sync a specific kanban collection
 */
async function syncKanbanCollectionCommand(item?: StatusItem): Promise<void> {
  const provider = item?.provider;
  const projectKey = item?.projectKey;

  if (!provider || !projectKey) {
    vscode.window.showWarningMessage('No collection selected for sync.');
    return;
  }

  const apiClient = new OperatorApiClient();

  try {
    await apiClient.health();
  } catch {
    vscode.window.showErrorMessage(
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
      vscode.window.showWarningMessage(`${message}, ${result.errors.length} errors`);
    } else {
      vscode.window.showInformationMessage(message);
    }
    await refreshAllProviders();
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to sync collection: ${msg}`);
  }
}

/**
 * Command: Add a Jira project to an existing workspace
 */
async function addJiraProjectCommand(workspaceKey?: string): Promise<void> {
  await addJiraProject(extensionContext, workspaceKey);
  await refreshAllProviders();
}

/**
 * Command: Add a Linear team to an existing workspace
 */
async function addLinearTeamCommand(workspaceKey?: string): Promise<void> {
  await addLinearTeam(extensionContext, workspaceKey);
  await refreshAllProviders();
}

/**
 * Command: Reveal .tickets directory in the OS file explorer
 */
async function revealTicketsDirCommand(): Promise<void> {
  if (!currentTicketsDir) {
    vscode.window.showWarningMessage('No .tickets directory found.');
    return;
  }

  const uri = vscode.Uri.file(currentTicketsDir);
  await vscode.commands.executeCommand('revealFileInOS', uri);
}

/**
 * Extension deactivation
 */
export function deactivate(): void {
  webhookServer?.stop();
  terminalManager?.dispose();
}
