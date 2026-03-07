import React, { useState } from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import TextField from '@mui/material/TextField';
import Chip from '@mui/material/Chip';
import Collapse from '@mui/material/Collapse';
import { MappingPanel } from './MappingPanel';
import type { ProjectSyncConfig } from '../../../src/generated/ProjectSyncConfig';
import type { IssueTypeSummary, CollectionResponse, ExternalIssueTypeSummary, IssueTypeResponse } from '../../types/messages';

interface ProjectRowProps {
  provider: string;
  domain: string;
  projectKey: string;
  project: ProjectSyncConfig;
  collections: CollectionResponse[];
  issueTypes: IssueTypeSummary[];
  externalTypes: ExternalIssueTypeSummary[] | undefined;
  selectedIssueType: IssueTypeResponse | null;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  onViewIssueType: (key: string) => void;
  sectionKey: string;
}

export function ProjectRow({
  provider,
  domain,
  projectKey,
  project,
  collections,
  issueTypes,
  externalTypes,
  selectedIssueType,
  onUpdate,
  onGetExternalIssueTypes,
  onViewIssueType,
  sectionKey,
}: ProjectRowProps) {
  const [expanded, setExpanded] = useState(false);

  const mappingCount = Object.keys(project.type_mappings ?? {}).length;

  const handleMappingChange = (externalName: string, operatorKey: string | '') => {
    const newMappings = { ...(project.type_mappings ?? {}) };
    if (operatorKey === '') {
      delete newMappings[externalName];
    } else {
      newMappings[externalName] = operatorKey;
    }
    onUpdate(sectionKey, `projects.${projectKey}.type_mappings`, newMappings);
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
          {(project.sync_statuses ?? []).map((status) => (
            <Chip key={status} label={status} size="small" variant="outlined" />
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
          <TextField
            size="small"
            label="Sync Statuses"
            value={(project.sync_statuses ?? []).join(', ')}
            onChange={(e) => {
              const statuses = e.target.value.split(',').map((s) => s.trim()).filter(Boolean);
              onUpdate(sectionKey, `projects.${projectKey}.sync_statuses`, statuses);
            }}
            placeholder="To Do, In Progress"
            fullWidth
            sx={{ mb: 1 }}
            helperText="Workflow statuses to sync (comma-separated)"
          />

          <MappingPanel
            provider={provider}
            domain={domain}
            projectKey={projectKey}
            collectionName={project.collection_name}
            typeMappings={project.type_mappings ?? {}}
            issueTypes={issueTypes}
            externalTypes={externalTypes}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            onMappingChange={handleMappingChange}
            onViewIssueType={onViewIssueType}
            selectedIssueType={selectedIssueType}
          />
        </Box>
      </Collapse>
    </Box>
  );
}
