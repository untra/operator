import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Chip, IconButton, SelectInput } from "../primitives";
import { MappingPanel } from "./MappingPanel";
import type { ProjectSyncConfig } from "../../../src/generated/ProjectSyncConfig";
import type { KanbanStatusMapping } from "../../../src/generated/KanbanStatusMapping";
import type {
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from "../../types/messages";

interface ProjectRowProps {
  provider: string;
  domain: string;
  projectKey: string;
  project: ProjectSyncConfig;
  collections: CollectionResponse[];
  issueTypes: IssueTypeSummary[];
  externalTypes: ExternalIssueTypeSummary[] | undefined;
  lookupEnabled: boolean;
  statuses: string[] | undefined;
  onUpdate: (key: string, value: unknown) => void;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  onGetKanbanStatuses: (provider: string, domain: string, projectKey: string) => void;
  onViewIssueType: () => void;
}

const NO_EXTERNAL_TYPES: ExternalIssueTypeSummary[] = [];
const NO_TYPE_MAPPINGS = {};

const OPERATOR_STATES = [
  { field: "todo", label: "Todo", helper: "Pulled into the queue; requeue pushes back here" },
  { field: "doing", label: "Doing", helper: "Pushed when a ticket is launched/claimed" },
  { field: "done", label: "Done", helper: "Pushed when a ticket completes" },
] as const;

export function ProjectRow({
  provider,
  domain,
  projectKey,
  project,
  collections,
  issueTypes,
  externalTypes,
  lookupEnabled,
  statuses,
  onUpdate,
  onGetExternalIssueTypes,
  onGetKanbanStatuses,
  onViewIssueType,
}: ProjectRowProps) {
  const [expanded, setExpanded] = useState(false);

  const mappingCount = Object.keys(project.type_mappings ?? {}).length;
  const statusMapping = useMemo<KanbanStatusMapping>(
    () => project.status_mapping ?? {},
    [project.status_mapping],
  );

  // Lazily discover the board's real columns the first time the row expands.
  useEffect(() => {
    if (lookupEnabled && expanded && statuses === undefined) {
      onGetKanbanStatuses(provider, domain, projectKey);
    }
  }, [lookupEnabled, domain, expanded, statuses, provider, projectKey, onGetKanbanStatuses]);

  const handleMappingChange = useCallback(
    (externalName: string, operatorKey: string) => {
      const newMappings = { ...project.type_mappings };
      if (operatorKey === "") {
        delete newMappings[externalName];
      } else {
        newMappings[externalName] = operatorKey;
      }
      onUpdate(`projects.${projectKey}.type_mappings`, newMappings);
    },
    [onUpdate, project.type_mappings, projectKey],
  );

  const handleStatusMappingChange = useCallback(
    (field: "todo" | "doing" | "done", column: string) => {
      const next: KanbanStatusMapping = { ...statusMapping };
      if (column === "") {
        delete next[field];
      } else {
        next[field] = column;
      }
      onUpdate(`projects.${projectKey}.status_mapping`, next);
    },
    [onUpdate, projectKey, statusMapping],
  );

  /** Discovered columns plus the currently-mapped value (so a stale mapping stays visible). */
  const optionsFor = (current: string | null | undefined): string[] => {
    const opts = [...(statuses ?? [])];
    if (current && !opts.includes(current)) {
      opts.push(current);
    }
    return opts;
  };

  const toggleExpanded = useCallback(() => setExpanded((current) => !current), [setExpanded]);
  const handleCollectionChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) =>
      onUpdate(`projects.${projectKey}.collection_name`, event.target.value),
    [onUpdate, projectKey],
  );

  return (
    <div className="op-mapping-row">
      <div className="op-row op-gap-2">
        <span className="op-body2 op-project-key">{projectKey}</span>

        <div className="op-collection-picker">
          <SelectInput
            label="Collection"
            value={project.collection_name || ""}
            onChange={handleCollectionChange}
          >
            <option value="">None</option>
            {collections.map((c) => (
              <option key={c.name} value={c.name}>
                {c.name}
                {c.is_active && " ✓"}
              </option>
            ))}
          </SelectInput>
        </div>

        <div className="op-row op-gap-05 op-flex-1">
          {OPERATOR_STATES.filter(({ field }) => statusMapping[field]).map(({ field, label }) => (
            <Chip key={field} label={`${label} → ${statusMapping[field]}`} variant="outlined" />
          ))}
        </div>

        {mappingCount > 0 && <Chip label={`${mappingCount} mapped`} variant="outlined" />}

        <IconButton
          aria-label={expanded ? "Collapse project" : "Expand project"}
          onClick={toggleExpanded}
        >
          <span className="op-body2 op-expander" data-expanded={expanded}>
            ▾
          </span>
        </IconButton>
      </div>

      {expanded && (
        <div className="op-project-detail">
          {!lookupEnabled && (
            <p>Live board lookup requires an API connection and a unique enabled project key.</p>
          )}
          <span className="op-caption op-text-secondary op-mb-05 op-block">
            Column Mapping - map operator's todo/doing/done to this board's columns
          </span>
          <div className="op-row op-gap-1 op-mb-1">
            {OPERATOR_STATES.map(({ field, label, helper }) => (
              <StatusMappingSelect
                key={field}
                field={field}
                label={label}
                helper={helper}
                value={statusMapping[field]}
                options={optionsFor(statusMapping[field])}
                onChange={handleStatusMappingChange}
              />
            ))}
          </div>

          <MappingPanel
            provider={provider}
            domain={domain}
            projectKey={projectKey}
            collectionName={project.collection_name || ""}
            typeMappings={project.type_mappings ?? NO_TYPE_MAPPINGS}
            issueTypes={issueTypes}
            externalTypes={lookupEnabled ? externalTypes : NO_EXTERNAL_TYPES}
            onGetExternalIssueTypes={onGetExternalIssueTypes}
            onMappingChange={handleMappingChange}
            onViewIssueType={onViewIssueType}
          />
        </div>
      )}
    </div>
  );
}

function StatusMappingSelect({
  field,
  label,
  helper,
  value,
  options,
  onChange,
}: {
  field: "todo" | "doing" | "done";
  label: string;
  helper: string;
  value: string | null | undefined;
  options: string[];
  onChange: (field: "todo" | "doing" | "done", column: string) => void;
}) {
  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => onChange(field, event.target.value),
    [field, onChange],
  );
  return (
    <div title={helper} className="op-status-picker">
      <SelectInput label={label} value={value ?? ""} onChange={handleChange}>
        <option value="">Unmapped</option>
        {options.map((column) => (
          <option key={column} value={column}>
            {column}
          </option>
        ))}
      </SelectInput>
    </div>
  );
}
