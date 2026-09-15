/**
 * Presentation for `<operator-workflow-explorer>`: the split view on a collection's
 * docs page. Left rail lists the collection's issue types, right pane draws the
 * selected one's Operator workflow.
 *
 * Controlled - the custom element owns fetching, selection and error state. Styles
 * come from `elements.css`, which the docs bundle already loads globally.
 */

import { WorkflowGraph } from "../workflow/WorkflowGraph";
import type { IssueType } from "../generated/IssueType";

export interface ManifestEntry {
  key: string;
  schema_path: string;
}

export interface WorkflowExplorerViewProps {
  /** `null` while the collection manifest is still loading. */
  entries: ManifestEntry[] | null;
  selected: string | null;
  /** The selected issue type's document, or `null` while it loads. */
  document: IssueType | null;
  error?: string | null;
  onSelect: (key: string) => void;
}

export function WorkflowExplorerView({
  entries,
  selected,
  document,
  error,
  onSelect,
}: WorkflowExplorerViewProps) {
  if (error) {
    return <div className="workflow-explorer-error">Could not load this collection: {error}</div>;
  }
  if (!entries) {
    return <div className="workflow-explorer-loading">Loading collection…</div>;
  }
  if (entries.length === 0) {
    return <div className="workflow-explorer-loading">This collection defines no issue types.</div>;
  }

  return (
    <div className="workflow-explorer-body">
      <nav className="workflow-explorer-rail" aria-label="Issue types">
        <ul>
          {entries.map((entry) => (
            <li key={entry.key}>
              <RailButton entry={entry} selected={entry.key === selected} onSelect={onSelect} />
            </li>
          ))}
        </ul>
      </nav>
      <div className="workflow-explorer-canvas">
        {document ? (
          <>
            <h3 className="workflow-explorer-title">
              {document.name} <code>{document.key}</code>
            </h3>
            {document.description && (
              <p className="workflow-explorer-description">{document.description}</p>
            )}
            {/* The docs prose column is narrow; vertical keeps labels legible. */}
            <WorkflowGraph issueType={document} vertical />
          </>
        ) : (
          <div className="workflow-explorer-loading">Loading workflow…</div>
        )}
      </div>
    </div>
  );
}

function RailButton({
  entry,
  selected,
  onSelect,
}: {
  entry: ManifestEntry;
  selected: boolean;
  onSelect: (key: string) => void;
}) {
  return (
    <button
      type="button"
      aria-current={selected}
      className={selected ? "is-selected" : undefined}
      onClick={() => onSelect(entry.key)}
    >
      {entry.key}
    </button>
  );
}
