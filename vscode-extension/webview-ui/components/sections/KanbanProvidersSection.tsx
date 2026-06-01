import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Link from '@mui/material/Link';
import { SectionHeader } from '../SectionHeader';
import { LinkOutCard } from '../LinkOutCard';
import { ProviderCard } from '../kanban/ProviderCard';
import type {
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from '../../types/messages';
import type { KanbanConfig } from '../../../src/generated/KanbanConfig';
import type { JiraConfig } from '../../../src/generated/JiraConfig';
import type { LinearConfig } from '../../../src/generated/LinearConfig';

interface KanbanProvidersSectionProps {
  kanban: KanbanConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
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
  onOpenOperatorUi: (route: 'issuetypes' | 'projects') => void;
}

const DEFAULT_JIRA: JiraConfig = { enabled: false, api_key_env: 'OPERATOR_JIRA_API_KEY', email: '', projects: {} };
const DEFAULT_LINEAR: LinearConfig = { enabled: false, api_key_env: 'OPERATOR_LINEAR_API_KEY', projects: {} };

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
  onOpenOperatorUi,
}: KanbanProvidersSectionProps) {
  // Iterate all Jira domains
  const jiraEntries = Object.entries(kanban.jira ?? {});
  const hasJira = jiraEntries.length > 0;
  const defaultJiraDomain = 'your-org.atlassian.net';

  // Iterate all Linear workspaces
  const linearEntries = Object.entries(kanban.linear ?? {});
  const hasLinear = linearEntries.length > 0;
  const defaultLinearTeam = 'default-team';

  // Viewing an issue type now links out to the hosted Operator UI.
  const handleViewIssueType = () => {
    onOpenOperatorUi('issuetypes');
  };

  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-kanban" title="Kanban Providers" />
      <Typography color="text.secondary" gutterBottom>
        Configure kanban board integrations for ticket management. For more details see the{' '}
        <Link href="https://operator.untra.io/getting-started/kanban/">kanban documentation</Link>
      </Typography>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
        {/* Jira providers */}
        {hasJira ? (
          jiraEntries.map(([domain, config]) => (
            <ProviderCard
              key={`jira-${domain}`}
              type="jira"
              domain={domain}
              config={config as JiraConfig}
              onUpdate={onUpdate}
              onValidate={onValidateJira}
              validationResult={jiraResult}
              validating={validatingJira}
              collections={collections}
              issueTypes={issueTypes}
              externalIssueTypes={externalIssueTypes}
              onGetExternalIssueTypes={onGetExternalIssueTypes}
              onViewIssueType={handleViewIssueType}
            />
          ))
        ) : (
          <ProviderCard
            type="jira"
            domain={defaultJiraDomain}
            config={DEFAULT_JIRA}
            onUpdate={onUpdate}
            onValidate={onValidateJira}
            validationResult={jiraResult}
            validating={validatingJira}
            collections={collections}
            issueTypes={issueTypes}
            externalIssueTypes={externalIssueTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            onViewIssueType={handleViewIssueType}
          />
        )}

        {/* Linear providers */}
        {hasLinear ? (
          linearEntries.map(([teamId, config]) => (
            <ProviderCard
              key={`linear-${teamId}`}
              type="linear"
              domain={teamId}
              config={config as LinearConfig}
              onUpdate={onUpdate}
              onValidate={onValidateLinear}
              validationResult={linearResult}
              validating={validatingLinear}
              collections={collections}
              issueTypes={issueTypes}
              externalIssueTypes={externalIssueTypes}
              onGetExternalIssueTypes={onGetExternalIssueTypes}
              onViewIssueType={handleViewIssueType}
            />
          ))
        ) : (
          <ProviderCard
            type="linear"
            domain={defaultLinearTeam}
            config={DEFAULT_LINEAR}
            onUpdate={onUpdate}
            onValidate={onValidateLinear}
            validationResult={linearResult}
            validating={validatingLinear}
            collections={collections}
            issueTypes={issueTypes}
            externalIssueTypes={externalIssueTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            onViewIssueType={handleViewIssueType}
          />
        )}
      </Box>

      {/* Issue types & collections now live in the hosted Operator UI */}
      {apiReachable && (
        <Box sx={{ mt: 3 }}>
          <LinkOutCard
            id="section-issuetypes"
            title="Issue Types & Collections"
            description="Create and manage issue types and collections in the Operator UI."
            onOpen={() => onOpenOperatorUi('issuetypes')}
          />
        </Box>
      )}
    </Box>
  );
}
