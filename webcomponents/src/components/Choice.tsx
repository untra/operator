/**
 * A selectable card, used throughout the onboarding wizard wherever the operator
 * picks one option from a small set.
 */

import type { ReactNode } from "react";
import styles from "./Choice.module.css";

export interface ChoiceGroupProps {
  children: ReactNode;
}

/** Responsive grid the cards sit in. */
export function ChoiceGroup({ children }: ChoiceGroupProps) {
  return <div className={styles.choices}>{children}</div>;
}

export interface ChoiceProps<Value extends string> {
  value: Value;
  selected: boolean;
  onSelect: (value: Value) => void;
  children: ReactNode;
}

export function Choice<Value extends string>({
  value,
  selected,
  onSelect,
  children,
}: ChoiceProps<Value>) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      className={selected ? `${styles.choice} ${styles.selected}` : styles.choice}
      onClick={() => onSelect(value)}
    >
      {children}
    </button>
  );
}
