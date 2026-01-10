"use strict";
/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Displays Operator connection status and session information.
 * Checks for vscode-session.json (webhook) and api-session.json (API).
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
exports.StatusTreeProvider = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs/promises"));
const api_client_1 = require("./api-client");
/**
 * TreeDataProvider for status information
 */
class StatusTreeProvider {
    _onDidChangeTreeData = new vscode.EventEmitter();
    onDidChangeTreeData = this._onDidChangeTreeData.event;
    webhookStatus = { running: false };
    apiStatus = { connected: false };
    ticketsDir;
    async setTicketsDir(dir) {
        this.ticketsDir = dir;
        await this.refresh();
    }
    async refresh() {
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
    async checkWebhookStatus() {
        if (!this.ticketsDir) {
            this.webhookStatus = { running: false };
            return;
        }
        const webhookSessionFile = path.join(this.ticketsDir, 'operator', 'vscode-session.json');
        try {
            const content = await fs.readFile(webhookSessionFile, 'utf-8');
            const session = JSON.parse(content);
            this.webhookStatus = {
                running: true,
                version: session.version,
                port: session.port,
                workspace: session.workspace,
                sessionFile: webhookSessionFile,
            };
        }
        catch {
            this.webhookStatus = { running: false };
        }
    }
    /**
     * Check API status - tries session file first, then falls back to configured URL
     * Works even without ticketsDir by using the configured apiUrl
     */
    async checkApiStatus() {
        // Try session file first if ticketsDir exists
        if (this.ticketsDir) {
            const apiSessionFile = path.join(this.ticketsDir, 'operator', 'api-session.json');
            try {
                const content = await fs.readFile(apiSessionFile, 'utf-8');
                const session = JSON.parse(content);
                const apiUrl = `http://localhost:${session.port}`;
                if (await this.tryHealthCheck(apiUrl, session.version)) {
                    return;
                }
            }
            catch {
                // Session file doesn't exist or is invalid, fall through to configured URL
            }
        }
        // Always try configured URL as fallback (works without ticketsDir)
        const apiUrl = await (0, api_client_1.discoverApiUrl)(this.ticketsDir);
        await this.tryHealthCheck(apiUrl);
    }
    /**
     * Attempt a health check against the given API URL
     * Returns true if successful, false otherwise
     */
    async tryHealthCheck(apiUrl, sessionVersion) {
        try {
            const response = await fetch(`${apiUrl}/api/v1/health`);
            if (response.ok) {
                const health = await response.json();
                const port = new URL(apiUrl).port;
                this.apiStatus = {
                    connected: true,
                    version: health.version || sessionVersion,
                    port: port ? parseInt(port, 10) : 7008,
                    url: apiUrl,
                };
                return true;
            }
        }
        catch {
            // Health check failed
        }
        this.apiStatus = { connected: false };
        return false;
    }
    getTreeItem(element) {
        return element;
    }
    getChildren() {
        const items = [];
        // REST API status
        if (this.apiStatus.connected) {
            items.push(new StatusItem('API', 'Connected', 'pass', `Operator REST API at ${this.apiStatus.url}`));
            if (this.apiStatus.version) {
                items.push(new StatusItem('API Version', this.apiStatus.version, 'versions'));
            }
            if (this.apiStatus.port) {
                items.push(new StatusItem('API Port', this.apiStatus.port.toString(), 'plug'));
            }
        }
        else {
            items.push(new StatusItem('API', 'Disconnected', 'error', 'Operator REST API not running. Use "Operator: Download Operator" command if not installed.'));
        }
        // Webhook server status
        if (this.webhookStatus.running) {
            items.push(new StatusItem('Webhook', 'Running', 'pass', 'Local webhook server for terminal management'));
            if (this.webhookStatus.port) {
                items.push(new StatusItem('Webhook Port', this.webhookStatus.port.toString(), 'plug'));
            }
        }
        else {
            items.push(new StatusItem('Webhook', 'Stopped', 'circle-slash', 'Local webhook server not running'));
        }
        // Tickets directory
        if (this.ticketsDir) {
            items.push(new StatusItem('Tickets', path.basename(this.ticketsDir), 'folder'));
        }
        else {
            items.push(new StatusItem('Tickets', 'Not found', 'folder', 'No .tickets directory found'));
        }
        return items;
    }
}
exports.StatusTreeProvider = StatusTreeProvider;
/**
 * TreeItem for status display
 */
class StatusItem extends vscode.TreeItem {
    constructor(label, value, icon, tooltip) {
        super(label, vscode.TreeItemCollapsibleState.None);
        this.description = value;
        this.tooltip = tooltip || `${label}: ${value}`;
        this.iconPath = new vscode.ThemeIcon(icon);
    }
}
//# sourceMappingURL=status-provider.js.map