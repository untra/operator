/**
 * MCP connection logic for Operator VS Code extension.
 *
 * Discovers the local Operator API, fetches the MCP descriptor,
 * and registers the Operator MCP server. The registration path depends on
 * the host IDE:
 *
 * - **VS Code (and other Code OSS forks without a special MCP path)** -
 *   writes workspace-scope `mcp.servers.operator` (`.vscode/mcp.json`) and,
 *   when stdio is advertised, workspace-root `.mcp.json` (`mcpServers.operator`).
 *   Offers Copilot Global as a follow-up. SSE stays on workspace settings only.
 *
 * - **Cursor** - writes a user-scope entry to `~/.cursor/mcp.json` under
 *   `mcpServers.operator`. Cursor's MCP UI surfaces this user-scope config,
 *   not VS Code's workspace `mcp.servers`. Stdio-only (Cursor's `mcpServers`
 *   shape does not support an SSE URL); errors out with an actionable message
 *   when the operator descriptor does not advertise stdio.
 */

import * as vscode from "vscode";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { ApiError, discoverApiUrl, OperatorApiClient } from "./api-client";

/**
 * Stdio entrypoint advertised by the Operator MCP descriptor when
 * `[mcp].stdio_advertised = true` is set in the operator config.
 * Matches the Rust `StdioCommand` DTO.
 */
export interface StdioCommand {
  command: string;
  args: string[];
  cwd: string;
}

/**
 * MCP server descriptor returned by the Operator API.
 * Matches the Rust McpDescriptorResponse DTO. The `stdio` field is omitted
 * when the operator config has `[mcp].stdio_advertised = false`.
 */
export interface McpDescriptorResponse {
  server_name: string;
  server_id: string;
  version: string;
  transport_url: string;
  label: string;
  openapi_url: string | null;
  stdio?: StdioCommand;
}

/** Host IDE branches the extension knows how to register MCP servers in. */
export type HostApp = "cursor" | "vscode" | "other";

export const COPILOT_GLOBAL_ACTION = "Also write Copilot Global";

/**
 * Indirection layer for the small pieces of platform state that need to be
 * stubbed across function boundaries in tests. Sinon stubs cannot intercept
 * intra-file direct calls, so `connectMcpServer` and the registration
 * functions invoke these via `_testable.fn()` to allow stubbing.
 */
export const _testable = {
  /**
   * Returns `vscode.env.appName` (or "" if unavailable). The `vscode-test`
   * electron host returns its own string, so production code must NOT
   * branch on the raw value - go through `detectHostApp()` below.
   *
   * Observed values:
   * - Stock VS Code: "Visual Studio Code" (or "Visual Studio Code - Insiders")
   * - Cursor: "Cursor" (verify at runtime - see cursor.md Pre-Flight)
   */
  rawAppName(): string {
    return vscode.env.appName ?? "";
  },
  /** Default location of Cursor's user-scope MCP config. */
  cursorMcpConfigPath(): string {
    return path.join(os.homedir(), ".cursor", "mcp.json");
  },
  /** Workspace-root portable MCP file (`<folder>/.mcp.json`). */
  workspacePortableMcpPath(): string | undefined {
    const folder = vscode.workspace.workspaceFolders?.[0];
    if (!folder) {
      return undefined;
    }
    return path.join(folder.uri.fsPath, ".mcp.json");
  },
  /** Copilot Global portable MCP file. */
  copilotGlobalConfigPath(): string {
    const home = process.env.COPILOT_HOME;
    if (home && home.length > 0) {
      return path.join(home, "mcp-config.json");
    }
    return path.join(os.homedir(), ".copilot", "mcp-config.json");
  },
};

/** Detect which IDE the extension is running inside. */
export function detectHostApp(): HostApp {
  const name = _testable.rawAppName();
  if (name.startsWith("Cursor")) {
    return "cursor";
  }
  if (name.startsWith("Visual Studio Code")) {
    return "vscode";
  }
  return "other";
}

/** Public accessor for the default Cursor MCP config path. */
export function cursorMcpConfigPath(): string {
  return _testable.cursorMcpConfigPath();
}

/** Public accessor for the workspace-root portable MCP path. */
export function workspacePortableMcpPath(): string | undefined {
  return _testable.workspacePortableMcpPath();
}

/** Public accessor for the Copilot Global MCP config path. */
export function copilotGlobalConfigPath(): string {
  return _testable.copilotGlobalConfigPath();
}

