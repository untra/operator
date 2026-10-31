/**
 * `Operator: Sign In` / `Operator: Sign Out`.
 *
 * Sign-in is deliberately a command rather than something a 401 triggers: the
 * browser hand-off is a deliberate act the user starts, not a surprise.
 */

import * as vscode from "vscode";
import { OperatorApiClient } from "../api-client";
import { approvalTarget, VerificationOriginError } from "../connection";
import type { CredentialProvider } from "./credentials";
import type { DeviceFlowOutcome } from "./device-flow";
import { runDeviceFlow } from "./device-flow";
import type { TokenStore } from "./token-store";

export { VerificationOriginError };

const COPY_CODE = "Copy code";

/** Open the device-approval page. Loopback URLs are port-forwarded; a public origin is opened unchanged. */
export async function openApprovalPage(apiUrl: string, verificationUri: string): Promise<void> {
  const target = approvalTarget(apiUrl, verificationUri);
  if (!target.ok) {
    throw new VerificationOriginError(target.message);
  }
  const uri = vscode.Uri.parse(target.url);
  const external = target.portForward ? await vscode.env.asExternalUri(uri) : uri;
  await vscode.env.openExternal(external);
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
): Promise<DeviceFlowOutcome | undefined> {
  if (await alreadyAuthenticated(apiUrl, credentials)) {
    void vscode.window.showInformationMessage(
      `Already authenticated with Operator at ${apiUrl}; no sign-in needed.`,
    );
    return undefined;
  }

  let outcome: DeviceFlowOutcome;
  try {
    outcome = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: "Operator sign-in",
        cancellable: true,
      },
      (progress, token) =>
        runDeviceFlow(apiUrl, store, {
          onCode: async (authorization) => {
            const target = approvalTarget(apiUrl, authorization.verification_uri_complete);
            const host = target.ok ? target.host : apiUrl;
            progress.report({
              message: `Approve code ${authorization.user_code} on ${host}`,
            });
            await openApprovalPage(apiUrl, authorization.verification_uri_complete);
            void vscode.window
              .showInformationMessage(
                `Approve Operator sign-in code ${authorization.user_code} on ${host}`,
                COPY_CODE,
              )
              .then((choice) => {
                if (choice === COPY_CODE) {
                  return vscode.env.clipboard.writeText(authorization.user_code);
                }
                return undefined;
              });
          },
          isCancelled: () => token.isCancellationRequested,
          sleep,
          now: Date.now,
        }),
    );
  } catch (err) {
    if (err instanceof VerificationOriginError) {
      return { status: "error", message: err.message };
    }
    throw err;
  }

  const message = describeOutcome(apiUrl, outcome);
  if (outcome.status === "approved") {
    void vscode.window.showInformationMessage(message);
  } else if (outcome.status !== "cancelled") {
    void vscode.window.showErrorMessage(message);
  }
  return outcome;
}

export async function signOut(apiUrl: string, store: TokenStore): Promise<void> {
  await store.clear(apiUrl);
  void vscode.window.showInformationMessage(`Signed out of Operator at ${apiUrl}.`);
}
