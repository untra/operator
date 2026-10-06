import { useCallback, useMemo, useState, type ChangeEvent } from "react";
import { Link, useParams } from "react-router-dom";
import {
  MarkdownView,
  StepProgress,
  openTargets,
  sessionTargets,
  ticketFileTargets,
  type OpenTarget,
  type SessionTarget,
} from "@operator/webcomponents";
import type { TicketAgent } from "@operator/bindings/TicketAgent";
import type { TicketDetailResponse } from "@operator/bindings/TicketDetailResponse";
import { ApiError } from "../api-client";
import { STATUS_POLL_MS, useApiMutation, useApiQuery } from "../api";
import {
  approveReviewMutation,
  focusSessionMutation,
  issueTypeDocumentQuery,
  moveTicketMutation,
  rejectReviewMutation,
  ticketQuery,
} from "../api/definitions";
import { BrandIcon } from "../components/BrandIcon";
import { useTicketLaunch } from "../components/use-ticket-launch";
import { useHost } from "../host";
import styles from "./TicketPage.module.css";

const MOVE_OPTIONS = [
  { value: "queued", label: "Queue" },
  { value: "running", label: "In progress" },
  { value: "awaiting", label: "Awaiting input" },
  { value: "completed", label: "Completed" },
];

const NO_STEPS: string[] = [];

const PROVIDER_LABEL: Record<string, string> = {
  jira: "Jira",
  linear: "Linear",
  github: "GitHub",
  gitlab: "GitLab",
  openspec: "OpenSpec",
};

function OpenGroup({
  label,
  path,
  targets,
  remoteHost,
}: {
  label: string;
  path: string;
  targets: OpenTarget[];
  remoteHost?: string | null;
}) {
  const host = useHost();
  const open = useCallback(
    (target: OpenTarget) => {
      if (host.opensFiles && !remoteHost) {
        host.openFile(path);
      } else {
        host.openExternal(target.url);
      }
    },
    [host, path, remoteHost],
  );
  return (
    <fieldset className={styles.openRow}>
      <legend className={styles.label}>{label}</legend>
      {targets.map((target) => (
        <OpenButton key={target.editor} target={target} onOpen={open} />
      ))}
      <code className={styles.path}>{remoteHost ? `${remoteHost}:${path}` : path}</code>
    </fieldset>
  );
}

function OpenButton({
  target,
  onOpen,
}: {
  target: OpenTarget;
  onOpen: (target: OpenTarget) => void;
}) {
  const onClick = useCallback(() => onOpen(target), [onOpen, target]);
  return (
    <button type="button" className={styles.chip} onClick={onClick}>
      {target.editor !== "jetbrains" && <BrandIcon name={target.editor} />}
      {target.label}
    </button>
  );
}

function SessionAction({ agentId, target }: { agentId: string; target: SessionTarget }) {
  const host = useHost();
  const focus = useApiMutation(focusSessionMutation);
  const [copied, setCopied] = useState(false);
  const onFocus = useCallback(() => focus.mutate({ agentId }), [agentId, focus]);
  const onOpen = useCallback(() => {
    if (target.kind === "open-url") {
      host.openExternal(target.url);
    }
  }, [host, target]);
  const onCopy = useCallback(() => {
    if (target.kind === "command") {
      void navigator.clipboard?.writeText(target.command).then(() => setCopied(true));
    }
  }, [target]);

  if (target.kind === "open-url") {
    return (
      <button type="button" className={styles.chip} onClick={onOpen}>
        {target.label}
      </button>
    );
  }
  if (target.kind === "focus-api") {
    return (
      <>
        <button type="button" className={styles.chip} onClick={onFocus} disabled={focus.isPending}>
          {target.label}
        </button>
        {focus.error && <span className={styles.error}>{focus.error.message}</span>}
      </>
    );
  }
  if (target.kind === "command") {
    return (
      <button type="button" className={styles.chip} onClick={onCopy} title="Copy">
        <code>{target.command}</code>
        {copied ? " ✓" : ""}
      </button>
    );
  }
  return (
    <span className={styles.muted}>
      {target.label} <code>{target.detail}</code>
    </span>
  );
}

