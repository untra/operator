import { useCallback } from "react";
import { QueueView } from "@operator/webcomponents";
import { STATUS_POLL_MS, useApiQuery } from "../api";
import { kanbanQuery } from "../api/definitions";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { useRightPanel } from "../right-panel";
import { CONCEPTS } from "../concepts";
import { TicketDetailPanel } from "../components/TicketDetailPanel";
import { TicketCreatePanel } from "../components/TicketCreatePanel";

const QUEUE = CONCEPTS.queue;

export function QueuePage() {
  const { open } = useRightPanel();
  const board = useApiQuery(kanbanQuery(), { pollIntervalMs: STATUS_POLL_MS });

  const openTicket = useCallback(
    (ticket: KanbanTicketCard) =>
      open(<TicketDetailPanel key={ticket.id} ticket={ticket} />, ticket.id),
    [open],
  );

  const onCreated = useCallback(() => undefined, []);

  const createTicket = useCallback(
    () => open(<TicketCreatePanel onCreated={onCreated} />, "new-ticket"),
    [open, onCreated],
  );

  return (
    <QueueView
      header={{
        title: QUEUE.label,
        summary: QUEUE.summary,
        docsUrl: QUEUE.docsUrl,
        icon: QUEUE.icon,
      }}
      board={
        board.isLoading
          ? { status: "loading", message: "Loading queue..." }
          : board.data
            ? { status: "ready", data: board.data }
            : { status: "empty", message: "No tickets" }
      }
      error={board.error?.message ?? null}
      updatedLabel={board.data ? new Date(board.data.last_updated).toLocaleTimeString() : undefined}
      onOpenTicket={openTicket}
      onCreateTicket={createTicket}
    />
  );
}
