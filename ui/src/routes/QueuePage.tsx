import { useEffect, useState } from 'react';
import { OperatorApi } from '../api-client';
import type { KanbanBoardResponse } from '../api-client';
import { useHost } from '../host';
import { CONCEPTS } from '../concepts';
import { PageHeader } from '../components/PageHeader';
import { KanbanBoard } from '../components/KanbanBoard';
import styles from './QueuePage.module.css';

const QUEUE = CONCEPTS.queue;

const POLL_INTERVAL_MS = 3000;

export function QueuePage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [board, setBoard] = useState<KanbanBoardResponse | null>(null);
  const [loading, setLoading] = useState(true);
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
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    };

    refresh();
    const timer = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [api]);

  if (loading) return <div className={styles.loading}>Loading queue...</div>;

  return (
    <div className={styles.page}>
      <PageHeader
        title={QUEUE.label}
        summary={QUEUE.summary}
        docsUrl={QUEUE.docsUrl}
        icon={QUEUE.icon}
      />

      {error && <div className={styles.error}>{error}</div>}

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
