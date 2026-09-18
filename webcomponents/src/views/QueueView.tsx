import { useMemo } from "react";
import type { KanbanBoardResponse } from "../generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import { AsyncState, type AsyncValue } from "../components/AsyncState";
import { KanbanBoard } from "../components/KanbanBoard";
import { KanbanFilterBar } from "../components/KanbanFilterBar";
import { PageHeader, type PageHeaderProps } from "../components/PageHeader";
import {
  facetsFromBoard,
  filterBoard,
  hasActiveFilters,
  useKanbanFilters,
} from "../shared/kanban-filters";
import styles from "./QueueView.module.css";

export interface QueueViewProps {
  header: PageHeaderProps;
  board: AsyncValue<KanbanBoardResponse>;
  error?: string | null;
  updatedLabel?: string;
  onOpenTicket?: (ticket: KanbanTicketCard) => void;
  /** Opens the create form. Omitted on surfaces that cannot write. */
  onCreateTicket?: () => void;
}

export function QueueView({
  header,
  board,
  error,
  updatedLabel,
  onOpenTicket,
  onCreateTicket,
}: QueueViewProps) {
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
          <FilterableBoard
            board={data}
            updatedLabel={updatedLabel}
            onOpenTicket={onOpenTicket}
            onCreateTicket={onCreateTicket}
          />
        )}
      </AsyncState>
    </div>
  );
}

function FilterableBoard({
  board,
  updatedLabel,
  onOpenTicket,
  onCreateTicket,
}: {
  board: KanbanBoardResponse;
  updatedLabel?: string;
  onOpenTicket?: (ticket: KanbanTicketCard) => void;
  onCreateTicket?: () => void;
}) {
  const { state, setState, clear } = useKanbanFilters();
  // The board object is replaced on every poll, so both derivations rerun.
  const facets = useMemo(() => facetsFromBoard(board), [board]);
  const filtered = useMemo(() => filterBoard(board, state), [board, state]);

  const active = hasActiveFilters(state);
  const countLabel = active
    ? `${filtered.total_count} of ${board.total_count} tickets`
    : `${board.total_count} tickets`;

  return (
    <>
      <KanbanFilterBar facets={facets} state={state} onChange={setState} onClear={clear} />
      <div className={styles.meta}>
        <output>
          {countLabel}
          {updatedLabel ? ` · updated ${updatedLabel}` : ""}
        </output>
        {onCreateTicket && (
          <button type="button" className={styles.newTicketBtn} onClick={onCreateTicket}>
            + New ticket
          </button>
        )}
      </div>
      {active && filtered.total_count === 0 ? (
        <div className={styles.noMatches}>
          No tickets match these filters.
          <button type="button" className={styles.linkBtn} onClick={clear}>
            Clear filters
          </button>
        </div>
      ) : (
        <KanbanBoard board={filtered} onOpenTicket={onOpenTicket} />
      )}
    </>
  );
}
