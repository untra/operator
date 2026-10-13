import { useCallback, useState } from "react";
import type { Attention } from "@operator/bindings/Attention";
import type { ConnectOption } from "@operator/bindings/ConnectOption";
import type { ConnectPlan } from "@operator/bindings/ConnectPlan";
import { useApiMutation } from "../api";
import { focusSessionMutation } from "../api/definitions";
import { useHost } from "../host";
import { connectAction } from "../session-links";
import styles from "./ConnectPanel.module.css";

const ATTENTION_TEXT: Record<Attention["reason"], string> = {
  awaiting_input: "Agent is waiting for you",
  review: "Review pending",
  orphaned: "Session lost",
  idle: "No output for a while",
};

export function attentionText(attention: Attention): string {
  const head = ATTENTION_TEXT[attention.reason];
  return attention.detail ? `${head} · ${attention.detail}` : head;
}

/** Connect controls for an in-progress agent: preferred link, alternatives, attach commands. */
export function ConnectPanel({
  agentId,
  plan,
  attention,
}: {
  agentId: string;
  plan: ConnectPlan;
  attention?: Attention | null;
}) {
  const host = useHost();
  const focus = useApiMutation(focusSessionMutation);
  const [copied, setCopied] = useState<string | null>(null);

  const run = useCallback(
    (option: ConnectOption) => {
      const action = connectAction(option);
      switch (action.kind) {
        case "open-url":
          host.openExternal(action.url);
          break;
        case "focus-api":
          focus.mutate({ agentId });
          break;
        case "copy":
          void navigator.clipboard
            ?.writeText(action.text)
            .then(() => setCopied(action.text))
            .catch(() => setCopied(null));
          break;
        case "none":
          break;
      }
    },
    [agentId, focus, host],
  );

  const preferred = plan.preferred;
  const others = plan.options.filter(
    (o) => o.kind !== "command" && !(preferred && sameOption(o, preferred)),
  );
  const commands = plan.options.filter((o) => o.kind === "command" && o.command);

  return (
    <section className={styles.panel} aria-label="Connect">
      {attention && (
        <output className={styles.banner} data-reason={attention.reason}>
          <span>{attentionText(attention)}</span>
          {preferred && (
            <button type="button" className={styles.primary} onClick={() => run(preferred)}>
              {preferred.label}
            </button>
          )}
        </output>
      )}

      <div className={styles.header}>
        <h2>Connect</h2>
        <span className={styles.phase} data-phase={plan.phase}>
          {plan.phase}
        </span>
      </div>

      {plan.options.length === 0 && (
        <p className={styles.muted}>No session to attach to or resume.</p>
      )}

      <div className={styles.actions}>
        {preferred && !attention && (
          <button type="button" className={styles.primary} onClick={() => run(preferred)}>
            {preferred.label}
          </button>
        )}
        {others.map((o) => (
          <button
            key={`${o.interface}-${o.kind}-${o.label}`}
            type="button"
            className={styles.secondary}
            onClick={() => run(o)}
            disabled={!o.available || (o.kind === "focus_api" && focus.isPending)}
            title={o.unavailable_reason}
          >
            {o.label}
          </button>
        ))}
      </div>
      {focus.error && <div className={styles.error}>{focus.error.message}</div>}

      {commands.map((o) => (
        <div key={`${o.interface}-${o.command}`} className={styles.command}>
          <span className={styles.commandLabel}>{o.label}</span>
          <code>{o.command}</code>
          <button type="button" className={styles.secondary} onClick={() => run(o)}>
            {copied === o.command ? "Copied" : "Copy"}
          </button>
        </div>
      ))}
    </section>
  );
}

function sameOption(a: ConnectOption, b: ConnectOption): boolean {
  return a.interface === b.interface && a.kind === b.kind && a.label === b.label;
}
