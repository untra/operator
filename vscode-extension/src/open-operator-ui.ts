/**
 * open-operator-ui — link out to the daemon-hosted Operator UI.
 *
 * The full Operator UI (issue types, projects, kanban board, queue, agents,
 * dashboard) is served by the running daemon at the same localhost port as the
 * REST API. Rather than reimplement those surfaces in the extension webview, we
 * open the hosted UI in VS Code's built-in Simple Browser.
 *
 * Because the UI lives at the API port, a stopped daemon means there is nothing
 * to render — we health-probe first and surface an actionable message instead of
 * opening a blank tab.
 */

import * as vscode from 'vscode';
import { discoverApiUrl } from './api-client';

/** Sections of the hosted UI we can deep-link to (hash routes from ui/src/main.tsx). */
export type OperatorUiRoute =
  | 'dashboard'
  | 'issuetypes'
  | 'projects'
  | 'kanban'
  | 'queue'
  | 'config';

const ROUTE_HASH: Record<OperatorUiRoute, string> = {
  dashboard: '#/',
  issuetypes: '#/issuetypes',
  projects: '#/projects',
  kanban: '#/kanban',
  queue: '#/queue',
  config: '#/config',
};

/**
 * Open a section of the daemon-hosted Operator UI in VS Code's Simple Browser.
 *
 * @param ticketsDir the active `.tickets` directory, used to discover the
 *   daemon's dynamic port via `api-session.json`.
 * @param route the hosted-UI section to open.
 */
export async function openOperatorUi(
  ticketsDir: string | undefined,
  route: OperatorUiRoute
): Promise<void> {
  const apiUrl = await discoverApiUrl(ticketsDir);

  // The hosted UI is served by the daemon; if it's down there is nothing to
  // show. Probe health before opening so the user gets an actionable message
  // rather than a blank Simple Browser tab.
  let reachable: boolean;
  try {
    const res = await fetch(`${apiUrl}/api/v1/health`);
    reachable = res.ok;
  } catch {
    reachable = false;
  }
  if (!reachable) {
    const choice = await vscode.window.showErrorMessage(
      'The Operator daemon is not running, so the Operator UI is unavailable. ' +
        'Start the daemon, then try again.',
      'Start Operator Server'
    );
    if (choice === 'Start Operator Server') {
      await vscode.commands.executeCommand('operator.startOperatorServer');
    }
    return;
  }

  // Map the local URL for remote / SSH / Codespaces port forwarding. The hash
  // route is appended after mapping because asExternalUri can drop a fragment.
  const external = await vscode.env.asExternalUri(vscode.Uri.parse(apiUrl));
  const base = external.toString().replace(/\/+$/, '');
  const url = `${base}/${ROUTE_HASH[route]}`;

  try {
    await vscode.commands.executeCommand('simpleBrowser.show', url);
  } catch {
    // Simple Browser unavailable for some reason — fall back to the OS browser.
    await vscode.env.openExternal(vscode.Uri.parse(url));
  }
}
