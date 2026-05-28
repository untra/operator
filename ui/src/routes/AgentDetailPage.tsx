import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { OperatorApi } from '../api-client';
import type { AgentDetailResponse } from '../api-client';
import { useHost } from '../host';
import styles from './AgentDetailPage.module.css';

export function AgentDetailPage() {
  const { id } = useParams<{ id: string }>();
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [agent, setAgent] = useState<AgentDetailResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    const load = () => {
      api.getAgent(id).then(setAgent).catch((e) => setError(e.message));
    };
    load();
    const interval = setInterval(load, 5000);
    return () => clearInterval(interval);
  }, [api, id]);

  if (error) return <div className={styles.error}>Error: {error}</div>;
  if (!agent) return <div className={styles.loading}>Loading...</div>;

  const elapsed = formatElapsed(agent.started_at);

  return (
    <div className={styles.page}>
      <div className={styles.breadcrumb}>
        <Link to="/">Dashboard</Link> / Agent {agent.id.slice(0, 8)}
      </div>

      <h1 className={styles.title}>
        {agent.ticket_id}
        <span className={styles.statusBadge} data-status={agent.status}>
          {agent.status}
        </span>
      </h1>

      <div className={styles.grid}>
        <Field label="Project" value={agent.project} />
        <Field label="Type" value={agent.ticket_type} />
        <Field label="Tool" value={agent.llm_tool} />
        <Field label="Model" value={agent.llm_model} />
        <Field label="Wrapper" value={agent.session_wrapper} />
        <Field label="Launch Mode" value={agent.launch_mode} />
        <Field label="Step" value={agent.current_step} />
        <Field label="Review" value={agent.review_state} />
        <Field label="Elapsed" value={elapsed} />
        <Field label="Paired" value={agent.paired ? 'Yes' : 'No'} />
      </div>

      {agent.pr_url && (
        <div className={styles.prSection}>
          <h2>Pull Request</h2>
          <a href={agent.pr_url} target="_blank" rel="noopener noreferrer">
            {agent.pr_url}
          </a>
          {agent.pr_status && (
            <span className={styles.prStatus}>{agent.pr_status}</span>
          )}
        </div>
      )}

      {agent.completed_steps.length > 0 && (
        <div className={styles.stepsSection}>
          <h2>Completed Steps</h2>
          <ol className={styles.stepsList}>
            {agent.completed_steps.map((step, i) => (
              <li key={i} className={styles.stepDone}>{step}</li>
            ))}
            {agent.current_step && (
              <li className={styles.stepActive}>{agent.current_step}</li>
            )}
          </ol>
        </div>
      )}

      {agent.worktree_path && (
        <div className={styles.meta}>
          <strong>Worktree:</strong> <code>{agent.worktree_path}</code>
        </div>
      )}

      <div className={styles.meta}>
        <strong>Last Activity:</strong> {new Date(agent.last_activity).toLocaleString()}
      </div>
    </div>
  );
}

function Field({ label, value }: { label: string; value: string | null | undefined }) {
  return (
    <div className={styles.field}>
      <div className={styles.fieldLabel}>{label}</div>
      <div className={styles.fieldValue}>{value ?? '—'}</div>
    </div>
  );
}

function formatElapsed(startedAt: string): string {
  const ms = Date.now() - new Date(startedAt).getTime();
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ${secs % 60}s`;
  const hrs = Math.floor(mins / 60);
  return `${hrs}h ${mins % 60}m`;
}
