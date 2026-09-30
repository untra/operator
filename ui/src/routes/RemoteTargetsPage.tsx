import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PremiumPaywall } from "@operator/webcomponents";
import type { TargetDef } from "@operator/bindings/TargetDef";
import type { TargetResponse } from "../api-client";
import { useApiMutation, useApiQuery } from "../api";
import {
  licenseQuery,
  probeTargetMutation,
  removeTargetMutation,
  saveTargetMutation,
  targetsQuery,
} from "../api/definitions";
import form from "./onboarding/OnboardingPage.module.css";
import styles from "./RemoteTargetsPage.module.css";

const DEFAULT_CODER = {
  template: "",
  url_env: "CODER_URL",
  token_env: "CODER_SESSION_TOKEN",
  name_prefix: "op",
  stop_on_complete: true,
  create_timeout_secs: 300n,
};
const emptyTarget = (): TargetDef => ({ name: "", kind: "ssh", ssh_alias: "", workdir: "" });

/// This page manages remote targets only; local and docker are built in.
const remoteOnly = (targets: TargetResponse[]): TargetResponse[] =>
  targets.filter((target) => target.kind === "ssh" || target.kind === "coder");

type TargetRowProps = {
  target: TargetResponse;
  busy: boolean;
  onProbe: (target: TargetResponse) => void;
  onEdit: (target: TargetResponse) => void;
  onRemove: (target: TargetResponse) => void;
};

/// One row, so the per-target handlers are created in this component's scope
/// rather than inside the parent's `map`.
function TargetRow({ target, busy, onProbe, onEdit, onRemove }: TargetRowProps) {
  const probe = useCallback(() => onProbe(target), [onProbe, target]);
  const edit = useCallback(() => onEdit(target), [onEdit, target]);
  const remove = useCallback(() => onRemove(target), [onRemove, target]);
  return (
    <li className={styles.target}>
      <strong>{target.display_name ?? target.name}</strong>
      <span className={styles.meta}>
        {" "}
        · {target.kind} · {target.entitled ? "Available" : "License required"}
      </span>
      <div className={styles.actions}>
        <button type="button" disabled={busy || !target.entitled} onClick={probe}>
          Check connection
        </button>
        {target.user_declared && (
          <>
            <button type="button" disabled={busy || !target.entitled} onClick={edit}>
              Edit
            </button>
            <button type="button" disabled={busy} onClick={remove}>
              Remove
            </button>
          </>
        )}
      </div>
    </li>
  );
}

