"use strict";
/**
 * Launch manager for Operator VS Code extension
 *
 * Orchestrates ticket launching and relaunching, coordinating between
 * terminal management, ticket parsing, and command building.
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
exports.LaunchManager = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const ticket_parser_1 = require("./ticket-parser");
const launch_command_1 = require("./launch-command");
/**
 * Manages launching and relaunching tickets
 */
class LaunchManager {
    terminalManager;
    ticketsDir;
    constructor(terminalManager) {
        this.terminalManager = terminalManager;
    }
    /**
     * Set the tickets directory
     */
    setTicketsDir(dir) {
        this.ticketsDir = dir;
    }
    /**
     * Launch a ticket with options
     */
    async launchTicket(ticket, options) {
        // Parse ticket metadata
        const metadata = await (0, ticket_parser_1.parseTicketMetadata)(ticket.filePath);
        if (!metadata) {
            throw new Error(`Could not parse ticket metadata: ${ticket.filePath}`);
        }
        const terminalName = (0, launch_command_1.buildTerminalName)(ticket.id);
        // Check if terminal already exists
        if (this.terminalManager.exists(terminalName)) {
            const choice = await vscode.window.showWarningMessage(`Terminal '${terminalName}' already exists`, 'Focus Existing', 'Kill and Relaunch');
            if (choice === 'Focus Existing') {
                await this.terminalManager.focus(terminalName);
                return;
            }
            else if (choice === 'Kill and Relaunch') {
                await this.terminalManager.kill(terminalName);
            }
            else {
                return; // Cancelled
            }
        }
        // Get session ID for resume
        const sessionId = options.resumeSession ? (0, ticket_parser_1.getCurrentSessionId)(metadata) : undefined;
        // Determine working directory
        const workingDir = metadata.worktree_path || this.getProjectDir(ticket);
        // Build the command
        const ticketRelPath = path.relative(workingDir, ticket.filePath);
        const command = (0, launch_command_1.buildLaunchCommand)(ticketRelPath, metadata, options, sessionId);
        // Create terminal and send command
        await this.terminalManager.create({
            name: terminalName,
            workingDir,
        });
        await this.terminalManager.send(terminalName, command);
        await this.terminalManager.focus(terminalName);
        const resumeMsg = sessionId ? ' (resuming session)' : '';
        vscode.window.showInformationMessage(`Launched agent for ${ticket.id}${resumeMsg}`);
    }
    /**
     * Offer to relaunch when terminal not found
     */
    async offerRelaunch(ticket) {
        const metadata = await (0, ticket_parser_1.parseTicketMetadata)(ticket.filePath);
        const sessionId = metadata ? (0, ticket_parser_1.getCurrentSessionId)(metadata) : undefined;
        const options = ['Launch Fresh'];
        if (sessionId) {
            options.push('Resume Session');
        }
        options.push('Cancel');
        const choice = await vscode.window.showWarningMessage(`Terminal for '${ticket.id}' not found`, ...options);
        if (choice === 'Launch Fresh') {
            await this.launchTicket(ticket, {
                model: 'sonnet',
                yoloMode: false,
                resumeSession: false,
            });
        }
        else if (choice === 'Resume Session') {
            await this.launchTicket(ticket, {
                model: 'sonnet',
                yoloMode: false,
                resumeSession: true,
            });
        }
    }
    /**
     * Get project directory from ticket info
     */
    getProjectDir(ticket) {
        // Default to parent of .tickets directory
        if (this.ticketsDir) {
            return path.dirname(this.ticketsDir);
        }
        // Fall back to ticket's parent directory
        return path.dirname(path.dirname(ticket.filePath));
    }
}
exports.LaunchManager = LaunchManager;
//# sourceMappingURL=launch-manager.js.map