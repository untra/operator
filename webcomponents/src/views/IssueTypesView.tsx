import { useMemo } from "react";
import type { IssueType } from "../generated/IssueType";
import type { IssueTypeResponse } from "../generated/IssueTypeResponse";
import type { IssueTypeSummary } from "../generated/IssueTypeSummary";
import { AsyncState, type AsyncValue } from "../components/AsyncState";
import { PageHeader, type PageHeaderProps } from "../components/PageHeader";
import { WorkflowGraph } from "../workflow/WorkflowGraph";
import styles from "./IssueTypesView.module.css";

export type IssueTypeViewMode = "steps" | "graph";

export interface IssueTypesViewProps {
  header: PageHeaderProps;
  issueTypes: AsyncValue<IssueTypeSummary[]>;
  selected: IssueTypeResponse | null;
  workflow: AsyncValue<IssueType>;
  mode: IssueTypeViewMode;
  error?: string | null;
  onSelect: (key: string) => void;
  onModeChange: (mode: IssueTypeViewMode) => void;
}

export function IssueTypesView({
  header,
  issueTypes,
  selected,
  workflow,
  mode,
  error,
  onSelect,
  onModeChange,
}: IssueTypesViewProps) {
  const errorValue = useMemo<AsyncValue<never> | null>(
    () => (error ? { status: "error", message: error } : null),
    [error],
  );
  return (
    <div className={styles.page}>
      <PageHeader {...header} />
      {errorValue && <AsyncState value={errorValue}>{() => null}</AsyncState>}
      <div className={styles.split}>
        <AsyncState value={issueTypes}>
          {(items) => (
            <div className={styles.list}>
              {items.map((item) => (
                <button
                  type="button"
                  key={item.key}
                  className={`${styles.item} ${selected?.key === item.key ? styles.selectedItem : ""}`}
                  onClick={() => onSelect(item.key)}
                >
                  <span className={styles.glyph}>{item.glyph}</span>
                  <span>
                    <span className={styles.itemName}>{item.name}</span>
                    <span className={styles.itemMeta}>
                      {item.key} &middot; {item.mode} &middot; {item.stepCount} steps
                    </span>
                  </span>
                </button>
              ))}
            </div>
          )}
        </AsyncState>
        <div className={styles.detail}>
          {selected ? (
            <>
              <h2 className={styles.detailTitle}>
                <span className={styles.detailGlyph}>{selected.glyph}</span>
                {selected.name}
              </h2>
              <p className={styles.detailDesc}>{selected.description}</p>
              <div className={styles.kvGrid}>
                <span className={styles.label}>Key</span>
                <span>{selected.key}</span>
                <span className={styles.label}>Mode</span>
                <span>{selected.mode}</span>
                <span className={styles.label}>Source</span>
                <span>{selected.source}</span>
                <span className={styles.label}>Steps</span>
                <span>{selected.steps.length}</span>
              </div>
              {selected.steps.length > 0 && (
                <div className={styles.steps}>
                  <div className={styles.stepsHead}>
                    <h3 className={styles.stepsTitle}>Workflow</h3>
                    <div className={styles.toggle} role="tablist" aria-label="Workflow view">
                      {(["steps", "graph"] as const).map((nextMode) => (
                        <button
                          type="button"
                          role="tab"
                          aria-selected={mode === nextMode}
                          key={nextMode}
                          className={mode === nextMode ? styles.toggleActive : styles.toggleBtn}
                          onClick={() => onModeChange(nextMode)}
                        >
                          {nextMode === "steps" ? "Steps" : "Graph"}
                        </button>
                      ))}
                    </div>
                  </div>
                  {mode === "steps" ? (
                    <ol className={styles.stepList}>
                      {selected.steps.map((step) => (
                        <li key={step.name} className={styles.step}>
                          <span className={styles.stepName}>{step.display_name ?? step.name}</span>
                          <span className={styles.stepMeta}>{step.review_type} review</span>
                        </li>
                      ))}
                    </ol>
                  ) : (
                    <AsyncState value={workflow}>
                      {(issueType) => <WorkflowGraph issueType={issueType} />}
                    </AsyncState>
                  )}
                </div>
              )}
            </>
          ) : (
            <div className={styles.placeholder}>Select an issue type to view details.</div>
          )}
        </div>
      </div>
    </div>
  );
}
