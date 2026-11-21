/**
 * How this window reaches Operator.
 *
 * `local` is the free install on this machine. `remote` is a server the user
 * already runs. Remote mode never downloads, updates, or starts a binary.
 */

import { isLoopbackUrl } from "./auth/credentials";

export const CONNECTION_MODE_LOCAL = "local";
export const CONNECTION_MODE_REMOTE = "remote";
export type ConnectionMode = typeof CONNECTION_MODE_LOCAL | typeof CONNECTION_MODE_REMOTE;

export const CHOOSE_CONNECTION_COMMAND = "operator.chooseConnection";
export const CONNECTION_CHOSEN_STATE = "operator.connectionChosen";
export const DOWNLOADS_PAGE = "https://operator.untra.io/downloads/";

export const PICKER_MESSAGE =
  "This editor is free. Running Operator on this machine is the free local install. " +
  "A server you connect to is one you operate; SSH, Coder, and other remote targets " +
  "are entitled by that server's license.";

export const PICK_RUNNING = "Running on this machine";
export const PICK_HERE = "Run for this folder";
export const PICK_REMOTE = "Connect to a server";

export type NormalizeReason =
  | "empty"
  | "invalid"
  | "http"
  | "userinfo"
  | "query"
  | "fragment"
  | "path"
  | "loopback";

export type StartupAction =
  | "attach"
  | "sign-in"
  | "start-local"
  | "report-unreachable"
  | "prompt-choose"
  | "idle";

interface Inspected<T> {
  globalValue?: T;
  workspaceValue?: T;
  workspaceFolderValue?: T;
}

export interface OperatorConnectionSettings {
  mode: ConnectionMode;
  explicitApiUrl: string | undefined;
}

export function inspectedValue<T>(inspected: Inspected<T> | undefined): T | undefined {
  return inspected?.workspaceFolderValue ?? inspected?.workspaceValue ?? inspected?.globalValue;
}

export function connectionModeFromInspect(
  inspected: Inspected<string> | undefined,
): ConnectionMode {
  return inspectedValue(inspected) === CONNECTION_MODE_REMOTE
    ? CONNECTION_MODE_REMOTE
    : CONNECTION_MODE_LOCAL;
}

export function readConnectionSettings(config: {
  inspect<T>(section: string): Inspected<T> | undefined;
}): OperatorConnectionSettings {
  return {
    mode: connectionModeFromInspect(config.inspect<string>("connectionMode")),
    explicitApiUrl: inspectedValue(config.inspect<string>("apiUrl")),
  };
}

/** Remote mode, or any saved non-loopback URL, uses the remote rules. */
export function effectiveMode(mode: ConnectionMode, apiUrl: string | undefined): ConnectionMode {
  if (mode === CONNECTION_MODE_REMOTE) {
    return CONNECTION_MODE_REMOTE;
  }
  if (apiUrl && !isLoopbackUrl(apiUrl)) {
    return CONNECTION_MODE_REMOTE;
  }
  return CONNECTION_MODE_LOCAL;
}

export function usingRemoteServer(settings: OperatorConnectionSettings): boolean {
  return effectiveMode(settings.mode, settings.explicitApiUrl) === CONNECTION_MODE_REMOTE;
}

export function downloadsApply(settings: OperatorConnectionSettings): boolean {
  return !usingRemoteServer(settings);
}

export function normalizeRemoteOrigin(
  input: string,
): { ok: true; origin: string } | { ok: false; reason: NormalizeReason } {
  const trimmed = input.trim();
  if (!trimmed) {
    return { ok: false, reason: "empty" };
  }
  const raw = /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmed) ? trimmed : `https://${trimmed}`;
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    return { ok: false, reason: "invalid" };
  }
  if (url.username || url.password) {
    return { ok: false, reason: "userinfo" };
  }
  if (isLoopbackUrl(url.origin)) {
    return { ok: false, reason: "loopback" };
  }
  if (url.protocol === "http:") {
    return { ok: false, reason: "http" };
  }
  if (url.protocol !== "https:") {
    return { ok: false, reason: "invalid" };
  }
  if (url.search) {
    return { ok: false, reason: "query" };
  }
  if (url.hash) {
    return { ok: false, reason: "fragment" };
  }
  if (url.pathname !== "/" && url.pathname !== "") {
    return { ok: false, reason: "path" };
  }
  return { ok: true, origin: url.origin };
}

