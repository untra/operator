import React, { useRef } from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import { SidebarNav, type NavItem } from './SidebarNav';
import { OperatorBrand } from './OperatorBrand';
import { PrimaryConfigSection } from './sections/PrimaryConfigSection';
import { CodingAgentsSection } from './sections/CodingAgentsSection';
import { KanbanProvidersSection } from './sections/KanbanProvidersSection';
import { GitRepositoriesSection } from './sections/GitRepositoriesSection';
import type { WebviewConfig, JiraValidationInfo, LinearValidationInfo } from '../types/messages';

const NAV_ITEMS: NavItem[] = [
  { id: 'section-primary', label: 'Primary' },
  { id: 'section-agents', label: 'Coding Agents' },
  { id: 'section-kanban', label: 'Kanban' },
  { id: 'section-git', label: 'Git Repos' },
];

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
}: ConfigPageProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  return (
    <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <SidebarNav items={NAV_ITEMS} scrollContainerRef={scrollRef} />

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
          sessions_wrapper={(config.config.sessions as Record<string, unknown>)?.wrapper as string ?? 'vscode'}
          onUpdate={onUpdate}
          onBrowseFolder={onBrowseFolder}
        />
        <KanbanProvidersSection
          kanban={config.config.kanban as Record<string, unknown>}
          onUpdate={onUpdate}
          onValidateJira={onValidateJira}
          onValidateLinear={onValidateLinear}
          jiraResult={jiraResult}
          linearResult={linearResult}
          validatingJira={validatingJira}
          validatingLinear={validatingLinear}
        />
        <CodingAgentsSection
          agents={config.config.agents as Record<string, unknown>}
          llm_tools={config.config.llm_tools as Record<string, unknown>}
          onUpdate={onUpdate}
          onDetectTools={onDetectTools}
        />
        <GitRepositoriesSection
          git={config.config.git as Record<string, unknown>}
          projects={(config.config.projects as string[]) ?? []}
          onUpdate={onUpdate}
        />
      </Box>
    </Box>
  );
}
