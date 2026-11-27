/**
 * `Operator: Sign In` / `Operator: Sign Out`.
 *
 * Sign-in is deliberately a command rather than something a 401 triggers.
 * The browser hand-off is a deliberate act the user starts.
 */

import * as vscode from "vscode";
import { OperatorApiClient } from "../api-client";
import { approvalTarget, VerificationOriginError } from "../connection";
import type { CredentialProvider } from "./credentials";
import type { DeviceFlowOutcome } from "./device-flow";
import { runDeviceFlow } from "./device-flow";
import type { DeviceAuthorizationResponse } from "../generated";
import type { TokenStore } from "./token-store";

export { VerificationOriginError };

const APPROVAL_PAGE_TIMEOUT_MS = 5_000;
const PLACEHOLDER_MARKER = "operator:placeholder";

export interface PendingApproval {
  code: string;
  address: string;
  reopen(): Promise<void>;
  cancel(): void;
}

let pending: PendingApproval | undefined;

export function pendingApproval(): PendingApproval | undefined {
  return pending;
}

/** Open the device-approval page. Loopback URLs are port-forwarded; a public origin is opened unchanged. */
export async function openApprovalPage(apiUrl: string, verificationUri: string): Promise<boolean> {
  const target = approvalTarget(apiUrl, verificationUri);
  if (!target.ok) {
    throw new VerificationOriginError(target.message);
  }
  const uri = vscode.Uri.parse(target.url);
  const external = target.portForward ? await vscode.env.asExternalUri(uri) : uri;
  return vscode.env.openExternal(external);
}

async function approvalAddress(
  apiUrl: string,
  authorization: DeviceAuthorizationResponse,
  signal: AbortSignal,
): Promise<string> {
  const manual = approvalTarget(apiUrl, authorization.verification_uri);
  const complete = approvalTarget(apiUrl, authorization.verification_uri_complete);
  if (!manual.ok) {
    throw new VerificationOriginError(manual.message);
  }
  if (!complete.ok) {
    throw new VerificationOriginError(complete.message);
  }
  const response = await fetch(manual.url, {
    redirect: "manual",
    signal: AbortSignal.any([AbortSignal.timeout(APPROVAL_PAGE_TIMEOUT_MS), signal]),
  });
  if (
    response.status === 404 ||
    (response.ok &&
      (!response.headers.get("content-type")?.includes("text/html") ||
        (await response.text()).includes(PLACEHOLDER_MARKER)))
  ) {
    throw new Error(
      `The approval page is missing on ${manual.host}. Ask the administrator to serve the built Operator web UI at this origin.`,
    );
  }
  const uri = vscode.Uri.parse(manual.url);
  return manual.portForward ? (await vscode.env.asExternalUri(uri)).toString(true) : manual.url;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** A credential that already works means the browser round-trip is unnecessary. */
export async function alreadyAuthenticated(
  apiUrl: string,
  credentials: CredentialProvider,
): Promise<boolean> {
  if (!(await credentials.bearer(apiUrl))) {
    return false;
  }
  try {
    await new OperatorApiClient(apiUrl).currentSession();
    return true;
  } catch {
    return false;
  }
}

export function describeOutcome(apiUrl: string, outcome: DeviceFlowOutcome): string {
  switch (outcome.status) {
    case "approved":
      return `Signed in to Operator at ${apiUrl}.`;
    case "denied":
      return "Sign-in was declined in the browser.";
    case "expired":
      return "The sign-in code expired before it was approved. Run Operator: Sign In again.";
    case "cancelled":
      return "Sign-in cancelled.";
    case "error":
      return `Sign-in failed: ${outcome.message}`;
    default: {
      const exhaustive: never = outcome;
      throw new Error(`Unknown device flow outcome: ${JSON.stringify(exhaustive)}`);
    }
  }
}

export async function signIn(
  apiUrl: string,
  credentials: CredentialProvider,
  store: TokenStore,
  signal?: AbortSignal,
  redraw: () => void = () => {},
): Promise<DeviceFlowOutcome | undefined> {
  if (!apiUrl || signal?.aborted) {
    return undefined;
  }
  if (await alreadyAuthenticated(apiUrl, credentials)) {
    if (signal?.aborted) {
      return undefined;
    }
    void vscode.window.showInformationMessage(
      `Already authenticated with Operator at ${apiUrl}; no sign-in needed.`,
    );
    return undefined;
  }

  let outcome: DeviceFlowOutcome;
  let approval: PendingApproval | undefined;
  const controller = new AbortController();
  const active = signal ? AbortSignal.any([controller.signal, signal]) : controller.signal;
  const clear = () => {
    if (approval && pending === approval) {
      pending = undefined;
      redraw();
    }
  };
  const cancel = () => {
    controller.abort();
  };
  active.addEventListener("abort", clear, { once: true });
  try {
    outcome = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: "Operator sign-in",
        cancellable: true,
      },
      async (progress, token) => {
        const listener = token.onCancellationRequested(cancel);
        try {
          return await runDeviceFlow(apiUrl, store, {
            onCode: async (authorization) => {
              const address = await approvalAddress(apiUrl, authorization, active);
              if (active.aborted) {
                return;
              }
              const reopen = async () => {
                if (pending !== approval || active.aborted) {
                  return;
                }
                const opened = await openApprovalPage(
                  apiUrl,
                  authorization.verification_uri_complete,
                ).catch(() => false);
                if (!opened && !active.aborted && pending === approval) {
                  void vscode.window.showWarningMessage(
                    `Open ${address} in your browser and enter ${authorization.user_code}. You can reopen it from Operator Connections.`,
                  );
                }
              };
              approval = { code: authorization.user_code, address, reopen, cancel };
              pending = approval;
              redraw();
              progress.report({
                message: `Compare code ${authorization.user_code} on ${new URL(apiUrl).host}, then select Approve in your browser. Reopen the page from Operator Connections.`,
              });
              await reopen();
            },
            isCancelled: () => active.aborted || token.isCancellationRequested,
            sleep,
            now: Date.now,
          });
        } finally {
          listener.dispose();
        }
      },
    );
  } catch (err) {
    outcome = active.aborted
      ? { status: "cancelled" }
      : { status: "error", message: err instanceof Error ? err.message : String(err) };
  } finally {
    active.removeEventListener("abort", clear);
    clear();
  }

  if (signal?.aborted) {
    return undefined;
  }
  const message = describeOutcome(apiUrl, outcome);
  if (outcome.status === "approved") {
    void vscode.window.showInformationMessage(message);
  } else if (outcome.status === "expired") {
    void vscode.window.showWarningMessage(message, "Start Again").then((choice) => {
      if (choice && !signal?.aborted) {
        return vscode.commands.executeCommand("operator.signIn");
      }
      return undefined;
    });
  } else if (outcome.status !== "cancelled") {
    void vscode.window.showErrorMessage(message);
  }
  return outcome;
}

export async function signOut(apiUrl: string, store: TokenStore): Promise<void> {
  await store.clear(apiUrl);
  void vscode.window.showInformationMessage(`Signed out of Operator at ${apiUrl}.`);
}
