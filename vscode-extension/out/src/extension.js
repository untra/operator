"use strict";
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
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs/promises"));
const terminal_manager_1 = require("./terminal-manager");
const webhook_server_1 = require("./webhook-server");
const ticket_provider_1 = require("./ticket-provider");
const status_provider_1 = require("./status-provider");
const launch_manager_1 = require("./launch-manager");
const launch_dialog_1 = require("./launch-dialog");
const ticket_parser_1 = require("./ticket-parser");
const api_client_1 = require("./api-client");
let terminalManager;
let webhookServer;
let statusBarItem;
let launchManager;
// TreeView providers
let statusProvider;
let inProgressProvider;
let queueProvider;
let completedProvider;
// Current tickets directory
let currentTicketsDir;
/**
 * Extension activation
 */
async function activate(context) {
    terminalManager = new terminal_manager_1.TerminalManager();
    webhookServer = new webhook_server_1.WebhookServer(terminalManager);
    launchManager = new launch_manager_1.LaunchManager(terminalManager);
    // Create status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'operator.showStatus';
    context.subscriptions.push(statusBarItem);
    // Create TreeView providers
    statusProvider = new status_provider_1.StatusTreeProvider();
    inProgressProvider = new ticket_provider_1.TicketTreeProvider('in-progress', terminalManager);
    queueProvider = new ticket_provider_1.TicketTreeProvider('queue');
    completedProvider = new ticket_provider_1.TicketTreeProvider('completed');
    // Register TreeViews
    context.subscriptions.push(vscode.window.registerTreeDataProvider('operator-status', statusProvider), vscode.window.registerTreeDataProvider('operator-in-progress', inProgressProvider), vscode.window.registerTreeDataProvider('operator-queue', queueProvider), vscode.window.registerTreeDataProvider('operator-completed', completedProvider));
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('operator.startServer', startServer), vscode.commands.registerCommand('operator.stopServer', stopServer), vscode.commands.registerCommand('operator.showStatus', showStatus), vscode.commands.registerCommand('operator.refreshTickets', refreshAllProviders), vscode.commands.registerCommand('operator.focusTicket', focusTicketTerminal), vscode.commands.registerCommand('operator.openTicket', openTicketFile), vscode.commands.registerCommand('operator.launchTicket', launchTicketCommand), vscode.commands.registerCommand('operator.launchTicketWithOptions', launchTicketWithOptionsCommand), vscode.commands.registerCommand('operator.relaunchTicket', relaunchTicketCommand), vscode.commands.registerCommand('operator.launchTicketFromEditor', launchTicketFromEditorCommand), vscode.commands.registerCommand('operator.launchTicketFromEditorWithOptions', launchTicketFromEditorWithOptionsCommand), vscode.commands.registerCommand('operator.downloadOperator', downloadOperatorCommand));
    // Find tickets directory (check parent first, then workspace)
    currentTicketsDir = await findParentTicketsDir();
    await setTicketsDir(currentTicketsDir);
    // Set up file watcher if tickets directory exists
    if (currentTicketsDir) {
        const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(currentTicketsDir, '**/*.md'));
        watcher.onDidChange(() => refreshAllProviders());
        watcher.onDidCreate(() => refreshAllProviders());
        watcher.onDidDelete(() => refreshAllProviders());
        context.subscriptions.push(watcher);
    }
    // Auto-start if configured
    const autoStart = vscode.workspace
        .getConfiguration('operator')
        .get('autoStart', true);
    if (autoStart) {
        await startServer();
    }
    updateStatusBar();
}
/**
 * Find .tickets directory - check parent directory first, then workspace
 */
async function findParentTicketsDir() {
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
    }
    catch {
        // Parent doesn't have .tickets, check workspace
    }
    // Fall back to configured tickets directory in workspace
    const configuredDir = vscode.workspace
        .getConfiguration('operator')
        .get('ticketsDir', '.tickets');
    const ticketsPath = path.isAbsolute(configuredDir)
        ? configuredDir
        : path.join(workspaceFolder.uri.fsPath, configuredDir);
    try {
        await fs.access(ticketsPath);
        return ticketsPath;
    }
    catch {
        // .tickets directory doesn't exist yet - create it
        try {
            await fs.mkdir(ticketsPath, { recursive: true });
            return ticketsPath;
        }
        catch {
            return undefined;
        }
    }
}
/**
 * Find the .tickets directory in the workspace (for webhook session file)
 */
