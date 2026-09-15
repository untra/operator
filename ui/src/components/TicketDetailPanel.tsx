import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import type { ConfigurationResponse } from "@operator/bindings/ConfigurationResponse";
import type { DelegatorResponse } from "@operator/bindings/DelegatorResponse";
import type { LaunchTicketResponse } from "@operator/bindings/LaunchTicketResponse";
import { LaunchForm, TicketDetailView } from "@operator/webcomponents";
import type { LaunchFormValue } from "@operator/webcomponents";
import { OperatorApi } from "../api-client";
import { useHost } from "../host";
import { useRightPanel } from "../right-panel";
import { wrapperSessionLink } from "../session-links";
import type { IssueType } from "@operator/bindings/IssueType";
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
  const [api] = useState(() => new OperatorApi(host));

  // Launch form state.
  const [delegator, setDelegator] = useState<string>(""); // '' = default chain
  const [wrapper, setWrapper] = useState<string>(""); // '' = configured default
  const [target, setTarget] = useState<string>(""); // '' = delegator's target
  const [yolo, setYolo] = useState(false);

  const [config, setConfig] = useState<ConfigurationResponse | null>(null);
  const [delegators, setDelegators] = useState<DelegatorResponse[]>([]);
  const [targets, setTargets] = useState<string[]>([]);
  const [workflow, setWorkflow] = useState<IssueType | null>(null);
  const [workflowError, setWorkflowError] = useState<string | null>(null);

  const [launching, setLaunching] = useState(false);
  const [result, setResult] = useState<LaunchTicketResponse | null>(null);
  const [launchError, setLaunchError] = useState<string | null>(null);

  // Focus action state (cmux: calls the control-plane focus endpoint).
  const [focusBusy, setFocusBusy] = useState(false);
  const [focusError, setFocusError] = useState<string | null>(null);
  const [focused, setFocused] = useState(false);
  const launchRequest = useRef(0);
  const focusRequest = useRef(0);

  useEffect(
    () => () => {
      launchRequest.current += 1;
      focusRequest.current += 1;
    },
    [],
  );

  // Config (delegator names + the configured control wrapper) for the dropdowns.
  useEffect(() => {
    let cancelled = false;
    Promise.all([api.getConfiguration(), api.listDelegators(), api.executionTargets()])
      .then(([configuration, delegatorResponse, targetResponse]) => {
        if (!cancelled) {
          setConfig(configuration);
          setDelegators(delegatorResponse.delegators);
          setTargets(
            targetResponse.targets.filter((item) => item.available).map((item) => item.name),
          );
        }
        return undefined;
      })
      .catch(() => !cancelled && setConfig(null));
    return () => {
      cancelled = true;
    };
  }, [api]);

  // The Operator workflow this ticket's issue type defines. Rendered from the
  // native document, so this graph matches the one on the docs site exactly.
  useEffect(() => {
    let cancelled = false;
    api
      .getIssueTypeDocument(ticket.ticket_type)
      .then((doc) => !cancelled && setWorkflow(doc))
      .catch((e) => {
        if (!cancelled) {
          setWorkflowError(e instanceof Error ? e.message : "Failed to load workflow");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api, ticket.ticket_type]);

  const defaultWrapperLabel = config?.launch.session_wrapper ?? "configured";

  const onLaunch = useCallback(() => {
    const request = ++launchRequest.current;
    setLaunching(true);
    setLaunchError(null);
    api
      .launchTicket(ticket.id, {
        delegator: delegator || null,
        provider: null,
        model: null,
        model_server: null,
        yolo_mode: yolo,
        wrapper: wrapper || null,
        retry_reason: null,
        resume_session_id: null,
        target: target || null,
      })
      .then((response) => {
        if (request === launchRequest.current) {
          setResult(response);
        }
        return undefined;
      })
      .catch((e) => {
        if (request === launchRequest.current) {
          setLaunchError(e instanceof Error ? e.message : "Launch failed");
        }
      })
      .finally(() => {
        if (request === launchRequest.current) {
          setLaunching(false);
        }
      });
  }, [api, delegator, target, ticket.id, wrapper, yolo]);

  const onFocus = useCallback(
    (agentId: string) => {
      const request = ++focusRequest.current;
      setFocusBusy(true);
      setFocusError(null);
      api
        .focusSession(agentId)
        .then(() => {
          if (request === focusRequest.current) {
            setFocused(true);
          }
          return undefined;
        })
        .catch((e) => {
          if (request === focusRequest.current) {
            setFocusError(e instanceof Error ? e.message : "Focus failed");
          }
        })
        .finally(() => {
          if (request === focusRequest.current) {
            setFocusBusy(false);
          }
        });
    },
    [api],
  );

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
    if (result) {
      onFocus(result.agent_id);
    }
  }, [onFocus, result]);
  const formValue: LaunchFormValue = { delegator, wrapper, target, yolo };
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
            disabled={focusBusy}
          >
            {focusBusy ? "Focusing…" : focused ? `${link.label} ✓` : link.label}
          </button>
          {focusError && <div className={styles.error}>{focusError}</div>}
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
      launchControls={
        result ? undefined : (
          <LaunchForm
            value={formValue}
            delegators={delegators}
            targets={targets}
            defaultWrapperLabel={defaultWrapperLabel}
            busy={launching}
            error={launchError}
            onChange={handleFormChange}
            onSubmit={onLaunch}
          />
        )
      }
      launchedTicketId={result?.ticket_id}
      launchActions={launchActions}
      workflow={
        workflowError
          ? { status: "error", message: workflowError }
          : workflow
            ? { status: "ready", data: workflow }
            : { status: "loading", message: "Loading workflow…" }
      }
    />
  );
}
