import { useEffect, useState } from 'react';
import { OperatorApi } from '../api-client';
import type { HealthResponse, QueueStatusResponse, ActiveAgentsResponse } from '../api-client';
import { useHost } from '../host';
import styles from './DashboardPage.module.css';

export function DashboardPage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [queue, setQueue] = useState<QueueStatusResponse | null>(null);
  const [agents, setAgents] = useState<ActiveAgentsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.health().then(setHealth).catch((e) => setError(e.message));
    api.queueStatus().then(setQueue).catch(() => {});
    api.activeAgents().then(setAgents).catch(() => {});
  }, [api]);

  return (
    <div className={styles.page}>
      <h1 className={styles.title}>Dashboard</h1>

      {error && <div className={styles.error}>API: {error}</div>}

      {health && (
        <div className={styles.statusBanner}>
          API: {health.status} &middot; v{health.version}
        </div>
      )}

      <div className={styles.cards}>
        <Card label="Queued" value={queue?.queued} />
        <Card label="In Progress" value={queue?.in_progress} />
        <Card label="Awaiting" value={queue?.awaiting} />
        <Card label="Completed" value={queue?.completed} />
      </div>

      {agents && agents.count > 0 && (
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Active Agents ({agents.count})</h2>
          <table className={styles.table}>
            <thead>
              <tr>
                <th>ID</th>
                <th>Ticket</th>
                <th>Project</th>
                <th>Status</th>
                <th>Step</th>
              </tr>
            </thead>
            <tbody>
              {agents.agents.map((a) => (
                <tr key={a.id}>
                  <td>{a.id}</td>
                  <td>{a.ticket_id}</td>
                  <td>{a.project}</td>
                  <td>{a.status}</td>
                  <td>{a.current_step ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}
    </div>
  );
}

function Card({ label, value }: { label: string; value?: number }) {
  return (
    <div className={styles.card}>
      <div className={styles.cardValue}>{value ?? '—'}</div>
      <div className={styles.cardLabel}>{label}</div>
    </div>
  );
}
