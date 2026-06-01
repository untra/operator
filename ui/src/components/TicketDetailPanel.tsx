import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { KanbanTicketCard } from '@operator/bindings/KanbanTicketCard';
import type { Config } from '@operator/bindings/Config';
import type { LaunchTicketResponse } from '@operator/bindings/LaunchTicketResponse';
import { OperatorApi } from '../api-client';
import { useHost } from '../host';
import { useRightPanel } from '../right-panel';
import { wrapperSessionLink } from '../session-links';
import { WorkflowGraphView } from './WorkflowGraphView';
import styles from './TicketDetailPanel.module.css';

/**
 * Right-panel contents for a kanban ticket: detail, the issue-type workflow
 * graph (the launch steps), and a full launch form. Replaces the old centered
 * WorkflowModal — the graph now lives alongside the controls to launch the
 * ticket. After a launch, surfaces session links contextual to the operator's
 * control wrapper (clickable for VS Code/cmux, read-only for tmux/zellij).
 */
export function TicketDetailPanel({ ticket }: { ticket: KanbanTicketCard }) {
  const host = useHost();
  const navigate = useNavigate();
  const { close } = useRightPanel();
  const [api] = useState(() => new OperatorApi(host));

  // Launch form state.
  const [delegator, setDelegator] = useState<string>(''); // '' = default chain
  const [wrapper, setWrapper] = useState<string>(''); // '' = configured default
  const [yolo, setYolo] = useState(false);

  const [config, setConfig] = useState<Config | null>(null);
  const [workflow, setWorkflow] = useState<string | null>(null);
  const [workflowError, setWorkflowError] = useState<string | null>(null);

  const [launching, setLaunching] = useState(false);
  const [result, setResult] = useState<LaunchTicketResponse | null>(null);
  const [launchError, setLaunchError] = useState<string | null>(null);

  // Focus action state (cmux: calls the control-plane focus endpoint).
  const [focusBusy, setFocusBusy] = useState(false);
  const [focusError, setFocusError] = useState<string | null>(null);
  const [focused, setFocused] = useState(false);

  // Config (delegator names + the configured control wrapper) for the dropdowns.
  useEffect(() => {
    let cancelled = false;
    api
      .getConfiguration()
      .then((c) => !cancelled && setConfig(c))
      .catch(() => !cancelled && setConfig(null));
    return () => {
      cancelled = true;
    };
  }, [api]);

  // The issue-type workflow graph for this ticket (same source the modal used).
  useEffect(() => {
    let cancelled = false;
    api
      .exportWorkflow(ticket.id)
      .then((r) => !cancelled && setWorkflow(r.contents))
      .catch((e) => {
        if (!cancelled) setWorkflowError(e instanceof Error ? e.message : 'Failed to load workflow');
      });
    return () => {
      cancelled = true;
    };
  }, [api, ticket.id]);

  const defaultWrapperLabel = config?.sessions.wrapper ?? 'configured';
  const delegators = useMemo(() => config?.delegators ?? [], [config]);

  const onLaunch = () => {
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
      })
      .then((r) => setResult(r))
      .catch((e) => setLaunchError(e instanceof Error ? e.message : 'Launch failed'))
      .finally(() => setLaunching(false));
  };

  const onFocus = (agentId: string) => {
    setFocusBusy(true);
    setFocusError(null);
    api
      .focusSession(agentId)
      .then(() => setFocused(true))
      .catch((e) => setFocusError(e instanceof Error ? e.message : 'Focus failed'))
      .finally(() => setFocusBusy(false));
  };

  const link = result ? wrapperSessionLink(result) : null;

  return (
    <div className={styles.panel}>
      {/* Detail */}
      <div className={styles.detail}>
        <div className={styles.detailRow}>
          <span className={styles.ticketType}>{ticket.ticket_type}</span>
          <span className={styles.ticketId}>{ticket.id}</span>
        </div>
        <p className={styles.summary}>{ticket.summary}</p>
        <p className={styles.meta}>
          {ticket.project} &middot; {ticket.step_display_name ?? ticket.step}
        </p>
      </div>

      {/* Launch form */}
      {!result && (
        <div className={styles.form}>
          <label className={styles.field}>
            <span className={styles.fieldLabel}>Delegator</span>
            <select
              className={styles.select}
              value={delegator}
              onChange={(e) => setDelegator(e.target.value)}
            >
              <option value="">Default (auto)</option>
              {delegators.map((d) => (
                <option key={d.name} value={d.name}>
                  {d.display_name ?? d.name}
                </option>
              ))}
            </select>
          </label>

          <label className={styles.field}>
            <span className={styles.fieldLabel}>Wrapper</span>
            <select
              className={styles.select}
              value={wrapper}
              onChange={(e) => setWrapper(e.target.value)}
            >
              <option value="">Default ({defaultWrapperLabel})</option>
              <option value="tmux">tmux</option>
              <option value="vscode">vscode</option>
              <option value="cmux">cmux</option>
              <option value="zellij">zellij</option>
            </select>
          </label>

          <label className={styles.checkboxField}>
            <input type="checkbox" checked={yolo} onChange={(e) => setYolo(e.target.checked)} />
            <span>YOLO mode (auto-accept prompts)</span>
          </label>

          {launchError && <div className={styles.error}>{launchError}</div>}

          <button
            type="button"
            className={styles.launchBtn}
            onClick={onLaunch}
            disabled={launching}
          >
            {launching ? 'Launching…' : 'Launch ▸'}
          </button>
        </div>
      )}

      {/* Launch result + session links */}
      {result && (
        <div className={styles.result}>
          <div className={styles.resultHeader}>Launched ✓ {result.ticket_id}</div>
          <button
            type="button"
            className={styles.linkBtn}
            onClick={() => {
              navigate(`/agent/${encodeURIComponent(result.agent_id)}`);
              close();
            }}
          >
            Open agent detail
          </button>
          {link?.kind === 'open-url' && (
            <button
              type="button"
              className={styles.linkBtn}
              onClick={() => host.openExternal(link.url)}
            >
              {link.label}
            </button>
          )}
          {link?.kind === 'focus-api' && (
            <>
              <button
                type="button"
                className={styles.linkBtn}
                onClick={() => onFocus(result.agent_id)}
                disabled={focusBusy}
              >
                {focusBusy ? 'Focusing…' : focused ? `${link.label} ✓` : link.label}
              </button>
              {focusError && <div className={styles.error}>{focusError}</div>}
            </>
          )}
          {link?.kind === 'display' && (
            <div className={styles.sessionRef}>
              <span className={styles.fieldLabel}>{link.label}</span>
              <code>{link.detail}</code>
            </div>
          )}
        </div>
      )}

      {/* Issue-type workflow graph (the launch steps) */}
      <div className={styles.graphSection}>
        <div className={styles.graphLabel}>Workflow steps</div>
        {workflowError && <div className={styles.error}>{workflowError}</div>}
        {!workflowError && !workflow && <div className={styles.loading}>Loading workflow…</div>}
        {workflow && <WorkflowGraphView contents={workflow} />}
      </div>
    </div>
  );
}
