import * as path from "node:path";
import { existsSync } from "node:fs";
import * as vscode from "vscode";

const DEFAULT_TICKETS_DIR = ".tickets";

export function workspaceConfigTickets(): string | undefined {
  const folder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!folder || !vscode.workspace.isTrusted) {
    return undefined;
  }
  const configured =
    vscode.workspace.getConfiguration("operator").get<string>("ticketsDir") || DEFAULT_TICKETS_DIR;
  const dir = path.resolve(folder, configured);
  const relative = path.relative(folder, dir);
  if (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    return undefined;
  }
  return existsSync(path.join(dir, "operator", "config.toml")) ? dir : undefined;
}

export function resolveTicketsDirectory(workingDirectory?: string): string {
  const config = vscode.workspace.getConfiguration("operator");
  const own = workspaceConfigTickets();
  if (!workingDirectory && own && !config.inspect<string>("workingDirectory")?.workspaceValue) {
    return own;
  }
  const directory = workingDirectory ?? config.get<string>("workingDirectory");
  const tickets = config.get<string>("ticketsDir") || DEFAULT_TICKETS_DIR;
  if (path.isAbsolute(tickets)) {
    return tickets;
  }
  if (directory) {
    return path.resolve(directory, tickets);
  }
  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspace) {
    return "";
  }
  if (tickets !== DEFAULT_TICKETS_DIR) {
    return path.resolve(workspace, tickets);
  }
  const parent = path.join(path.dirname(workspace), tickets);
  if (existsSync(parent)) {
    return parent;
  }
  let current = workspace;
  while (current !== path.dirname(current)) {
    const candidate = path.join(current, tickets);
    if (existsSync(candidate)) {
      return candidate;
    }
    current = path.dirname(current);
  }
  return parent;
}

export function resolveWorkingDirectory(): string {
  if (
    workspaceConfigTickets() &&
    !vscode.workspace.getConfiguration("operator").inspect<string>("workingDirectory")
      ?.workspaceValue
  ) {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? "";
  }
  const explicit = vscode.workspace.getConfiguration("operator").get<string>("workingDirectory");
  if (explicit) {
    return explicit;
  }
  const tickets = resolveTicketsDirectory();
  return tickets ? path.dirname(tickets) : "";
}

export function getResolvedConfigPath(): string {
  const tickets = resolveTicketsDirectory();
  return tickets ? path.join(tickets, "operator", "config.toml") : "";
}

export function configFileExists(ticketsDir?: string): Promise<boolean> {
  return Promise.resolve(
    existsSync(
      ticketsDir ? path.join(ticketsDir, "operator", "config.toml") : getResolvedConfigPath(),
    ),
  );
}
