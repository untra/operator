import type { KanbanBoardResponse } from '@operator/bindings/KanbanBoardResponse';
import type { KanbanTicketCard } from '@operator/bindings/KanbanTicketCard';
import styles from './KanbanBoard.module.css';

/**
 * Three-column kanban board mirroring the operator TUI's ticket columns:
 * TODO QUEUE / IN PROGRESS / DONE. The API's `awaiting` tickets are folded
 * into IN PROGRESS (with a distinct paused indicator), matching the TUI which
 * keeps awaiting tickets in the in-progress panel.
 */
export function KanbanBoard({ board }: { board: KanbanBoardResponse }) {
  const inProgress = [...board.running, ...board.awaiting];
  return (
    <div className={styles.columns}>
      <Column title="TODO QUEUE" tickets={board.queue} />
      <Column title="IN PROGRESS" tickets={inProgress} />
      <Column title="DONE" tickets={board.done} />
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
          <Card key={t.id} ticket={t} />
        ))}
        {tickets.length === 0 && <div className={styles.empty}>No tickets</div>}
      </div>
    </div>
  );
}

function Card({ ticket }: { ticket: KanbanTicketCard }) {
  return (
    <div className={styles.card} data-priority={priorityKey(ticket.priority)}>
      <div className={styles.cardHeader}>
        <span className={styles.statusIcon}>{statusIcon(ticket.status)}</span>
        <span className={styles.ticketType}>{ticket.ticket_type}</span>
        <span className={styles.ticketId}>{ticket.id}</span>
      </div>
      <div className={styles.cardSummary}>{ticket.summary}</div>
      <div className={styles.cardMeta}>
        {ticket.project} &middot; {ticket.step_display_name ?? ticket.step}
      </div>
    </div>
  );
}

function statusIcon(status: string): string {
  switch (status) {
    case 'running':
      return '▶'; // ▶
    case 'awaiting':
    case 'waiting':
    case 'blocked':
      return '⏸'; // ⏸
    case 'completed':
    case 'done':
      return '✓'; // ✓
    default:
      return '•'; // • queued
  }
}

/** Maps "P0-critical".."P3-low" to a stable key for priority-colored styling. */
function priorityKey(priority: string): string {
  const match = priority.match(/^P([0-3])/i);
  return match ? `p${match[1]}` : 'p2';
}
