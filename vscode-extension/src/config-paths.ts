/**
 * Dynamic config path resolution for Operator VS Code Extension
 *
 * Config lives at: <workdir>/.tickets/operator/config.toml
 * The working directory is determined from the VS Code setting
 * `operator.workingDirectory`, falling back to the workspace parent.
 */

import * as path from 'path';
import * as fs from 'fs/promises';
import * as vscode from 'vscode';

/** Build the config.toml path from a working directory */
export function getConfigPath(workingDir: string): string {
  return path.join(workingDir, '.tickets', 'operator', 'config.toml');
}

/** Build the config directory (containing config.toml) from a working directory */
export function getConfigDir(workingDir: string): string {
  return path.join(workingDir, '.tickets', 'operator');
}

/** Resolve the working directory from settings or workspace */
export function resolveWorkingDirectory(): string {
  // Check operator.workingDirectory setting first
  const configured = vscode.workspace
    .getConfiguration('operator')
    .get<string>('workingDirectory');
  if (configured) {
    return configured;
  }

  // Fall back to parent of the first workspace folder
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (workspaceFolder) {
    return path.dirname(workspaceFolder.uri.fsPath);
  }

  return '';
}

/** Get the resolved config.toml path using current settings */
export function getResolvedConfigPath(): string {
  const workDir = resolveWorkingDirectory();
  if (!workDir) {
    return '';
  }
  return getConfigPath(workDir);
}

/** Check whether config.toml exists and is readable */
export async function configFileExists(): Promise<boolean> {
  const configPath = getResolvedConfigPath();
  if (!configPath) {
    return false;
  }
  try {
    await fs.access(configPath);
    return true;
  } catch {
    return false;
  }
}
