/**
 * What a palette command does when this window is attached to a remote Operator.
 * Local mode always returns `local` so the existing handler runs.
 */

import * as vscode from "vscode";
import { CHOOSE_CONNECTION_COMMAND, readConnectionSettings, usingRemoteServer } from "./connection";
import { openOperatorUi, type OperatorUiRoute } from "./open-operator-ui";

export type RemoteCommandTarget =
  | { kind: "local" }
  | { kind: "ui"; route: OperatorUiRoute }
  | { kind: "message"; message: string; chooseConnection: boolean };

const UI_ROUTES: Record<string, OperatorUiRoute> = {
  "operator.runSetup": "setup",
  "operator.revealTicketsDir": "queue",
  "operator.openSettings": "config",
  "operator.openCreateDelegator": "delegators",
  "operator.detectLlmTools": "llm",
  "operator.startGitOnboarding": "git",
  "operator.configureGitHub": "git",
  "operator.configureGitLab": "git",
  "operator.checkKanbanConnection": "kanban",
};

const REFUSALS: Record<string, { message: string; chooseConnection: boolean }> = {
  "operator.selectWorkingDirectory": {
    message: "The working directory belongs to the Operator server.",
    chooseConnection: true,
  },
  "operator.startWebhookServer": {
    message:
      "This webhook is how a local server hands a terminal to the editor. A remote server does not call this laptop.",
    chooseConnection: false,
  },
  "operator.downloadOperator": {
    message:
      "This window is connected to a remote Operator. Download and update apply to a local install. Run Operator: Choose Connection to switch.",
    chooseConnection: true,
  },
  "operator.startOperatorServer": {
    message:
      "This window uses a remote Operator, so a local server is not started. Run Operator: Choose Connection to run Operator on this machine.",
    chooseConnection: true,
  },
};

export function remoteCommandTarget(command: string, remote: boolean): RemoteCommandTarget {
  if (!remote) {
    return { kind: "local" };
  }
  const route = UI_ROUTES[command];
  if (route) {
    return { kind: "ui", route };
  }
  const refusal = REFUSALS[command];
  if (refusal) {
    return {
      kind: "message",
      message: refusal.message,
      chooseConnection: refusal.chooseConnection,
    };
  }
  return { kind: "local" };
}

export type LaunchClientAction = "run-terminal" | "server-session" | "refuse-local-command";

/** A remote window must not run a command whose working directory is on the server. */
export function launchClientAction(
  remote: boolean,
  executedServerSide: boolean,
  command: string,
): LaunchClientAction {
  if (executedServerSide || (remote && command.trim() === "")) {
    return "server-session";
  }
  if (remote) {
    return "refuse-local-command";
  }
  return "run-terminal";
}

/** Remote launches leave the wrapper unset so the server uses its own session target. */
export function launchWrapper(remote: boolean): string | null {
  return remote ? null : "vscode";
}

export function focusMissAction(remote: boolean): "relaunch" | "open-queue" {
  return remote ? "open-queue" : "relaunch";
}

export function ticketOpenAction(
  remote: boolean,
  ticketId: string | undefined,
): "file" | "ui" | "unavailable" {
  if (!remote) {
    return "file";
  }
  return ticketId ? "ui" : "unavailable";
}

/** Header click when Configuration is not ready. */
export function notReadyHeaderCommand(remote: boolean, hasWorkingDirectory: boolean): string {
  if (remote) {
    return CHOOSE_CONNECTION_COMMAND;
  }
  return hasWorkingDirectory ? "operator.runSetup" : "operator.selectWorkingDirectory";
}

const HEADER_TITLES: Record<string, string> = {
  [CHOOSE_CONNECTION_COMMAND]: "Choose Connection",
  "operator.runSetup": "Run Operator Setup",
  "operator.selectWorkingDirectory": "Select Working Directory",
};

export function headerCommand(
  remote: boolean,
  hasWorkingDirectory: boolean,
): { command: string; title: string } {
  const command = notReadyHeaderCommand(remote, hasWorkingDirectory);
  return { command, title: HEADER_TITLES[command] ?? "Choose Connection" };
}

export interface RemoteCommandDeps {
  openUi: (route: OperatorUiRoute) => Promise<void>;
  inform: (message: string, chooseConnection: boolean) => Promise<void>;
}

/** Run the remote branch. Returns true when the local handler must not run. */
export async function presentRemoteCommand(
  target: RemoteCommandTarget,
  deps: RemoteCommandDeps,
): Promise<boolean> {
  if (target.kind === "local") {
    return false;
  }
  if (target.kind === "ui") {
    await deps.openUi(target.route);
    return true;
  }
  await deps.inform(target.message, target.chooseConnection);
  return true;
}

export async function guardRemoteCommand(
  command: string,
  ticketsDir: string | undefined,
): Promise<boolean> {
  const remote = usingRemoteServer(
    readConnectionSettings(vscode.workspace.getConfiguration("operator")),
  );
  return presentRemoteCommand(remoteCommandTarget(command, remote), {
    openUi: (route) => openOperatorUi(ticketsDir, route),
    inform: async (message, chooseConnection) => {
      if (!chooseConnection) {
        void vscode.window.showInformationMessage(message);
        return;
      }
      const choice = await vscode.window.showInformationMessage(message, "Choose Connection");
      if (choice === "Choose Connection") {
        await vscode.commands.executeCommand(CHOOSE_CONNECTION_COMMAND);
      }
    },
  });
}
