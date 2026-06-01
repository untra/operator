// Describes how to reach a launched agent's session, contextual to the wrapper
// the operator control plane is running inside.
//
// The three wrapper families need three different mechanisms:
//  - VS Code registers a URI scheme, so its terminal is focused by opening
//    `vscode://untra.operator-terminals/focus-session?name=...` (the extension's
//    URI handler reveals the tab).
//  - cmux has NO browser URL scheme (per cmux docs — commands are palette /
//    shortcut / button driven). Instead operator, which runs inside cmux, focuses
//    the pane by shelling out to `cmux focus-workspace` via the control-plane
//    endpoint `POST /api/v1/agents/{id}/focus`. So the UI calls the API, not a URL.
//  - tmux / zellij have neither a URL scheme nor a control-plane focus path, so
//    their session is shown read-only.

import type { LaunchTicketResponse } from './api-client';

/** Publisher.name of the operator VS Code extension — the URI authority. */
const VSCODE_EXTENSION_ID = 'untra.operator-terminals';

export type WrapperSessionLink =
  /** Clickable deep-link opened via host.openExternal (VS Code). */
  | { kind: 'open-url'; label: string; url: string }
  /** Focus via the operator control-plane API (cmux). */
  | { kind: 'focus-api'; label: string }
  /** Read-only session reference, no action (tmux/zellij). */
  | { kind: 'display'; label: string; detail: string };

export function wrapperSessionLink(res: LaunchTicketResponse): WrapperSessionLink {
  switch (res.session_wrapper) {
    case 'vscode':
      return {
        kind: 'open-url',
        label: 'Focus VS Code terminal',
        url: `vscode://${VSCODE_EXTENSION_ID}/focus-session?name=${encodeURIComponent(
          res.terminal_name,
        )}`,
      };
    case 'cmux':
      return { kind: 'focus-api', label: 'Focus cmux session' };
    default: {
      // tmux / zellij / terminal — no URI scheme, no control-plane focus; show the ref.
      const ref = [res.terminal_name, res.session_window_ref, res.session_context_ref]
        .filter(Boolean)
        .join(' · ');
      return { kind: 'display', label: 'Session', detail: ref || res.terminal_name };
    }
  }
}
