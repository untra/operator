/**
 * MCP connection logic for Operator VS Code extension.
 *
 * Discovers the local Operator API, fetches the MCP descriptor,
 * builds a vscode:// deep link, and opens it to register the
 * Operator MCP server in VS Code.
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
 * Build a VS Code MCP deep link URI from an MCP descriptor.
 *
 * The deep link format is:
 *   vscode://modelcontextprotocol.mcp/connect?config=<base64-json>
 *
 * Where the JSON config contains:
 *   { name, type: "sse", url: transport_url }
 */
export function buildMcpDeepLink(
  descriptor: McpDescriptorResponse
): vscode.Uri {
  const config = {
    name: descriptor.server_name,
    type: 'sse',
    url: descriptor.transport_url,
  };

  const base64 = Buffer.from(JSON.stringify(config)).toString('base64');
  return vscode.Uri.parse(
    `vscode://modelcontextprotocol.mcp/connect?config=${base64}`
  );
}

/**
 * Connect Operator as an MCP server in VS Code.
 *
 * Discovers the running API, fetches the MCP descriptor,
 * builds a deep link, and opens it.
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

    // 3. Build and open the deep link
    const uri = buildMcpDeepLink(descriptor);

    const opened = await vscode.env.openExternal(uri);
    if (!opened) {
      void vscode.window.showErrorMessage(
        'Failed to open MCP connection. VS Code may not support MCP deep links in this version.'
      );
    }
  } catch (err) {
    const message =
      err instanceof Error ? err.message : 'Failed to connect MCP server';
    void vscode.window.showErrorMessage(message);
  }
}
