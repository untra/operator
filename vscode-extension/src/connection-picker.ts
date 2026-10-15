/**
 * First-run choice: attach to a local Operator, run one for this folder, or
 * save a remote origin. The command persists the choice; this module only
 * sequences the prompts.
 */

import * as vscode from "vscode";
import {
  PICK_HERE,
  PICK_REMOTE,
  PICK_RUNNING,
  PICKER_MESSAGE,
  normalizeFailureMessage,
  normalizeRemoteOrigin,
} from "./connection";

export type ProbeResult = "ok" | "auth" | "down";

export interface ConnectionPickerDeps {
  currentRemoteUrl(): string;
  discoverLocal(): Promise<string>;
  probe(url: string): Promise<ProbeResult>;
  saveLocal(): Promise<void>;
  saveRemote(origin: string): Promise<void>;
  markChosen(): Promise<void>;
  signIn(url: string): Promise<void>;
  runLocal(): Promise<void>;
}

export async function runConnectionPicker(deps: ConnectionPickerDeps): Promise<void> {
  const choice = await vscode.window.showInformationMessage(
    PICKER_MESSAGE,
    PICK_RUNNING,
    PICK_HERE,
    PICK_REMOTE,
  );

  if (choice === PICK_RUNNING) {
    await attachRunningOperator(deps);
    return;
  }
  if (choice === PICK_HERE) {
    await deps.saveLocal();
    await deps.markChosen();
    await deps.runLocal();
    return;
  }
  if (choice === PICK_REMOTE) {
    await connectRemote(deps);
  }
}

async function attachRunningOperator(deps: ConnectionPickerDeps): Promise<void> {
  const url = await deps.discoverLocal();
  const probe = await deps.probe(url);
  if (probe === "down") {
    const again = await vscode.window.showWarningMessage(
      `No Operator is listening at ${url}.`,
      PICK_HERE,
    );
    if (again !== PICK_HERE) {
      return;
    }
    await deps.saveLocal();
    await deps.markChosen();
    await deps.runLocal();
    return;
  }
  await deps.saveLocal();
  await deps.markChosen();
  if (probe === "auth") {
    await deps.signIn(url);
    return;
  }
  void vscode.window.showInformationMessage(`Using the Operator already running at ${url}.`);
}

async function connectRemote(deps: ConnectionPickerDeps): Promise<void> {
  const entered = await vscode.window.showInputBox({
    title: "Operator server",
    prompt: "https address of the Operator you already run",
    value: deps.currentRemoteUrl(),
    ignoreFocusOut: true,
  });
  if (!entered) {
    return;
  }
  const normalized = normalizeRemoteOrigin(entered);
  if (!normalized.ok) {
    void vscode.window.showErrorMessage(normalizeFailureMessage(normalized.reason));
    return;
  }
  await deps.saveRemote(normalized.origin);
  await deps.markChosen();
  const probe = await deps.probe(normalized.origin);
  if (probe === "down") {
    void vscode.window.showWarningMessage(`Can't reach Operator at ${normalized.origin}.`);
    return;
  }
  if (probe === "auth") {
    await deps.signIn(normalized.origin);
    return;
  }
  void vscode.window.showInformationMessage(`Connected to Operator at ${normalized.origin}.`);
}
