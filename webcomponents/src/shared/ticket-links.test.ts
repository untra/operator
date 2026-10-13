/**
 * Editor and session links are pure string building, so every scheme is pinned
 * here rather than discovered by clicking.
 */

import { describe, expect, test } from "bun:test";

import {
  focusSessionUrl,
  openTargets,
  sessionTargets,
  ticketFileTargets,
  type SessionAgent,
} from "./ticket-links";

function agent(over: Partial<SessionAgent>): SessionAgent {
  return {
    id: "agent-1",
    session_wrapper: null,
    session_name: null,
    session_window_ref: null,
    session_context_ref: null,
    attach_command: null,
    ...over,
  };
}

describe("openTargets", () => {
  test("links a local path into every editor", () => {
    expect(openTargets("/work/api/FEAT-1 notes.md")).toEqual([
      { editor: "vscode", label: "VS Code", url: "vscode://file/work/api/FEAT-1%20notes.md" },
      { editor: "cursor", label: "Cursor", url: "cursor://file/work/api/FEAT-1%20notes.md" },
      { editor: "zed", label: "Zed", url: "zed://file/work/api/FEAT-1%20notes.md" },
      {
        editor: "jetbrains",
        label: "JetBrains",
        url: "idea://open?file=%2Fwork%2Fapi%2FFEAT-1%20notes.md",
      },
    ]);
  });

  test("normalizes a Windows path", () => {
    expect(openTargets("C:\\work\\api")[0]?.url).toBe("vscode://file/C:/work/api");
  });

  test("links a remote path over SSH and drops editors without a remote form", () => {
    expect(openTargets("/srv/worktrees/feat-1", "builder")).toEqual([
      {
        editor: "vscode",
        label: "VS Code",
        url: "vscode://vscode-remote/ssh-remote+builder/srv/worktrees/feat-1",
      },
      {
        editor: "cursor",
        label: "Cursor",
        url: "cursor://vscode-remote/ssh-remote+builder/srv/worktrees/feat-1",
      },
      { editor: "zed", label: "Zed", url: "zed://ssh/builder/srv/worktrees/feat-1" },
    ]);
  });
});

describe("sessionTargets", () => {
  test("a VS Code session focuses through the extension in VS Code and Cursor", () => {
    expect(sessionTargets(agent({ session_wrapper: "vscode", session_name: "op FEAT-1" }))).toEqual(
      [
        {
          kind: "open-url",
          label: "Focus in VS Code",
          url: "vscode://untra.operator-terminals/focus-session?name=op%20FEAT-1",
        },
        {
          kind: "open-url",
          label: "Focus in Cursor",
          url: "cursor://untra.operator-terminals/focus-session?name=op%20FEAT-1",
        },
      ],
    );
  });

  test("cmux focuses through the API", () => {
    expect(sessionTargets(agent({ session_wrapper: "cmux" }))).toEqual([
      { kind: "focus-api", label: "Focus cmux session" },
    ]);
  });

  test("tmux focuses through the API and offers the attach command", () => {
    expect(
      sessionTargets(
        agent({
          session_wrapper: "tmux",
          session_name: "op-feat-1",
          attach_command: "tmux -L operator attach -t op-feat-1",
        }),
      ),
    ).toEqual([
      { kind: "focus-api", label: "Focus tmux session" },
      { kind: "command", label: "Attach", command: "tmux -L operator attach -t op-feat-1" },
    ]);
  });

  test("zellij names the tab to switch to", () => {
    expect(
      sessionTargets(agent({ session_wrapper: "zellij", session_name: "api-feat-1" })),
    ).toEqual([{ kind: "display", label: "zellij tab", detail: "api-feat-1" }]);
  });

  test("an agent with no session offers nothing", () => {
    expect(sessionTargets(agent({ session_wrapper: "vscode" }))).toEqual([]);
  });
});

test("focusSessionUrl encodes the terminal name", () => {
  expect(focusSessionUrl("vscode", "a&b")).toBe(
    "vscode://untra.operator-terminals/focus-session?name=a%26b",
  );
});

test("ticketFileTargets opens VS Code and Cursor through the Operator extension", () => {
  expect(ticketFileTargets("FEAT-1", "/work/t.md").map((t) => t.url)).toEqual([
    "vscode://untra.operator-terminals/open-ticket?id=FEAT-1",
    "cursor://untra.operator-terminals/open-ticket?id=FEAT-1",
    "zed://file/work/t.md",
    "idea://open?file=%2Fwork%2Ft.md",
  ]);
});
