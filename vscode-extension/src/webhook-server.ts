/**
 * HTTP webhook server for Operator communication
 *
 * Provides REST API endpoints for terminal management.
 * Listens on localhost only for security.
 * Supports dynamic port binding with session file registration.
 */

import * as http from "node:http";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { randomBytes, randomUUID, timingSafeEqual } from "node:crypto";
import * as os from "node:os";
import * as path from "node:path";
import * as fs from "node:fs/promises";
import * as vscode from "vscode";
import { PROTECT_REGISTRATION_DIRECTORY_SCRIPT } from "./generated/runtime-scripts";
import type { TerminalManager } from "./terminal-manager";
import type {
  TerminalCreateOptions,
  SendCommandRequest,
  HealthResponse,
  SuccessResponse,
  ExistsResponse,
  ActivityResponse,
  ListResponse,
  SessionInfo,
} from "./types";

const DEFAULT_WEBHOOK_PORT = 7009;
const MAX_TCP_PORT = 65535;
const TOKEN_BYTES = 32;
const PRIVATE_DIRECTORY = 0o700;
const PRIVATE_FILE = 0o600;

const VERSION = "0.2.17";

async function protectWindowsDirectory(dir: string): Promise<void> {
  const system = process.env.SystemRoot;
  if (!system) {
    throw new Error("Missing Windows system directory");
  }
  await promisify(execFile)(
    path.join(system, "System32", "WindowsPowerShell", "v1.0", "powershell.exe"),
    ["-NoProfile", "-NonInteractive", "-Command", PROTECT_REGISTRATION_DIRECTORY_SCRIPT],
    { env: { ...process.env, OPERATOR_TERMINAL_REGISTRATIONS: dir }, windowsHide: true },
  );
}

/**
 * HTTP server for operator <-> extension communication
 */
export class WebhookServer {
  private server: http.Server | null = null;
  private terminalManager: TerminalManager;
  private configuredPort: number;
  private actualPort = 0;
  private sessionFilePath: string | null = null;

  private bearer?: string;

  constructor(
    terminalManager: TerminalManager,
    private readonly registrations = path.join(os.homedir(), ".operator", "vscode-sessions"),
  ) {
    this.terminalManager = terminalManager;
    const port = vscode.workspace
      .getConfiguration("operator")
      .get("webhookPort", DEFAULT_WEBHOOK_PORT);
    this.configuredPort =
      Number.isInteger(port) && port > 0 && port <= MAX_TCP_PORT ? port : DEFAULT_WEBHOOK_PORT;
  }

  async start(ticketsDir?: string): Promise<void> {
    if (this.server) {
      return;
    }
    this.bearer = randomBytes(TOKEN_BYTES).toString("base64url");
    try {
      await this.tryListen(this.configuredPort);
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code === "EADDRINUSE") {
        console.log(`Port ${this.configuredPort} in use, requesting available port...`);
        await this.tryListen(0); // Let OS assign a port
      } else {
        throw err;
      }
    }

