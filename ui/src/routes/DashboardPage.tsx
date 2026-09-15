import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { DashboardView } from "@operator/webcomponents";
import { OperatorApi } from "../api-client";
import type { HealthResponse, QueueStatusResponse, KanbanBoardResponse } from "../api-client";
import { useHost } from "../host";
import { CONCEPTS } from "../concepts";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { useRightPanel } from "../right-panel";
import { TicketDetailPanel } from "../components/TicketDetailPanel";

const DASHBOARD = CONCEPTS.dashboard;

const POLL_INTERVAL_MS = 3000;

export function DashboardPage() {
  const host = useHost();
  const { open } = useRightPanel();
  const [api] = useState(() => new OperatorApi(host));
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [queue, setQueue] = useState<QueueStatusResponse | null>(null);
  const [board, setBoard] = useState<KanbanBoardResponse | null>(null);
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
        });
      api
        .queueStatus()
        .then((q) => !cancelled && setQueue(q))
        .catch(() => {});
      api
        .health()
        .then((h) => !cancelled && setHealth(h))
        .catch(() => {});
    };

    refresh();
    const timer = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [api]);

  return (
    <DashboardView
      header={{
        title: DASHBOARD.label,
        summary: DASHBOARD.summary,
        docsUrl: DASHBOARD.docsUrl,
        icon: DASHBOARD.icon,
      }}
      health={health}
      queue={queue}
      board={board}
      error={error}
      updatedLabel={board ? new Date(board.last_updated).toLocaleTimeString() : undefined}
      statusLink={<Link to="/status">View all sections →</Link>}
      onOpenTicket={openTicket}
    />
  );
}
