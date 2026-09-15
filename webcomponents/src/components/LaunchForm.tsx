import type { DelegatorResponse } from "../generated/DelegatorResponse";
import styles from "./LaunchForm.module.css";

export interface LaunchFormValue {
  delegator: string;
  wrapper: string;
  target: string;
  yolo: boolean;
}

export interface LaunchFormProps {
  value: LaunchFormValue;
  delegators: DelegatorResponse[];
  targets: string[];
  defaultWrapperLabel: string;
  busy?: boolean;
  disabled?: boolean;
  error?: string | null;
  onChange: (value: LaunchFormValue) => void;
  onSubmit: () => void;
}

const WRAPPERS = ["tmux", "vscode", "cmux", "zellij"] as const;

export function LaunchForm({
  value,
  delegators,
  targets,
  defaultWrapperLabel,
  busy = false,
  disabled = false,
  error,
  onChange,
  onSubmit,
}: LaunchFormProps) {
  const update = <K extends keyof LaunchFormValue>(key: K, next: LaunchFormValue[K]) =>
    onChange({ ...value, [key]: next });
  return (
    <form
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <label className={styles.field}>
        <span className={styles.fieldLabel}>Delegator</span>
        <select
          className={styles.select}
          value={value.delegator}
          onChange={(event) => update("delegator", event.target.value)}
          disabled={busy || disabled}
        >
          <option value="">Default (auto)</option>
          {delegators.map((delegator) => (
            <option key={delegator.name} value={delegator.name}>
              {delegator.display_name ?? delegator.name}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.field}>
        <span className={styles.fieldLabel}>Wrapper</span>
        <select
          className={styles.select}
          value={value.wrapper}
          onChange={(event) => update("wrapper", event.target.value)}
          disabled={busy || disabled}
        >
          <option value="">Default ({defaultWrapperLabel})</option>
          {WRAPPERS.map((wrapper) => (
            <option key={wrapper} value={wrapper}>
              {wrapper}
            </option>
          ))}
        </select>
      </label>
      {targets.length > 0 && (
        <label className={styles.field}>
          <span className={styles.fieldLabel}>Target</span>
          <select
            className={styles.select}
            value={value.target}
            onChange={(event) => update("target", event.target.value)}
            disabled={busy || disabled}
          >
            <option value="">Delegator&apos;s target (auto)</option>
            {targets.map((target) => (
              <option key={target} value={target}>
                {target}
              </option>
            ))}
          </select>
        </label>
      )}
      <label className={styles.checkboxField}>
        <input
          type="checkbox"
          checked={value.yolo}
          onChange={(event) => update("yolo", event.target.checked)}
          disabled={busy || disabled}
        />
        <span>YOLO mode (auto-accept prompts)</span>
      </label>
      {error && <div className={styles.error}>{error}</div>}
      <button type="submit" className={styles.launchBtn} disabled={busy || disabled}>
        {busy ? "Launching…" : "Launch ▸"}
      </button>
    </form>
  );
}
