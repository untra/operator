import { useCallback, useEffect, useRef, useState } from "react";
import { IssueTypesView } from "@operator/webcomponents";
import { OperatorApi } from "../api-client";
import type { IssueTypeSummary, IssueTypeResponse } from "../api-client";
import { useHost } from "../host";
import { CONCEPTS } from "../concepts";
import type { IssueType } from "@operator/bindings/IssueType";

const ISSUE_TYPES = CONCEPTS.issuetypes;

export function IssueTypesPage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [issueTypes, setIssueTypes] = useState<IssueTypeSummary[]>([]);
  const [selected, setSelected] = useState<IssueTypeResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"steps" | "graph">("steps");
  const [document_, setDocument] = useState<IssueType | null>(null);
  const selectionRequest = useRef(0);

  useEffect(() => {
    api
      .listIssueTypes()
      .then(setIssueTypes)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [api]);

  // Lazily fetch the native Operator workflow document when the graph opens.
  // Same bytes the docs site renders, so the two graphs cannot disagree.
  useEffect(() => {
    if (view !== "graph" || !selected) {
      return undefined;
    }
    if (document_?.key === selected.key) {
      return undefined;
    }
    let cancelled = false;
    api
      .getIssueTypeDocument(selected.key)
      .then((doc) => {
        if (!cancelled) {
          setDocument(doc);
        }
        return undefined;
      })
      .catch((e) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : "Failed to load workflow");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api, view, selected, document_]);

  const handleSelect = useCallback(
    async (key: string) => {
      const request = ++selectionRequest.current;
      try {
        const detail = await api.getIssueType(key);
        if (request === selectionRequest.current) {
          setSelected(detail);
          setError(null);
        }
      } catch (e) {
        if (request === selectionRequest.current) {
          setError(e instanceof Error ? e.message : "Failed to load issue type");
        }
      }
    },
    [api],
  );

  const selectIssueType = useCallback(
    (key: string) => {
      void handleSelect(key);
    },
    [handleSelect],
  );

  return (
    <IssueTypesView
      header={{
        title: ISSUE_TYPES.label,
        summary: ISSUE_TYPES.summary,
        docsUrl: ISSUE_TYPES.docsUrl,
        icon: ISSUE_TYPES.icon,
      }}
      issueTypes={
        loading
          ? { status: "loading", message: "Loading issue types..." }
          : issueTypes.length > 0
            ? { status: "ready", data: issueTypes }
            : { status: "empty", message: "No issue types available." }
      }
      selected={selected}
      workflow={
        document_ && selected && document_.key === selected.key
          ? { status: "ready", data: document_ }
          : { status: "loading", message: "Loading workflow graph…" }
      }
      mode={view}
      error={error}
      onSelect={selectIssueType}
      onModeChange={setView}
    />
  );
}
