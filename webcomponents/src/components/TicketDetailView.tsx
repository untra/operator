import type { ReactNode } from "react";
import type { IssueType } from "../generated/IssueType";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import { WorkflowGraph } from "../workflow/WorkflowGraph";
import { AsyncState, type AsyncValue } from "./AsyncState";
import styles from "./TicketDetailView.module.css";

export interface TicketDetailViewProps {
  ticket: KanbanTicketCard;
  launchControls?: ReactNode;
  launchedTicketId?: string;
  launchActions?: ReactNode;
  workflow: AsyncValue<IssueType>;
}

export function TicketDetailView({
  ticket,
  launchControls,
  launchedTicketId,
  launchActions,
  workflow,
}: TicketDetailViewProps) {
  return (
    <div className={styles.panel}>
      <div className={styles.detail}>
        <div className={styles.detailRow}>
          <span className={styles.ticketType}>{ticket.ticket_type}</span>
          <span className={styles.ticketId}>{ticket.id}</span>
        </div>
        <p className={styles.summary}>{ticket.summary}</p>
        <p className={styles.meta}>
          {ticket.project} &middot; {ticket.step_display_name ?? ticket.step}
        </p>
      </div>
      {launchControls}
      {launchedTicketId && (
        <div className={styles.result}>
          <div className={styles.resultHeader}>Launched ✓ {launchedTicketId}</div>
          {launchActions && <div className={styles.actions}>{launchActions}</div>}
        </div>
      )}
      <div className={styles.graphSection}>
        <div className={styles.graphLabel}>Workflow steps</div>
        <AsyncState value={workflow}>
          {(issueType) => <WorkflowGraph issueType={issueType} />}
        </AsyncState>
      </div>
    </div>
  );
}
