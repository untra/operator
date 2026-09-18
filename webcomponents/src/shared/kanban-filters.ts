/**
 * Client-side board filtering.
 *
 * The board endpoint returns everything; narrowing it is view state, so it
 * lives here as a pure transform plus a hook that remembers the choice per
 * viewer. Empty arrays mean "no filter" - there are no sentinel values.
 */

import { useCallback, useState } from "react";

import type { KanbanBoardResponse } from "../generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import type { TicketPriority } from "../generated/TicketPriority";
import { PRIORITY_ORDER } from "./ticket-fields";

const STORAGE_KEY = "operator-queue-filters";

const COLUMNS = ["queue", "running", "awaiting", "done"] as const;

export interface KanbanFilterState {
  search: string;
  projects: string[];
  types: string[];
  priorities: TicketPriority[];
}

export const DEFAULT_FILTER_STATE: KanbanFilterState = {
  search: "",
  projects: [],
  types: [],
  priorities: [],
};

/** The options a bar can offer, derived from the board rather than hardcoded. */
export interface KanbanFacets {
  projects: string[];
  types: string[];
  priorities: TicketPriority[];
}

function allCards(board: KanbanBoardResponse): KanbanTicketCard[] {
  return COLUMNS.flatMap((column) => board[column]);
}

function distinct(values: string[]): string[] {
  return [...new Set(values)].toSorted();
}

export function hasActiveFilters(state: KanbanFilterState): boolean {
  return (
    state.search.trim().length > 0 ||
    state.projects.length > 0 ||
    state.types.length > 0 ||
    state.priorities.length > 0
  );
}

export function facetsFromBoard(board: KanbanBoardResponse): KanbanFacets {
  const cards = allCards(board);
  const present = new Set(cards.map((c) => c.priority));
  return {
    projects: distinct(cards.map((c) => c.project)),
    types: distinct(cards.map((c) => c.ticket_type)),
    priorities: PRIORITY_ORDER.filter((p) => present.has(p)),
  };
}

/** Same matching the docs-site catalog search uses: every term, anywhere. */
function matchesSearch(card: KanbanTicketCard, terms: string[]): boolean {
  if (terms.length === 0) {
    return true;
  }
  const haystack = `${card.id} ${card.summary}`.toLowerCase();
  return terms.every((term) => haystack.includes(term));
}

function matchesFacet(value: string, selected: string[]): boolean {
  return selected.length === 0 || selected.includes(value);
}

export function filterBoard(
  board: KanbanBoardResponse,
  state: KanbanFilterState,
): KanbanBoardResponse {
  if (!hasActiveFilters(state)) {
    return board;
  }
  const terms = state.search.toLowerCase().split(/\s+/).filter(Boolean);
  const keep = (card: KanbanTicketCard) =>
    matchesSearch(card, terms) &&
    matchesFacet(card.project, state.projects) &&
    matchesFacet(card.ticket_type, state.types) &&
    matchesFacet(card.priority, state.priorities);

  const queue = board.queue.filter(keep);
  const running = board.running.filter(keep);
  const awaiting = board.awaiting.filter(keep);
  const done = board.done.filter(keep);

  return {
    queue,
    running,
    awaiting,
    done,
    total_count: queue.length + running.length + awaiting.length + done.length,
    last_updated: board.last_updated,
  };
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((v) => typeof v === "string");
}

/** Storage can be absent or throw in a restricted webview; never trust it. */
function readStoredFilters(): KanbanFilterState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed: unknown = JSON.parse(raw);
      if (parsed && typeof parsed === "object") {
        const candidate = parsed as Partial<Record<keyof KanbanFilterState, unknown>>;
        return {
          search: typeof candidate.search === "string" ? candidate.search : "",
          projects: isStringArray(candidate.projects) ? candidate.projects : [],
          types: isStringArray(candidate.types) ? candidate.types : [],
          priorities: isStringArray(candidate.priorities)
            ? candidate.priorities.filter((p): p is TicketPriority =>
                (PRIORITY_ORDER as string[]).includes(p),
              )
            : [],
        };
      }
    }
  } catch {
    // localStorage unavailable (e.g. restricted webview) - fall through
  }
  return DEFAULT_FILTER_STATE;
}

export interface KanbanFiltersHandle {
  state: KanbanFilterState;
  setState: (next: KanbanFilterState) => void;
  clear: () => void;
}

export function useKanbanFilters(): KanbanFiltersHandle {
  const [state, setStateRaw] = useState<KanbanFilterState>(readStoredFilters);

  const setState = useCallback((next: KanbanFilterState) => {
    setStateRaw(next);
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      // ignore persistence failures
    }
  }, []);

  const clear = useCallback(() => setState(DEFAULT_FILTER_STATE), [setState]);

  return { state, setState, clear };
}
