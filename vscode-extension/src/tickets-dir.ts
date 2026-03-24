import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

/**
 * Find .tickets directory - check parent directory first, then workspace
 */
export async function findParentTicketsDir(): Promise<string | undefined> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  // First check parent directory for .tickets (monorepo setup)
  const parentDir = path.dirname(workspaceFolder.uri.fsPath);
  const parentTicketsPath = path.join(parentDir, '.tickets');

  try {
    await fs.access(parentTicketsPath);
    return parentTicketsPath;
  } catch {
    // Parent doesn't have .tickets, check workspace
  }

  // Fall back to configured tickets directory in workspace
  const configuredDir = vscode.workspace
    .getConfiguration('operator')
    .get<string>('ticketsDir', '.tickets');

  const ticketsPath = path.isAbsolute(configuredDir)
    ? configuredDir
    : path.join(workspaceFolder.uri.fsPath, configuredDir);

  try {
    await fs.access(ticketsPath);
    return ticketsPath;
  } catch {
    return undefined;
  }
}

/**
 * Find the .tickets directory for webhook session file.
 * Walks up from workspace to find existing .tickets, or creates in parent (org level).
 */
export async function findTicketsDir(): Promise<string | undefined> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  const configuredDir = vscode.workspace
    .getConfiguration('operator')
    .get<string>('ticketsDir', '.tickets');

  // If absolute path configured, check if it exists
  if (path.isAbsolute(configuredDir)) {
    try {
      await fs.access(configuredDir);
      return configuredDir;
    } catch {
      return undefined;
    }
  }

  // Walk up from workspace to find existing .tickets directory
  let currentDir = workspaceFolder.uri.fsPath;
  const root = path.parse(currentDir).root;

  while (currentDir !== root) {
    const ticketsPath = path.join(currentDir, configuredDir);
    try {
      await fs.access(ticketsPath);
      return ticketsPath; // Found existing .tickets
    } catch {
      // Not found, try parent
      currentDir = path.dirname(currentDir);
    }
  }

  // Not found anywhere
  return undefined;
}

/**
 * Find the directory to run the operator server in.
 * Prefers parent directory if it has .tickets/operator/, otherwise uses workspace.
 */
export async function findOperatorServerDir(): Promise<string | undefined> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    return undefined;
  }

  const workspaceDir = workspaceFolder.uri.fsPath;
  const parentDir = path.dirname(workspaceDir);

  // Check if parent has .tickets/operator/ (initialized operator setup)
  const parentOperatorPath = path.join(parentDir, '.tickets', 'operator');
  try {
    await fs.access(parentOperatorPath);
    return parentDir; // Parent has initialized operator
  } catch {
    // Parent doesn't have .tickets/operator
  }

  // Fall back to workspace directory
  return workspaceDir;
}
