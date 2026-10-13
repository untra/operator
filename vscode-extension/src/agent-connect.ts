/**
 * `vscode://untra.operator-terminals/connect?agent=..&authority=..&path=..`
 *
 * Opens an agent's workspace (over Remote-SSH when `authority` is set) and
 * starts a terminal attached to its session, or resuming its conversation.
 * The window that receives the link asks the Operator API what to run, then
 * hands it to the window that owns the workspace through `globalState`.
 */

import * as vscode from "vscode";
import type { ConnectTerminal } from "./generated";

export interface ConnectRequest {
  agent: string;
  authority?: string;
  path: string;
}

export interface PendingConnect extends ConnectRequest {
  terminal: ConnectTerminal;
  at: number;
}

export interface WorkspaceRef {
  authority?: string;
  path: string;
}

export const PENDING_CONNECT_KEY = "operator.pendingConnect";
const PENDING_TTL_MS = 2 * 60 * 1000;

export function parseConnect(params: URLSearchParams): ConnectRequest | undefined {
  const agent = params.get("agent");
  const path = params.get("path");
  if (!agent || !path) {
    return undefined;
  }
  return { agent, path, authority: params.get("authority") || undefined };
}

export function matchesWorkspace(req: ConnectRequest, folders: readonly WorkspaceRef[]): boolean {
  return folders.some(
    (f) =>
      (f.authority || undefined) === req.authority && trimSlash(f.path) === trimSlash(req.path),
  );
}

export function isFresh(pending: PendingConnect, now: number): boolean {
  return now - pending.at < PENDING_TTL_MS;
}

function trimSlash(p: string): string {
  return p.length > 1 ? p.replace(/\/+$/, "") : p;
}

function currentWorkspaces(): WorkspaceRef[] {
  return (vscode.workspace.workspaceFolders ?? []).map((f) => ({
    authority: f.uri.scheme === "vscode-remote" ? f.uri.authority : undefined,
    path: f.uri.path,
  }));
}

export interface ConnectDeps {
  state: vscode.Memento;
  fetchTerminal: (agentId: string) => Promise<ConnectTerminal>;
  openTerminal: (terminal: ConnectTerminal) => void;
}

/** Handle a connect link in this window. */
export async function handleConnect(req: ConnectRequest, deps: ConnectDeps): Promise<void> {
  const terminal = await deps.fetchTerminal(req.agent);
  if (matchesWorkspace(req, currentWorkspaces())) {
    deps.openTerminal(terminal);
    return;
  }
  const pending: PendingConnect = { ...req, terminal, at: Date.now() };
  await deps.state.update(PENDING_CONNECT_KEY, pending);
  const folder = req.authority
    ? vscode.Uri.from({ scheme: "vscode-remote", authority: req.authority, path: req.path })
    : vscode.Uri.file(req.path);
  await vscode.commands.executeCommand("vscode.openFolder", folder, { forceNewWindow: true });
}

/** On activation, finish a connect another window started for this workspace. */
export async function resumePendingConnect(deps: ConnectDeps): Promise<void> {
  const pending = deps.state.get<PendingConnect>(PENDING_CONNECT_KEY);
  if (!pending || !matchesWorkspace(pending, currentWorkspaces())) {
    return;
  }
  await deps.state.update(PENDING_CONNECT_KEY, undefined);
  if (isFresh(pending, Date.now())) {
    deps.openTerminal(pending.terminal);
  }
}

export interface TerminalOps {
  exists(name: string): boolean;
  focus(name: string): void;
  create(options: { name: string; workingDir?: string }): unknown;
  send(name: string, command: string): void;
}

/** Focus the agent's own terminal, or open one running the attach/resume command. */
export function openAgentTerminal(
  ops: TerminalOps,
  terminal: ConnectTerminal,
  warn: (message: string) => void,
): void {
  if (!terminal.command) {
    if (ops.exists(terminal.terminal_name)) {
      ops.focus(terminal.terminal_name);
    } else {
      warn(`Operator: no terminal named '${terminal.terminal_name}' to focus`);
    }
    return;
  }
  const verb = terminal.phase === "resumable" ? "resume" : "attach";
  const name = `${terminal.ticket_id} · ${verb}`;
  ops.create({ name, workingDir: terminal.cwd });
  ops.send(name, terminal.command);
}
