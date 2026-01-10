"use strict";
/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Displays Operator connection status and session information.
 * Checks for vscode-session.json to determine if Operator is running.
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
/**
 * TreeDataProvider for status information
 */
class StatusTreeProvider {
    _onDidChangeTreeData = new vscode.EventEmitter();
    onDidChangeTreeData = this._onDidChangeTreeData.event;
    status = { running: false };
    ticketsDir;
    async setTicketsDir(dir) {
        this.ticketsDir = dir;
        await this.refresh();
    }
    async refresh() {
        if (!this.ticketsDir) {
            this.status = { running: false };
            this._onDidChangeTreeData.fire(undefined);
            return;
        }
        // Check for vscode-session.json in .tickets/operator/
        const sessionFile = path.join(this.ticketsDir, 'operator', 'vscode-session.json');
        try {
            const content = await fs.readFile(sessionFile, 'utf-8');
            const session = JSON.parse(content);
            // Session file exists - server is running
            this.status = {
                running: true,
                version: session.version,
                port: session.port,
                workspace: session.workspace,
                sessionFile,
            };
        }
        catch {
            this.status = { running: false };
        }
        this._onDidChangeTreeData.fire(undefined);
    }
    getTreeItem(element) {
        return element;
    }
    getChildren() {
        const items = [];
        if (this.status.running) {
            items.push(new StatusItem('Status', 'Connected', 'pass', 'Webhook server is running'));
            if (this.status.version) {
                items.push(new StatusItem('Version', this.status.version, 'versions'));
            }
            if (this.status.port) {
                items.push(new StatusItem('Port', this.status.port.toString(), 'plug'));
            }
        }
        else {
            items.push(new StatusItem('Status', 'Disconnected', 'error', 'Webhook server not running'));
        }
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