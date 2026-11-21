import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PremiumPaywall, SelectInput, StringInput } from "@operator/webcomponents";
import type { TargetDef } from "@operator/bindings/TargetDef";
import type { TargetResponse } from "../api-client";
import { useApiMutation, useApiQuery } from "../api";
import { PREMIUM_PAGE_HREF, useLicenseQuery } from "../license-return";
import {
  probeTargetMutation,
  removeTargetMutation,
  saveTargetMutation,
  targetsQuery,
} from "../api/definitions";
import { envVarRules, requiredRules, targetDraftValid } from "../field-rules";
import form from "./onboarding/OnboardingPage.module.css";
import styles from "./RemoteTargetsPage.module.css";

const DEFAULT_CODER = {
  template: "",
  url_env: "CODER_URL",
  token_env: "CODER_SESSION_TOKEN",
  name_prefix: "op",
  stop_on_complete: true,
  create_timeout_secs: 300n,
  delete_on_complete: false,
  delete_on_merge: false,
};
const TARGET_KINDS = [
  { value: "ssh", label: "SSH host" },
  { value: "coder", label: "Coder workspace" },
] as const;
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
  const license = useLicenseQuery();
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

  const setCommonField = useCallback(
    (key: "name" | "display_name") => (value: string) =>
      setDraft((current) => ({ ...current, [key]: value })),
    [],
  );
  const setSshField = useCallback(
    (key: "ssh_alias" | "workdir") => (value: string) =>
      setDraft((current) => (current.kind === "ssh" ? { ...current, [key]: value } : current)),
    [],
  );
  const setSshConfigPath = useCallback(
    (value: string) =>
      setDraft((current) =>
        current.kind === "ssh" ? { ...current, ssh_config_path: value || null } : current,
      ),
    [],
  );
  const setCoderField = useCallback(
    (key: "template" | "url_env" | "token_env") => (value: string) =>
      setDraft((current) => (current.kind === "coder" ? { ...current, [key]: value } : current)),
    [],
  );
  const setCoderWorkdir = useCallback(
    (value: string) =>
      setDraft((current) =>
        current.kind === "coder" ? { ...current, workdir: value || null } : current,
      ),
    [],
  );
  const selectKind = useCallback(
    (kind: "ssh" | "coder") =>
      setDraft((current) =>
        kind === "ssh"
          ? {
              name: current.name,
              display_name: current.display_name,
              kind: "ssh",
              ssh_alias: "",
              workdir: "",
            }
          : {
              name: current.name,
              display_name: current.display_name,
              kind: "coder",
              ...DEFAULT_CODER,
            },
      ),
    [],
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
      {loading && <p className={styles.status}>Loading remote targets...</p>}
      {license.data && !license.data.premium && (
        <PremiumPaywall
          feature="Remote targets"
          premiumHref={PREMIUM_PAGE_HREF}
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
          <StringInput
            label="Name"
            rules={requiredRules}
            value={draft.name}
            disabled={busy || !!editing}
            onChange={setCommonField("name")}
          />
          <StringInput
            label="Display name"
            value={draft.display_name ?? ""}
            disabled={busy}
            onChange={setCommonField("display_name")}
          />
          <SelectInput
            label="Type"
            options={TARGET_KINDS}
            value={draft.kind === "ssh" || draft.kind === "coder" ? draft.kind : ""}
            disabled={busy || !!editing}
            onChange={selectKind}
          />
          {draft.kind === "ssh" && (
            <>
              <StringInput
                label="SSH alias"
                rules={requiredRules}
                value={draft.ssh_alias}
                onChange={setSshField("ssh_alias")}
              />
              <StringInput
                label="Remote project root"
                rules={requiredRules}
                value={draft.workdir}
                onChange={setSshField("workdir")}
              />
              <StringInput
                label="SSH configuration path (optional)"
                value={draft.ssh_config_path ?? ""}
                onChange={setSshConfigPath}
              />
            </>
          )}
          {draft.kind === "coder" && (
            <>
              <StringInput
                label="Template"
                rules={requiredRules}
                value={draft.template}
                onChange={setCoderField("template")}
              />
              <StringInput
                label="Deployment URL environment variable"
                rules={envVarRules}
                value={draft.url_env}
                onChange={setCoderField("url_env")}
              />
              <StringInput
                label="Session token environment variable"
                rules={envVarRules}
                value={draft.token_env}
                onChange={setCoderField("token_env")}
              />
              <StringInput
                label="Remote project root (optional)"
                value={draft.workdir ?? ""}
                onChange={setCoderWorkdir}
              />
            </>
          )}
          <button type="submit" disabled={busy || !targetDraftValid(draft)}>
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