async function findTicketsDir() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        return undefined;
    }
    // Check configured tickets directory
    const configuredDir = vscode.workspace
        .getConfiguration('operator')
        .get('ticketsDir', '.tickets');
    const ticketsPath = path.isAbsolute(configuredDir)
        ? configuredDir
        : path.join(workspaceFolder.uri.fsPath, configuredDir);
    try {
        await fs.access(ticketsPath);
        return ticketsPath;
    }
    catch {
        // .tickets directory doesn't exist yet - create it
        try {
            await fs.mkdir(ticketsPath, { recursive: true });
            return ticketsPath;
        }
        catch {
            return undefined;
        }
    }
}
/**
 * Set tickets directory for all providers
 */
async function setTicketsDir(dir) {
    await statusProvider.setTicketsDir(dir);
    await inProgressProvider.setTicketsDir(dir);
    await queueProvider.setTicketsDir(dir);
    await completedProvider.setTicketsDir(dir);
    launchManager.setTicketsDir(dir);
}
/**
 * Refresh all TreeView providers
 */
async function refreshAllProviders() {
    await statusProvider.refresh();
    await inProgressProvider.refresh();
    await queueProvider.refresh();
    await completedProvider.refresh();
}
/**
 * Focus a terminal by name, or offer relaunch if not found
 */
async function focusTicketTerminal(terminalName, ticket) {
    if (terminalManager.exists(terminalName)) {
        await terminalManager.focus(terminalName);
    }
    else if (ticket) {
        await launchManager.offerRelaunch(ticket);
    }
    else {
        vscode.window.showWarningMessage(`Terminal '${terminalName}' not found`);
    }
}
/**
 * Open a ticket file
 */
function openTicketFile(filePath) {
    vscode.workspace.openTextDocument(filePath).then((doc) => {
        vscode.window.showTextDocument(doc);
    });
}
/**
 * Start the webhook server
 */
async function startServer() {
    if (webhookServer.isRunning()) {
        vscode.window.showInformationMessage('Operator webhook server already running');
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
            vscode.window.showInformationMessage(`Operator webhook server started on port ${port} (configured port ${configuredPort} was in use)`);
        }
        else {
            vscode.window.showInformationMessage(`Operator webhook server started on port ${port}`);
        }
        updateStatusBar();
    }
    catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error';
        vscode.window.showErrorMessage(`Failed to start webhook server: ${msg}`);
    }
}
/**
 * Stop the webhook server
 */
async function stopServer() {
    await webhookServer.stop();
    vscode.window.showInformationMessage('Operator webhook server stopped');
    updateStatusBar();
}
/**
 * Show server status
 */
function showStatus() {
    const running = webhookServer.isRunning();
    const port = webhookServer.getPort();
    const configuredPort = webhookServer.getConfiguredPort();
    const terminals = terminalManager.list();
    let message;
    if (running) {
        if (port !== configuredPort) {
            message = `Operator server running on port ${port} (fallback from ${configuredPort})\nManaged terminals: ${terminals.length}`;
        }
        else {
            message = `Operator server running on port ${port}\nManaged terminals: ${terminals.length}`;
        }
    }
    else {
        message = 'Operator server stopped';
    }
    vscode.window.showInformationMessage(message);
}
/**
 * Update status bar appearance
 */
function updateStatusBar() {
    if (webhookServer.isRunning()) {
        const port = webhookServer.getPort();
        statusBarItem.text = `$(terminal) Operator :${port}`;
        statusBarItem.tooltip = `Operator webhook server running on port ${port}`;
        statusBarItem.backgroundColor = undefined;
    }
    else {
        statusBarItem.text = '$(terminal) Operator (off)';
        statusBarItem.tooltip = 'Operator webhook server stopped';
        statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
    }
    statusBarItem.show();
}
/**
 * Command: Launch ticket (quick, uses defaults)
 */
