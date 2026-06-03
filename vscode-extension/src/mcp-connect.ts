/**
 * MCP connection logic for Operator VS Code extension.
 *
 * Discovers the local Operator API, fetches the MCP descriptor,
 * and registers the Operator MCP server. The registration path depends on
 * the host IDE:
 *
 * - **VS Code (and other Code OSS forks without a special MCP path)** —
 *   writes a workspace-scope `mcp.servers.operator` entry via
 *   `vscode.workspace.getConfiguration('mcp').update('servers', ...)`. When
 *   the operator descriptor advertises stdio, the entry uses the stdio shape;
 *   otherwise it falls back to SSE (preserves existing behavior).
 *
 * - **Cursor** — writes a user-scope entry to `~/.cursor/mcp.json` under
 *   `mcpServers.operator`. Cursor's MCP UI surfaces this user-scope config,
 *   not VS Code's workspace `mcp.servers`. Stdio-only (Cursor's `mcpServers`
 *   shape does not support an SSE URL); errors out with an actionable message
 *   when the operator descriptor does not advertise stdio.
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import * as os from 'os';
import * as path from 'path';
import { discoverApiUrl } from './api-client';

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
export type HostApp = 'cursor' | 'vscode' | 'other';

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
   * branch on the raw value — go through `detectHostApp()` below.
   *
   * Observed values:
   * - Stock VS Code: "Visual Studio Code" (or "Visual Studio Code - Insiders")
   * - Cursor: "Cursor" (verify at runtime — see cursor.md Pre-Flight)
   */
  rawAppName(): string {
    return vscode.env.appName ?? '';
  },
  /** Default location of Cursor's user-scope MCP config. */
  cursorMcpConfigPath(): string {
    return path.join(os.homedir(), '.cursor', 'mcp.json');
  },
};

/** Detect which IDE the extension is running inside. */
export function detectHostApp(): HostApp {
  const name = _testable.rawAppName();
  if (name.startsWith('Cursor')) {
    return 'cursor';
  }
  if (name.startsWith('Visual Studio Code')) {
    return 'vscode';
  }
  return 'other';
}

/** Public accessor for the default Cursor MCP config path. */
export function cursorMcpConfigPath(): string {
  return _testable.cursorMcpConfigPath();
}

/**
 * Fetch the MCP descriptor from the Operator API.
 *
 * @param apiUrl - Base URL of the Operator API (e.g. "http://localhost:7008")
 * @returns The MCP descriptor
 * @throws Error if the API is unreachable or the descriptor endpoint fails
 */
export async function fetchMcpDescriptor(
  apiUrl: string
): Promise<McpDescriptorResponse> {
  const url = `${apiUrl}/api/v1/mcp/descriptor`;

  let response: Response;
  try {
    response = await fetch(url);
  } catch (err) {
    throw new Error(
      `Operator API is not running at ${apiUrl}. Start the server first.`,
      { cause: err },
    );
  }

  if (!response.ok) {
    throw new Error(
      `MCP descriptor unavailable (HTTP ${response.status}). ` +
        'Ensure Operator is updated to a version that supports MCP.'
    );
  }

  return (await response.json()) as McpDescriptorResponse;
}

/**
 * Check whether an MCP server named "operator" is already registered
 * in VS Code workspace settings. (Cursor users should check
 * `~/.cursor/mcp.json` directly — VS Code's API does not see that file.)
 */
export function isMcpServerRegistered(): boolean {
  const mcpConfig = vscode.workspace.getConfiguration('mcp');
  const servers = mcpConfig.get<Record<string, unknown>>('servers') || {};
  return 'operator' in servers;
}

/**
 * Build the workspace-scope server entry for VS Code's `mcp.servers`,
 * preferring the stdio transport when the descriptor advertises it.
 */
function buildVscodeServerEntry(
  descriptor: McpDescriptorResponse
): Record<string, unknown> {
  if (descriptor.stdio) {
    return {
      type: 'stdio',
      command: descriptor.stdio.command,
      args: descriptor.stdio.args,
      cwd: descriptor.stdio.cwd,
    };
  }
  return {
    type: 'sse',
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
  descriptor: McpDescriptorResponse
): Promise<void> {
  const mcpConfig = vscode.workspace.getConfiguration('mcp');
  const servers = mcpConfig.get<Record<string, unknown>>('servers') || {};

  servers['operator'] = buildVscodeServerEntry(descriptor);

  await mcpConfig.update(
    'servers',
    servers,
    vscode.ConfigurationTarget.Workspace
  );

  const transport = descriptor.stdio ? 'stdio' : 'sse';
  const detail = descriptor.stdio
    ? `${descriptor.stdio.command} ${descriptor.stdio.args.join(' ')}`
    : descriptor.transport_url;
  void vscode.window.showInformationMessage(
    `Operator MCP server registered (${transport}: ${detail})`
  );
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
  configPath: string = _testable.cursorMcpConfigPath()
): Promise<void> {
  if (!descriptor.stdio) {
    void vscode.window.showErrorMessage(
      'Operator MCP stdio entrypoint is not advertised. Set ' +
        '`[mcp].stdio_advertised = true` in your operator config and restart ' +
        'the API, or use stock VS Code which can connect over SSE.'
    );
    return;
  }

  await fs.mkdir(path.dirname(configPath), { recursive: true });

  let existing: Record<string, unknown> = {};
  try {
    const raw = await fs.readFile(configPath, 'utf-8');
    const parsed = JSON.parse(raw) as unknown;
    if (parsed !== null && typeof parsed === 'object') {
      existing = parsed as Record<string, unknown>;
    }
  } catch (err: unknown) {
    const e = err as NodeJS.ErrnoException;
    if (e.code !== 'ENOENT') {
      void vscode.window.showErrorMessage(
        `Could not parse existing ${configPath}: ${e.message}. ` +
          'Please fix or remove the file and retry.'
      );
      return;
    }
  }

  const existingServers = existing.mcpServers;
  const mcpServers =
    existingServers && typeof existingServers === 'object'
      ? { ...(existingServers as Record<string, unknown>) }
      : {};

  mcpServers['operator'] = {
    command: descriptor.stdio.command,
    args: descriptor.stdio.args,
    cwd: descriptor.stdio.cwd,
  };

  const merged = { ...existing, mcpServers };
  await fs.writeFile(
    configPath,
    JSON.stringify(merged, null, 2) + '\n',
    'utf-8'
  );

  void vscode.window.showInformationMessage(
    `Operator MCP server registered in ${configPath} (stdio). ` +
      'You may need to restart Cursor or toggle the server in ' +
      'Cursor Settings → MCP.'
  );
}

/**
 * Connect Operator as an MCP server in the host IDE.
 *
 * Discovers the running API, fetches the descriptor, then dispatches to
 * either the Cursor user-scope path (`~/.cursor/mcp.json`) or the VS Code
 * workspace-scope path (`mcp.servers`) based on the detected host.
 */
export async function connectMcpServer(
  ticketsDir: string | undefined
): Promise<void> {
  try {
    const apiUrl = await discoverApiUrl(ticketsDir);

    let descriptor: McpDescriptorResponse;
    try {
      descriptor = await fetchMcpDescriptor(apiUrl);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch MCP descriptor';
      void vscode.window.showErrorMessage(message);
      return;
    }

    if (detectHostApp() === 'cursor') {
      await registerInCursorUserConfig(descriptor);
    } else {
      await registerInVscodeWorkspaceConfig(descriptor);
    }
  } catch (err) {
    const message =
      err instanceof Error ? err.message : 'Failed to connect MCP server';
    void vscode.window.showErrorMessage(message);
  }
}
