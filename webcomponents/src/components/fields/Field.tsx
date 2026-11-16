import { useCallback, useId, useState, type ReactNode } from "react";
import { validate, type Validator } from "./validators";
import styles from "./Field.module.css";

export interface FieldBaseProps<T> {
  label: ReactNode;
  rules?: readonly Validator<T>[];
  /** Guidance shown under the control while it has no error. */
  hint?: ReactNode;
  /** An external error, e.g. from the server; shown regardless of touch state. */
  error?: string | null;
  /** Show rule errors before the control is blurred, e.g. after a submit attempt. */
  showErrors?: boolean;
  disabled?: boolean;
  name?: string;
  /** Extra class on the field wrapper, for page-level spacing. */
  className?: string;
}

type FieldRuleProps<T> = Pick<FieldBaseProps<T>, "rules" | "error" | "showErrors">;

/** Rule errors appear after the first blur, then track every change. */
export function useFieldState<T>(value: T, props: FieldRuleProps<T>) {
  const [touched, setTouched] = useState(false);
  const onBlur = useCallback(() => setTouched(true), []);
  const ruleError = validate(value, props.rules ?? []);
  const shown = touched || props.showErrors ? ruleError : null;
  return { onBlur, message: props.error ?? shown };
}

export function controlClass(message: string | null): string {
  return message ? `${styles.control} ${styles.invalid}` : styles.control;
}

interface FieldShellProps {
  label: ReactNode;
  className?: string;
  hint?: ReactNode;
  message: string | null;
  children: (aria: {
    id: string;
    "aria-invalid": boolean;
    "aria-describedby": string;
  }) => ReactNode;
}

export function FieldShell({ label, hint, message, className, children }: FieldShellProps) {
  const id = useId();
  const messageId = `${id}-message`;
  return (
    <div className={className ? `${styles.field} ${className}` : styles.field}>
      <label className={styles.label} htmlFor={id}>
        {label}
      </label>
      {children({ id, "aria-invalid": message !== null, "aria-describedby": messageId })}
      <small id={messageId} className={message ? styles.error : styles.hint} aria-live="polite">
        {message ? <em>{message}</em> : hint}
      </small>
    </div>
  );
}
