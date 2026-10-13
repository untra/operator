import styles from "./StepProgress.module.css";

export type StepState = "done" | "current" | "pending";

export type StepLike = { name: string; display_name?: string | null };

export type StepProgressProps = {
  steps: StepLike[];
  current?: string | null;
  completed: string[];
  /** The ticket is completed, so every step is done. */
  finished?: boolean;
};

const GLYPH: Record<StepState, string> = { done: "✓", current: "●", pending: "○" };

/** Each step's state; steps before the current one count as done. */
export function stepStates(
  steps: StepLike[],
  current: string | null | undefined,
  completed: string[],
  finished = false,
): Array<{ name: string; label: string; state: StepState }> {
  const currentIndex = steps.findIndex((s) => s.name === current);
  return steps.map((step, index) => {
    const done =
      finished || completed.includes(step.name) || (currentIndex >= 0 && index < currentIndex);
    const state: StepState = done ? "done" : step.name === current ? "current" : "pending";
    return { name: step.name, label: step.display_name ?? step.name, state };
  });
}

export function StepProgress({ steps, current, completed, finished }: StepProgressProps) {
  return (
    <ol className={styles.steps}>
      {stepStates(steps, current, completed, finished).map((step) => (
        <li
          key={step.name}
          className={styles.step}
          data-state={step.state}
          aria-current={step.state === "current" ? "step" : undefined}
        >
          <span className={styles.glyph} aria-hidden="true">
            {GLYPH[step.state]}
          </span>
          {step.label}
        </li>
      ))}
    </ol>
  );
}
