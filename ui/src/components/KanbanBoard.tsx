import type { KanbanBoardResponse } from '@operator/bindings/KanbanBoardResponse';
import type { KanbanTicketCard } from '@operator/bindings/KanbanTicketCard';
import { useRightPanel } from '../right-panel';
import { TicketDetailPanel } from './TicketDetailPanel';
import styles from './KanbanBoard.module.css';

/**
 * Three-column kanban board mirroring the operator TUI's ticket columns:
 * TODO QUEUE / IN PROGRESS / DONE. The API's `awaiting` tickets are folded
 * into IN PROGRESS (with a distinct paused indicator), matching the TUI which
 * keeps awaiting tickets in the in-progress panel.
 *
 * Cards in the TODO and IN PROGRESS columns are clickable: they open the
 * right-hand detail sidepanel with that ticket's detail, launch form, and
 * issue-type workflow graph. DONE cards are not interactive.
 */
export function KanbanBoard({ board }: { board: KanbanBoardResponse }) {
  const { open } = useRightPanel();
  const openTicket = (ticket: KanbanTicketCard) =>
    open(<TicketDetailPanel ticket={ticket} />, ticket.id);

  const inProgress = [...board.running, ...board.awaiting];
  return (
    <div className={styles.columns}>
      <Column title="TODO QUEUE" tickets={board.queue} onOpen={openTicket} />
      <Column title="IN PROGRESS" tickets={inProgress} onOpen={openTicket} />
      <Column title="DONE" tickets={board.done} />
    </div>
  );
}

function Column({
  title,
  tickets,
  onOpen,
}: {
  title: string;
  tickets: KanbanTicketCard[];
  onOpen?: (ticket: KanbanTicketCard) => void;
}) {
  return (
    <div className={styles.column}>
      <div className={styles.columnHeader}>
        {title} <span className={styles.count}>({tickets.length})</span>
      </div>
      <div className={styles.cardList}>
        {tickets.map((t) => (
          <Card key={t.id} ticket={t} onOpen={onOpen} />
        ))}
        {tickets.length === 0 && <div className={styles.empty}>No tickets</div>}
      </div>
    </div>
  );
}

function Card({
  ticket,
  onOpen,
}: {
  ticket: KanbanTicketCard;
  onOpen?: (ticket: KanbanTicketCard) => void;
}) {
  const inner = (
    <>
      <div className={styles.cardHeader}>
        <span className={styles.statusIcon}>{statusIcon(ticket.status)}</span>
        <span className={styles.ticketType}>{ticket.ticket_type}</span>
        <span className={styles.ticketId}>{ticket.id}</span>
      </div>
      <div className={styles.cardSummary}>{ticket.summary}</div>
      <div className={styles.cardMeta}>
        {ticket.project} &middot; {ticket.step_display_name ?? ticket.step}
      </div>
    </>
  );

  if (!onOpen) {
    return (
      <div className={styles.card} data-priority={priorityKey(ticket.priority)}>
        {inner}
      </div>
    );
  }
  return (
    <button
      type="button"
      className={`${styles.card} ${styles.cardClickable}`}
      data-priority={priorityKey(ticket.priority)}
      onClick={() => onOpen(ticket)}
      title="Open ticket detail"
    >
      {inner}
    </button>
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
