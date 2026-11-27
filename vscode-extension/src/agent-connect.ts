import * as vscode from "vscode";
import type { AttachReference, ConnectTerminal, TerminalAction } from "./generated";

export interface ConnectRequest {
  agent: string;
}

export interface PendingConnect extends ConnectRequest {
  attach: AttachReference;
  origin: string;
  at: number;
}

export interface WorkspaceRef {
  authority?: string;
  path: string;
}

export const PENDING_CONNECT_KEY = "operator.pendingConnect";
const REFERENCE_PREFIX = `${PENDING_CONNECT_KEY}.`;
const PENDING_TTL_MS = 2 * 60 * 1000;

export function parseConnect(params: URLSearchParams): ConnectRequest | undefined {
  const agent = params.get("agent");
  return agent ? { agent } : undefined;
}

export function matchesWorkspace(req: WorkspaceRef, folders: readonly WorkspaceRef[]): boolean {
  return folders.some(
    (f) => f.authority === req.authority && trimSlash(f.path) === trimSlash(req.path),
  );
}

export function isFresh(pending: PendingConnect, now: number): boolean {
  const age = now - pending.at;
  return age >= 0 && age < PENDING_TTL_MS;
}

function trimSlash(p: string): string {
  return p.length > 1 ? p.replace(/\/+$/, "") : p;
}

function currentWorkspaces(): WorkspaceRef[] {
  return (vscode.workspace.workspaceFolders ?? [])
    .filter((f) => f.uri.scheme === "file" || f.uri.scheme === "vscode-remote")
    .map((f) => ({
      authority: f.uri.scheme === "vscode-remote" ? f.uri.authority : undefined,
      path: f.uri.scheme === "file" ? f.uri.fsPath : f.uri.path,
    }));
}

export interface ConnectDeps {
  state: vscode.Memento;
  connection(): { origin: string; signal: AbortSignal; selection: { kind: string } };
  issue: (agent: string) => Promise<AttachReference>;
  redeem: (agent: string, attach: AttachReference) => Promise<ConnectTerminal>;
  openTerminal: (terminal: ConnectTerminal) => void;
}

function remoteWorkspace(target: WorkspaceRef): boolean {
  return (
    /^ssh-remote\+[a-zA-Z0-9][a-zA-Z0-9_.@:-]*$/.test(target.authority ?? "") &&
    target.path.startsWith("/") &&
    !target.path.includes("\0")
  );
}

function destination(attach: AttachReference): WorkspaceRef {
  return { path: attach.destination.path, authority: attach.destination.authority ?? undefined };
}

async function openAuthorized(pending: PendingConnect, deps: ConnectDeps): Promise<void> {
  const { origin, signal, selection } = deps.connection();
  if (
    !vscode.workspace.isTrusted ||
    !origin ||
    origin !== pending.origin ||
    !isFresh(pending, Date.now())
  ) {
    return;
  }
  const target = destination(pending.attach);
  if (selection.kind === "hosted" && !remoteWorkspace(target)) {
    return;
  }
  if (!matchesWorkspace(target, currentWorkspaces())) {
    return;
  }
  const terminal = await deps.redeem(pending.agent, pending.attach);
  signal.throwIfAborted();
  if (
    !vscode.workspace.isTrusted ||
    !matchesWorkspace(target, currentWorkspaces()) ||
    !terminal.cwd ||
    !matchesWorkspace({ path: terminal.cwd, authority: terminal.authority }, [target])
  ) {
    return;
  }
  deps.openTerminal(terminal);
}