function Review({ agent }: { agent: TicketAgent }) {
  const approve = useApiMutation(approveReviewMutation);
  const reject = useApiMutation(rejectReviewMutation);
  const [reason, setReason] = useState("");
  const onApprove = useCallback(() => approve.mutate({ agentId: agent.id }), [agent.id, approve]);
  const onReject = useCallback(
    () => reject.mutate({ agentId: agent.id, reason }),
    [agent.id, reason, reject],
  );
  const onReason = useCallback((e: ChangeEvent<HTMLInputElement>) => setReason(e.target.value), []);
  const busy = approve.isPending || reject.isPending;
  return (
    <fieldset className={styles.actions}>
      <legend className={styles.label}>Review</legend>
      <button type="button" disabled={busy} onClick={onApprove}>
        Approve
      </button>
      <input aria-label="Rejection reason" value={reason} onChange={onReason} />
      <button type="button" disabled={busy || !reason.trim()} onClick={onReject}>
        Reject
      </button>
      {(approve.error ?? reject.error) && (
        <span className={styles.error}>{(approve.error ?? reject.error)?.message}</span>
      )}
    </fieldset>
  );
}

function Launch({ ticketId }: { ticketId: string }) {
  const [open, setOpen] = useState(false);
  const { controls } = useTicketLaunch(ticketId);
  const toggle = useCallback(() => setOpen((v) => !v), []);
  return (
    <>
      <button type="button" onClick={toggle} aria-expanded={open}>
        Launch…
      </button>
      {open && <div className={styles.launch}>{controls}</div>}
    </>
  );
}

function Move({ ticketId }: { ticketId: string }) {
  const move = useApiMutation(moveTicketMutation);
  const onChange = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) => {
      if (e.target.value) {
        move.mutate({ ticketId, status: e.target.value });
      }
    },
    [move, ticketId],
  );
  return (
    <>
      <select aria-label="Move to" value="" onChange={onChange} disabled={move.isPending}>
        <option value="">Move to…</option>
        {MOVE_OPTIONS.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
      {move.error && <span className={styles.error}>{move.error.message}</span>}
    </>
  );
}

function ExternalIssue({ ticket }: { ticket: TicketDetailResponse }) {
  const host = useHost();
  const url = ticket.external_url;
  const onOpen = useCallback(() => {
    if (url) {
      host.openExternal(url);
    }
  }, [host, url]);
  if (!url) {
    return null;
  }
  const provider = ticket.external_provider ?? "";
  return (
    <button type="button" className={styles.chip} onClick={onOpen}>
      {provider && <BrandIcon name={provider} />}
      {PROVIDER_LABEL[provider] ?? provider} {ticket.external_id ?? "issue"} ↗
    </button>
  );
}