export function normalizeFailureMessage(reason: NormalizeReason): string {
  switch (reason) {
    case "empty":
      return "Enter the server address.";
    case "invalid":
      return "That address is not a URL.";
    case "http":
      return "A remote Operator must use https.";
    case "userinfo":
      return "Enter the address without a username or password. Sign in after the URL is saved.";
    case "query":
    case "fragment":
    case "path":
      return "Enter only the server origin, for example https://operator.example.com.";
    case "loopback":
      return "Loopback addresses stay in local mode. Pick Run for this folder, or enter the server's public https address.";
    default: {
      const exhaustive: never = reason;
      return exhaustive;
    }
  }
}

export function serverOrigin(apiUrl: string): string | undefined {
  try {
    return new URL(apiUrl).origin;
  } catch {
    return undefined;
  }
}

export function swaggerUrl(apiUrl: string): string | undefined {
  const origin = serverOrigin(apiUrl);
  return origin ? `${origin}/swagger-ui` : undefined;
}

export function startupAction(input: {
  mode: ConnectionMode;
  apiUrl: string;
  reachable: boolean;
  authenticated: boolean;
  autoStart: boolean;
  hasConfig: boolean;
}): StartupAction {
  if (input.reachable && input.authenticated) {
    return "attach";
  }
  if (input.reachable && !input.authenticated) {
    return "sign-in";
  }
  const remote = effectiveMode(input.mode, input.apiUrl || undefined) === CONNECTION_MODE_REMOTE;
  if (remote) {
    return input.apiUrl ? "report-unreachable" : "idle";
  }
  if (input.autoStart && input.hasConfig) {
    return "start-local";
  }
  if (input.autoStart && !input.hasConfig) {
    return "prompt-choose";
  }
  return "idle";
}

export function shouldStartWebhook(
  mode: ConnectionMode,
  apiUrl: string,
  hasConfig: boolean,
): boolean {
  return effectiveMode(mode, apiUrl || undefined) === CONNECTION_MODE_LOCAL && hasConfig;
}

/** Local adopt rules. Remote servers are not compared to a laptop binary or folder name. */
export function localAdoptMismatch(input: {
  remote: boolean;
  expectedVersion?: string;
  foundVersion?: string;
  localDirectoryName?: string;
  foundDirectoryName?: string;
}): { kind: "version" | "project"; detail: string } | undefined {
  if (input.remote) {
    return undefined;
  }
  if (input.expectedVersion && input.foundVersion && input.expectedVersion !== input.foundVersion) {
    return {
      kind: "version",
      detail: `running v${input.foundVersion}, this binary is v${input.expectedVersion}`,
    };
  }
  if (
    input.localDirectoryName &&
    input.foundDirectoryName &&
    input.localDirectoryName !== input.foundDirectoryName
  ) {
    return {
      kind: "project",
      detail: `serves '${input.foundDirectoryName}', this workspace is '${input.localDirectoryName}'`,
    };
  }
  return undefined;
}

export class VerificationOriginError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "VerificationOriginError";
  }
}

export function approvalTarget(
  apiUrl: string,
  verificationUri: string,
): { ok: true; url: string; host: string; portForward: boolean } | { ok: false; message: string } {
  let api: URL;
  let verification: URL;
  try {
    api = new URL(apiUrl);
    verification = new URL(verificationUri);
  } catch {
    return { ok: false, message: "The server sent a sign-in link that is not a URL." };
  }
  if (verification.origin !== api.origin) {
    return {
      ok: false,
      message:
        `The server sent a sign-in link on ${verification.origin}, which is not ${api.origin}. ` +
        `Set [rest_api].public_url on that server to ${api.origin}.`,
    };
  }
  return {
    ok: true,
    url: verificationUri,
    host: verification.host,
    portForward: isLoopbackUrl(api.origin),
  };
}

export function gateRemoteMcp(
  apiOrigin: string,
  transportUrl: string,
): { ok: true; transportUrl: string } | { ok: false; message: string } {
  let api: URL;
  let transport: URL;
  try {
    api = new URL(apiOrigin);
    transport = new URL(transportUrl);
  } catch {
    return {
      ok: false,
      message:
        "The MCP address is not a URL. Set [rest_api].public_url on the server to its public https origin.",
    };
  }
  const publicMismatch =
    transport.protocol !== "https:" ||
    isLoopbackUrl(transport.origin) ||
    transport.origin !== api.origin;
  if (publicMismatch) {
    return {
      ok: false,
      message:
        `The MCP address ${transportUrl} is not ${api.origin}. ` +
        "Set [rest_api].public_url on the server to that public https origin.",
    };
  }
  return { ok: true, transportUrl };
}
