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
  /** Span the whole grid row, for an option that leads the set. */
  wide?: boolean;
  /** Already in effect and not something to pick; renders without a click. */
  locked?: boolean;
}

export function Choice<Value extends string>({
  value,
  selected,
  onSelect,
  children,
  wide = false,
  locked = false,
}: ChoiceProps<Value>) {
  const className = [
    styles.choice,
    selected ? styles.selected : "",
    wide ? styles.wide : "",
    locked ? styles.locked : "",
  ]
    .filter(Boolean)
    .join(" ");
  if (locked) {
    return (
      <div className={className} aria-current="true">
        {children}
      </div>
    );
  }
  return (
    <button
      type="button"
      aria-pressed={selected}
      className={className}
      onClick={() => onSelect(value)}
    >
      {children}
    </button>
  );
}
