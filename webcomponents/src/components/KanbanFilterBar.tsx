import { useCallback } from "react";

import type { TicketPriority } from "../generated/TicketPriority";
import {
  hasActiveFilters,
  type KanbanFacets,
  type KanbanFilterState,
} from "../shared/kanban-filters";
import styles from "./KanbanFilterBar.module.css";

export interface KanbanFilterBarProps {
  facets: KanbanFacets;
  state: KanbanFilterState;
  onChange: (next: KanbanFilterState) => void;
  onClear: () => void;
}

function toggle<T extends string>(selected: T[], value: T): T[] {
  return selected.includes(value) ? selected.filter((v) => v !== value) : [...selected, value];
}

export function KanbanFilterBar({ facets, state, onChange, onClear }: KanbanFilterBarProps) {
  const onSearch = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => onChange({ ...state, search: e.target.value }),
    [onChange, state],
  );
  const onProject = useCallback(
    (value: string) => onChange({ ...state, projects: toggle(state.projects, value) }),
    [onChange, state],
  );
  const onType = useCallback(
    (value: string) => onChange({ ...state, types: toggle(state.types, value) }),
    [onChange, state],
  );
  const onPriority = useCallback(
    (value: string) =>
      onChange({ ...state, priorities: toggle(state.priorities, value as TicketPriority) }),
    [onChange, state],
  );

  return (
    <div className={styles.bar}>
      <input
        type="search"
        className={styles.search}
        value={state.search}
        placeholder="Search tickets"
        aria-label="Search tickets"
        onChange={onSearch}
      />
      <Facet
        label="Project"
        options={facets.projects}
        selected={state.projects}
        onToggle={onProject}
      />
      <Facet label="Type" options={facets.types} selected={state.types} onToggle={onType} />
      <Facet
        label="Priority"
        options={facets.priorities}
        selected={state.priorities}
        onToggle={onPriority}
      />
      {hasActiveFilters(state) && (
        <button type="button" className={styles.clear} onClick={onClear}>
          Clear filters
        </button>
      )}
    </div>
  );
}

/** A facet with fewer than two options cannot narrow anything, so it is hidden. */
function Facet({
  label,
  options,
  selected,
  onToggle,
}: {
  label: string;
  options: string[];
  selected: string[];
  onToggle: (value: string) => void;
}) {
  if (options.length < 2) {
    return null;
  }
  return (
    <fieldset className={styles.facet}>
      <legend className={styles.facetLabel}>{label}</legend>
      <div className={styles.group}>
        {options.map((option) => (
          <FacetOption
            key={option}
            option={option}
            pressed={selected.includes(option)}
            onToggle={onToggle}
          />
        ))}
      </div>
    </fieldset>
  );
}

function FacetOption({
  option,
  pressed,
  onToggle,
}: {
  option: string;
  pressed: boolean;
  onToggle: (value: string) => void;
}) {
  const onClick = useCallback(() => onToggle(option), [onToggle, option]);
  return (
    <button
      type="button"
      aria-pressed={pressed}
      className={pressed ? styles.optionActive : styles.option}
      onClick={onClick}
    >
      {option}
    </button>
  );
}