/**
 * Register Operator in Copilot Global (`$COPILOT_HOME/mcp-config.json` or
 * `~/.copilot/mcp-config.json`). Stdio-only.
 */
export async function registerInCopilotGlobalConfig(
  descriptor: McpDescriptorResponse,
  configPath?: string,
): Promise<McpServersWriteResult | { ok: false; path: string; reason: "no-stdio" }> {
  if (!descriptor.stdio) {
    return { ok: false, path: "", reason: "no-stdio" };
  }
  const target = configPath ?? _testable.copilotGlobalConfigPath();
  return writeMcpServersOperatorEntry(target, descriptor.stdio);
}

export type PortableRegisterSkip = {
  ok: false;
  path: string;
  reason: "no-stdio" | "no-workspace";
};

/**
 * Register Operator in the workspace-root portable `.mcp.json`.
 * Stdio-only: skip when the descriptor has no stdio or no workspace folder.
 */
export async function registerInVscodeWorkspacePortableConfig(
  descriptor: McpDescriptorResponse,
  configPath?: string,
): Promise<McpServersWriteResult | PortableRegisterSkip> {
  if (!descriptor.stdio) {
    return { ok: false, path: "", reason: "no-stdio" };
  }
  const target = configPath ?? _testable.workspacePortableMcpPath();
  if (!target) {
    return { ok: false, path: "", reason: "no-workspace" };
  }
  return writeMcpServersOperatorEntry(target, descriptor.stdio);
}

export type McpServersWriteResult =
  | { ok: true; path: string }
  | { ok: false; path: string; reason: "parse" | "io"; message: string };

/**
 * Merge `mcpServers.operator` into a portable MCP file (Cursor, workspace
 * `.mcp.json`, Copilot Global). Preserves sibling servers and other top-level
 * keys. Does not write `type` or `url`.
 */
export async function writeMcpServersOperatorEntry(
  configPath: string,
  stdio: StdioCommand,
): Promise<McpServersWriteResult> {
  await fs.mkdir(path.dirname(configPath), { recursive: true });

  let existing: Record<string, unknown> = {};
  try {
    const raw = await fs.readFile(configPath, "utf-8");
    const parsed = JSON.parse(raw) as unknown;
    if (parsed !== null && typeof parsed === "object") {
      existing = parsed as Record<string, unknown>;
    }
  } catch (err: unknown) {
    const e = err as NodeJS.ErrnoException;
    if (e.code !== "ENOENT") {
      return {
        ok: false,
        path: configPath,
        reason: "parse",
        message: e.message,
      };
    }
  }

  const existingServers = existing.mcpServers;
  const mcpServers =
    existingServers && typeof existingServers === "object"
      ? { ...(existingServers as Record<string, unknown>) }
      : {};

  mcpServers.operator = {
    command: stdio.command,
    args: stdio.args,
    cwd: stdio.cwd,
  };

  try {
    await fs.writeFile(
      configPath,
      `${JSON.stringify({ ...existing, mcpServers }, null, 2)}\n`,
      "utf-8",
    );
  } catch (err: unknown) {
    const e = err as Error;
    return { ok: false, path: configPath, reason: "io", message: e.message };
  }

  return { ok: true, path: configPath };
}

/**
 * Fetch the MCP descriptor from the Operator API.
 *
 * @param apiUrl - Base URL of the Operator API (e.g. "http://localhost:7008")
 * @returns The MCP descriptor
 * @throws Error if the API is unreachable or the descriptor endpoint fails
 */
export async function fetchMcpDescriptor(apiUrl: string): Promise<McpDescriptorResponse> {
  const client = new OperatorApiClient(apiUrl);
  try {
    const descriptor = await client.mcpDescriptor();
    return { ...descriptor, stdio: descriptor.stdio ?? undefined };
  } catch (err) {
    if (err instanceof ApiError) {
      throw new Error(
        `MCP descriptor unavailable (HTTP ${err.status}). ` +
          "Ensure Operator is updated to a version that supports MCP.",
        { cause: err },
      );
    }
    throw new Error(`Operator API is not running at ${apiUrl}. Start the server first.`, {
      cause: err,
    });
  }
}

/**
 * Check whether an MCP server named "operator" is already registered
 * in VS Code workspace settings. (Cursor users should check
 * `~/.cursor/mcp.json` directly - VS Code's API does not see that file.)
 */
export function isMcpServerRegistered(): boolean {
  const mcpConfig = vscode.workspace.getConfiguration("mcp");
  const servers = mcpConfig.get<Record<string, unknown>>("servers") ?? {};
  return "operator" in servers;
}

