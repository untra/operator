"use strict";
/**
 * Operator REST API client
 *
 * Provides methods to communicate with the Operator REST API
 * for launching tickets and checking health status.
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
exports.OperatorApiClient = void 0;
exports.discoverApiUrl = discoverApiUrl;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs/promises"));
const path = __importStar(require("path"));
/**
 * Discover Operator API URL from session file or configuration
 *
 * Checks in order:
 * 1. .tickets/operator/api-session.json (written by running Operator)
 * 2. VSCode configuration operator.apiUrl
 */
async function discoverApiUrl(ticketsDir) {
    // Try to read api-session.json from tickets directory
    if (ticketsDir) {
        const sessionFile = path.join(ticketsDir, 'operator', 'api-session.json');
        try {
            const content = await fs.readFile(sessionFile, 'utf-8');
            const session = JSON.parse(content);
            return `http://localhost:${session.port}`;
        }
        catch {
            // Session file doesn't exist or is invalid, fall through
        }
    }
    // Fall back to configured URL
    const config = vscode.workspace.getConfiguration('operator');
    return config.get('apiUrl', 'http://localhost:7008');
}
/**
 * Client for the Operator REST API
 */
class OperatorApiClient {
    baseUrl;
    constructor(baseUrl) {
        const config = vscode.workspace.getConfiguration('operator');
        this.baseUrl = baseUrl || config.get('apiUrl', 'http://localhost:7008');
    }
    /**
     * Check if the Operator API is available
     */
    async health() {
        const response = await fetch(`${this.baseUrl}/api/v1/health`);
        if (!response.ok) {
            throw new Error('Operator API not available');
        }
        return (await response.json());
    }
    /**
     * Launch a ticket via the Operator API
     *
     * Claims the ticket, sets up worktree if needed, and returns
     * the command to execute in a terminal.
     */
    async launchTicket(ticketId, options) {
        const response = await fetch(`${this.baseUrl}/api/v1/tickets/${encodeURIComponent(ticketId)}/launch`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                provider: options.provider,
                model: options.model,
                yolo_mode: options.yolo_mode ?? false,
                wrapper: options.wrapper,
            }),
        });
        if (!response.ok) {
            const error = (await response.json().catch(() => ({
                error: 'unknown',
                message: `HTTP ${response.status}: ${response.statusText}`,
            })));
            throw new Error(error.message);
        }
        return (await response.json());
    }
}
exports.OperatorApiClient = OperatorApiClient;
//# sourceMappingURL=api-client.js.map