    try {
      await this.writeSessionFile(ticketsDir);
    } catch (error) {
      await this.stop();
      throw error;
    }
  }

  /**
   * Attempt to listen on a specific port
   */
  private tryListen(port: number): Promise<void> {
    return new Promise((resolve, reject) => {
      const server = http.createServer((req, res) => {
        void this.handleRequest(req, res);
      });

      server.once("error", reject);

      server.listen(port, "127.0.0.1", () => {
        this.server = server;
        const addr = server.address();
        this.actualPort = typeof addr === "object" && addr ? addr.port : port;
        console.log(`Operator webhook server listening on port ${this.actualPort}`);
        resolve();
      });
    });
  }

  private async writeSessionFile(ticketsDir?: string): Promise<void> {
    if (this.sessionFilePath || !this.bearer) {
      return;
    }
    const sessionInfo: SessionInfo = {
      bearer: this.bearer,
      wrapper: "vscode",
      port: this.actualPort,
      pid: process.pid,
      version: VERSION,
      startedAt: new Date().toISOString(),
      workspace: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? ticketsDir ?? process.cwd(),
    };

    await fs.mkdir(this.registrations, { recursive: true, mode: PRIVATE_DIRECTORY });
    if (process.platform === "win32") {
      await protectWindowsDirectory(this.registrations);
    }
    const info = await fs.lstat(this.registrations);
    const expectedHome = path.join(os.homedir(), ".operator", "vscode-sessions");
    if (this.registrations === expectedHome) {
      const parent = await fs.lstat(path.dirname(this.registrations));
      if (
        !parent.isDirectory() ||
        (process.getuid && (parent.uid !== process.getuid() || (parent.mode & 0o022) !== 0))
      ) {
        throw new Error(
          "Terminal registration parent is not owned and controlled by the current user",
        );
      }
    }
    if (
      !info.isDirectory() ||
      (process.getuid && (info.uid !== process.getuid() || (info.mode & 0o077) !== 0))
    ) {
      throw new Error("Terminal registration requires an owner-private directory");
    }
    const file = path.join(this.registrations, `${randomUUID()}.json`);
    const handle = await fs.open(file, "wx", PRIVATE_FILE);
    this.sessionFilePath = file;
    try {
      await handle.writeFile(JSON.stringify(sessionInfo));
    } finally {
      await handle.close();
    }
  }

  /**
   * Stop the webhook server and clean up session file
   */
  async stop(): Promise<void> {
    this.bearer = undefined;
    // Clean up session file
    if (this.sessionFilePath) {
      try {
        await fs.unlink(this.sessionFilePath);
        console.log(`Session file removed: ${this.sessionFilePath}`);
      } catch {
        /* ignore - file may not exist */
      }
      this.sessionFilePath = null;
    }

    // Stop server
    return new Promise((resolve) => {
      if (this.server) {
        this.server.closeAllConnections();
        this.server.close(() => {
          this.server = null;
          this.actualPort = 0;
          resolve();
        });
      } else {
        resolve();
      }
    });
  }

  /**
   * Check if server is running
   */
  isRunning(): boolean {
    return this.server !== null;
  }

  getRegistrationPath(): string | null {
    return this.sessionFilePath;
  }

  getPort(): number {
    return this.actualPort;
  }

  /**
   * Re-write the session file if the server is running but the file was lost
   */
  async ensureSessionFile(ticketsDir: string): Promise<void> {
    if (!this.server) {
      return;
    }
    if (this.sessionFilePath && !(await fs.stat(this.sessionFilePath).catch(() => undefined))) {
      this.sessionFilePath = null;
    }
    await this.writeSessionFile(ticketsDir);
  }

  /**
   * Get the configured port preference
   */
  getConfiguredPort(): number {
    return this.configuredPort;
  }

  /**
   * Handle incoming HTTP requests
   */
  private async handleRequest(req: http.IncomingMessage, res: http.ServerResponse) {
    if (req.headers.origin !== undefined || req.headers["sec-fetch-site"] !== undefined) {
      res.writeHead(403).end();
      return;
    }
    try {
      const urlPath = new URL(req.url ?? "/", `http://localhost:${this.actualPort}`).pathname;
      // Health check
      if (urlPath === "/health" && req.method === "GET") {
        const response: HealthResponse = {
          status: "ok",
          version: VERSION,
          port: this.actualPort,
        };
        return this.sendJson(res, response);
      }

      const bearer = this.bearer;
      const expected = Buffer.from(`Bearer ${bearer ?? ""}`);
      const supplied = Buffer.from(req.headers.authorization ?? "");
      if (
        !this.bearer ||
        expected.length !== supplied.length ||
        !timingSafeEqual(expected, supplied)
      ) {
        res.writeHead(401).end();
        return;
      }

      // Create terminal
      if (urlPath === "/terminal/create" && req.method === "POST") {
        const body = await this.parseBody<TerminalCreateOptions>(req);
        if (bearer !== this.bearer) {
          res.writeHead(401).end();
          return;
        }
        this.terminalManager.create(body);
        const response: SuccessResponse = { success: true, name: body.name };
        return this.sendJson(res, response);
      }

      // Send command to terminal
      if (urlPath.startsWith("/terminal/") && urlPath.endsWith("/send") && req.method === "POST") {
        const name = this.extractName(urlPath, "/terminal/", "/send");
        const body = await this.parseBody<SendCommandRequest>(req);
        if (bearer !== this.bearer) {
          res.writeHead(401).end();
          return;
        }
        this.terminalManager.send(name, body.command);
        const response: SuccessResponse = { success: true };
        return this.sendJson(res, response);
      }

      // Show terminal (reveal without focus)
      if (urlPath.startsWith("/terminal/") && urlPath.endsWith("/show") && req.method === "POST") {
        const name = this.extractName(urlPath, "/terminal/", "/show");
        this.terminalManager.show(name);
        const response: SuccessResponse = { success: true };
        return this.sendJson(res, response);
      }

      // Focus terminal (reveal with focus)
      if (urlPath.startsWith("/terminal/") && urlPath.endsWith("/focus") && req.method === "POST") {
        const name = this.extractName(urlPath, "/terminal/", "/focus");
        this.terminalManager.focus(name);
        const response: SuccessResponse = { success: true };
        return this.sendJson(res, response);
      }

      // Kill terminal
      if (
        urlPath.startsWith("/terminal/") &&
        urlPath.endsWith("/kill") &&
        req.method === "DELETE"
      ) {
        const name = this.extractName(urlPath, "/terminal/", "/kill");
        this.terminalManager.kill(name);
        const response: SuccessResponse = { success: true };
        return this.sendJson(res, response);
      }

      // Check if terminal exists
      if (urlPath.startsWith("/terminal/") && urlPath.endsWith("/exists") && req.method === "GET") {
        const name = this.extractName(urlPath, "/terminal/", "/exists");
        const response: ExistsResponse = {
          exists: this.terminalManager.exists(name),
        };
        return this.sendJson(res, response);
      }

      // Get terminal activity
      if (
        urlPath.startsWith("/terminal/") &&
        urlPath.endsWith("/activity") &&
        req.method === "GET"
      ) {
        const name = this.extractName(urlPath, "/terminal/", "/activity");
        const response: ActivityResponse = {
          activity: this.terminalManager.getActivity(name),
        };
        return this.sendJson(res, response);
      }

      // List all terminals
      if (urlPath === "/terminal/list" && req.method === "GET") {
        const response: ListResponse = {
          terminals: this.terminalManager.list(),
        };
        return this.sendJson(res, response);
      }

      // 404 Not Found
      res.writeHead(404);
      res.end(JSON.stringify({ error: "Not found" }));
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      res.writeHead(500);
      res.end(JSON.stringify({ error: message }));
    }
  }

  /**
   * Extract terminal name from path
   */
  private extractName(urlPath: string, prefix: string, suffix: string): string {
    return decodeURIComponent(urlPath.slice(prefix.length, -suffix.length));
  }

  /**
   * Parse JSON request body
   */
  private async parseBody<T>(req: http.IncomingMessage): Promise<T> {
    let body = "";
    for await (const chunk of req) {
      body += chunk;
    }
    try {
      return JSON.parse(body || "{}") as T;
    } catch {
      throw new Error("Invalid JSON");
    }
  }

  /**
   * Send JSON response
   */
  private sendJson(res: http.ServerResponse, data: object): void {
    res.setHeader("Content-Type", "application/json");
    res.writeHead(200);
    res.end(JSON.stringify(data));
  }
}
