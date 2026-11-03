import type { ReactNode } from "react";
import styles from "./AsyncState.module.css";

export type AsyncValue<T> =
  | { status: "initial"; message?: string }
  | { status: "loading"; message?: string }
  | { status: "empty"; message?: string }
  | { status: "error"; message: string }
  | { status: "ready"; data: T };

export interface AsyncStateProps<T> {
  value: AsyncValue<T>;
  children: (data: T) => ReactNode;
}

export function AsyncState<T>({ value, children }: AsyncStateProps<T>) {
  if (value.status === "ready") {
    return (
      <>
        <output className={styles.silent} aria-live="polite" aria-atomic="true" />
        <div className={styles.silent} role="alert" aria-atomic="true" />
        {children(value.data)}
      </>
    );
  }
  const message =
    value.message ??
    (value.status === "initial"
      ? "Waiting to load."
      : value.status === "loading"
        ? "Loading…"
        : "No data available.");
  return (
    <>
      <output
        aria-live="polite"
        aria-atomic="true"
        className={value.status !== "error" ? styles.state : styles.silent}
      >
        {value.status !== "error" ? message : null}
      </output>
      <div
        role="alert"
        aria-atomic="true"
        className={value.status === "error" ? `${styles.state} ${styles.error}` : styles.silent}
      >
        {value.status === "error" ? message : null}
      </div>
    </>
  );
}
