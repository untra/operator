import type { ReactNode } from "react";
import styles from "./RightPanel.module.css";

export interface RightPanelProps {
  title?: string | null;
  children: ReactNode;
  onClose: () => void;
}

export function RightPanel({ title, children, onClose }: RightPanelProps) {
  return (
    <aside className={styles.panel} aria-label={title ?? "Detail panel"}>
      <div className={styles.header}>
        <span className={styles.title}>{title}</span>
        <button
          type="button"
          className={styles.close}
          onClick={onClose}
          aria-label="Close panel"
          title="Close panel"
        >
          ✕
        </button>
      </div>
      <div className={styles.body}>{children}</div>
    </aside>
  );
}
