/**
 * open-operator-ui - link out to the daemon-hosted Operator UI.
 *
 * The full Operator UI (issue types, projects, kanban board, queue, agents,
 * dashboard) is served by the running daemon at the same localhost port as the
 * REST API. Rather than reimplement those surfaces in the extension webview, we
 * open the hosted UI in VS Code's built-in Simple Browser.
 *
 * Because the UI lives at the API port, a stopped daemon means there is nothing
 * to render - we health-probe first and surface an actionable message instead of
 * opening a blank tab.
 */

import * as vscode from "vscode";
import { discoverApiUrl, OperatorApiClient } from "./api-client";
import { isLoopbackUrl } from "./auth/credentials";

/** Sections of the hosted UI we can deep-link to (hash routes from ui/src/main.tsx). */
export type OperatorUiRoute =
  | "dashboard"
  | "issuetypes"
  | "projects"
  | "kanban"
  | "queue"
  | "config"
  | "workflows"
  | "remote-targets"
  | "campaigns"
  | "license"
  | "setup"
  | "delegators"
  | "llm"
  | "git"
  | "tickets";

const ROUTE_HASH: Record<OperatorUiRoute, string> = {
  dashboard: "#/",
  issuetypes: "#/issuetypes",
  projects: "#/projects",
  kanban: "#/kanban",
  queue: "#/queue",
  config: "#/config",
  workflows: "#/workflows",
  "remote-targets": "#/remote-targets",
  campaigns: "#/campaigns",
  license: "#/settings/license",
  setup: "#/setup",
  delegators: "#/delegators",
  llm: "#/llm",
  git: "#/git",
  tickets: "#/tickets",
};

/**
 * Open a section of the daemon-hosted Operator UI in VS Code's Simple Browser.
 *
 * @param ticketsDir the active `.tickets` directory, used to discover the
 *   daemon's dynamic port via `api-session.json`.
 * @param route the hosted-UI section to open.
 * @param id an item within that section, e.g. a campaign id.
 */
export async function openOperatorUi(
  ticketsDir: string | undefined,
  route: OperatorUiRoute,
  id?: string,
): Promise<void> {
  const apiUrl = await discoverApiUrl(ticketsDir);

  // The hosted UI is served by the daemon; if it's down there is nothing to
  // show. Probe health before opening so the user gets an actionable message
  // rather than a blank Simple Browser tab.
  if (!apiUrl) {
    void vscode.window.showErrorMessage(
      "No Operator server is configured. Run Operator: Choose Connection.",
    );
    return;
  }

  if (!(await new OperatorApiClient(apiUrl).isReachable())) {
    if (isLoopbackUrl(apiUrl)) {
      const choice = await vscode.window.showErrorMessage(
        "The Operator daemon is not running, so the Operator UI is unavailable. " +
          "Start the daemon, then try again.",
        "Start Operator Server",
      );
      if (choice === "Start Operator Server") {
        await vscode.commands.executeCommand("operator.startOperatorServer");
      }
    } else {
      void vscode.window.showErrorMessage(`Can't reach Operator at ${apiUrl}.`);
    }
    return;
  }

  const detail = id ? `/${encodeURIComponent(id)}` : "";
  if (!isLoopbackUrl(apiUrl)) {
    const url = `${apiUrl.replace(/\/+$/, "")}/${ROUTE_HASH[route]}${detail}`;
    await vscode.env.openExternal(vscode.Uri.parse(url));
    return;
  }

  // Map the local URL for remote / SSH / Codespaces port forwarding. The hash
  // route is appended after mapping because asExternalUri can drop a fragment.
  const external = await vscode.env.asExternalUri(vscode.Uri.parse(apiUrl));
  const base = external.toString().replace(/\/+$/, "");
  const detail = id ? `/${encodeURIComponent(id)}` : "";
  const url = `${base}/${ROUTE_HASH[route]}${detail}`;

  try {
    await vscode.commands.executeCommand("simpleBrowser.show", url);
  } catch {
    // Simple Browser unavailable for some reason - fall back to the OS browser.
    await vscode.env.openExternal(vscode.Uri.parse(url));
  }
}
