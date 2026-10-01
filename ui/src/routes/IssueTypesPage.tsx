import { useCallback, useState } from "react";
import { IssueTypesView } from "@operator/webcomponents";
import { useApiQuery } from "../api";
import { issueTypeDocumentQuery, issueTypeQuery, issueTypesQuery } from "../api/definitions";
import { CONCEPTS } from "../concepts";

const ISSUE_TYPES = CONCEPTS.issuetypes;

export function IssueTypesPage() {
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [view, setView] = useState<"steps" | "graph">("steps");
  const list = useApiQuery(issueTypesQuery());
  const detail = useApiQuery(issueTypeQuery(selectedKey ?? ""), { enabled: Boolean(selectedKey) });
  const document = useApiQuery(issueTypeDocumentQuery(selectedKey ?? ""), {
    enabled: view === "graph" && Boolean(selectedKey),
  });
  const issueTypes = list.data ?? [];
  const selected = detail.data;
  const error = list.error?.message ?? detail.error?.message ?? document.error?.message ?? null;

  const selectIssueType = useCallback((key: string) => {
    setSelectedKey(key);
  }, []);

  return (
    <IssueTypesView
      header={{
        title: ISSUE_TYPES.label,
        summary: ISSUE_TYPES.summary,
        docsUrl: ISSUE_TYPES.docsUrl,
        icon: ISSUE_TYPES.icon,
      }}
      issueTypes={
        list.isLoading
          ? { status: "loading", message: "Loading issue types..." }
          : issueTypes.length > 0
            ? { status: "ready", data: issueTypes }
            : { status: "empty", message: "No issue types available." }
      }
      selected={selected}
      workflow={
        document.data && selected && document.data.key === selected.key
          ? { status: "ready", data: document.data }
          : { status: "loading", message: "Loading workflow graph…" }
      }
      mode={view}
      error={error}
      onSelect={selectIssueType}
      onModeChange={setView}
    />
  );
}
