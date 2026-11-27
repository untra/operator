import * as vscode from "vscode";
import { currentConnection } from "../../src/connection";
import { waitFor } from "./hosted-identity";

export async function configureEditor(origin: string) {
  await vscode.extensions.getExtension("untra.operator-terminals")!.activate();
  await vscode.workspace
    .getConfiguration("operator")
    .update("apiUrl", origin, vscode.ConfigurationTarget.Global);
  await waitFor(() => currentConnection().origin === origin || undefined, "hosted origin");
}
