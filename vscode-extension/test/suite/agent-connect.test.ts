/**
 * Connect links: workspace matching, pending hand-off freshness, and which
 * terminal an agent's connect opens.
 */

import * as assert from "node:assert";
import {
  isFresh,
  matchesWorkspace,
  openAgentTerminal,
  type PendingConnect,
  type TerminalOps,
} from "../../src/agent-connect";
import type { ConnectTerminal } from "../../src/generated";

function ops(existing: string[] = []): TerminalOps & { calls: string[] } {
  const calls: string[] = [];
  return {
    calls,
    exists: (name) => existing.includes(name),
    focus: (name) => {
      calls.push(`focus:${name}`);
    },
    create: (o) => {
      calls.push(`create:${o.name}:${o.workingDir ?? ""}`);
    },
    send: (name, command) => {
      calls.push(`send:${name}:${command}`);
    },
  };
}

const live: ConnectTerminal = {
  phase: "live",
  ticket_id: "FEAT-12",
  terminal_name: "op-FEAT-12",
  cwd: "/home/me/proj",
  command: "tmux attach -t op-FEAT-12",
};

suite("agent-connect", () => {
  test("matches a remote workspace by authority and path", () => {
    const req = { agent: "a", authority: "ssh-remote+devbox", path: "/home/me/proj/" };
    assert.ok(matchesWorkspace(req, [{ authority: "ssh-remote+devbox", path: "/home/me/proj" }]));
    assert.ok(!matchesWorkspace(req, [{ path: "/home/me/proj" }]));
  });

  test("pending connects expire", () => {
    const pending: PendingConnect = { agent: "a", path: "/p", terminal: live, at: 0 };
    assert.ok(isFresh(pending, 1000));
    assert.ok(!isFresh(pending, 10 * 60 * 1000));
  });

  test("a live session opens an attach terminal", () => {
    const o = ops();
    openAgentTerminal(o, live, () => undefined);
    assert.deepStrictEqual(o.calls, [
      "create:FEAT-12 · attach:/home/me/proj",
      "send:FEAT-12 · attach:tmux attach -t op-FEAT-12",
    ]);
  });

  test("no command focuses the agent's own terminal", () => {
    const o = ops(["op-FEAT-12"]);
    openAgentTerminal(o, { ...live, command: undefined }, () => undefined);
    assert.deepStrictEqual(o.calls, ["focus:op-FEAT-12"]);
  });
});
