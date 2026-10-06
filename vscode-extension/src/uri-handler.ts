/**
 * Routes `vscode://untra.operator-terminals/<path>?...` deep links from the
 * Operator UI to extension actions.
 */

import type * as vscode from "vscode";
import { parseConnect, type ConnectRequest } from "./agent-connect";

export interface UriActions {
  focusSession: (terminalName: string) => void;
  openTicket: (ticketId: string) => void;
  connect: (request: ConnectRequest) => void;
  warn: (message: string) => void;
}

export function routeOperatorUri(uri: vscode.Uri, actions: UriActions): void {
  const params = new URLSearchParams(uri.query);
  if (uri.path === "/focus-session") {
    const name = params.get("name");
    if (name) {
      actions.focusSession(name);
    } else {
      actions.warn("Operator: focus-session link is missing a terminal name");
    }
  } else if (uri.path === "/open-ticket") {
    const id = params.get("id");
    if (id) {
      actions.openTicket(id);
    } else {
      actions.warn("Operator: open-ticket link is missing a ticket id");
    }
  } else if (uri.path === "/connect") {
    const request = parseConnect(params);
    if (request) {
      actions.connect(request);
    } else {
      actions.warn("Operator: connect link is missing an agent or path");
    }
  }
}
