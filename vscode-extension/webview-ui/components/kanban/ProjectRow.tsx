import React, { useEffect, useState } from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Chip from '@mui/material/Chip';
import Collapse from '@mui/material/Collapse';
import { MappingPanel } from './MappingPanel';
import type { ProjectSyncConfig } from '../../../src/generated/ProjectSyncConfig';
import type { KanbanStatusMapping } from '../../../src/generated/KanbanStatusMapping';
import type { IssueTypeSummary, CollectionResponse, ExternalIssueTypeSummary } from '../../types/messages';

interface ProjectRowProps {
  provider: string;
  domain: string;
  projectKey: string;
  project: ProjectSyncConfig;
  collections: CollectionResponse[];
  issueTypes: IssueTypeSummary[];
  externalTypes: ExternalIssueTypeSummary[] | undefined;
  statuses: string[] | undefined;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  onGetKanbanStatuses: (provider: string, projectKey: string) => void;
  onViewIssueType: () => void;
  sectionKey: string;
}

const OPERATOR_STATES = [
  { field: 'todo', label: 'Todo', helper: 'Pulled into the queue; requeue pushes back here' },
  { field: 'doing', label: 'Doing', helper: 'Pushed when a ticket is launched/claimed' },
  { field: 'done', label: 'Done', helper: 'Pushed when a ticket completes' },
] as const;

export function ProjectRow({
  provider,
  domain,
  projectKey,
  project,
  collections,
  issueTypes,
  externalTypes,
  statuses,
  onUpdate,
  onGetExternalIssueTypes,
  onGetKanbanStatuses,
  onViewIssueType,
  sectionKey,
}: ProjectRowProps) {
  const [expanded, setExpanded] = useState(false);

  const mappingCount = Object.keys(project.type_mappings ?? {}).length;
  const statusMapping: KanbanStatusMapping = project.status_mapping ?? {};

  // Lazily discover the board's real columns the first time the row expands.
  useEffect(() => {
    if (expanded && statuses === undefined) {
      onGetKanbanStatuses(provider, projectKey);
    }
  }, [expanded, statuses, provider, projectKey, onGetKanbanStatuses]);

  const handleMappingChange = (externalName: string, operatorKey: string | '') => {
    const newMappings = { ...(project.type_mappings ?? {}) };
    if (operatorKey === '') {
      delete newMappings[externalName];
    } else {
      newMappings[externalName] = operatorKey;
    }
    onUpdate(sectionKey, `projects.${projectKey}.type_mappings`, newMappings);
  };

  const handleStatusMappingChange = (field: 'todo' | 'doing' | 'done', column: string) => {
    const next: KanbanStatusMapping = { ...statusMapping };
    if (column === '') {
      delete next[field];
    } else {
      next[field] = column;
    }
    onUpdate(sectionKey, `projects.${projectKey}.status_mapping`, next);
  };

  /** Discovered columns plus the currently-mapped value (so a stale mapping stays visible). */
  const optionsFor = (current: string | null | undefined): string[] => {
    const opts = [...(statuses ?? [])];
    if (current && !opts.includes(current)) {
      opts.push(current);
    }
    return opts;
  };

  return (
    <Box sx={{ borderBottom: '1px solid', borderColor: 'divider', py: 1 }}>
      <Box
        sx={{ display: 'flex', alignItems: 'center', gap: 2, cursor: 'pointer' }}
        onClick={() => setExpanded(!expanded)}
      >
        <Typography variant="body2" fontWeight={600} sx={{ minWidth: 80 }}>
          {projectKey}
        </Typography>

        <FormControl size="small" sx={{ minWidth: 160 }} onClick={(e) => e.stopPropagation()}>
          <InputLabel sx={{ fontSize: '0.8rem' }}>Collection</InputLabel>
          <Select
            value={project.collection_name || ''}
            label="Collection"
            onChange={(e) => onUpdate(sectionKey, `projects.${projectKey}.collection_name`, e.target.value)}
            sx={{ '& .MuiSelect-select': { py: 0.5, fontSize: '0.85rem' } }}
          >
            <MenuItem value="">
              <em>None</em>
            </MenuItem>
            {collections.map((c) => (
              <MenuItem key={c.name} value={c.name}>
                {c.name}
                {c.is_active && ' ✓'}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <Box sx={{ display: 'flex', gap: 0.5, flex: 1 }} onClick={(e) => e.stopPropagation()}>
          {OPERATOR_STATES.filter(({ field }) => statusMapping[field]).map(({ field, label }) => (
            <Chip
              key={field}
              label={`${label} → ${statusMapping[field]}`}
              size="small"
              variant="outlined"
            />
          ))}
        </Box>

        {mappingCount > 0 && (
          <Chip
            label={`${mappingCount} mapped`}
            size="small"
            color="info"
            variant="outlined"
          />
        )}

        <IconButton size="small" sx={{ transform: expanded ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }}>
          <Typography variant="body2">▾</Typography>
        </IconButton>
      </Box>

      <Collapse in={expanded}>
        <Box sx={{ pl: 2, pt: 1 }}>
          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 0.5 }}>
            Column Mapping — map operator's todo/doing/done to this board's columns
          </Typography>
          <Box sx={{ display: 'flex', gap: 1, mb: 1 }}>
            {OPERATOR_STATES.map(({ field, label, helper }) => (
              <FormControl key={field} size="small" sx={{ minWidth: 160, flex: 1 }}>
                <InputLabel sx={{ fontSize: '0.8rem' }}>{label}</InputLabel>
                <Select
                  value={statusMapping[field] ?? ''}
                  label={label}
                  onChange={(e) => handleStatusMappingChange(field, e.target.value)}
                  sx={{ '& .MuiSelect-select': { py: 0.5, fontSize: '0.85rem' } }}
                  title={helper}
                >
                  <MenuItem value="">
                    <em>Unmapped</em>
                  </MenuItem>
                  {optionsFor(statusMapping[field]).map((column) => (
                    <MenuItem key={column} value={column}>
                      {column}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            ))}
          </Box>

          <MappingPanel
            provider={provider}
            domain={domain}
            projectKey={projectKey}
            collectionName={project.collection_name||''}
            typeMappings={project.type_mappings ?? {}}
            issueTypes={issueTypes}
            externalTypes={externalTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            onMappingChange={handleMappingChange}
            onViewIssueType={onViewIssueType}
          />
        </Box>
      </Collapse>
    </Box>
  );
}
