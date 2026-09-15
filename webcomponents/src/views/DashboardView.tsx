import type { ReactNode } from "react";
import type { HealthResponse } from "../generated/HealthResponse";
import type { KanbanBoardResponse } from "../generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import type { QueueStatusResponse } from "../generated/QueueStatusResponse";
import { KanbanBoard } from "../components/KanbanBoard";
import { PageHeader, type PageHeaderProps } from "../components/PageHeader";
import styles from "./DashboardView.module.css";

export interface DashboardViewProps {
  header: PageHeaderProps;
  health: HealthResponse | null;
  queue: QueueStatusResponse | null;
  board: KanbanBoardResponse | null;
  error?: string | null;
  updatedLabel?: string;
  statusLink: ReactNode;
  onOpenTicket?: (ticket: KanbanTicketCard) => void;
}

export function DashboardView({
  header,
  health,
  queue,
  board,
  error,
  updatedLabel,
  statusLink,
  onOpenTicket,
}: DashboardViewProps) {
  const metrics = [
    ["Queued", queue?.queued],
    ["In Progress", queue?.in_progress],
    ["Awaiting", queue?.awaiting],
    ["Completed", queue?.completed],
  ] as const;
  return (
    <div className={styles.page}>
      <PageHeader {...header} />
      <div className={styles.subBar}>
        {health && (
          <span className={styles.statusBanner}>
            API: {health.status} &middot; v{health.version}
          </span>
        )}
        <span className={styles.statusLink}>{statusLink}</span>
      </div>
      {error && <div className={styles.error}>API: {error}</div>}
      <div className={styles.cards}>
        {metrics.map(([label, value]) => (
          <div className={styles.card} key={label}>
            <div className={styles.cardValue}>{value ?? "-"}</div>
            <div className={styles.cardLabel}>{label}</div>
          </div>
        ))}
      </div>
      {board && (
        <>
          <div className={styles.meta}>
            {board.total_count} tickets{updatedLabel ? ` · updated ${updatedLabel}` : ""}
          </div>
          <KanbanBoard board={board} onOpenTicket={onOpenTicket} />
        </>
      )}
    </div>
  );
}