export function RemoteTargetsPage() {
  const navigate = useNavigate();
  const targetsResult = useApiQuery(targetsQuery());
  const license = useApiQuery(licenseQuery());
  const save = useApiMutation(saveTargetMutation);
  const remove = useApiMutation(removeTargetMutation);
  const probe = useApiMutation(probeTargetMutation);
  const [draft, setDraft] = useState<TargetDef>(emptyTarget);
  const [editing, setEditing] = useState<string>();
  const [message, setMessage] = useState<string | null>(null);
  const targets = remoteOnly(targetsResult.data?.targets ?? []);
  const busy = save.isPending || remove.isPending || probe.isPending;

  const onProbe = useCallback(
    (target: TargetResponse) => {
      setMessage(null);
      probe.mutate(
        { name: target.name },
        {
          onSuccess: (result) => {
            setMessage(
              result.reachable
                ? "Target is reachable."
                : (result.message ?? "Target is unreachable"),
            );
          },
          onError: (cause) => {
            setMessage(cause.message);
          },
        },
      );
    },
    [probe],
  );

  const onEdit = useCallback((target: TargetResponse) => {
    const {
      premium: _premium,
      entitled: _entitled,
      user_declared: _declared,
      ...definition
    } = target;
    setDraft(definition);
    setEditing(target.name);
  }, []);

  const onRemove = useCallback(
    (target: TargetResponse) => {
      setMessage(null);
      remove.mutate(
        { name: target.name },
        {
          onSuccess: () => {
            setMessage("Target removed.");
          },
          onError: (cause) => {
            setMessage(cause.message);
          },
        },
      );
    },
    [remove],
  );

  const onAddLicense = useCallback(() => {
    void navigate("/settings/license");
  }, [navigate]);

  const onSubmit = useCallback(
    (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setMessage(null);
      save.mutate(
        { target: draft, existingName: editing },
        {
          onSuccess: () => {
            setDraft(emptyTarget());
            setEditing(undefined);
            setMessage("Target saved.");
          },
          onError: (cause) => {
            setMessage(cause.message);
          },
        },
      );
    },
    [draft, editing, save],
  );

  const onCancelEdit = useCallback(() => {
    setEditing(undefined);
    setDraft(emptyTarget());
  }, []);

  const statusMessage = message ?? targetsResult.error?.message ?? license.error?.message ?? null;
  const loading = targetsResult.isLoading || license.isLoading;

  return (
    <main className={styles.page}>
      <h1 className={styles.heading}>
        Remote targets <span className={styles.badge}>Premium</span>
      </h1>
      <p>Register SSH hosts and Coder workspaces for delegators to run agents remotely.</p>
      {statusMessage && <output className={styles.status}>{statusMessage}</output>}
      {loading && <p className={styles.status}>Loading remote targets…</p>}
      {license.data && !license.data.premium && (
        <PremiumPaywall
          feature="Remote targets"
          purchaseUrl={license.data.purchase_url}
          onAddLicense={onAddLicense}
        />
      )}
      {!loading && !targetsResult.error && targets.length === 0 && (
        <p className={styles.status}>No remote targets are registered.</p>
      )}
      <ul className={styles.targets}>
        {targets.map((target) => (
          <TargetRow
            key={target.name}
            target={target}
            busy={busy}
            onProbe={onProbe}
            onEdit={onEdit}
            onRemove={onRemove}
          />
        ))}
      </ul>
      {license.data?.premium && (
        <form className={form.form} onSubmit={onSubmit}>
          <h2>{editing ? "Edit target" : "Register target"}</h2>
          <label>
            Name
            <input
              required
              value={draft.name}
              disabled={busy || !!editing}
              onChange={(event) => setDraft({ ...draft, name: event.target.value })}
            />
          </label>
          <label>
            Display name
            <input
              value={draft.display_name ?? ""}
              disabled={busy}
              onChange={(event) => setDraft({ ...draft, display_name: event.target.value })}
            />
          </label>
          <label>
            Type
            <select
              value={draft.kind}
              disabled={busy || !!editing}
              onChange={(event) =>
                setDraft(
                  event.target.value === "ssh"
                    ? {
                        name: draft.name,
                        display_name: draft.display_name,
                        kind: "ssh",
                        ssh_alias: "",
                        workdir: "",
                      }
                    : {
                        name: draft.name,
                        display_name: draft.display_name,
                        kind: "coder",
                        ...DEFAULT_CODER,
                      },
                )
              }
            >
              <option value="ssh">SSH host</option>
              <option value="coder">Coder workspace</option>
            </select>
          </label>
          {draft.kind === "ssh" && (
            <>
              <label>
                SSH alias
                <input
                  required
                  value={draft.ssh_alias}
                  onChange={(event) => setDraft({ ...draft, ssh_alias: event.target.value })}
                />
              </label>
              <label>
                Remote project root
                <input
                  required
                  value={draft.workdir}
                  onChange={(event) => setDraft({ ...draft, workdir: event.target.value })}
                />
              </label>
              <label>
                SSH configuration path (optional)
                <input
                  value={draft.ssh_config_path ?? ""}
                  onChange={(event) =>
                    setDraft({ ...draft, ssh_config_path: event.target.value || null })
                  }
                />
              </label>
            </>
          )}
          {draft.kind === "coder" && (
            <>
              <label>
                Template
                <input
                  required
                  value={draft.template}
                  onChange={(event) => setDraft({ ...draft, template: event.target.value })}
                />
              </label>
              <label>
                Deployment URL environment variable
                <input
                  required
                  value={draft.url_env}
                  onChange={(event) => setDraft({ ...draft, url_env: event.target.value })}
                />
              </label>
              <label>
                Session token environment variable
                <input
                  required
                  value={draft.token_env}
                  onChange={(event) => setDraft({ ...draft, token_env: event.target.value })}
                />
              </label>
              <label>
                Remote project root (optional)
                <input
                  value={draft.workdir ?? ""}
                  onChange={(event) => setDraft({ ...draft, workdir: event.target.value || null })}
                />
              </label>
            </>
          )}
          <button type="submit" disabled={busy}>
            Save target
          </button>
          {editing && (
            <button type="button" disabled={busy} onClick={onCancelEdit}>
              Cancel edit
            </button>
          )}
        </form>
      )}
    </main>
  );
}
