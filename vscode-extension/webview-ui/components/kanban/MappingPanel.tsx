import React, { useEffect, useMemo, useState } from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import { MappingRow } from './MappingRow';
import type { ExternalIssueTypeSummary, IssueTypeSummary, IssueTypeResponse } from '../../types/messages';

interface MappingPanelProps {
  provider: string;
  domain: string;
  projectKey: string;
  collectionName: string;
  typeMappings: { [key: string]: string | undefined };
  issueTypes: IssueTypeSummary[];
  externalTypes: ExternalIssueTypeSummary[] | undefined;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  onMappingChange: (externalName: string, operatorKey: string | '') => void;
  onViewIssueType: (key: string) => void;
  selectedIssueType: IssueTypeResponse | null;
}

function autoMap(externalName: string, operatorTypes: IssueTypeSummary[]): string | null {
  const name = externalName.toLowerCase();
  const rules: [RegExp, string][] = [
    [/bug|defect|fix|issue/, 'FIX'],
    [/story|feature|enhancement/, 'FEAT'],
    [/task|subtask|item|card/, 'TASK'],
    [/spike|research|milestone/, 'SPIKE'],
    [/incident|investigation|initiative/, 'INV'],
  ];
  for (const [pattern, key] of rules) {
    if (pattern.test(name) && operatorTypes.some(t => t.key === key)) {
      return key;
    }
  }
  return null;
}

export function MappingPanel({
  provider,
  domain,
  projectKey,
  collectionName,
  typeMappings,
  issueTypes,
  externalTypes,
  onGetExternalIssueTypes,
  onMappingChange,
  onViewIssueType,
  selectedIssueType,
}: MappingPanelProps) {
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!externalTypes) {
      setLoading(true);
      onGetExternalIssueTypes(provider, domain, projectKey);
    }
  }, [provider, domain, projectKey, externalTypes, onGetExternalIssueTypes]);

  useEffect(() => {
    if (externalTypes) {
      setLoading(false);
    }
  }, [externalTypes]);

  const autoMappings = useMemo(() => {
    const map = new Map<string, string | null>();
    if (externalTypes) {
      for (const et of externalTypes) {
        map.set(et.name, autoMap(et.name, issueTypes));
      }
    }
    return map;
  }, [externalTypes, issueTypes]);

  if (loading || !externalTypes) {
    return (
      <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress size={20} />
        <Typography variant="body2" color="text.secondary" sx={{ ml: 1 }}>
          Loading issue types from {provider}...
        </Typography>
      </Box>
    );
  }

  if (externalTypes.length === 0) {
    return (
      <Alert severity="info" sx={{ mt: 1 }}>
        No issue types found in {provider} project {projectKey}
      </Alert>
    );
  }

  return (
    <Box sx={{ mt: 1 }}>
      <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>
        Issue Type Mappings for {projectKey}
        {collectionName && ` (collection: ${collectionName})`}
      </Typography>
      {externalTypes.map((et) => {
        const autoKey = autoMappings.get(et.name) ?? null;
        const overrideKey = typeMappings[et.name] ?? null;
        return (
          <MappingRow
            key={et.id}
            external={et}
            operatorTypes={issueTypes}
            selectedKey={overrideKey}
            autoMatchedKey={autoKey}
            selectedIssueTypeDetail={selectedIssueType}
            onSelect={onMappingChange}
            onViewIssueType={onViewIssueType}
          />
        );
      })}
    </Box>
  );
}
