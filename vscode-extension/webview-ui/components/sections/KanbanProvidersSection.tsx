import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Link from '@mui/material/Link';
import { SectionHeader } from '../SectionHeader';
import { ProviderCard } from '../kanban/ProviderCard';
import { CollectionsSubSection } from '../kanban/CollectionsSubSection';
import { IssueTypeDrawer } from '../kanban/IssueTypeDrawer';
import type {
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from '../../types/messages';
import type { KanbanConfig } from '../../../src/generated/KanbanConfig';
import type { JiraConfig } from '../../../src/generated/JiraConfig';
import type { LinearConfig } from '../../../src/generated/LinearConfig';
import type { CreateIssueTypeRequest } from '../../../src/generated/CreateIssueTypeRequest';
import type { UpdateIssueTypeRequest } from '../../../src/generated/UpdateIssueTypeRequest';

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
  issueTypesLoading: boolean;
  issueTypeError: string | null;
  collections: CollectionResponse[];
  collectionsLoading: boolean;
  collectionsError: string | null;
  externalIssueTypes: Map<string, ExternalIssueTypeSummary[]>;
  selectedIssueType: IssueTypeResponse | null;
  drawerOpen: boolean;
  drawerMode: 'view' | 'edit' | 'create';
  onGetIssueTypes: () => void;
  onGetIssueType: (key: string) => void;
  onGetCollections: () => void;
  onActivateCollection: (name: string) => void;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  onCreateIssueType: (request: CreateIssueTypeRequest) => void;
  onUpdateIssueType: (key: string, request: UpdateIssueTypeRequest) => void;
  onDeleteIssueType: (key: string) => void;
  onOpenDrawer: (mode: 'view' | 'edit' | 'create', issueType?: IssueTypeResponse) => void;
  onCloseDrawer: () => void;
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
  issueTypesLoading: _issueTypesLoading,
  issueTypeError: _issueTypeError,
  collections,
  collectionsLoading,
  collectionsError,
  externalIssueTypes,
  selectedIssueType,
  drawerOpen,
  drawerMode,
  onGetIssueTypes: _onGetIssueTypes,
  onGetIssueType,
  onGetCollections,
  onActivateCollection,
  onGetExternalIssueTypes,
  onCreateIssueType,
  onUpdateIssueType,
  onDeleteIssueType,
  onOpenDrawer,
  onCloseDrawer,
}: KanbanProvidersSectionProps) {
  // Iterate all Jira domains
  const jiraEntries = Object.entries(kanban.jira ?? {});
  const hasJira = jiraEntries.length > 0;
  const defaultJiraDomain = 'your-org.atlassian.net';

  // Iterate all Linear workspaces
  const linearEntries = Object.entries(kanban.linear ?? {});
  const hasLinear = linearEntries.length > 0;
  const defaultLinearTeam = 'default-team';

  const handleViewIssueType = (key: string) => {
    onGetIssueType(key);
    // The selectedIssueType will be set via message handler
    // We need to find it in the current list for immediate open
    onOpenDrawer('view');
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
              selectedIssueType={selectedIssueType}
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
            selectedIssueType={selectedIssueType}
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
              selectedIssueType={selectedIssueType}
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
            selectedIssueType={selectedIssueType}
            onGetExternalIssueTypes={onGetExternalIssueTypes}

            onViewIssueType={handleViewIssueType}
          />
        )}
      </Box>

      {/* Collections & Issue Types (shown when API is reachable) */}
      {apiReachable && (
        <CollectionsSubSection
          collections={collections}
          collectionsLoading={collectionsLoading}
          collectionsError={collectionsError}
          issueTypes={issueTypes}
          onActivateCollection={onActivateCollection}
          onGetCollections={onGetCollections}
          onViewIssueType={handleViewIssueType}
          onCreateIssueType={() => onOpenDrawer('create')}
        />
      )}

      {/* Issue Type Drawer */}
      <IssueTypeDrawer
        open={drawerOpen}
        mode={drawerMode}
        issueType={selectedIssueType}
        onClose={onCloseDrawer}
        onCreate={onCreateIssueType}
        onUpdate={onUpdateIssueType}
        onDelete={onDeleteIssueType}
      />
    </Box>
  );
}
