import React, { useCallback, useState } from "react";
import { SectionHeader } from "../SectionHeader";
import { LinkOutCard } from "../LinkOutCard";
import { ProviderCard } from "../kanban/ProviderCard";
import { useStableKeys } from "../../hooks/useStableKeys";
import type {
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from "../../types/messages";
import type { KanbanConfig } from "../../../src/generated/KanbanConfig";
import { Button, TextInput, SelectInput } from "../primitives";

interface KanbanProvidersSectionProps {
  kanban: KanbanConfig;
  onUpdate: (section: string, key: string, value: unknown, instanceKey?: string) => void;
  apiReachable: boolean;
  issueTypes: IssueTypeSummary[];
  collections: CollectionResponse[];
  externalIssueTypes: Map<string, ExternalIssueTypeSummary[]>;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  kanbanStatuses: Map<string, string[]>;
  onGetKanbanStatuses: (provider: string, domain: string, projectKey: string) => void;
  onOpenOperatorUi: (route: "issuetypes" | "projects") => void;
}

export function KanbanProvidersSection({
  kanban,
  onUpdate,
  apiReachable,
  issueTypes,
  collections,
  externalIssueTypes,
  onGetExternalIssueTypes,
  kanbanStatuses,
  onGetKanbanStatuses,
  onOpenOperatorUi,
}: KanbanProvidersSectionProps) {
  const jiraEntries = Object.entries(kanban.jira);
  const linearEntries = Object.entries(kanban.linear);
  const [provider, setProvider] = useState("jira");
  const [instance, setInstance] = useState("");
  const inline = provider === "jira" || provider === "linear";
  const selectProvider = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => setProvider(event.target.value),
    [],
  );
  const changeInstance = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setInstance(event.target.value),
    [],
  );
  const addInstance = useCallback(() => {
    onUpdate(`kanban.${provider}`, "addInstance", true, instance.trim());
    setInstance("");
  }, [onUpdate, provider, instance]);
  const jiraIds = useStableKeys(jiraEntries.map(([domain]) => domain));
  const linearIds = useStableKeys(linearEntries.map(([teamId]) => teamId));

  const handleOpenIssueTypes = useCallback(
    () => onOpenOperatorUi("issuetypes"),
    [onOpenOperatorUi],
  );

  return (
    <div className="op-mb-4">
      <SectionHeader id="section-kanban" title="Kanban Providers" />
      <p className="op-body1 op-text-secondary op-mb-1">
        Configure kanban board integrations for ticket management. For more details see the{" "}
        <a href="https://operator.untra.io/getting-started/kanban/">kanban documentation</a>
      </p>

      <div className="op-col op-gap-3">
        <SelectInput label="Add provider" value={provider} onChange={selectProvider}>
          {["jira", "linear", "github", "gitea", "openspec"].map((slug) => (
            <option key={slug} value={slug}>
              {slug}
            </option>
          ))}
        </SelectInput>
        {inline ? (
          <>
            <TextInput
              label={provider === "jira" ? "Jira domain" : "Linear workspace slug"}
              value={instance}
              onChange={changeInstance}
            />
            <Button
              disabled={!instance.trim() || Object.hasOwn(kanban[provider], instance.trim())}
              onClick={addInstance}
            >
              Add instance
            </Button>
          </>
        ) : (
          <a href={`https://operator.untra.io/getting-started/kanban/${provider}/`}>
            Configure {provider}{" "}
            {provider === "gitea"
              ? "(import only)"
              : provider === "openspec"
                ? "(experimental, pull only)"
                : "Projects"}
          </a>
        )}
        {(["github", "gitea", "openspec", "forgejo"] as const).flatMap((slug) =>
          Object.keys(kanban[slug]).map((name) => (
            <p key={slug + name}>
              {slug} · {name}
              {slug === "forgejo" && " (reserved, read-only)"}
            </p>
          )),
        )}
        {jiraEntries.map(([domain, config]) => (
          <ProviderCard
            key={`jira-${jiraIds.get(domain) ?? domain}`}
            instances={kanban.jira}
            type="jira"
            domain={domain}
            apiReachable={apiReachable}
            config={config}
            onUpdate={onUpdate}
            collections={collections}
            issueTypes={issueTypes}
            externalIssueTypes={externalIssueTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            kanbanStatuses={kanbanStatuses}
            onGetKanbanStatuses={onGetKanbanStatuses}
            onViewIssueType={handleOpenIssueTypes}
          />
        ))}

        {linearEntries.map(([teamId, config]) => (
          <ProviderCard
            key={`linear-${linearIds.get(teamId) ?? teamId}`}
            instances={kanban.linear}
            type="linear"
            domain={teamId}
            apiReachable={apiReachable}
            config={config}
            onUpdate={onUpdate}
            collections={collections}
            issueTypes={issueTypes}
            externalIssueTypes={externalIssueTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            kanbanStatuses={kanbanStatuses}
            onGetKanbanStatuses={onGetKanbanStatuses}
            onViewIssueType={handleOpenIssueTypes}
          />
        ))}
      </div>

      {/* Issue types & collections now live in the hosted Operator UI */}
      {apiReachable && (
        <div className="op-mt-3">
          <LinkOutCard
            id="section-issuetypes"
            title="Issue Types & Collections"
            description="Create and manage issue types and collections in the Operator UI."
            onOpen={handleOpenIssueTypes}
          />
        </div>
      )}
    </div>
  );
}
