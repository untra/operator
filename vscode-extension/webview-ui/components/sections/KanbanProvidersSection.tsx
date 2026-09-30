import React, { useCallback } from "react";
import { SectionHeader } from "../SectionHeader";
import { LinkOutCard } from "../LinkOutCard";
import { ProviderCard } from "../kanban/ProviderCard";
import { useStableKeys } from "../../hooks/useStableKeys";
import type {
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from "../../types/messages";
import type { KanbanConfig } from "../../../src/generated/KanbanConfig";
import type { JiraConfig } from "../../../src/generated/JiraConfig";
import type { LinearConfig } from "../../../src/generated/LinearConfig";

interface KanbanProvidersSectionProps {
  kanban: KanbanConfig;
  onUpdate: (section: string, key: string, value: unknown, instanceKey?: string) => void;
  onValidateJira: (domain: string, email: string, apiToken: string) => void;
  onValidateLinear: (apiKey: string) => void;
  jiraResult: JiraValidationInfo | null;
  linearResult: LinearValidationInfo | null;
  validatingJira: boolean;
  validatingLinear: boolean;
  apiReachable: boolean;
  issueTypes: IssueTypeSummary[];
  collections: CollectionResponse[];
  externalIssueTypes: Map<string, ExternalIssueTypeSummary[]>;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  kanbanStatuses: Map<string, string[]>;
  onGetKanbanStatuses: (provider: string, projectKey: string) => void;
  onOpenOperatorUi: (route: "issuetypes" | "projects") => void;
}

const DEFAULT_JIRA: JiraConfig = {
  enabled: false,
  api_key_env: "OPERATOR_JIRA_API_KEY",
  email: "",
  projects: {},
};
const DEFAULT_LINEAR: LinearConfig = {
  enabled: false,
  api_key_env: "OPERATOR_LINEAR_API_KEY",
  projects: {},
};
const DEFAULT_JIRA_DOMAIN = "your-org.atlassian.net";
const DEFAULT_LINEAR_TEAM = "default-team";

/** Render a placeholder card under the default instance key until one is configured. */
function entriesOrDefault<W>(
  map: Record<string, W> | undefined,
  defaultKey: string,
  defaultValue: W,
): Array<[string, W]> {
  const entries = Object.entries(map ?? {});
  return entries.length > 0 ? entries : [[defaultKey, defaultValue]];
}

export function KanbanProvidersSection({
  kanban,
  onUpdate,
  onValidateJira,
  onValidateLinear,
  jiraResult,
  linearResult,
  validatingJira,
  validatingLinear,
  apiReachable,
  issueTypes,
  collections,
  externalIssueTypes,
  onGetExternalIssueTypes,
  kanbanStatuses,
  onGetKanbanStatuses,
  onOpenOperatorUi,
}: KanbanProvidersSectionProps) {
  const jiraEntries = entriesOrDefault(kanban.jira, DEFAULT_JIRA_DOMAIN, DEFAULT_JIRA);
  const linearEntries = entriesOrDefault(kanban.linear, DEFAULT_LINEAR_TEAM, DEFAULT_LINEAR);
  const jiraIds = useStableKeys(jiraEntries.map(([domain]) => domain));
  const linearIds = useStableKeys(linearEntries.map(([teamId]) => teamId));

  // Viewing an issue type now links out to the hosted Operator UI.
  const handleViewIssueType = useCallback(() => {
    onOpenOperatorUi("issuetypes");
  }, [onOpenOperatorUi]);
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
        {jiraEntries.map(([domain, config]) => (
          <ProviderCard
            key={`jira-${jiraIds.get(domain) ?? domain}`}
            type="jira"
            domain={domain}
            config={config}
            onUpdate={onUpdate}
            onValidate={onValidateJira}
            validationResult={jiraResult}
            validating={validatingJira}
            collections={collections}
            issueTypes={issueTypes}
            externalIssueTypes={externalIssueTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            kanbanStatuses={kanbanStatuses}
            onGetKanbanStatuses={onGetKanbanStatuses}
            onViewIssueType={handleViewIssueType}
          />
        ))}

        {linearEntries.map(([teamId, config]) => (
          <ProviderCard
            key={`linear-${linearIds.get(teamId) ?? teamId}`}
            type="linear"
            domain={teamId}
            config={config}
            onUpdate={onUpdate}
            onValidate={onValidateLinear}
            validationResult={linearResult}
            validating={validatingLinear}
            collections={collections}
            issueTypes={issueTypes}
            externalIssueTypes={externalIssueTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            kanbanStatuses={kanbanStatuses}
            onGetKanbanStatuses={onGetKanbanStatuses}
            onViewIssueType={handleViewIssueType}
          />
        ))}
      </div>

      {/* Issue types & collections now live in the hosted Operator UI */}
      {apiReachable && (
        <div style={{ marginTop: 24 }}>
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