async function launchTicketCommand() {
    const tickets = queueProvider.getTickets();
    if (tickets.length === 0) {
        vscode.window.showInformationMessage('No tickets in queue');
        return;
    }
    const ticket = await (0, launch_dialog_1.showTicketPicker)(tickets);
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
 */
async function launchTicketWithOptionsCommand() {
    const tickets = queueProvider.getTickets();
    if (tickets.length === 0) {
        vscode.window.showInformationMessage('No tickets in queue');
        return;
    }
    const ticket = await (0, launch_dialog_1.showTicketPicker)(tickets);
    if (!ticket) {
        return;
    }
    const metadata = await (0, ticket_parser_1.parseTicketMetadata)(ticket.filePath);
    const hasSession = metadata ? !!(0, ticket_parser_1.getCurrentSessionId)(metadata) : false;
    const options = await (0, launch_dialog_1.showLaunchOptionsDialog)(ticket, hasSession);
    if (!options) {
        return;
    }
    await launchManager.launchTicket(ticket, options);
}
/**
 * Command: Relaunch in-progress ticket
 */
async function relaunchTicketCommand(ticket) {
    await launchManager.offerRelaunch(ticket);
}
/**
 * Check if a file path is a ticket file in the .tickets directory
 */
function isTicketFile(filePath) {
    const normalized = filePath.replace(/\\/g, '/');
    return ((normalized.includes('.tickets/queue/') ||
        normalized.includes('.tickets/in-progress/')) &&
        normalized.endsWith('.md'));
}
/**
 * Command: Launch ticket from the active editor
 *
 * Uses the Operator API to properly claim the ticket and track state.
 */
async function launchTicketFromEditorCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
    }
    const filePath = editor.document.uri.fsPath;
    if (!isTicketFile(filePath)) {
        vscode.window.showWarningMessage('Current file is not a ticket in .tickets/ directory');
        return;
    }
    const metadata = await (0, ticket_parser_1.parseTicketMetadata)(filePath);
    if (!metadata?.id) {
        vscode.window.showErrorMessage('Could not parse ticket ID from file');
        return;
    }
    const apiClient = new api_client_1.OperatorApiClient();
    // Check if Operator API is running
    try {
        await apiClient.health();
    }
    catch {
        vscode.window.showErrorMessage('Operator API not running. Start operator first.');
        return;
    }
    // Launch via Operator API
    try {
        const response = await apiClient.launchTicket(metadata.id, {
            wrapper: 'vscode',
            model: 'sonnet',
        });
        // Create terminal and execute command
        await terminalManager.create({
            name: response.terminal_name,
            workingDir: response.working_directory,
        });
        await terminalManager.send(response.terminal_name, response.command);
        await terminalManager.focus(response.terminal_name);
        const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
        vscode.window.showInformationMessage(`Launched agent for ${response.ticket_id}${worktreeMsg}`);
        // Refresh ticket providers to reflect the change
        await refreshAllProviders();
    }
    catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error';
        vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
    }
}
/**
 * Command: Launch ticket from editor with options dialog
 */
async function launchTicketFromEditorWithOptionsCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
    }
    const filePath = editor.document.uri.fsPath;
    if (!isTicketFile(filePath)) {
        vscode.window.showWarningMessage('Current file is not a ticket in .tickets/ directory');
        return;
    }
    const metadata = await (0, ticket_parser_1.parseTicketMetadata)(filePath);
    if (!metadata?.id) {
        vscode.window.showErrorMessage('Could not parse ticket ID from file');
        return;
    }
    // Create a minimal TicketInfo for the dialog
    const ticketType = metadata.id.split('-')[0];
    const ticketStatus = (metadata.status === 'in-progress' || metadata.status === 'completed')
        ? metadata.status
        : 'queue';
    const ticketInfo = {
        id: metadata.id,
        type: ticketType || 'TASK',
        title: 'Ticket from editor',
        status: ticketStatus,
        filePath: filePath,
    };
    const hasSession = !!(0, ticket_parser_1.getCurrentSessionId)(metadata);
    const options = await (0, launch_dialog_1.showLaunchOptionsDialog)(ticketInfo, hasSession);
    if (!options) {
        return;
    }
    const apiClient = new api_client_1.OperatorApiClient();
    // Check if Operator API is running
    try {
        await apiClient.health();
    }
    catch {
        vscode.window.showErrorMessage('Operator API not running. Start operator first.');
        return;
    }
    // Launch via Operator API
    try {
        const response = await apiClient.launchTicket(metadata.id, {
            wrapper: 'vscode',
            model: options.model,
            yolo_mode: options.yoloMode,
        });
        // Create terminal and execute command
        await terminalManager.create({
            name: response.terminal_name,
            workingDir: response.working_directory,
        });
        await terminalManager.send(response.terminal_name, response.command);
        await terminalManager.focus(response.terminal_name);
        const worktreeMsg = response.worktree_created ? ' (worktree created)' : '';
        vscode.window.showInformationMessage(`Launched agent for ${response.ticket_id}${worktreeMsg}`);
        // Refresh ticket providers to reflect the change
        await refreshAllProviders();
    }
    catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error';
        vscode.window.showErrorMessage(`Failed to launch: ${msg}`);
    }
}
/**
 * Command: Open Operator download page
 */
function downloadOperatorCommand() {
    vscode.env.openExternal(vscode.Uri.parse('https://operator.untra.io/downloads/'));
}
/**
 * Extension deactivation
 */
function deactivate() {
    webhookServer?.stop();
    terminalManager?.dispose();
}
//# sourceMappingURL=extension.js.map