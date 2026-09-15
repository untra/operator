import { useCallback } from "react";
import { KanbanBoard as SharedKanbanBoard } from "@operator/webcomponents";
import type { KanbanBoardResponse } from "@operator/bindings/KanbanBoardResponse";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { useRightPanel } from "../right-panel";
import { TicketDetailPanel } from "./TicketDetailPanel";

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
  const openTicket = useCallback(
    (ticket: KanbanTicketCard) =>
      open(<TicketDetailPanel key={ticket.id} ticket={ticket} />, ticket.id),
    [open],
  );

  return <SharedKanbanBoard board={board} onOpenTicket={openTicket} />;
}
