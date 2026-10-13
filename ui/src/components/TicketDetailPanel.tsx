import { useCallback, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { TicketDetailView, type AsyncValue } from "@operator/webcomponents";
import type { IssueType } from "@operator/bindings/IssueType";
import { AGENT_POLL_MS, useApiMutation, useApiQuery } from "../api";
import {
  activeAgentsQuery,
  focusSessionMutation,
  issueTypeDocumentQuery,
} from "../api/definitions";
import { ConnectPanel } from "./ConnectPanel";
import { useHost } from "../host";
import { useRightPanel } from "../right-panel";
import { wrapperSessionLink } from "../session-links";
import styles from "./TicketDetailPanel.module.css";
import { useTicketLaunch } from "./use-ticket-launch";

/**
 * Right-panel contents for a kanban ticket: detail, the issue-type workflow
 * graph (the launch steps), and a full launch form. Replaces the old centered
 * WorkflowModal - the graph now lives alongside the controls to launch the
 * ticket. After a launch, surfaces session links contextual to the operator's
 * control wrapper (clickable for VS Code/cmux, read-only for tmux/zellij).
 */
export function TicketDetailPanel({ ticket }: { ticket: KanbanTicketCard }) {
  const host = useHost();
  const navigate = useNavigate();
  const { close } = useRightPanel();

  const [focused, setFocused] = useState(false);
  const { controls: launchControls, result } = useTicketLaunch(ticket.id);
  const workflow = useApiQuery(issueTypeDocumentQuery(ticket.ticket_type));
  const focus = useApiMutation(focusSessionMutation);
  const isActive = ticket.status !== "queued" && ticket.status !== "completed";
  const agents = useApiQuery(activeAgentsQuery(), {
    enabled: isActive || Boolean(result),
    pollIntervalMs: AGENT_POLL_MS,
  });
  const agent = agents.data?.agents.find((a) => a.ticket_id === ticket.id);

  const link = useMemo(() => (result ? wrapperSessionLink(result) : null), [result]);
  const openAgentDetail = useCallback(() => {
    if (!result) {
      return;
    }
    void navigate(`/agent/${encodeURIComponent(result.agent_id)}`);
    close();
  }, [close, navigate, result]);
  const openTicketPage = useCallback(() => {
    void navigate(`/tickets/${encodeURIComponent(ticket.id)}`);
    close();
  }, [close, navigate, ticket.id]);
  const openSession = useCallback(() => {
    if (link?.kind === "open-url") {
      host.openExternal(link.url);
    }
  }, [host, link]);
  const focusSession = useCallback(() => {
    if (!result) {
      return;
    }
    focus.mutate(
      { agentId: result.agent_id },
      {
        onSuccess: () => {
          setFocused(true);
        },
      },
    );
  }, [focus, result]);
  const isFinished = ticket.status === "completed";
  const workflowValue = useMemo<AsyncValue<IssueType>>(
    () =>
      workflow.error
        ? { status: "error", message: workflow.error.message }
        : workflow.data
          ? { status: "ready", data: workflow.data }
          : { status: "loading", message: "Loading workflow…" },
    [workflow.data, workflow.error],
  );

  const launchActions = result ? (
    <>
      <button type="button" className={styles.linkBtn} onClick={openAgentDetail}>
        Open agent detail
      </button>
      {link?.kind === "open-url" && (
        <button type="button" className={styles.linkBtn} onClick={openSession}>
          {link.label}
        </button>
      )}
      {link?.kind === "focus-api" && (
        <>
          <button
            type="button"
            className={styles.linkBtn}
            onClick={focusSession}
            disabled={focus.isPending}
          >
            {focus.isPending ? "Focusing…" : focused ? `${link.label} ✓` : link.label}
          </button>
          {focus.error && <div className={styles.error}>{focus.error.message}</div>}
        </>
      )}
      {link?.kind === "display" && (
        <div className={styles.sessionRef}>
          <span className={styles.fieldLabel}>{link.label}</span>
          <code>{link.detail}</code>
        </div>
      )}
    </>
  ) : undefined;

  return (
    <>
      <button type="button" className={styles.linkBtn} onClick={openTicketPage}>
        Open ticket page
      </button>
      {agent && (
        <ConnectPanel agentId={agent.id} plan={agent.connect} attention={agent.attention} />
      )}
      <TicketDetailView
        ticket={ticket}
        launchControls={result || isFinished ? undefined : launchControls}
        launchedTicketId={result?.ticket_id}
        launchActions={launchActions}
        workflow={workflowValue}
      />
    </>
  );
}
