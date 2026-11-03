import styles from "./HealthIndicator.module.css";

const LABELS: Readonly<Record<string, string>> = {
  green: "Healthy",
  yellow: "Needs attention",
  red: "Error",
  gray: "Not applicable",
};

export function HealthIndicator({ health }: { health: string }) {
  return (
    <span className={styles.health}>
      <span className={styles.dot} data-health={health} aria-hidden="true" />
      {LABELS[health] ?? "Unknown"}
    </span>
  );
}
