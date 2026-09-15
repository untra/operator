import { useCallback, useMemo } from "react";
import type { KanbanBoardResponse } from "../generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import styles from "./KanbanBoard.module.css";

export interface KanbanBoardProps {
  board: KanbanBoardResponse;
  onOpenTicket?: (ticket: KanbanTicketCard) => void;
}

export function KanbanBoard({ board, onOpenTicket }: KanbanBoardProps) {
  const inProgress = useMemo(
    () => [...board.running, ...board.awaiting],
    [board.running, board.awaiting],
  );
  return (
    <div className={styles.columns}>
      <Column title="TODO QUEUE" tickets={board.queue} onOpen={onOpenTicket} />
      <Column title="IN PROGRESS" tickets={inProgress} onOpen={onOpenTicket} />
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
    <section className={styles.column} aria-label={title}>
      <div className={styles.columnHeader}>
        {title} <span className={styles.count}>({tickets.length})</span>
      </div>
      <div className={styles.cardList}>
        {tickets.map((ticket) => (
          <Card key={ticket.id} ticket={ticket} onOpen={onOpen} />
        ))}
        {tickets.length === 0 && <div className={styles.empty}>No tickets</div>}
      </div>
    </section>
  );
}

function Card({
  ticket,
  onOpen,
}: {
  ticket: KanbanTicketCard;
  onOpen?: (ticket: KanbanTicketCard) => void;
}) {
  const handleOpen = useCallback(() => onOpen?.(ticket), [onOpen, ticket]);
  const contents = (
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
        {contents}
      </div>
    );
  }
  return (
    <button
      type="button"
      className={`${styles.card} ${styles.cardClickable}`}
      data-priority={priorityKey(ticket.priority)}
      onClick={handleOpen}
      title="Open ticket detail"
    >
      {contents}
    </button>
  );
}

function statusIcon(status: string): string {
  switch (status) {
    case "running":
      return "▶";
    case "awaiting":
    case "waiting":
    case "blocked":
      return "⏸";
    case "completed":
    case "done":
      return "✓";
    default:
      return "•";
  }
}

function priorityKey(priority: string): string {
  const match = priority.match(/^P([0-3])/i);
  return match ? `p${match[1]}` : "p2";
}
