import React, { useMemo, useRef } from 'react';
import { Button } from './primitives';
import { SidebarNav, type NavItem } from './SidebarNav';
import { OperatorBrand } from './OperatorBrand';
import { LinkOutCard } from './LinkOutCard';
import { PrimaryConfigSection } from './sections/PrimaryConfigSection';
import { CodingAgentsSection } from './sections/CodingAgentsSection';
import { ModelProvidersSection } from './sections/ModelProvidersSection';
import { KanbanProvidersSection } from './sections/KanbanProvidersSection';
import { GitRepositoriesSection } from './sections/GitRepositoriesSection';
import type {
  WebviewConfig,
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from '../types/messages';

interface ConfigPageProps {
  config: WebviewConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onBrowseFolder: (field: string) => void;
  onOpenFile: (filePath: string) => void;
  onStartSetup: () => void;
  onValidateJira: (domain: string, email: string, apiToken: string) => void;
  onValidateLinear: (apiKey: string) => void;
  onDetectTools: () => void;
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
  onOpenOperatorUi: (route: 'issuetypes' | 'projects') => void;
}

export function ConfigPage({
  config,
  onUpdate,
  onBrowseFolder,
  onOpenFile,
  onStartSetup,
  onValidateJira,
  onValidateLinear,
  onDetectTools,
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
}: ConfigPageProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const hasWorkDir = Boolean(config.working_directory);
  // When no config file exists yet (or it's empty), nudge the user into the
  // setup walkthrough instead of trying to open a non-existent config.toml.
  const needsSetup = !config.config_exists;

  const navItems: NavItem[] = useMemo(() => [
    { id: 'section-primary', label: 'Workspace Configuration' },
    { id: 'section-kanban', label: 'Kanban Providers', disabled: !hasWorkDir },
    { id: 'section-agents', label: 'Coding Agents', disabled: !hasWorkDir },
    { id: 'section-model-providers', label: 'Model Providers', disabled: !apiReachable },
    { id: 'section-git', label: 'Git Version Control', disabled: !hasWorkDir },
    { id: 'section-projects', label: 'Operator Managed Projects', disabled: !apiReachable },
  ], [hasWorkDir, apiReachable]);

  return (
    <div style={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <SidebarNav items={navItems} scrollContainerRef={scrollRef} />

      <div ref={scrollRef} style={{ flexGrow: 1, overflow: 'auto', padding: '16px 24px' }}>
        <div className="op-row op-space-between op-mb-2">
          <h1 className="op-h6">
            <OperatorBrand /> Settings
          </h1>
          {needsSetup ? (
            <Button variant="contained" accent="sage" onClick={onStartSetup}>
              start operator setup
            </Button>
          ) : (
            <Button
              variant="outlined"
              accent="sage"
              onClick={() => onOpenFile(config.config_path)}
              disabled={!config.working_directory}
            >
              edit config.toml
            </Button>
          )}
        </div>
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
          collections={collections}
          externalIssueTypes={externalIssueTypes}
          onGetExternalIssueTypes={onGetExternalIssueTypes}
          kanbanStatuses={kanbanStatuses}
          onGetKanbanStatuses={onGetKanbanStatuses}
          onOpenOperatorUi={onOpenOperatorUi}
        />
        <CodingAgentsSection
          agents={config.config.agents}
          llm_tools={config.config.llm_tools}
          onUpdate={onUpdate}
          onDetectTools={onDetectTools}
        />
        <ModelProvidersSection
          detectedTools={config.config.llm_tools.detected.map((t) => t.name)}
          apiReachable={apiReachable}
        />
        <GitRepositoriesSection
          git={config.config.git}
          onUpdate={onUpdate}
        />
        <LinkOutCard
          id="section-projects"
          title="Operator Managed Projects"
          description="Browse, assess, and open managed projects in the Operator UI."
          onOpen={() => onOpenOperatorUi('projects')}
        />
      </div>
    </div>
  );
}
