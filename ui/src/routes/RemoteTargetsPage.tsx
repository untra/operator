import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PremiumPaywall } from "@operator/webcomponents";
import type { TargetDef } from "@operator/bindings/TargetDef";
import { OperatorApi, type LicenseResponse, type TargetResponse } from "../api-client";
import { useHost } from "../host";
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
  const host = useHost();
  const api = useMemo(() => new OperatorApi(host), [host]);
  const navigate = useNavigate();
  const [targets, setTargets] = useState<TargetResponse[]>([]);
  const [license, setLicense] = useState<LicenseResponse | null>(null);
  const [draft, setDraft] = useState<TargetDef>(emptyTarget);
  const [editing, setEditing] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    const [targetResult, licenseResult] = await Promise.all([api.targets(), api.license()]);
    setTargets(remoteOnly(targetResult.targets));
    setLicense(licenseResult);
  }, [api]);

  useEffect(() => {
    let cancelled = false;
    Promise.all([api.targets(), api.license()])
      .then(([targetResult, licenseResult]) => {
        if (!cancelled) {
          setTargets(remoteOnly(targetResult.targets));
          setLicense(licenseResult);
        }
        return undefined;
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          setMessage(cause instanceof Error ? cause.message : "Could not load targets");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api]);

  const act = useCallback(
    async (operation: () => Promise<unknown>, success: string) => {
      setBusy(true);
      setMessage(null);
      try {
        await api.refreshCsrf();
        await operation();
        await refresh();
        setMessage(success);
      } catch (cause) {
        setMessage(cause instanceof Error ? cause.message : "Target operation failed");
      } finally {
        setBusy(false);
      }
    },
    [api, refresh],
  );

  const onProbe = useCallback(
    (target: TargetResponse) => {
      void act(async () => {
        const result = await api.probeTarget(target.name);
        if (!result.reachable) {
          throw new Error(result.message ?? "Target is unreachable");
        }
      }, "Target is reachable.");
    },
    [act, api],
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
      void act(() => api.removeTarget(target.name), "Target removed.");
    },
    [act, api],
  );

  const onAddLicense = useCallback(() => {
    void navigate("/settings/license");
  }, [navigate]);

  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();
      void act(async () => {
        await api.saveTarget(draft, editing);
        setDraft(emptyTarget());
        setEditing(undefined);
      }, "Target saved.");
    },
    [act, api, draft, editing],
  );

  const onCancelEdit = useCallback(() => {
    setEditing(undefined);
    setDraft(emptyTarget());
  }, []);

  return (
    <main className={styles.page}>
      <h1 className={styles.heading}>
        Remote targets <span className={styles.badge}>Premium</span>
      </h1>
      <p>Register SSH hosts and Coder workspaces for delegators to run agents remotely.</p>
      {message && <output className={styles.status}>{message}</output>}
      {license && !license.premium && (
        <PremiumPaywall
          feature="Remote targets"
          purchaseUrl={license.purchase_url}
          onAddLicense={onAddLicense}
        />
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
      {license?.premium && (
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
