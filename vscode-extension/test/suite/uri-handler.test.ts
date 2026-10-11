/**
 * Deep links from the Operator UI: each path reaches the right action, and a
 * link missing its parameter warns instead of acting.
 */

import * as assert from "node:assert";
import * as vscode from "vscode";
import { routeOperatorUri, type UriActions } from "../../src/uri-handler";

function recorder(): UriActions & { calls: string[] } {
  const calls: string[] = [];
  return {
    calls,
    focusSession: (name) => {
      calls.push(`focus:${name}`);
    },
    openTicket: (id) => {
      calls.push(`open:${id}`);
    },
    connect: (req) => {
      calls.push(`connect:${req.agent}:${req.authority ?? ""}:${req.path}`);
    },
    warn: (message) => {
      calls.push(`warn:${message}`);
    },
  };
}

function uri(path: string, query: string): vscode.Uri {
  return vscode.Uri.from({ scheme: "vscode", authority: "untra.operator-terminals", path, query });
}

suite("routeOperatorUri", () => {
  test("open-ticket opens the ticket by id", () => {
    const actions = recorder();
    routeOperatorUri(uri("/open-ticket", "id=FEAT-12"), actions);
    assert.deepStrictEqual(actions.calls, ["open:FEAT-12"]);
  });

  test("focus-session focuses the named terminal", () => {
    const actions = recorder();
    routeOperatorUri(uri("/focus-session", "name=op%20FEAT-12"), actions);
    assert.deepStrictEqual(actions.calls, ["focus:op FEAT-12"]);
  });

  test("a link without its parameter warns", () => {
    const actions = recorder();
    routeOperatorUri(uri("/open-ticket", ""), actions);
    assert.deepStrictEqual(actions.calls, [
      "warn:Operator: open-ticket link is missing a ticket id",
    ]);
  });

  test("unknown paths are ignored", () => {
    const actions = recorder();
    routeOperatorUri(uri("/something-else", "id=1"), actions);
    assert.deepStrictEqual(actions.calls, []);
  });

  test("connect passes agent, authority and path", () => {
    const actions = recorder();
    routeOperatorUri(
      uri("/connect", "agent=a1&authority=ssh-remote%2Bdevbox&path=%2Fhome%2Fme%2Fproj"),
      actions,
    );
    assert.deepStrictEqual(actions.calls, ["connect:a1:ssh-remote+devbox:/home/me/proj"]);
  });

  test("connect without a path warns", () => {
    const actions = recorder();
    routeOperatorUri(uri("/connect", "agent=a1"), actions);
    assert.deepStrictEqual(actions.calls, [
      "warn:Operator: connect link is missing an agent or path",
    ]);
  });
});
