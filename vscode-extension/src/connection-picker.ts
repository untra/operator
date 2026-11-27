import * as vscode from "vscode";
import {
  normalizeFailureMessage,
  normalizeRemoteOrigin,
  CONNECTION_MODE_REMOTE,
  PICK_LOCAL,
  PICK_REMOTE,
} from "./connection";

const PICK_OFF = "Off";

export interface ConnectionPickerDeps {
  userMode?: string;
  userUrl?: string;
  suggestion?: string;
  saveOff(): Promise<void>;
  saveLocal(): Promise<void>;
  saveRemote(origin: string, workspace: boolean): Promise<void>;
}

export async function runConnectionPicker(
  deps: ConnectionPickerDeps,
  preferred?: typeof PICK_LOCAL | typeof PICK_REMOTE,
): Promise<void> {
  const local = {
    label: PICK_LOCAL,
    description: "Use the Operator configuration on this computer",
  };
  const remote = {
    label: PICK_REMOTE,
    description: "Connect to a hosted Operator server",
  };
  const off = {
    label: PICK_OFF,
    description: "Keep Operator off for this workspace",
  };
  const choice =
    preferred ??
    (
      await vscode.window.showQuickPick(
        deps.userMode === CONNECTION_MODE_REMOTE ? [remote, local, off] : [local, remote, off],
        {
          title: `Operator connection for ${vscode.workspace.name ?? "this window"}`,
          placeHolder: "Choose how to use Operator in this workspace",
        },
      )
    )?.label;
  if (choice === PICK_OFF) {
    await deps.saveOff();
    return;
  }
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
    title: "Connect to a remote Operator server",
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
