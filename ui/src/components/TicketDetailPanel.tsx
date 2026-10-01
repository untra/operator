import { useCallback, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { LaunchForm, TicketDetailView } from "@operator/webcomponents";
import type { LaunchFormValue } from "@operator/webcomponents";
import { useApiMutation, useApiQuery } from "../api";
import {
  configurationQuery,
  delegatorsQuery,
  executionTargetsQuery,
  focusSessionMutation,
  issueTypeDocumentQuery,
  launchTicketMutation,
} from "../api/definitions";
import { useHost } from "../host";
import { useRightPanel } from "../right-panel";
import { wrapperSessionLink } from "../session-links";
import styles from "./TicketDetailPanel.module.css";

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

  const [delegator, setDelegator] = useState<string>("");
  const [wrapper, setWrapper] = useState<string>("");
  const [target, setTarget] = useState<string>("");
  const [yolo, setYolo] = useState(false);
  const [focused, setFocused] = useState(false);

  const config = useApiQuery(configurationQuery());
  const delegators = useApiQuery(delegatorsQuery());
  const targets = useApiQuery(executionTargetsQuery());
  const workflow = useApiQuery(issueTypeDocumentQuery(ticket.ticket_type));
  const launch = useApiMutation(launchTicketMutation);
  const focus = useApiMutation(focusSessionMutation);

  const defaultWrapperLabel = config.data?.launch.session_wrapper ?? "configured";
  const result = launch.data;

  const onLaunch = useCallback(() => {
    launch.mutate({
      ticketId: ticket.id,
      options: {
        delegator: delegator || null,
        provider: null,
        model: null,
        model_server: null,
        yolo_mode: yolo,
        wrapper: wrapper || null,
        retry_reason: null,
        resume_session_id: null,
        target: target || null,
      },
    });
  }, [delegator, launch, target, ticket.id, wrapper, yolo]);

  const handleFormChange = useCallback((value: LaunchFormValue) => {
    setDelegator(value.delegator);
    setWrapper(value.wrapper);
    setTarget(value.target);
    setYolo(value.yolo);
  }, []);

  const link = useMemo(() => (result ? wrapperSessionLink(result) : null), [result]);
  const openAgentDetail = useCallback(() => {
    if (!result) {
      return;
    }
    void navigate(`/agent/${encodeURIComponent(result.agent_id)}`);
    close();
  }, [close, navigate, result]);
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
  const formValue: LaunchFormValue = { delegator, wrapper, target, yolo };
  const launchLoading = config.isLoading || delegators.isLoading || targets.isLoading;
  const launchError =
    config.error?.message ?? delegators.error?.message ?? targets.error?.message ?? null;

  const launchControls = launchLoading ? (
    <div className={styles.loading}>Loading launch configuration…</div>
  ) : launchError ? (
    <div className={styles.error}>Launch options unavailable: {launchError}</div>
  ) : (
    <LaunchForm
      value={formValue}
      delegators={delegators.data?.delegators ?? []}
      targets={(targets.data?.targets ?? [])
        .filter((item) => item.available)
        .map((item) => item.name)}
      defaultWrapperLabel={defaultWrapperLabel}
      busy={launch.isPending}
      error={launch.error?.message ?? null}
      onChange={handleFormChange}
      onSubmit={onLaunch}
    />
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
    <TicketDetailView
      ticket={ticket}
      launchControls={result || isFinished ? undefined : launchControls}
      launchedTicketId={result?.ticket_id}
      launchActions={launchActions}
      workflow={
        workflow.error
          ? { status: "error", message: workflow.error.message }
          : workflow.data
            ? { status: "ready", data: workflow.data }
            : { status: "loading", message: "Loading workflow…" }
      }
    />
  );
}
