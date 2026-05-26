import { useEffect, useState } from 'react';
import { useHost } from '../host';
import styles from './QueuePage.module.css';
import type { KanbanBoardResponse } from '@operator/bindings/KanbanBoardResponse';
import type { KanbanTicketCard } from '@operator/bindings/KanbanTicketCard';

export function QueuePage() {
  const host = useHost();
  const [board, setBoard] = useState<KanbanBoardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch(`${host.baseUrl()}/api/v1/queue/kanban`)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json() as Promise<KanbanBoardResponse>;
      })
      .then(setBoard)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [host]);

  if (loading) return <div className={styles.loading}>Loading queue...</div>;

  return (
    <div className={styles.page}>
      <h1 className={styles.title}>Queue</h1>

      {error && <div className={styles.error}>{error}</div>}

      {board && (
        <>
          <div className={styles.meta}>
            {board.total_count} tickets &middot; updated {new Date(board.last_updated).toLocaleTimeString()}
          </div>
          <div className={styles.columns}>
            <Column title="Queue" tickets={board.queue} />
            <Column title="Running" tickets={board.running} />
            <Column title="Awaiting" tickets={board.awaiting} />
            <Column title="Done" tickets={board.done} />
          </div>
        </>
      )}
    </div>
  );
}

function Column({ title, tickets }: { title: string; tickets: KanbanTicketCard[] }) {
  return (
    <div className={styles.column}>
      <div className={styles.columnHeader}>
        {title} <span className={styles.count}>({tickets.length})</span>
      </div>
      <div className={styles.cardList}>
        {tickets.map((t) => (
          <div key={t.id} className={styles.card}>
            <div className={styles.cardHeader}>
              <span className={styles.ticketType}>{t.ticket_type}</span>
              <span className={styles.ticketId}>{t.id}</span>
            </div>
            <div className={styles.cardSummary}>{t.summary}</div>
            <div className={styles.cardMeta}>
              {t.project} &middot; {t.step_display_name ?? t.step}
            </div>
          </div>
        ))}
        {tickets.length === 0 && <div className={styles.empty}>No tickets</div>}
      </div>
    </div>
  );
}
