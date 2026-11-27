import * as sinon from "sinon";
import * as vscode from "vscode";
import { currentConnection } from "../../src/connection";
import { waitFor } from "./hosted-identity";

export async function configureEditor(origin: string) {
  await vscode.extensions.getExtension("untra.operator-terminals")!.activate();
  const enterOrigin = sinon.stub(vscode.window, "showInputBox").resolves(origin);
  try {
    await vscode.commands.executeCommand("operator.chooseHostedConnection");
  } finally {
    enterOrigin.restore();
  }
  await waitFor(() => currentConnection().origin === origin || undefined, "hosted origin");
}