export interface McpRegistrationPresence {
  vscodeWorkspace: boolean;
  workspacePortable: boolean;
  copilotGlobal: boolean;
  cursor: boolean;
}

export function isVscodeWorkspaceStdio(): boolean | undefined {
  const servers =
    vscode.workspace.getConfiguration("mcp").get<Record<string, unknown>>("servers") ?? {};
  const entry = servers.operator;
  if (entry === null || typeof entry !== "object") {
    return undefined;
  }
  const type = (entry as Record<string, unknown>).type;
  if (type === "stdio") {
    return true;
  }
  if (type === "sse") {
    return false;
  }
  return undefined;
}

async function fileHasOperator(configPath: string | undefined): Promise<boolean> {
  if (!configPath) {
    return false;
  }
  try {
    const raw = await fs.readFile(configPath, "utf-8");
    const parsed = JSON.parse(raw) as unknown;
    if (parsed === null || typeof parsed !== "object") {
      return false;
    }
    const servers = (parsed as Record<string, unknown>).mcpServers;
    if (servers === null || typeof servers !== "object") {
      return false;
    }
    return "operator" in (servers as Record<string, unknown>);
  } catch {
    return false;
  }
}

export async function inspectMcpRegistration(): Promise<McpRegistrationPresence> {
  return {
    vscodeWorkspace: isMcpServerRegistered(),
    workspacePortable: await fileHasOperator(_testable.workspacePortableMcpPath()),
    copilotGlobal: await fileHasOperator(_testable.copilotGlobalConfigPath()),
    cursor: await fileHasOperator(_testable.cursorMcpConfigPath()),
  };
}

export function mcpStatusDescription(
  presence: McpRegistrationPresence,
  host: HostApp,
  workspaceEntryIsStdio: boolean | undefined,
): { connected: boolean; description: string; tooltip: string } {
  if (host === "cursor") {
    if (presence.cursor) {
      return {
        connected: true,
        description: "~/.cursor/mcp.json",
        tooltip: "Operator MCP is registered in Cursor user MCP config.",
      };
    }
    return {
      connected: false,
      description: "Connect",
      tooltip: "Connect Operator as MCP server in Cursor",
    };
  }

  const connected =
    presence.vscodeWorkspace || presence.workspacePortable || presence.copilotGlobal;
  if (!connected) {
    return {
      connected: false,
      description: "Connect",
      tooltip: "Connect Operator as MCP server in VS Code",
    };
  }

  if (presence.copilotGlobal && !presence.vscodeWorkspace && !presence.workspacePortable) {
    return {
      connected: true,
      description: "Copilot Global",
      tooltip: "Operator MCP is registered in Copilot Global.",
    };
  }

  const tokens: string[] = [];
  const tooltipParts: string[] = [];
  if (presence.vscodeWorkspace) {
    if (workspaceEntryIsStdio === false) {
      tokens.push("editor (SSE)");
      tooltipParts.push(
        "SSE in workspace settings. Portable files skipped because stdio is not advertised.",
      );
    } else {
      tokens.push("editor");
      tooltipParts.push(
        "Registered in workspace MCP settings. Copilot CLI does not read .vscode/mcp.json.",
      );
    }
  }
  if (presence.workspacePortable) {
    tokens.push(".mcp.json");
    tooltipParts.push("Agent Host / Copilot CLI can see workspace .mcp.json.");
  }
  if (presence.copilotGlobal) {
    tokens.push("Copilot");
    tooltipParts.push("Also in Copilot Global.");
  }

  return {
    connected: true,
    description: tokens.join(" · "),
    tooltip: tooltipParts.join(" "),
  };
}

/**
 * Build the workspace-scope server entry for VS Code's `mcp.servers`,
 * preferring the stdio transport when the descriptor advertises it.
 */
function buildVscodeServerEntry(descriptor: McpDescriptorResponse): Record<string, unknown> {
  if (descriptor.stdio) {
    return {
      type: "stdio",
      command: descriptor.stdio.command,
      args: descriptor.stdio.args,
      cwd: descriptor.stdio.cwd,
    };
  }
  return {
    type: "sse",
    url: descriptor.transport_url,
  };
}

/**
 * Register Operator under VS Code's workspace `mcp.servers` setting.
 *
 * Prefers the stdio shape when the descriptor advertises it; otherwise
 * preserves the legacy SSE registration so old operator builds keep working.
 */
