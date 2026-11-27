/**
 * How this window reaches Operator.
 *
 * `local` is the free install on this machine. `remote` is a server the user
 * already runs. Remote mode never downloads, updates, or starts a binary.
 */

import { isLoopbackUrl } from "./auth/credentials";

export const CONNECTION_MODE_LOCAL = "local";
export const CONNECTION_MODE_REMOTE = "remote";

export const CHOOSE_CONNECTION_COMMAND = "operator.chooseConnection";
export const DOWNLOADS_PAGE = "https://operator.untra.io/downloads/";

export const PICK_LOCAL = "Use Operator on this computer";
export const PICK_REMOTE = "Connect to Operator server";

export type NormalizeReason =
  | "empty"
  | "invalid"
  | "http"
  | "userinfo"
  | "query"
  | "fragment"
  | "path"
  | "loopback";

export function normalizeRemoteOrigin(
  input: string,
): { ok: true; origin: string } | { ok: false; reason: NormalizeReason } {
  const trimmed = input.trim();
  if (!trimmed) {
    return { ok: false, reason: "empty" };
  }
  const raw = trimmed.includes("://") ? trimmed : `https://${trimmed}`;
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
  if (raw.includes("?")) {
    return { ok: false, reason: "query" };
  }
  if (raw.includes("#")) {
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

export const APPROVED_WORKSPACE_ORIGIN = "operator.approvedWorkspaceOrigin";

export type ConnectionSelection =
  | { kind: "undecided" }
  | { kind: "invalid"; reason: NormalizeReason }
  | { kind: "local"; origin?: string }
  | { kind: "hosted"; origin: string };

export function resolveConnection(input: {
  userMode?: string;
  userUrl?: string;
  suggestion?: string;
  approvedOrigin?: string;
}): ConnectionSelection {
  const suggestion = normalizeRemoteOrigin(input.suggestion ?? "");
  if (suggestion.ok && suggestion.origin === input.approvedOrigin) {
    return { kind: "hosted", origin: suggestion.origin };
  }
  const userUrl = input.userUrl?.trim();
  if (userUrl && input.userMode !== CONNECTION_MODE_REMOTE && isLoopbackUrl(userUrl)) {
    const url = new URL(userUrl);
    if (
      (url.protocol !== "http:" && url.protocol !== "https:") ||
      url.username ||
      url.password ||
      url.pathname !== "/" ||
      url.search ||
      url.hash
    ) {
      return { kind: "invalid", reason: "invalid" };
    }
    return { kind: "local", origin: url.origin };
  }
  if (userUrl || input.userMode === CONNECTION_MODE_REMOTE) {
    const hosted = normalizeRemoteOrigin(userUrl ?? "");
    return hosted.ok
      ? { kind: "hosted", origin: hosted.origin }
      : { kind: "invalid", reason: hosted.reason };
  }
  return input.userMode === CONNECTION_MODE_LOCAL ? { kind: "local" } : { kind: "undecided" };
}

let selection: ConnectionSelection = { kind: "undecided" };
let origin = "";
let controller = new AbortController();

export function resetConnection(next: ConnectionSelection): void {
  controller.abort();
  controller = new AbortController();
  selection = next;
  origin = "";
}

export function setConnectionOrigin(url: string): void {
  origin = url;
}

export function currentConnection() {
  return { selection, origin, signal: controller.signal };
}
