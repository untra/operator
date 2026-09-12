/**
 * `Operator: Sign In` / `Operator: Sign Out`.
 *
 * Sign-in is deliberately a command rather than something a 401 triggers: the
 * browser hand-off is a deliberate act the user starts, not a surprise.
 */

import * as vscode from 'vscode';
import { OperatorApiClient } from '../api-client';
import type { CredentialProvider } from './credentials';
import type { DeviceFlowOutcome} from './device-flow';
import { runDeviceFlow } from './device-flow';
import type { TokenStore } from './token-store';

const COPY_CODE = 'Copy code';

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** A credential that already works means the browser round-trip is unnecessary. */
export async function alreadyAuthenticated(
  apiUrl: string,
  credentials: CredentialProvider
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
    case 'approved':
      return `Signed in to Operator at ${apiUrl}.`;
    case 'denied':
      return 'Sign-in was declined in the browser.';
    case 'expired':
      return 'The sign-in code expired before it was approved. Run Operator: Sign In again.';
    case 'cancelled':
      return 'Sign-in cancelled.';
    case 'error':
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
  store: TokenStore
): Promise<DeviceFlowOutcome | undefined> {
  if (await alreadyAuthenticated(apiUrl, credentials)) {
    void vscode.window.showInformationMessage(
      `Already authenticated with Operator at ${apiUrl}; no sign-in needed.`
    );
    return undefined;
  }

  const outcome = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Operator sign-in',
      cancellable: true,
    },
    (progress, token) =>
      runDeviceFlow(apiUrl, store, {
        onCode: async (authorization) => {
          progress.report({
            message: `Approve code ${authorization.user_code} in the browser`,
          });
          const external = await vscode.env.asExternalUri(
            vscode.Uri.parse(authorization.verification_uri_complete)
          );
          await vscode.env.openExternal(external);
          void vscode.window
            .showInformationMessage(
              `Operator sign-in code: ${authorization.user_code}`,
              COPY_CODE
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
      })
  );

  const message = describeOutcome(apiUrl, outcome);
  if (outcome.status === 'approved') {
    void vscode.window.showInformationMessage(message);
  } else if (outcome.status !== 'cancelled') {
    void vscode.window.showErrorMessage(message);
  }
  return outcome;
}

export async function signOut(apiUrl: string, store: TokenStore): Promise<void> {
  await store.clear(apiUrl);
  void vscode.window.showInformationMessage(`Signed out of Operator at ${apiUrl}.`);
}
