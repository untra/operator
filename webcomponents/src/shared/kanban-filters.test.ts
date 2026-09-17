/**
 * Board filtering is a pure transform of the payload, which is the whole point
 * of keeping it out of the components.
 */

import { describe, expect, test } from "bun:test";

import type { KanbanBoardResponse } from "../generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../generated/KanbanTicketCard";
import {
  DEFAULT_FILTER_STATE,
  facetsFromBoard,
  filterBoard,
  hasActiveFilters,
  type KanbanFilterState,
} from "./kanban-filters";

function card(over: Partial<KanbanTicketCard> & { id: string }): KanbanTicketCard {
  return {
    summary: "A ticket",
    ticket_type: "FEAT",
    project: "operator",
    status: "queued",
    step: "plan",
    step_display_name: "Plan",
    priority: "P2-medium",
    timestamp: "20260101-1200",
    filename: `${over.id}.md`,
    ...over,
  };
}

const board: KanbanBoardResponse = {
  queue: [
    card({ id: "FEAT-1", summary: "Add audit history", project: "operator" }),
    card({
      id: "TASK-2",
      summary: "Rotate signing key",
      ticket_type: "TASK",
      priority: "P0-critical",
    }),
  ],
  running: [
    card({
      id: "FIX-3",
      summary: "Stale queue results",
      ticket_type: "FIX",
      project: "gamesvc",
      status: "running",
    }),
  ],
  awaiting: [
    card({
      id: "INV-4",
      summary: "Compare retry behaviour",
      ticket_type: "INV",
      project: "gamesvc",
      status: "awaiting",
    }),
  ],
  done: [
    card({
      id: "SPIKE-5",
      summary: "Prototype layout",
      ticket_type: "SPIKE",
      status: "completed",
      priority: "P3-low",
    }),
  ],
  total_count: 5,
  last_updated: "2026-01-01T12:00:00Z",
};

function withFilter(over: Partial<KanbanFilterState>): KanbanFilterState {
  return { ...DEFAULT_FILTER_STATE, ...over };
}

describe("filterBoard", () => {
  test("the default state is inert", () => {
    expect(filterBoard(board, DEFAULT_FILTER_STATE)).toEqual(board);
    expect(hasActiveFilters(DEFAULT_FILTER_STATE)).toBe(false);
  });

  test("search matches summary and id, case-insensitively", () => {
    expect(filterBoard(board, withFilter({ search: "AUDIT" })).queue.map((c) => c.id)).toEqual([
      "FEAT-1",
    ]);
    expect(filterBoard(board, withFilter({ search: "task-2" })).queue.map((c) => c.id)).toEqual([
      "TASK-2",
    ]);
  });

  test("search requires every term to match", () => {
    expect(filterBoard(board, withFilter({ search: "rotate key" })).total_count).toBe(1);
    expect(filterBoard(board, withFilter({ search: "rotate nonsense" })).total_count).toBe(0);
  });

  test("filters by project", () => {
    const filtered = filterBoard(board, withFilter({ projects: ["gamesvc"] }));
    expect(filtered.queue).toEqual([]);
    expect(filtered.running.map((c) => c.id)).toEqual(["FIX-3"]);
    expect(filtered.awaiting.map((c) => c.id)).toEqual(["INV-4"]);
  });

  test("filters by type, OR within the facet", () => {
    const filtered = filterBoard(board, withFilter({ types: ["FEAT", "FIX"] }));
    expect(filtered.total_count).toBe(2);
  });

  test("filters by priority", () => {
    const filtered = filterBoard(board, withFilter({ priorities: ["P0-critical"] }));
    expect(filtered.queue.map((c) => c.id)).toEqual(["TASK-2"]);
  });

  test("ANDs across facets", () => {
    expect(
      filterBoard(board, withFilter({ projects: ["gamesvc"], types: ["FIX"] })).total_count,
    ).toBe(1);
    expect(
      filterBoard(board, withFilter({ projects: ["operator"], types: ["FIX"] })).total_count,
    ).toBe(0);
  });

  test("recomputes total_count from what is shown", () => {
    expect(filterBoard(board, withFilter({ projects: ["gamesvc"] })).total_count).toBe(2);
  });

  test("filtering everything out is distinguishable from an empty queue", () => {
    const filtered = filterBoard(board, withFilter({ search: "nothing matches this" }));
    expect(filtered.total_count).toBe(0);
    expect(hasActiveFilters(withFilter({ search: "nothing matches this" }))).toBe(true);
  });

  test("keeps last_updated so the meta line stays honest", () => {
    expect(filterBoard(board, withFilter({ types: ["FEAT"] })).last_updated).toBe(
      board.last_updated,
    );
  });
});

describe("facetsFromBoard", () => {
  test("derives deduped, sorted options from the board itself", () => {
    const facets = facetsFromBoard(board);
    expect(facets.projects).toEqual(["gamesvc", "operator"]);
    expect(facets.types).toEqual(["FEAT", "FIX", "INV", "SPIKE", "TASK"]);
  });

  test("offers priorities most urgent first, only those present", () => {
    expect(facetsFromBoard(board).priorities).toEqual(["P0-critical", "P2-medium", "P3-low"]);
  });
});

describe("hasActiveFilters", () => {
  test("whitespace-only search is not a filter", () => {
    expect(hasActiveFilters(withFilter({ search: "   " }))).toBe(false);
  });

  test("any non-empty facet counts", () => {
    expect(hasActiveFilters(withFilter({ types: ["FEAT"] }))).toBe(true);
  });
});
