/**
 * Where a ticket can be opened: its files in an editor, and its agent's
 * terminal session. Pure URL building, shared by every surface.
 */

import type { TicketAgent } from "../generated/TicketAgent";

export type EditorId = "vscode" | "cursor" | "zed" | "jetbrains";

export type OpenTarget = { editor: EditorId; label: string; url: string };

export type SessionTarget =
  /** A deep link handed to the host's openExternal. */
  | { kind: "open-url"; label: string; url: string }
  /** Focus through `POST /api/v1/agents/{id}/focus`. */
  | { kind: "focus-api"; label: string }
  /** A shell command to copy. */
  | { kind: "command"; label: string; command: string }
  /** A read-only reference. */
  | { kind: "display"; label: string; detail: string };

export type SessionAgent = Pick<
  TicketAgent,
  | "id"
  | "session_wrapper"
  | "session_name"
  | "session_window_ref"
  | "session_context_ref"
  | "attach_command"
>;

/** Publisher.name of the Operator VS Code extension, the URI authority. */
const EXTENSION_ID = "untra.operator-terminals";

function slashPath(path: string): string {
  const forward = path.replaceAll("\\", "/");
  return forward.startsWith("/") ? forward : `/${forward}`;
}

/** Deep links that open `path` in each editor; `remoteHost` switches to SSH forms. */
export function openTargets(path: string, remoteHost?: string | null): OpenTarget[] {
  const encoded = encodeURI(slashPath(path));
  if (remoteHost) {
    const host = encodeURIComponent(remoteHost);
    return [
      {
        editor: "vscode",
        label: "VS Code",
        url: `vscode://vscode-remote/ssh-remote+${host}${encoded}`,
      },
      {
        editor: "cursor",
        label: "Cursor",
        url: `cursor://vscode-remote/ssh-remote+${host}${encoded}`,
      },
      { editor: "zed", label: "Zed", url: `zed://ssh/${host}${encoded}` },
    ];
  }
  return [
    { editor: "vscode", label: "VS Code", url: `vscode://file${encoded}` },
    { editor: "cursor", label: "Cursor", url: `cursor://file${encoded}` },
    { editor: "zed", label: "Zed", url: `zed://file${encoded}` },
    {
      editor: "jetbrains",
      label: "JetBrains",
      url: `idea://open?file=${encodeURIComponent(path.replaceAll("\\", "/"))}`,
    },
  ];
}

/** Like `openTargets`, but VS Code and Cursor open the ticket through the Operator extension. */
export function ticketFileTargets(ticketId: string, path: string): OpenTarget[] {
  const id = encodeURIComponent(ticketId);
  return openTargets(path).map((target) =>
    target.editor === "vscode" || target.editor === "cursor"
      ? { ...target, url: `${target.editor}://${EXTENSION_ID}/open-ticket?id=${id}` }
      : target,
  );
}

/** The Operator extension's link that reveals a terminal by name. */
export function focusSessionUrl(scheme: "vscode" | "cursor", name: string): string {
  return `${scheme}://${EXTENSION_ID}/focus-session?name=${encodeURIComponent(name)}`;
}

/** Ways to reach the agent's terminal, by the wrapper it runs in. */
export function sessionTargets(agent: SessionAgent): SessionTarget[] {
  const name = agent.session_name;
  switch (agent.session_wrapper ?? "tmux") {
    case "vscode":
      return name
        ? [
            { kind: "open-url", label: "Focus in VS Code", url: focusSessionUrl("vscode", name) },
            { kind: "open-url", label: "Focus in Cursor", url: focusSessionUrl("cursor", name) },
          ]
        : [];
    case "cmux":
      return [{ kind: "focus-api", label: "Focus cmux session" }];
    case "zellij":
      return name ? [{ kind: "display", label: "zellij tab", detail: name }] : [];
    case "tmux": {
      const targets: SessionTarget[] = name
        ? [{ kind: "focus-api", label: "Focus tmux session" }]
        : [];
      return agent.attach_command
        ? [...targets, { kind: "command", label: "Attach", command: agent.attach_command }]
        : targets;
    }
    default:
      return name ? [{ kind: "display", label: "Session", detail: name }] : [];
  }
}