function Dependencies({ ticket }: { ticket: TicketDetailResponse }) {
  if (ticket.dependencies.length === 0 && ticket.dependents.length === 0) {
    return null;
  }
  return (
    <section className={styles.card} aria-label="Dependencies">
      {ticket.dependencies.length > 0 && (
        <>
          <h2>Depends on</h2>
          <ul className={styles.list}>
            {ticket.dependencies.map((dep) => (
              <li key={dep.reference}>
                <span aria-hidden="true">{dep.satisfied ? "✓ " : "⏳ "}</span>
                {dep.ticket_id ? (
                  <Link to={`/tickets/${encodeURIComponent(dep.ticket_id)}`}>{dep.ticket_id}</Link>
                ) : (
                  <span>{dep.reference}</span>
                )}
                <span className={styles.muted}> {dep.column ?? "not synced"}</span>
              </li>
            ))}
          </ul>
        </>
      )}
      {ticket.dependents.length > 0 && (
        <>
          <h2>Needed by</h2>
          <ul className={styles.list}>
            {ticket.dependents.map((id) => (
              <li key={id}>
                <Link to={`/tickets/${encodeURIComponent(id)}`}>{id}</Link>
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}

function AgentCard({ agent }: { agent: TicketAgent }) {
  const host = useHost();
  const pr = agent.pr_url;
  const onPr = useCallback(() => {
    if (pr) {
      host.openExternal(pr);
    }
  }, [host, pr]);
  return (
    <section className={styles.card} aria-label="Agent">
      <h2>Agent</h2>
      <p>
        <Link to={`/agent/${encodeURIComponent(agent.id)}`}>{agent.status}</Link>
        {agent.llm_tool && (
          <span className={styles.muted}>
            {" "}
            · {agent.llm_tool}
            {agent.llm_model ? `/${agent.llm_model}` : ""}
          </span>
        )}
      </p>
      {pr && (
        <button type="button" className={styles.chip} onClick={onPr}>
          PR {agent.pr_number ? `#${agent.pr_number}` : ""} {agent.pr_status ?? ""} ↗
        </button>
      )}
      {agent.last_message && <p className={styles.muted}>{agent.last_message}</p>}
    </section>
  );
}

function TicketView({ ticket }: { ticket: TicketDetailResponse }) {
  const workflow = useApiQuery(issueTypeDocumentQuery(ticket.ticket_type));
  const agent = ticket.agent ?? null;
  const worktree = agent?.worktree_path ?? ticket.worktree_path ?? null;
  const sessions = useMemo(() => (agent ? sessionTargets(agent) : []), [agent]);
  const fileTargets = useMemo(
    () => ticketFileTargets(ticket.id, ticket.filepath),
    [ticket.id, ticket.filepath],
  );
  const remoteHost = agent?.remote_host;
  const worktreeTargets = useMemo(
    () => (worktree ? openTargets(worktree, remoteHost) : []),
    [worktree, remoteHost],
  );

  return (
    <>
      <header className={styles.header}>
        <h1 className={styles.title}>
          {ticket.id} <span className={styles.summary}>{ticket.summary}</span>
        </h1>
        <div className={styles.badges}>
          <span className={styles.badge} data-column={ticket.column}>
            {ticket.column}
          </span>
          <span className={styles.badge}>{ticket.priority}</span>
          <span className={styles.badge}>{ticket.ticket_type}</span>
          <span className={styles.badge}>{ticket.project}</span>
        </div>
        <div className={styles.meta}>
          {ticket.campaign && (
            <Link to={`/campaigns/${encodeURIComponent(ticket.campaign)}`}>{ticket.campaign}</Link>
          )}
          <ExternalIssue ticket={ticket} />
          {ticket.branch && <code>{ticket.branch}</code>}
        </div>
      </header>

      <div className={styles.actions}>
        {ticket.column === "queue" && !agent && <Launch ticketId={ticket.id} />}
        <Move ticketId={ticket.id} />
        {agent?.status === "awaiting_input" && agent.review_state && <Review agent={agent} />}
      </div>

      <div className={styles.open}>
        <OpenGroup label="Open ticket file" path={ticket.filepath} targets={fileTargets} />
        {worktree && (
          <OpenGroup
            label="Open worktree"
            path={worktree}
            targets={worktreeTargets}
            remoteHost={agent?.remote_host}
          />
        )}
        {agent && sessions.length > 0 && (
          <fieldset className={styles.openRow}>
            <legend className={styles.label}>Session</legend>
            {sessions.map((target) => (
              <SessionAction key={target.label} agentId={agent.id} target={target} />
            ))}
          </fieldset>
        )}
      </div>

      <div className={styles.columns}>
        <MarkdownView source={ticket.body} className={styles.body} />
        <aside className={styles.side}>
          {workflow.data && (
            <section className={styles.card} aria-label="Steps">
              <h2>Steps</h2>
              <StepProgress
                steps={workflow.data.steps}
                current={agent?.current_step ?? ticket.step}
                completed={agent?.completed_steps ?? NO_STEPS}
                finished={ticket.column === "completed"}
              />
            </section>
          )}
          {agent && <AgentCard agent={agent} />}
          <Dependencies ticket={ticket} />
        </aside>
      </div>
    </>
  );
}

export function TicketPage() {
  const { id = "" } = useParams<{ id: string }>();
  const query = useApiQuery(ticketQuery(id), {
    enabled: Boolean(id),
    pollIntervalMs: STATUS_POLL_MS,
  });

  let content;
  if (query.error) {
    content =
      query.error instanceof ApiError && query.error.status === 404 ? (
        <p className={styles.status}>Ticket not found</p>
      ) : (
        <output className={styles.status}>{query.error.message}</output>
      );
  } else if (query.data) {
    content = <TicketView ticket={query.data} />;
  } else {
    content = <p className={styles.status}>Loading ticket…</p>;
  }
  return <main className={styles.page}>{content}</main>;
}
