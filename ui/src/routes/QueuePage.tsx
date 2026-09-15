import { useCallback, useEffect, useState } from "react";
import { QueueView } from "@operator/webcomponents";
import { OperatorApi } from "../api-client";
import type { KanbanBoardResponse } from "../api-client";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { useHost } from "../host";
import { useRightPanel } from "../right-panel";
import { CONCEPTS } from "../concepts";
import { TicketDetailPanel } from "../components/TicketDetailPanel";

const QUEUE = CONCEPTS.queue;

const POLL_INTERVAL_MS = 3000;

export function QueuePage() {
  const host = useHost();
  const { open } = useRightPanel();
  const [api] = useState(() => new OperatorApi(host));
  const [board, setBoard] = useState<KanbanBoardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const openTicket = useCallback(
    (ticket: KanbanTicketCard) =>
      open(<TicketDetailPanel key={ticket.id} ticket={ticket} />, ticket.id),
    [open],
  );

  useEffect(() => {
    let cancelled = false;

    const refresh = () => {
      api
        .kanban()
        .then((b) => {
          if (!cancelled) {
            setBoard(b);
            setError(null);
          }
          return undefined;
        })
        .catch((e) => {
          if (!cancelled) {
            setError(e.message);
          }
        })
        .finally(() => {
          if (!cancelled) {
            setLoading(false);
          }
        });
    };

    refresh();
    const timer = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [api]);

  return (
    <QueueView
      header={{
        title: QUEUE.label,
        summary: QUEUE.summary,
        docsUrl: QUEUE.docsUrl,
        icon: QUEUE.icon,
      }}
      board={
        loading
          ? { status: "loading", message: "Loading queue..." }
          : board
            ? { status: "ready", data: board }
            : { status: "empty", message: "No tickets" }
      }
      error={error}
      updatedLabel={board ? new Date(board.last_updated).toLocaleTimeString() : undefined}
      onOpenTicket={openTicket}
    />
  );
}
