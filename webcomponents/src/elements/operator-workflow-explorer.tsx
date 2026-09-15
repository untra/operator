/**
 * `<operator-workflow-explorer>` - the split view on a collection's docs page.
 *
 * Left rail lists the collection's issue types; right pane draws the selected
 * one's Operator workflow. Everything is read from the hosted collection bundle
 * the docs generator publishes, so the docs site needs no API.
 *
 * Usage:
 *   <operator-workflow-explorer base="/collections/ralph_loop/"></operator-workflow-explorer>
 *
 * Optional `selected="PRD"` picks the initial issue type; otherwise the
 * manifest's first entry wins.
 */

import { StrictMode, useCallback, useEffect, useState } from "react";
import { createRoot, type Root } from "react-dom/client";

import { WorkflowExplorerView, type ManifestEntry } from "../views/WorkflowExplorerView";
import type { IssueType } from "../generated/IssueType";

interface Manifest {
  name?: string;
  issue_types?: ManifestEntry[];
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} for ${url}`);
  }
  return (await response.json()) as T;
}

function Explorer({ base, initial }: { base: string; initial?: string | null }) {
  const [entries, setEntries] = useState<ManifestEntry[] | null>(null);
  const [selected, setSelected] = useState<string | null>(initial ?? null);
  const [document_, setDocument] = useState<IssueType | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchJson<Manifest>(`${base}collection.json`)
      .then((manifest) => {
        if (!cancelled) {
          const list = manifest.issue_types ?? [];
          setEntries(list);
          setSelected((current) => current ?? list[0]?.key ?? null);
        }
        return undefined;
      })
      .catch((e: Error) => !cancelled && setError(e.message));
    return () => {
      cancelled = true;
    };
  }, [base]);

  useEffect(() => {
    if (!entries || !selected) {
      return undefined;
    }
    const entry = entries.find((e) => e.key === selected);
    if (!entry) {
      return undefined;
    }
    let cancelled = false;
    fetchJson<IssueType>(`${base}${entry.schema_path}`)
      .then((doc) => {
        if (!cancelled) {
          setDocument(doc);
        }
        return undefined;
      })
      .catch((e: Error) => !cancelled && setError(e.message));
    return () => {
      cancelled = true;
    };
  }, [base, entries, selected]);

  const handleSelect = useCallback((key: string) => {
    // Clear a stale failure so a later successful selection is not masked by it.
    setError(null);
    setSelected(key);
  }, []);

  const selectedDocument = document_?.key === selected ? document_ : null;

  return (
    <WorkflowExplorerView
      entries={entries}
      selected={selected}
      document={selectedDocument}
      error={error}
      onSelect={handleSelect}
    />
  );
}

export class OperatorWorkflowExplorer extends HTMLElement {
  private root?: Root;

  connectedCallback() {
    if (this.root) {
      return;
    }
    const base = this.getAttribute("base");
    if (!base) {
      this.textContent = 'operator-workflow-explorer: missing required "base" attribute.';
      return;
    }
    this.root = createRoot(this);
    this.root.render(
      <StrictMode>
        <Explorer
          base={base.endsWith("/") ? base : `${base}/`}
          initial={this.getAttribute("selected")}
        />
      </StrictMode>,
    );
  }

  disconnectedCallback() {
    // Defer: React throws if a root is unmounted while it is still rendering,
    // which happens when the element is moved rather than removed.
    const root = this.root;
    this.root = undefined;
    queueMicrotask(() => root?.unmount());
  }
}

export const OPERATOR_WORKFLOW_EXPLORER_TAG = "operator-workflow-explorer";
