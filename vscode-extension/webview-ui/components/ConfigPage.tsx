import React, { useMemo, useRef } from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import { SidebarNav, type NavItem } from './SidebarNav';
import { OperatorBrand } from './OperatorBrand';
import { PrimaryConfigSection } from './sections/PrimaryConfigSection';
import { CodingAgentsSection } from './sections/CodingAgentsSection';
import { KanbanProvidersSection } from './sections/KanbanProvidersSection';
import { GitRepositoriesSection } from './sections/GitRepositoriesSection';
import { ProjectsSection } from './sections/ProjectsSection';
import type {
  WebviewConfig,
  JiraValidationInfo,
  LinearValidationInfo,
  ProjectSummary,
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from '../types/messages';
import type { CreateIssueTypeRequest } from '../../src/generated/CreateIssueTypeRequest';
import type { UpdateIssueTypeRequest } from '../../src/generated/UpdateIssueTypeRequest';

interface ConfigPageProps {
  config: WebviewConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onBrowseFolder: (field: string) => void;
  onOpenFile: (filePath: string) => void;
  onValidateJira: (domain: string, email: string, apiToken: string) => void;
  onValidateLinear: (apiKey: string) => void;
  onDetectTools: () => void;
  jiraResult: JiraValidationInfo | null;
  linearResult: LinearValidationInfo | null;
  validatingJira: boolean;
  validatingLinear: boolean;
  apiReachable: boolean;
  projects: ProjectSummary[];
  projectsLoading: boolean;
  projectsError: string | null;
  onAssessProject: (name: string) => void;
  onRefreshProjects: () => void;
  onOpenProject: (path: string) => void;
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

export function ConfigPage({
  config,
  onUpdate,
  onBrowseFolder,
  onOpenFile,
  onValidateJira,
  onValidateLinear,
  onDetectTools,
  jiraResult,
  linearResult,
  validatingJira,
  validatingLinear,
  apiReachable,
  projects,
  projectsLoading,
  projectsError,
  onAssessProject,
  onRefreshProjects,
  onOpenProject,
  issueTypes,
  issueTypesLoading,
  issueTypeError,
  collections,
  collectionsLoading,
  collectionsError,
  externalIssueTypes,
  selectedIssueType,
  drawerOpen,
  drawerMode,
  onGetIssueTypes,
  onGetIssueType,
  onGetCollections,
  onActivateCollection,
  onGetExternalIssueTypes,
  onCreateIssueType,
  onUpdateIssueType,
  onDeleteIssueType,
  onOpenDrawer,
  onCloseDrawer,
}: ConfigPageProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const hasWorkDir = Boolean(config.working_directory);

  const navItems: NavItem[] = useMemo(() => [
    { id: 'section-primary', label: 'Workspace Configuration' },
    { id: 'section-kanban', label: 'Kanban Providers', disabled: !hasWorkDir },
    { id: 'section-agents', label: 'Coding Agents', disabled: !hasWorkDir },
    { id: 'section-git', label: 'Git Version Control', disabled: !hasWorkDir },
    { id: 'section-projects', label: 'Operator Managed Projects', disabled: !apiReachable },
  ], [hasWorkDir, apiReachable]);

  return (
    <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <SidebarNav items={navItems} scrollContainerRef={scrollRef} />

      <Box
        ref={scrollRef}
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          px: 3,
          py: 2,
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
          <Typography variant="h6">
            <OperatorBrand /> Settings
          </Typography>
          <Button
            variant="outlined"
            onClick={() => onOpenFile(config.config_path)}
            disabled={!config.working_directory}
            sx={{
              borderColor: '#66AA99',
              color: '#66AA99',
              '&:hover': {
                borderColor: '#66AA99',
                backgroundColor: 'rgba(102, 170, 153, 0.08)',
              },
            }}
          >
            edit config.toml
          </Button>
        </Box>
        <PrimaryConfigSection
          working_directory={config.working_directory}
          sessions_wrapper={config.config.sessions.wrapper ?? 'vscode'}
          onUpdate={onUpdate}
          onBrowseFolder={onBrowseFolder}
        />
        <KanbanProvidersSection
          kanban={config.config.kanban}
          onUpdate={onUpdate}
          onValidateJira={onValidateJira}
          onValidateLinear={onValidateLinear}
          jiraResult={jiraResult}
          linearResult={linearResult}
          validatingJira={validatingJira}
          validatingLinear={validatingLinear}
          apiReachable={apiReachable}
          issueTypes={issueTypes}
          issueTypesLoading={issueTypesLoading}
          issueTypeError={issueTypeError}
          collections={collections}
          collectionsLoading={collectionsLoading}
          collectionsError={collectionsError}
          externalIssueTypes={externalIssueTypes}
          selectedIssueType={selectedIssueType}
          drawerOpen={drawerOpen}
          drawerMode={drawerMode}
          onGetIssueTypes={onGetIssueTypes}
          onGetIssueType={onGetIssueType}
          onGetCollections={onGetCollections}
          onActivateCollection={onActivateCollection}
          onGetExternalIssueTypes={onGetExternalIssueTypes}
          onCreateIssueType={onCreateIssueType}
          onUpdateIssueType={onUpdateIssueType}
          onDeleteIssueType={onDeleteIssueType}
          onOpenDrawer={onOpenDrawer}
          onCloseDrawer={onCloseDrawer}
        />
        <CodingAgentsSection
          agents={config.config.agents}
          llm_tools={config.config.llm_tools}
          onUpdate={onUpdate}
          onDetectTools={onDetectTools}
        />
        <GitRepositoriesSection
          git={config.config.git}
          onUpdate={onUpdate}
        />
        <ProjectsSection
          projects={projects}
          loading={projectsLoading}
          error={projectsError}
          onAssess={onAssessProject}
          onOpenProject={onOpenProject}
          onRefresh={onRefreshProjects}
        />
      </Box>
    </Box>
  );
}
