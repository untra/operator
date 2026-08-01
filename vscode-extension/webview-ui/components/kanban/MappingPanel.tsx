import React, { useEffect, useMemo, useState } from 'react';
import { Alert, Spinner } from '../primitives';
import { MappingRow } from './MappingRow';
import type { ExternalIssueTypeSummary, IssueTypeSummary } from '../../types/messages';

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
  onViewIssueType: () => void;
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
      <div className="op-row" style={{ padding: '16px 0', justifyContent: 'center' }}>
        <Spinner size={20} />
        <span className="op-body2 op-text-secondary" style={{ marginLeft: 8 }}>
          Loading issue types from {provider}...
        </span>
      </div>
    );
  }

  if (externalTypes.length === 0) {
    return (
      <Alert severity="info" className="op-mt-1">
        No issue types found in {provider} project {projectKey}
      </Alert>
    );
  }

  return (
    <div className="op-mt-1">
      <span className="op-caption op-text-secondary op-mb-1" style={{ display: 'block' }}>
        Issue Type Mappings for {projectKey}
        {collectionName && ` (collection: ${collectionName})`}
      </span>
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
            onSelect={onMappingChange}
            onViewIssueType={onViewIssueType}
          />
        );
      })}
    </div>
  );
}