export async function registerInVscodeWorkspaceConfig(
  descriptor: McpDescriptorResponse,
): Promise<void> {
  const mcpConfig = vscode.workspace.getConfiguration("mcp");
  const servers = mcpConfig.get<Record<string, unknown>>("servers") ?? {};

  servers["operator"] = buildVscodeServerEntry(descriptor);

  await mcpConfig.update("servers", servers, vscode.ConfigurationTarget.Workspace);
}

/**
 * Register Operator in Cursor's user-scope MCP config (`~/.cursor/mcp.json`).
 *
 * Stdio-only: Cursor's `mcpServers` shape does not accept an SSE URL. If the
 * descriptor does not advertise stdio, this function shows an actionable
 * error naming the `[mcp].stdio_advertised` config knob and returns without
 * touching the file.
 *
 * Merge semantics: any existing top-level keys and any existing
 * `mcpServers.*` entries are preserved; only the `mcpServers.operator`
 * key is set/overwritten. Bails on JSON parse failure rather than
 * overwriting a file the user has hand-edited.
 *
 * @param configPath - Override target file (tests pass a tempdir path).
 *                     Defaults to `cursorMcpConfigPath()`.
 */
export async function registerInCursorUserConfig(
  descriptor: McpDescriptorResponse,
  configPath: string = _testable.cursorMcpConfigPath(),
): Promise<void> {
  if (!descriptor.stdio) {
    void vscode.window.showErrorMessage(
      "Operator MCP stdio entrypoint is not advertised. Set " +
        "`[mcp].stdio_advertised = true` in your operator config and restart " +
        "the API, or use stock VS Code which can connect over SSE.",
    );
    return;
  }

  const result = await writeMcpServersOperatorEntry(configPath, descriptor.stdio);
  if (!result.ok) {
    if (result.reason === "parse") {
      void vscode.window.showErrorMessage(
        `Could not parse existing ${configPath}: ${result.message}. ` +
          "Please fix or remove the file and retry.",
      );
    } else {
      void vscode.window.showErrorMessage(
        `Could not write ${configPath}: ${result.message}`,
      );
    }
    return;
  }

  void vscode.window.showInformationMessage(
    `Operator MCP server registered in ${configPath} (stdio). ` +
      "You may need to restart Cursor or toggle the server in " +
      "Cursor Settings → MCP.",
  );
}

/**
 * Connect Operator as an MCP server in the host IDE.
 *
 * Cursor writes `~/.cursor/mcp.json`. VS Code keeps workspace `mcp.servers`
 * and, when stdio is advertised, also writes workspace-root `.mcp.json`.
 * Copilot Global is offered as a follow-up action.
 */
export async function connectMcpServer(ticketsDir: string | undefined): Promise<void> {
  try {
    const apiUrl = await discoverApiUrl(ticketsDir);

    let descriptor: McpDescriptorResponse;
    try {
      descriptor = await fetchMcpDescriptor(apiUrl);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to fetch MCP descriptor";
      void vscode.window.showErrorMessage(message);
      return;
    }

    if (detectHostApp() === "cursor") {
      await registerInCursorUserConfig(descriptor);
      return;
    }

    await registerInVscodeWorkspaceConfig(descriptor);
    const portable = await registerInVscodeWorkspacePortableConfig(descriptor);

    const lines: string[] = [];
    const transport = descriptor.stdio ? "stdio" : "sse";
    lines.push(`Operator MCP server registered in workspace settings (${transport}).`);
    if (portable.ok) {
      lines.push(
        `Also wrote ${portable.path}. This path is machine-local; do not commit .mcp.json.`,
      );
    } else if (descriptor.stdio) {
      lines.push(
        "Copilot CLI does not read .vscode/mcp.json. Open a folder or write Copilot Global.",
      );
    }

    if (descriptor.stdio) {
      const choice = await vscode.window.showInformationMessage(
        lines.join(" "),
        COPILOT_GLOBAL_ACTION,
      );
      if (choice === COPILOT_GLOBAL_ACTION) {
        const globalResult = await registerInCopilotGlobalConfig(descriptor);
        if (globalResult.ok) {
          void vscode.window.showInformationMessage(
            `Operator MCP registered in ${globalResult.path} (Copilot Global).`,
          );
        } else if ("message" in globalResult) {
          void vscode.window.showErrorMessage(
            `Could not write ${globalResult.path}: ${globalResult.message}`,
          );
        }
      }
    } else {
      void vscode.window.showInformationMessage(lines.join(" "));
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : "Failed to connect MCP server";
    void vscode.window.showErrorMessage(message);
  }
}
