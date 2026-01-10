"use strict";
/**
 * Terminal lifecycle management for Operator VS Code extension
 *
 * Manages terminal creation, disposal, and activity tracking.
 * Terminals are styled by ticket type with colors and icons.
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
exports.TerminalManager = void 0;
const vscode = __importStar(require("vscode"));
/**
 * Manages operator terminals with activity detection and styling
 */
class TerminalManager {
    terminals = new Map();
    activityState = new Map();
    createdAt = new Map();
    disposables = [];
    constructor() {
        // Track shell execution for activity detection
        this.disposables.push(vscode.window.onDidStartTerminalShellExecution((e) => {
            const name = this.findTerminalName(e.terminal);
            if (name && this.terminals.has(name)) {
                this.activityState.set(name, 'running');
            }
        }), vscode.window.onDidEndTerminalShellExecution((e) => {
            const name = this.findTerminalName(e.terminal);
            if (name && this.terminals.has(name)) {
                this.activityState.set(name, 'idle');
            }
        }), vscode.window.onDidCloseTerminal((t) => {
            const name = this.findTerminalName(t);
            if (name) {
                this.terminals.delete(name);
                this.activityState.delete(name);
                this.createdAt.delete(name);
            }
        }));
    }
    /**
     * Create a new terminal with Operator styling
     */
    async create(options) {
        const { name, workingDir, env } = options;
        // Dispose existing terminal with same name if present
        if (this.terminals.has(name)) {
            await this.kill(name);
        }
        // Use ticket-specific colors and icons
        const terminalOptions = {
            name,
            cwd: workingDir,
            color: this.getColorForName(name),
            iconPath: this.getIconForName(name),
            env: {
                ...env,
                OPERATOR_SESSION: name,
            },
        };
        const terminal = vscode.window.createTerminal(terminalOptions);
        this.terminals.set(name, terminal);
        this.activityState.set(name, 'idle');
        this.createdAt.set(name, Date.now());
        return terminal;
    }
    /**
     * Send a command to a terminal
     */
    async send(name, command) {
        const terminal = this.terminals.get(name);
        if (!terminal) {
            throw new Error(`Terminal '${name}' not found`);
        }
        terminal.sendText(command);
    }
    /**
     * Reveal a terminal without taking focus (show in panel)
     */
    async show(name) {
        const terminal = this.terminals.get(name);
        if (!terminal) {
            throw new Error(`Terminal '${name}' not found`);
        }
        terminal.show(true); // preserveFocus = true
    }
    /**
     * Focus a terminal (takes keyboard focus)
     */
    async focus(name) {
        const terminal = this.terminals.get(name);
        if (!terminal) {
            throw new Error(`Terminal '${name}' not found`);
        }
        terminal.show(false); // preserveFocus = false
    }
    /**
     * Kill/dispose a terminal
     */
    async kill(name) {
        const terminal = this.terminals.get(name);
        if (terminal) {
            terminal.dispose();
            this.terminals.delete(name);
            this.activityState.delete(name);
            this.createdAt.delete(name);
        }
    }
    /**
     * Check if terminal exists
     */
    exists(name) {
        return this.terminals.has(name);
    }
    /**
     * Get activity state
     */
    getActivity(name) {
        return this.activityState.get(name) ?? 'unknown';
    }
    /**
     * List all managed terminals
     */
    list() {
        const result = [];
        for (const [name] of this.terminals) {
            result.push({
                name,
                pid: undefined, // processId is a Thenable, would need async handling
                activity: this.activityState.get(name) ?? 'unknown',
                createdAt: this.createdAt.get(name) ?? Date.now(),
            });
        }
        return result;
    }
    /**
     * Color scheme based on ticket type
     */
    getColorForName(name) {
        // op-FEAT-123 -> cyan, op-FIX-123 -> red, etc.
        if (name.includes('FEAT')) {
            return new vscode.ThemeColor('terminal.ansiCyan');
        }
        if (name.includes('FIX')) {
            return new vscode.ThemeColor('terminal.ansiRed');
        }
        if (name.includes('TASK')) {
            return new vscode.ThemeColor('terminal.ansiGreen');
        }
        if (name.includes('SPIKE')) {
            return new vscode.ThemeColor('terminal.ansiMagenta');
        }
        if (name.includes('INV')) {
            return new vscode.ThemeColor('terminal.ansiYellow');
        }
        return new vscode.ThemeColor('terminal.ansiWhite');
    }
    /**
     * Icon based on ticket type
     */
    getIconForName(name) {
        if (name.includes('FEAT')) {
            return new vscode.ThemeIcon('sparkle');
        }
        if (name.includes('FIX')) {
            return new vscode.ThemeIcon('wrench');
        }
        if (name.includes('TASK')) {
            return new vscode.ThemeIcon('tasklist');
        }
        if (name.includes('SPIKE')) {
            return new vscode.ThemeIcon('beaker');
        }
        if (name.includes('INV')) {
            return new vscode.ThemeIcon('search');
        }
        return new vscode.ThemeIcon('terminal');
    }
    /**
     * Find terminal name by terminal instance
     */
    findTerminalName(terminal) {
        for (const [name, t] of this.terminals) {
            if (t === terminal) {
                return name;
            }
        }
        return undefined;
    }
    /**
     * Dispose all resources
     */
    dispose() {
        this.disposables.forEach((d) => d.dispose());
        this.terminals.forEach((t) => t.dispose());
        this.terminals.clear();
        this.activityState.clear();
        this.createdAt.clear();
    }
}
exports.TerminalManager = TerminalManager;
//# sourceMappingURL=terminal-manager.js.map