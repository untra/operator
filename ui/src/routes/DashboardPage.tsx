import { useEffect, useState } from 'react';
import { OperatorApi } from '../api-client';
import type { HealthResponse, QueueStatusResponse, KanbanBoardResponse } from '../api-client';
import { useHost } from '../host';
import { KanbanBoard } from '../components/KanbanBoard';
import styles from './DashboardPage.module.css';

const POLL_INTERVAL_MS = 3000;

export function DashboardPage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [queue, setQueue] = useState<QueueStatusResponse | null>(null);
  const [board, setBoard] = useState<KanbanBoardResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const refresh = () => {
      api
        .kanban()
        .then((b) => {
          if (cancelled) return;
          setBoard(b);
          setError(null);
        })
        .catch((e) => {
          if (!cancelled) setError(e.message);
        });
      api.queueStatus().then((q) => !cancelled && setQueue(q)).catch(() => {});
      api.health().then((h) => !cancelled && setHealth(h)).catch(() => {});
    };

    refresh();
    const timer = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [api]);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h1 className={styles.title}>Dashboard</h1>
        {health && (
          <span className={styles.statusBanner}>
            API: {health.status} &middot; v{health.version}
          </span>
        )}
      </div>

      {error && <div className={styles.error}>API: {error}</div>}

      <div className={styles.cards}>
        <Card label="Queued" value={queue?.queued} />
        <Card label="In Progress" value={queue?.in_progress} />
        <Card label="Awaiting" value={queue?.awaiting} />
        <Card label="Completed" value={queue?.completed} />
      </div>

      {board && (
        <>
          <div className={styles.meta}>
            {board.total_count} tickets &middot; updated{' '}
            {new Date(board.last_updated).toLocaleTimeString()}
          </div>
          <KanbanBoard board={board} />
        </>
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
