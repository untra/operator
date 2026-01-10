"use strict";
/**
 * HTTP webhook server for Operator communication
 *
 * Provides REST API endpoints for terminal management.
 * Listens on localhost only for security.
 * Supports dynamic port binding with session file registration.
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
exports.WebhookServer = void 0;
const http = __importStar(require("http"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs/promises"));
const vscode = __importStar(require("vscode"));
const VERSION = '0.1.10';
/**
 * HTTP server for operator <-> extension communication
 */
class WebhookServer {
    server = null;
    terminalManager;
    configuredPort;
    actualPort = 0;
    sessionFilePath = null;
    constructor(terminalManager) {
        this.terminalManager = terminalManager;
        this.configuredPort = vscode.workspace
            .getConfiguration('operator')
            .get('webhookPort', 7009);
    }
    /**
     * Start the webhook server with optional session file registration
     * @param ticketsDir Path to .tickets directory for session file
     */
    async start(ticketsDir) {
        // Try configured port first, fall back to port 0 (OS assigns)
        try {
            await this.tryListen(this.configuredPort);
        }
        catch (err) {
            if (err.code === 'EADDRINUSE') {
                console.log(`Port ${this.configuredPort} in use, requesting available port...`);
                await this.tryListen(0); // Let OS assign a port
            }
            else {
                throw err;
            }
        }
        // Write session file if tickets directory provided
        if (ticketsDir) {
            await this.writeSessionFile(ticketsDir);
        }
    }
    /**
     * Attempt to listen on a specific port
     */
    tryListen(port) {
        return new Promise((resolve, reject) => {
            this.server = http.createServer((req, res) => this.handleRequest(req, res));
            this.server.on('error', reject);
            this.server.listen(port, '127.0.0.1', () => {
                const addr = this.server.address();
                this.actualPort =
                    typeof addr === 'object' && addr ? addr.port : port;
                console.log(`Operator webhook server listening on port ${this.actualPort}`);
                resolve();
            });
        });
    }
    /**
     * Write session info file for Operator discovery
     */
    async writeSessionFile(ticketsDir) {
        const sessionInfo = {
            wrapper: 'vscode',
            port: this.actualPort,
            pid: process.pid,
            version: VERSION,
            started_at: new Date().toISOString(),
            workspace: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? process.cwd(),
        };
        const operatorDir = path.join(ticketsDir, 'operator');
        await fs.mkdir(operatorDir, { recursive: true });
        this.sessionFilePath = path.join(operatorDir, 'vscode-session.json');
        await fs.writeFile(this.sessionFilePath, JSON.stringify(sessionInfo, null, 2));
        console.log(`Session file written to ${this.sessionFilePath}`);
    }
    /**
     * Stop the webhook server and clean up session file
     */
    async stop() {
        // Clean up session file
        if (this.sessionFilePath) {
            try {
                await fs.unlink(this.sessionFilePath);
                console.log(`Session file removed: ${this.sessionFilePath}`);
            }
            catch {
                /* ignore - file may not exist */
            }
            this.sessionFilePath = null;
        }
        // Stop server
        return new Promise((resolve) => {
            if (this.server) {
                this.server.close(() => {
                    this.server = null;
                    this.actualPort = 0;
                    resolve();
                });
            }
            else {
                resolve();
            }
        });
    }
    /**
     * Check if server is running
     */
    isRunning() {
        return this.server !== null;
    }
    /**
     * Get the actual port number (may differ from configured if fallback used)
     */
    getPort() {
        return this.actualPort;
    }
    /**
     * Get the configured port preference
     */
    getConfiguredPort() {
        return this.configuredPort;
    }
    /**
     * Handle incoming HTTP requests
     */
    async handleRequest(req, res) {
        // CORS headers for local development
        res.setHeader('Access-Control-Allow-Origin', '*');
        res.setHeader('Access-Control-Allow-Methods', 'GET, POST, DELETE, OPTIONS');
        res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
        if (req.method === 'OPTIONS') {
            res.writeHead(200);
            res.end();
            return;
        }
        const url = new URL(req.url ?? '/', `http://localhost:${this.actualPort}`);
        const urlPath = url.pathname;
        try {
            // Health check
            if (urlPath === '/health' && req.method === 'GET') {
                const response = {
                    status: 'ok',
                    version: VERSION,
                    port: this.actualPort,
                };
                return this.sendJson(res, response);
            }
            // Create terminal
            if (urlPath === '/terminal/create' && req.method === 'POST') {
                const body = await this.parseBody(req);
                await this.terminalManager.create(body);
                const response = { success: true, name: body.name };
                return this.sendJson(res, response);
            }
            // Send command to terminal
            if (urlPath.startsWith('/terminal/') &&
                urlPath.endsWith('/send') &&
                req.method === 'POST') {
                const name = this.extractName(urlPath, '/terminal/', '/send');
                const body = await this.parseBody(req);
                await this.terminalManager.send(name, body.command);
                const response = { success: true };
                return this.sendJson(res, response);
            }
            // Show terminal (reveal without focus)
            if (urlPath.startsWith('/terminal/') &&
                urlPath.endsWith('/show') &&
                req.method === 'POST') {
                const name = this.extractName(urlPath, '/terminal/', '/show');
                await this.terminalManager.show(name);
                const response = { success: true };
                return this.sendJson(res, response);
            }
            // Focus terminal (reveal with focus)
            if (urlPath.startsWith('/terminal/') &&
                urlPath.endsWith('/focus') &&
                req.method === 'POST') {
                const name = this.extractName(urlPath, '/terminal/', '/focus');
                await this.terminalManager.focus(name);
                const response = { success: true };
                return this.sendJson(res, response);
            }
            // Kill terminal
            if (urlPath.startsWith('/terminal/') &&
                urlPath.endsWith('/kill') &&
                req.method === 'DELETE') {
                const name = this.extractName(urlPath, '/terminal/', '/kill');
                await this.terminalManager.kill(name);
                const response = { success: true };
                return this.sendJson(res, response);
            }
            // Check if terminal exists
            if (urlPath.startsWith('/terminal/') &&
                urlPath.endsWith('/exists') &&
                req.method === 'GET') {
                const name = this.extractName(urlPath, '/terminal/', '/exists');
                const response = {
                    exists: this.terminalManager.exists(name),
                };
                return this.sendJson(res, response);
            }
            // Get terminal activity
            if (urlPath.startsWith('/terminal/') &&
                urlPath.endsWith('/activity') &&
                req.method === 'GET') {
                const name = this.extractName(urlPath, '/terminal/', '/activity');
                const response = {
                    activity: this.terminalManager.getActivity(name),
                };
                return this.sendJson(res, response);
            }
            // List all terminals
            if (urlPath === '/terminal/list' && req.method === 'GET') {
                const response = {
                    terminals: this.terminalManager.list(),
                };
                return this.sendJson(res, response);
            }
            // 404 Not Found
            res.writeHead(404);
            res.end(JSON.stringify({ error: 'Not found' }));
        }
        catch (err) {
            const message = err instanceof Error ? err.message : 'Unknown error';
            res.writeHead(500);
            res.end(JSON.stringify({ error: message }));
        }
    }
    /**
     * Extract terminal name from path
     */
    extractName(urlPath, prefix, suffix) {
        return decodeURIComponent(urlPath.slice(prefix.length, -suffix.length));
    }
    /**
     * Parse JSON request body
     */
    parseBody(req) {
        return new Promise((resolve, reject) => {
            let body = '';
            req.on('data', (chunk) => (body += chunk));
            req.on('end', () => {
                try {
                    resolve(JSON.parse(body || '{}'));
                }
                catch {
                    reject(new Error('Invalid JSON'));
                }
            });
            req.on('error', reject);
        });
    }
    /**
     * Send JSON response
     */
    sendJson(res, data) {
        res.setHeader('Content-Type', 'application/json');
        res.writeHead(200);
        res.end(JSON.stringify(data));
    }
}
exports.WebhookServer = WebhookServer;
//# sourceMappingURL=webhook-server.js.map