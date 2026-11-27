import * as vscode from "vscode";
import {
  normalizeFailureMessage,
  normalizeRemoteOrigin,
  PICK_LOCAL,
  PICK_REMOTE,
} from "./connection";

export interface ConnectionPickerDeps {
  userUrl?: string;
  suggestion?: string;
  saveLocal(): Promise<void>;
  saveRemote(origin: string, workspace: boolean): Promise<void>;
}

export async function runConnectionPicker(
  deps: ConnectionPickerDeps,
  preferred?: typeof PICK_LOCAL | typeof PICK_REMOTE,
): Promise<void> {
  const choice =
    preferred ??
    (await vscode.window.showInformationMessage(
      "Choose how this window connects to Operator.",
      PICK_LOCAL,
      PICK_REMOTE,
    ));
  if (choice === PICK_LOCAL) {
    await deps.saveLocal();
    return;
  }
  if (choice !== PICK_REMOTE) {
    return;
  }

  const suggestion = normalizeRemoteOrigin(deps.suggestion ?? "");
  if (suggestion.ok) {
    const useSuggested = "Use This Server";
    const enterAddress = "Enter Another Address";
    const confirmed = await vscode.window.showInformationMessage(
      `This workspace suggests ${suggestion.origin} as its Operator server.`,
      { modal: true },
      useSuggested,
      enterAddress,
    );
    if (confirmed === useSuggested) {
      await deps.saveRemote(suggestion.origin, true);
      return;
    }
    if (confirmed !== enterAddress) {
      return;
    }
  }

  const entered = await vscode.window.showInputBox({
    title: PICK_REMOTE,
    prompt: "HTTPS origin of your organization's Operator server",
    value: deps.userUrl,
    ignoreFocusOut: true,
    validateInput: (input) => {
      const result = normalizeRemoteOrigin(input);
      return result.ok ? undefined : normalizeFailureMessage(result.reason);
    },
  });
  if (entered === undefined) {
    return;
  }
  const result = normalizeRemoteOrigin(entered);
  if (result.ok) {
    await deps.saveRemote(result.origin, false);
  }
}
