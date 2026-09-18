import { useCallback, useEffect, useRef, useState } from "react";
import { QueueView } from "@operator/webcomponents";
import { OperatorApi } from "../api-client";
import type { KanbanBoardResponse } from "../api-client";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { useHost } from "../host";
import { useRightPanel } from "../right-panel";
import { CONCEPTS } from "../concepts";
import { TicketDetailPanel } from "../components/TicketDetailPanel";
import { TicketCreatePanel } from "../components/TicketCreatePanel";

const QUEUE = CONCEPTS.queue;

const POLL_INTERVAL_MS = 3000;

export function QueuePage() {
  const host = useHost();
  const { open } = useRightPanel();
  const [api] = useState(() => new OperatorApi(host));
  const [board, setBoard] = useState<KanbanBoardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Guards every state write, so a poll or a post-write refresh that lands
  // after unmount is dropped instead of setting state on a dead component.
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  // Stable, so both the poll and a write can pull the board.
  const refresh = useCallback(() => {
    api
      .kanban()
      .then((b) => {
        if (mounted.current) {
          setBoard(b);
          setError(null);
        }
        return undefined;
      })
      .catch((e) => {
        if (mounted.current) {
          setError(e.message);
        }
      })
      .finally(() => {
        if (mounted.current) {
          setLoading(false);
        }
      });
  }, [api]);

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, POLL_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [refresh]);

  const openTicket = useCallback(
    (ticket: KanbanTicketCard) =>
      open(<TicketDetailPanel key={ticket.id} ticket={ticket} />, ticket.id),
    [open],
  );

  // A created ticket shows up immediately rather than on the next poll.
  const createTicket = useCallback(
    () => open(<TicketCreatePanel onCreated={refresh} />, "new-ticket"),
    [open, refresh],
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
        loading
          ? { status: "loading", message: "Loading queue..." }
          : board
            ? { status: "ready", data: board }
            : { status: "empty", message: "No tickets" }
      }
      error={error}
      updatedLabel={board ? new Date(board.last_updated).toLocaleTimeString() : undefined}
      onOpenTicket={openTicket}
      onCreateTicket={createTicket}
    />
  );
}