export async function handleConnect(req: ConnectRequest, deps: ConnectDeps): Promise<void> {
  const { origin, signal, selection } = deps.connection();
  if (!origin) {
    throw new Error("Choose an Operator connection first.");
  }
  const attach = await deps.issue(req.agent);
  signal.throwIfAborted();
  const target = destination(attach);
  if (selection.kind === "hosted" && !remoteWorkspace(target)) {
    throw new Error("A hosted terminal requires a remote workspace.");
  }
  const pending = { ...req, attach, origin, at: Date.now() };
  if (matchesWorkspace(target, currentWorkspaces())) {
    await openAuthorized(pending, deps);
    return;
  }
  const key = REFERENCE_PREFIX + attach.reference;
  await deps.state.update(key, pending);
  if (signal.aborted) {
    await deps.state.update(key, undefined);
    return;
  }
  const discard = () => {
    void deps.state.update(key, undefined);
  };
  signal.addEventListener("abort", discard, { once: true });
  const expiry = setTimeout(() => {
    signal.removeEventListener("abort", discard);
    discard();
  }, PENDING_TTL_MS);
  expiry.unref();
  const folder = target.authority
    ? vscode.Uri.from({ scheme: "vscode-remote", authority: target.authority, path: target.path })
    : vscode.Uri.file(target.path);
  try {
    await vscode.commands.executeCommand("vscode.openFolder", folder, { forceNewWindow: true });
  } catch (error) {
    clearTimeout(expiry);
    signal.removeEventListener("abort", discard);
    await deps.state.update(key, undefined);
    throw error;
  }
}

export async function resumePendingConnect(deps: ConnectDeps): Promise<void> {
  if (deps.state.get(PENDING_CONNECT_KEY)) {
    await deps.state.update(PENDING_CONNECT_KEY, undefined);
  }
  const { origin, signal } = deps.connection();
  for (const key of deps.state.keys().filter((k) => k.startsWith(REFERENCE_PREFIX))) {
    const pending = deps.state.get<PendingConnect>(key);
    if (!pending || !isFresh(pending, Date.now())) {
      await deps.state.update(key, undefined);
      continue;
    }
    if (
      signal.aborted ||
      pending.origin !== origin ||
      !matchesWorkspace(destination(pending.attach), currentWorkspaces())
    ) {
      continue;
    }
    await deps.state.update(key, undefined);
    await openAuthorized(pending, deps);
  }
}

export interface TerminalOps {
  exists(name: string): boolean;
  focus(name: string): void;
  create(
    options: { name: string; workingDir?: string },
    process?: { shellPath: string; shellArgs: string[] },
  ): unknown;
}

function terminalProcess(
  action: Exclude<TerminalAction, { kind: "focus" }>,
): { shellPath: string; shellArgs: string[] } | undefined {
  if (!action.session || action.session.startsWith("-") || action.session.includes("\0")) {
    return undefined;
  }
  switch (action.kind) {
    case "tmux":
      return {
        shellPath: "tmux",
        shellArgs: [
          ...(action.socket ? ["-L", action.socket] : []),
          "attach-session",
          "-t",
          `=${action.session}`,
        ],
      };
    case "zellij":
      return { shellPath: "zellij", shellArgs: ["attach", action.session] };
    case "resume":
      if (!["claude", "codex", "gemini"].includes(action.tool)) {
        return undefined;
      }
      return {
        shellPath: action.tool,
        shellArgs: [action.tool === "codex" ? "resume" : "--resume", action.session],
      };
    default:
      return undefined;
  }
}

export function openAgentTerminal(
  ops: TerminalOps,
  terminal: ConnectTerminal,
  warn: (s: string) => void,
): void {
  if (!terminal.action) {
    return warn("Operator: unsupported terminal action");
  }
  if (terminal.action.kind === "focus") {
    if (ops.exists(terminal.terminal_name)) {
      ops.focus(terminal.terminal_name);
    } else {
      warn(`Operator: no terminal named '${terminal.terminal_name}' to focus`);
    }
    return;
  }
  const process = terminalProcess(terminal.action);
  if (!process) {
    return warn("Operator: invalid terminal action");
  }
  const verb = terminal.phase === "resumable" ? "resume" : "attach";
  ops.create({ name: `${terminal.ticket_id} · ${verb}`, workingDir: terminal.cwd }, process);
}
