import React, { useCallback } from "react";
import { SelectInput } from "../primitives";
import type { ExternalIssueTypeSummary, IssueTypeSummary } from "../../types/messages";

interface MappingRowProps {
  external: ExternalIssueTypeSummary;
  operatorTypes: IssueTypeSummary[];
  selectedKey: string | null;
  autoMatchedKey: string | null;
  onSelect: (externalName: string, operatorKey: string) => void;
  onViewIssueType: () => void;
}

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
  const handleSelectionChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => onSelect(external.name, event.target.value),
    [external.name, onSelect],
  );

  return (
    <div className="op-mapping-row">
      <div className="op-row op-gap-2">
        {/* External type */}
        <div className="op-row op-gap-1 op-flex-1">
          {external.icon_url && <img src={external.icon_url} alt="" className="op-issue-icon" />}
          <span className="op-body2 op-medium">{external.name}</span>
        </div>

        {/* Arrow */}
        <span className="op-text-secondary op-mapping-arrow">→</span>

        {/* Operator type selector */}
        <div className="op-flex-1">
          <SelectInput value={effectiveKey ?? ""} onChange={handleSelectionChange}>
            <option value="">Unmapped</option>
            {operatorTypes.map((ot) => (
              <option key={ot.key} value={ot.key}>
                {ot.glyph} {ot.key} - {ot.name}
              </option>
            ))}
          </SelectInput>
          {autoMatchedKey && !isOverride && (
            <span className="op-caption op-text-secondary op-mapping-hint">auto-matched</span>
          )}
          {isOverride && <span className="op-caption op-mapping-override">custom override</span>}
        </div>
      </div>

      {/* View the mapped issue type in the hosted Operator UI */}
      {effectiveKey && (
        <div className="op-mapping-link">
          <button
            type="button"
            className="op-link op-caption op-text-left"
            onClick={onViewIssueType}
          >
            view issue type
          </button>
        </div>
      )}
    </div>
  );
}
