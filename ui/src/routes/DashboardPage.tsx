import { useCallback } from "react";
import { Link } from "react-router-dom";
import { DashboardView } from "@operator/webcomponents";
import { STATUS_POLL_MS, useApiQuery } from "../api";
import { healthQuery, kanbanQuery, queueStatusQuery } from "../api/definitions";
import { CONCEPTS } from "../concepts";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import { useRightPanel } from "../right-panel";
import { TicketDetailPanel } from "../components/TicketDetailPanel";

const DASHBOARD = CONCEPTS.dashboard;

export function DashboardPage() {
  const { open } = useRightPanel();
  const board = useApiQuery(kanbanQuery(), { pollIntervalMs: STATUS_POLL_MS });
  const queue = useApiQuery(queueStatusQuery(), { pollIntervalMs: STATUS_POLL_MS });
  const health = useApiQuery(healthQuery(), { pollIntervalMs: STATUS_POLL_MS });
  const error = [
    board.error && `Board: ${board.error.message}`,
    queue.error && `Queue: ${queue.error.message}`,
    health.error && `Health: ${health.error.message}`,
  ]
    .filter(Boolean)
    .join(" ");
  const openTicket = useCallback(
    (ticket: KanbanTicketCard) =>
      open(<TicketDetailPanel key={ticket.id} ticket={ticket} />, ticket.id),
    [open],
  );

  return (
    <DashboardView
      header={{
        title: DASHBOARD.label,
        summary: DASHBOARD.summary,
        docsUrl: DASHBOARD.docsUrl,
        icon: DASHBOARD.icon,
      }}
      health={health.data}
      queue={queue.data}
      board={board.data}
      loading={board.isLoading || queue.isLoading || health.isLoading}
      error={error || null}
      updatedLabel={board.data ? new Date(board.data.last_updated).toLocaleTimeString() : undefined}
      statusLink={<Link to="/status">View all sections →</Link>}
      onOpenTicket={openTicket}
    />
  );
}
