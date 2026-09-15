/**
 * The sign-in / setup / reset card shared by every unauthenticated route.
 *
 * Presentation only: each route keeps its own API calls, validation and navigation
 * and passes the fields, footer links and submit handler in.
 */

import type { ReactNode, SubmitEvent } from "react";
import styles from "./AuthCard.module.css";

export interface AuthCardProps {
  title: string;
  subtitle?: string;
  /** A failed attempt. */
  error?: string | null;
  /** A non-failure status message, e.g. "Check your email". */
  notice?: string | null;
  children: ReactNode;
  /** Submit control plus any other actions. Omitted on cards that only report an outcome. */
  actions?: ReactNode;
  /** Secondary navigation below the actions. */
  links?: ReactNode;
  onSubmit?: (event: SubmitEvent<HTMLFormElement>) => void;
}

export function AuthCard({
  title,
  subtitle,
  error,
  notice,
  children,
  actions,
  links,
  onSubmit,
}: AuthCardProps) {
  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={onSubmit}>
        <h1 className={styles.title}>{title}</h1>
        {subtitle && <p className={styles.subtitle}>{subtitle}</p>}
        {error && <p className={styles.error}>{error}</p>}
        {notice && <p className={styles.notice}>{notice}</p>}
        {children}
        {actions}
        {links && <div className={styles.links}>{links}</div>}
      </form>
    </div>
  );
}

export interface AuthFieldProps {
  label: string;
  /** Guidance shown under the input, e.g. a password policy. */
  hint?: ReactNode;
  type?: string;
  value: string;
  autoComplete?: string;
  maxLength?: number;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  onChange: (value: string) => void;
}

export function AuthField({ label, hint, value, onChange, ...input }: AuthFieldProps) {
  return (
    <label className={styles.field}>
      <span className={styles.label}>{label}</span>
      <input
        {...input}
        className={styles.input}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
      {hint && <span className={styles.hint}>{hint}</span>}
    </label>
  );
}

export interface AuthSubmitProps {
  busy?: boolean;
  busyLabel?: string;
  disabled?: boolean;
  children: ReactNode;
}

export function AuthSubmit({ busy, busyLabel, disabled, children }: AuthSubmitProps) {
  return (
    <button className={styles.button} type="submit" disabled={busy === true || disabled === true}>
      {busy ? (busyLabel ?? "Working…") : children}
    </button>
  );
}
