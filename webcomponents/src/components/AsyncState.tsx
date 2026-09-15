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
    return children(value.data);
  }
  const message =
    value.message ??
    (value.status === "initial"
      ? "Waiting to load."
      : value.status === "loading"
        ? "Loading…"
        : "No data available.");
  return (
    <div className={`${styles.state} ${value.status === "error" ? styles.error : ""}`}>
      {message}
    </div>
  );
}
