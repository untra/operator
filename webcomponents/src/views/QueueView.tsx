import { useMemo } from "react";
import type { KanbanBoardResponse } from "../generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import { AsyncState, type AsyncValue } from "../components/AsyncState";
import { KanbanBoard } from "../components/KanbanBoard";
import { PageHeader, type PageHeaderProps } from "../components/PageHeader";
import styles from "./QueueView.module.css";

export interface QueueViewProps {
  header: PageHeaderProps;
  board: AsyncValue<KanbanBoardResponse>;
  error?: string | null;
  updatedLabel?: string;
  onOpenTicket?: (ticket: KanbanTicketCard) => void;
}

export function QueueView({ header, board, error, updatedLabel, onOpenTicket }: QueueViewProps) {
  const errorValue = useMemo<AsyncValue<never> | null>(
    () => (error ? { status: "error", message: error } : null),
    [error],
  );
  return (
    <div className={styles.page}>
      <PageHeader {...header} />
      {errorValue && <AsyncState value={errorValue}>{() => null}</AsyncState>}
      <AsyncState value={board}>
        {(data) => (
          <>
            <div className={styles.meta}>
              {data.total_count} tickets{updatedLabel ? ` · updated ${updatedLabel}` : ""}
            </div>
            <KanbanBoard board={data} onOpenTicket={onOpenTicket} />
          </>
        )}
      </AsyncState>
    </div>
  );
}
