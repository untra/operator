"use strict";
/**
 * Ticket TreeDataProvider for Operator VS Code extension
 *
 * Displays tickets from .tickets directory in sidebar TreeViews.
 * Supports in-progress, queue, and completed ticket states.
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
exports.TicketItem = exports.TicketTreeProvider = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs/promises"));
/**
 * TreeDataProvider for ticket lists
 */
class TicketTreeProvider {
    status;
    terminalManager;
    _onDidChangeTreeData = new vscode.EventEmitter();
    onDidChangeTreeData = this._onDidChangeTreeData.event;
    tickets = [];
    ticketsDir;
    constructor(status, terminalManager) {
        this.status = status;
        this.terminalManager = terminalManager;
    }
    async setTicketsDir(dir) {
        this.ticketsDir = dir;
        await this.refresh();
    }
    async refresh() {
        if (!this.ticketsDir) {
            this.tickets = [];
            this._onDidChangeTreeData.fire(undefined);
            return;
        }
        const subDir = path.join(this.ticketsDir, this.status);
        try {
            const files = await fs.readdir(subDir);
            const mdFiles = files.filter((f) => f.endsWith('.md'));
            this.tickets = await Promise.all(mdFiles.map(async (file) => {
                const filePath = path.join(subDir, file);
                const content = await fs.readFile(filePath, 'utf-8');
                return this.parseTicket(file, filePath, content);
            }));
            // Sort by ticket ID
            this.tickets.sort((a, b) => a.id.localeCompare(b.id));
        }
        catch {
            this.tickets = [];
        }
        this._onDidChangeTreeData.fire(undefined);
    }
    parseTicket(filename, filePath, content) {
        // Parse ticket ID and type from filename: FEAT-123-title.md or FEAT-123.md
        const match = filename.match(/^(FEAT|FIX|TASK|SPIKE|INV)-(\d+)/i);
        const type = (match?.[1]?.toUpperCase() || 'TASK');
        const id = match ? `${match[1].toUpperCase()}-${match[2]}` : filename.replace('.md', '');
        // Parse title from first heading or frontmatter
        const titleMatch = content.match(/^#\s+(.+)$/m) || content.match(/^title:\s*(.+)$/m);
        const title = titleMatch?.[1]?.trim() || id;
        return {
            id,
            title,
            type,
            status: this.status,
            filePath,
            terminalName: this.status === 'in-progress' ? `op-${id}` : undefined,
        };
    }
    getTreeItem(element) {
        return element;
    }
    getChildren() {
        return this.tickets.map((ticket) => new TicketItem(ticket, this.terminalManager));
    }
    /**
     * Get all tickets (for launch command)
     */
    getTickets() {
        return [...this.tickets];
    }
}
exports.TicketTreeProvider = TicketTreeProvider;
/**
 * TreeItem representing a single ticket
 */
class TicketItem extends vscode.TreeItem {
    ticket;
    terminalManager;
    constructor(ticket, terminalManager) {
        super(ticket.title, vscode.TreeItemCollapsibleState.None);
        this.ticket = ticket;
        this.terminalManager = terminalManager;
        this.id = ticket.id;
        this.tooltip = `${ticket.id}: ${ticket.title}`;
        this.description = ticket.id;
        // Set icon based on ticket type
        this.iconPath = this.getIconForType(ticket.type);
        // Set context for menu commands
        this.contextValue = ticket.status;
        // Make in-progress items clickable to focus terminal (pass ticket for relaunch)
        if (ticket.status === 'in-progress' && ticket.terminalName) {
            this.command = {
                command: 'operator.focusTicket',
                title: 'Focus Terminal',
                arguments: [ticket.terminalName, ticket],
            };
        }
        else {
            // Queue and completed items open the file
            this.command = {
                command: 'operator.openTicket',
                title: 'Open Ticket',
                arguments: [ticket.filePath],
            };
        }
    }
    getIconForType(type) {
        switch (type) {
            case 'FEAT':
                return new vscode.ThemeIcon('sparkle', new vscode.ThemeColor('terminal.ansiCyan'));
            case 'FIX':
                return new vscode.ThemeIcon('wrench', new vscode.ThemeColor('terminal.ansiRed'));
            case 'TASK':
                return new vscode.ThemeIcon('tasklist', new vscode.ThemeColor('terminal.ansiGreen'));
            case 'SPIKE':
                return new vscode.ThemeIcon('beaker', new vscode.ThemeColor('terminal.ansiMagenta'));
            case 'INV':
                return new vscode.ThemeIcon('search', new vscode.ThemeColor('terminal.ansiYellow'));
            default:
                return new vscode.ThemeIcon('file');
        }
    }
}
exports.TicketItem = TicketItem;
//# sourceMappingURL=ticket-provider.js.map