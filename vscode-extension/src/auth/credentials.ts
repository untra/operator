/**
 * Where the extension's daemon credential comes from.
 *
 * Order of preference, per request:
 *
 * 1. The daemon's `local-token` file. A loopback daemon writes it owner-only to
 *    its state directory; the extension host runs as the same OS user, so being
 *    able to read it is the same proof the CLI relies on. It is read fresh
 *    every time because the daemon rotates it on each start.
 * 2. A device-flow access token held in SecretStorage, while it is still valid.
 * 3. A refresh of that token.
 *
 * The provider is module-level state set once in `activate()`. The extension
 * host activates once, so this is a single wiring point rather than a
 * constructor parameter threaded through every client, panel, and section.
 */

import { execFile } from "node:child_process";
import { constants } from "node:fs";
import { promisify } from "node:util";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { TokenRequest, TokenResponse, OAuthErrorResponse } from "../generated";
import { READ_CREDENTIAL_SCRIPT } from "../generated/runtime-scripts";
import type { TokenStore } from "./token-store";

/** The `client_id` device codes and refresh families are bound to server-side. */
export const CLIENT_ID = "vscode";
export const LOCAL_REGISTRATION_FILENAME = "local-registration.json";
/** Treat an access token as expired this long before it actually is. */
export const EXPIRY_SKEW_MS = 30_000;
export const TOKEN_PATH = "/api/v1/auth/token";

/** ts-rs types Rust `u64` as `bigint`, but JSON.parse delivers a number; normalize before arithmetic. */
export function secondsToMs(seconds: bigint | number): number {
  return Number(seconds) * 1000;
}

export interface CredentialProvider {
  /** The best credential currently available, or `undefined` if none. */
  bearer(apiUrl: string): Promise<string | undefined>;
  /** Exchange the stored refresh token after a 401. Returns the new access token. */
  refresh(apiUrl: string): Promise<string | undefined>;
  /** The active `.tickets` directory, used to locate the local token. */
  setTicketsDir(dir: string | undefined): void;
}

let active: CredentialProvider | undefined;

export function setCredentialProvider(provider: CredentialProvider): void {
  active = provider;
}

export function clearCredentialProvider(): void {
  active = undefined;
}

/** Throws rather than silently sending unauthenticated requests when activation forgot to wire one. */
export function credentialProvider(): CredentialProvider {
  if (!active) {
    throw new Error(
      "Operator credential provider is not configured; activate() must call setCredentialProvider()",
    );
  }
  return active;
}

/** Only a loopback daemon issues a local token, and it must never be sent anywhere else. */
export function isLoopbackUrl(apiUrl: string): boolean {
  try {
    const host = new URL(apiUrl).hostname.replace(/^\[|\]$/g, "");
    return host === "localhost" || host === "::1" || /^127\.\d+\.\d+\.\d+$/.test(host);
  } catch {
    return false;
  }
}

async function readPrivateRegistration(file: string): Promise<string | undefined> {
  if (process.platform === "win32") {
    const system = process.env.SystemRoot;
    if (!system) {
      return undefined;
    }
    const { stdout } = await promisify(execFile)(
      path.join(system, "System32", "WindowsPowerShell", "v1.0", "powershell.exe"),
      ["-NoProfile", "-NonInteractive", "-Command", READ_CREDENTIAL_SCRIPT],
      { env: { ...process.env, OPERATOR_PRIVATE_FILE: file }, windowsHide: true },
    );
    return stdout;
  }
  const handle = await fs.open(
    file,
    constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
  );
  try {
    const stat = await handle.stat();
    if (
      !stat.isFile() ||
      stat.uid !== process.getuid?.() ||
      (stat.mode & 0o077) !== 0 ||
      stat.size > 4096
    ) {
      return undefined;
    }
    return await handle.readFile("utf8");
  } finally {
    await handle.close();
  }
}

export async function readLocalToken(
  ticketsDir: string,
  apiUrl: string,
): Promise<string | undefined> {
  try {
    if (!isLoopbackUrl(apiUrl)) {
      return undefined;
    }
    const raw = await readPrivateRegistration(
      path.join(ticketsDir, "operator", LOCAL_REGISTRATION_FILENAME),
    );
    if (!raw) {
      return undefined;
    }
    const registration = JSON.parse(raw) as {
      origin?: string;
      token?: string;
      pid?: number;
      tickets?: string;
    };
    if (
      registration.origin !== apiUrl ||
      !registration.pid ||
      !Number.isSafeInteger(registration.pid) ||
      registration.pid < 1
    ) {
      return undefined;
    }
    if (
      !registration.tickets ||
      path.toNamespacedPath(registration.tickets) !==
        path.toNamespacedPath(await fs.realpath(ticketsDir))
    ) {
      return undefined;
    }
    process.kill(registration.pid, 0);
    return typeof registration.token === "string" && registration.token.trim()
      ? registration.token
      : undefined;
  } catch {
    return undefined;
  }
}

export class OperatorCredentials implements CredentialProvider {
  private ticketsDir: string | undefined;
  /** One refresh in flight per daemon: a second concurrent redemption of the same rotating token would revoke the whole family. */
  private readonly inflight = new Map<string, Promise<string | undefined>>();

  constructor(private readonly store: TokenStore) {}

  setTicketsDir(dir: string | undefined): void {
    this.ticketsDir = dir;
  }

  async bearer(apiUrl: string): Promise<string | undefined> {
    if (this.ticketsDir && isLoopbackUrl(apiUrl)) {
      const local = await readLocalToken(this.ticketsDir, apiUrl);
      if (local) {
        return local;
      }
    }

    const stored = await this.store.load(apiUrl);
    if (!stored) {
      return undefined;
    }
    if (stored.expires_at - Date.now() > EXPIRY_SKEW_MS) {
      return stored.access_token;
    }
    return this.refresh(apiUrl);
  }

  refresh(apiUrl: string): Promise<string | undefined> {
    const pending = this.inflight.get(apiUrl);
    if (pending) {
      return pending;
    }
    const run = this.redeem(apiUrl).finally(() => this.inflight.delete(apiUrl));
    this.inflight.set(apiUrl, run);
    return run;
  }

  private async redeem(apiUrl: string): Promise<string | undefined> {
    const stored = await this.store.load(apiUrl);
    if (!stored) {
      return undefined;
    }

    const body: TokenRequest = {
      grant_type: "refresh_token",
      refresh_token: stored.refresh_token,
      client_id: CLIENT_ID,
    };
    let response: Response;
    try {
      response = await fetch(`${apiUrl}${TOKEN_PATH}`, {
        method: "POST",
        redirect: "error",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
    } catch {
      // Transient network failure: keep the stored credential for next time.
      return undefined;
    }

    if (response.ok) {
      const token = (await response.json()) as TokenResponse;
      await this.store.save(apiUrl, {
        access_token: token.access_token,
        refresh_token: token.refresh_token ?? stored.refresh_token,
        expires_at: Date.now() + secondsToMs(token.expires_in),
        scopes: token.scopes,
      });
      return token.access_token;
    }

    const error = (await response.json().catch(() => ({}))) as Partial<OAuthErrorResponse>;
    if (error.error === "invalid_grant" || error.error === "invalid_client") {
      // The family is dead (expired, revoked, or reuse-detected); nothing to retry with.
      await this.store.clear(apiUrl);
    }
    return undefined;
  }
}
