import { useCallback, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { PremiumGate } from "@operator/webcomponents";
import type { BlockReason } from "@operator/bindings/BlockReason";
import type { CampaignLaunchOutcome } from "@operator/bindings/CampaignLaunchOutcome";
import type { CampaignMember } from "@operator/bindings/CampaignMember";
import type { CampaignStatusView } from "@operator/bindings/CampaignStatusView";
import { STATUS_POLL_MS, useApiMutation, useApiQuery } from "../api";
import {
  activeAgentsQuery,
  campaignQuery,
  campaignStatusQuery,
  launchCampaignMutation,
  licenseQuery,
  pauseCampaignMutation,
} from "../api/definitions";
import styles from "./CampaignsPage.module.css";

const LAUNCHABLE = new Set(["draft", "ready", "paused"]);

function blockedLabel(reason: BlockReason): string {
  if (reason.kind === "gate") {
    return `waiting on ${reason.refs.join(", ")}`;
  }
  if (reason.kind === "unsynced") {
    return `waiting on unsynced ${reason.refs.join(", ")}`;
  }
  return BLOCKED_LABELS[reason.kind];
}

const BLOCKED_LABELS = {
  dependencies: "waiting on dependencies",
  license: "needs a Premium license",
  capacity: "waiting for a free slot",
} as const;

function cap(value: number, limit: number | null | undefined): string {
  return limit == null ? String(value) : `${value} / ${limit}`;
}

function Member({
  member,
  status,
  agentId,
}: {
  member: CampaignMember;
  status: CampaignStatusView | undefined;
  agentId: string | undefined;
}) {
  const blocked = status?.report.blocked[member.id];
  const awaiting = status?.report.awaiting_operator.includes(member.id);
  const stalled = status?.report.stalled.includes(member.id);
  return (
    <div className={styles.member}>
      <div>
        <Link to={`/tickets/${encodeURIComponent(member.id)}`}>{member.id}</Link>{" "}
        <span className={styles.muted}>{member.column}</span>
        {agentId && (
          <>
            {" · "}
            <Link to={`/agent/${encodeURIComponent(agentId)}`}>agent</Link>
          </>
        )}
      </div>
      <div>{member.summary}</div>
      {member.depends_on.length > 0 && (
        <div className={styles.muted}>depends on {member.depends_on.join(", ")}</div>
      )}
      {blocked && <div className={styles.note}>{blockedLabel(blocked)}</div>}
      {awaiting && <div className={styles.note}>awaiting operator (paired)</div>}
      {stalled && <div className={styles.note}>stalled: in progress with no live agent</div>}
    </div>
  );
}

function LaunchPreview({
  outcome,
  busy,
  onConfirm,
  onCancel,
}: {
  outcome: CampaignLaunchOutcome;
  busy: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const blocked = outcome.violations.length > 0;
  return (
    <section className={styles.confirm} aria-label="Launch preview">
      <h2>Launch preview</h2>
      <p>
        {outcome.waves.length} {outcome.waves.length === 1 ? "wave" : "waves"}; the first launches{" "}
        {outcome.report.launched.length > 0 ? outcome.report.launched.join(", ") : "nothing yet"}.
      </p>
      {blocked && (
        <ul>
          {outcome.violations.map((v) => (
            <li key={v} className={styles.violation}>
              {v}
            </li>
          ))}
        </ul>
      )}
      <div className={styles.actions}>
        <button type="button" disabled={busy || blocked} onClick={onConfirm}>
          Confirm launch
        </button>
        <button type="button" disabled={busy} onClick={onCancel}>
          Cancel
        </button>
      </div>
    </section>
  );
}

function CampaignDetail({ id }: { id: string }) {
  const detail = useApiQuery(campaignQuery(id), { pollIntervalMs: STATUS_POLL_MS });
  const status = useApiQuery(campaignStatusQuery(id), { pollIntervalMs: STATUS_POLL_MS });
  const agents = useApiQuery(activeAgentsQuery());
  const launch = useApiMutation(launchCampaignMutation);
  const pause = useApiMutation(pauseCampaignMutation);
  const [preview, setPreview] = useState<CampaignLaunchOutcome | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const busy = launch.isPending || pause.isPending;

  const onPreview = useCallback(() => {
    setMessage(null);
    launch.mutate(
      { id, dryRun: true },
      { onSuccess: setPreview, onError: (cause) => setMessage(cause.message) },
    );
  }, [id, launch]);

  const onConfirm = useCallback(() => {
    launch.mutate(
      { id, dryRun: false },
      {
        onSuccess: (outcome) => {
          setPreview(null);
          setMessage(`Campaign ${outcome.status}.`);
        },
        onError: (cause) => setMessage(cause.message),
      },
    );
  }, [id, launch]);

  const onCancel = useCallback(() => setPreview(null), []);

  const onPause = useCallback(() => {
    pause.mutate(
      { id },
      {
        onSuccess: () => setMessage("Campaign paused."),
        onError: (cause) => setMessage(cause.message),
      },
    );
  }, [id, pause]);

  if (detail.error) {
    return <output className={styles.status}>{detail.error.message}</output>;
  }
  if (!detail.data) {
    return <p className={styles.status}>Loading campaign…</p>;
  }

  const { campaign, members, plan_error } = detail.data;
  const lifecycle = status.data?.status ?? campaign.status;
  const pauseReason = status.data?.pause_reason;
  const byId = new Map(members.map((m) => [m.id, m]));
  const agentFor = new Map((agents.data?.agents ?? []).map((a) => [a.ticket_id, a.id]));
  const counters = status.data?.counters;

  return (
    <>
      <h2 className={styles.heading}>
        {campaign.title}
        <span className={styles.state} data-status={lifecycle}>
          {lifecycle}
        </span>
      </h2>
      {pauseReason && <p className={styles.note}>Paused: {pauseReason}</p>}
      {plan_error && <p className={styles.violation}>{plan_error}</p>}
      {message && <output className={styles.status}>{message}</output>}
      <div className={styles.actions}>
        {LAUNCHABLE.has(lifecycle) && (
          <button type="button" disabled={busy} onClick={onPreview}>
            Launch…
          </button>
        )}
        {lifecycle === "running" && (
          <button type="button" disabled={busy} onClick={onPause}>
            Pause
          </button>
        )}
      </div>
      {preview && (
        <LaunchPreview outcome={preview} busy={busy} onConfirm={onConfirm} onCancel={onCancel} />
      )}
      <div className={styles.waves}>
        {detail.data.waves.map((wave, index) => (
          <section key={wave.join()} className={styles.wave} aria-label={`Wave ${index + 1}`}>
            <h3>Wave {index + 1}</h3>
            {wave.map((ticketId) => {
              const member = byId.get(ticketId);
              return member ? (
                <Member
                  key={ticketId}
                  member={member}
                  status={status.data ?? undefined}
                  agentId={agentFor.get(ticketId)}
                />
              ) : null;
            })}
          </section>
        ))}
      </div>
      {counters && status.data && (
        <section aria-label="Governance">
          <h2>Governance</h2>
          <dl className={styles.counters}>
            <dt>Launched</dt>
            <dd>{cap(counters.launched, counters.max_launches)}</dd>
            <dt>Active</dt>
            <dd>{cap(counters.active, counters.max_parallel)}</dd>
            <dt>Retries</dt>
            <dd>{cap(counters.retries, counters.max_retries)}</dd>
            <dt>Policy</dt>
            <dd>{status.data.policy.name}</dd>
          </dl>
        </section>
      )}
    </>
  );
}

export function CampaignDetailPage() {
  const { id = "" } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const license = useApiQuery(licenseQuery());
  const onAddLicense = useCallback(() => {
    void navigate("/settings/license");
  }, [navigate]);

  return (
    <main className={styles.page}>
      <div className={styles.muted}>
        <Link to="/campaigns">Campaigns</Link> / {id}
      </div>
      <h1 className={styles.heading}>
        {id} <span className={styles.badge}>Premium</span>
      </h1>
      {license.error && <output className={styles.status}>{license.error.message}</output>}
      <PremiumGate feature="Campaigns" license={license.data} onAddLicense={onAddLicense}>
        <CampaignDetail id={id} />
      </PremiumGate>
    </main>
  );
}
