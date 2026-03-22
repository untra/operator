/**
 * MCP connection logic for Operator VS Code extension.
 *
 * Discovers the local Operator API, fetches the MCP descriptor,
 * and registers the Operator MCP server in VS Code workspace settings.
 */

import * as vscode from 'vscode';
import { discoverApiUrl } from './api-client';

/**
 * MCP server descriptor returned by the Operator API.
 * Matches the Rust McpDescriptorResponse DTO.
 */
export interface McpDescriptorResponse {
  server_name: string;
  server_id: string;
  version: string;
  transport_url: string;
  label: string;
  openapi_url: string | null;
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
      `Operator API is not running at ${apiUrl}. Start the server first.`
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
 * in VS Code workspace settings.
 */
export function isMcpServerRegistered(): boolean {
  const mcpConfig = vscode.workspace.getConfiguration('mcp');
  const servers = mcpConfig.get<Record<string, unknown>>('servers') || {};
  return 'operator' in servers;
}

/**
 * Connect Operator as an MCP server in VS Code.
 *
 * Discovers the running API, fetches the MCP descriptor,
 * and writes the server config into VS Code workspace settings
 * under the `mcp.servers` key.
 */
export async function connectMcpServer(
  ticketsDir: string | undefined
): Promise<void> {
  try {
    // 1. Discover the API URL
    const apiUrl = await discoverApiUrl(ticketsDir);

    // 2. Fetch the MCP descriptor
    let descriptor: McpDescriptorResponse;
    try {
      descriptor = await fetchMcpDescriptor(apiUrl);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch MCP descriptor';
      void vscode.window.showErrorMessage(message);
      return;
    }

    // 3. Write MCP server config to workspace settings
    const mcpConfig = vscode.workspace.getConfiguration('mcp');
    const servers = mcpConfig.get<Record<string, unknown>>('servers') || {};

    servers['operator'] = {
      type: 'sse',
      url: descriptor.transport_url,
    };

    await mcpConfig.update(
      'servers',
      servers,
      vscode.ConfigurationTarget.Workspace
    );

    void vscode.window.showInformationMessage(
      `Operator MCP server registered (${descriptor.transport_url})`
    );
  } catch (err) {
    const message =
      err instanceof Error ? err.message : 'Failed to connect MCP server';
    void vscode.window.showErrorMessage(message);
  }
}
