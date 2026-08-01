import React from 'react';
import { SelectInput } from '../primitives';
import type { ExternalIssueTypeSummary, IssueTypeSummary } from '../../types/messages';

interface MappingRowProps {
  external: ExternalIssueTypeSummary;
  operatorTypes: IssueTypeSummary[];
  selectedKey: string | null;
  autoMatchedKey: string | null;
  onSelect: (externalName: string, operatorKey: string | '') => void;
  onViewIssueType: () => void;
}

const DIVIDER_BORDER = '1px solid var(--vscode-sideBar-border, var(--vscode-widget-border, #45454580))';
const INFO_COLOR = 'var(--vscode-textLink-foreground, #3794ff)';

export function MappingRow({
  external,
  operatorTypes,
  selectedKey,
  autoMatchedKey,
  onSelect,
  onViewIssueType,
}: MappingRowProps) {
  const effectiveKey = selectedKey ?? autoMatchedKey;
  const isOverride = selectedKey !== null && selectedKey !== autoMatchedKey;

  return (
    <div style={{ padding: '8px 0', borderBottom: DIVIDER_BORDER }}>
      <div className="op-row op-gap-2">
        {/* External type */}
        <div className="op-row op-gap-1" style={{ flex: 1 }}>
          {external.icon_url && <img src={external.icon_url} alt="" style={{ width: 16, height: 16 }} />}
          <span className="op-body2" style={{ fontWeight: 500 }}>
            {external.name}
          </span>
        </div>

        {/* Arrow */}
        <span className="op-text-secondary" style={{ padding: '0 8px' }}>→</span>

        {/* Operator type selector */}
        <div style={{ flex: 1 }}>
          <SelectInput
            value={effectiveKey ?? ''}
            onChange={(e) => onSelect(external.name, e.target.value)}
          >
            <option value="">Unmapped</option>
            {operatorTypes.map((ot) => (
              <option key={ot.key} value={ot.key}>
                {ot.glyph} {ot.key} — {ot.name}
              </option>
            ))}
          </SelectInput>
          {autoMatchedKey && !isOverride && (
            <span className="op-caption op-text-secondary" style={{ marginTop: 2, display: 'block' }}>
              auto-matched
            </span>
          )}
          {isOverride && (
            <span className="op-caption" style={{ marginTop: 2, display: 'block', color: INFO_COLOR }}>
              custom override
            </span>
          )}
        </div>
      </div>

      {/* View the mapped issue type in the hosted Operator UI */}
      {effectiveKey && (
        <div style={{ marginTop: 4, marginLeft: 32 }}>
          <button
            type="button"
            className="op-link op-caption"
            onClick={onViewIssueType}
            style={{ textAlign: 'left' }}
          >
            view issue type →
          </button>
        </div>
      )}
    </div>
  );
}
