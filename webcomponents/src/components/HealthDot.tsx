import styles from "./HealthDot.module.css";

export interface HealthDotProps {
  /** green, yellow, red, or gray. */
  health: string;
  className?: string;
}

export function HealthDot({ health, className }: HealthDotProps) {
  return (
    <span
      className={className ? `${styles.dot} ${className}` : styles.dot}
      data-health={health}
      aria-hidden="true"
    />
  );
}
