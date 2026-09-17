/**
 * The board renders a glyph and a CSS key per card. Both maps are keyed by a
 * generated union, so what is worth asserting is that every variant is covered
 * and nothing falls through to a default.
 */

import { describe, expect, test } from "bun:test";

import { PRIORITY_KEY, PRIORITY_ORDER, STATUS_GLYPH } from "./ticket-fields";

const STATUSES = ["queued", "running", "awaiting", "completed"];
const PRIORITIES = ["P0-critical", "P1-high", "P2-medium", "P3-low"] as const;

describe("STATUS_GLYPH", () => {
  test("covers every status exactly once", () => {
    expect(Object.keys(STATUS_GLYPH).toSorted()).toEqual(STATUSES.toSorted());
  });

  test("gives each status a distinct glyph", () => {
    const glyphs = Object.values(STATUS_GLYPH);
    expect(new Set(glyphs).size).toBe(glyphs.length);
  });
});

describe("PRIORITY_KEY", () => {
  test("covers every priority exactly once", () => {
    expect(Object.keys(PRIORITY_KEY).toSorted()).toEqual(PRIORITIES.toSorted() as string[]);
  });

  test("maps to the p0-p3 keys the stylesheet selects on", () => {
    expect(Object.values(PRIORITY_KEY)).toEqual(["p0", "p1", "p2", "p3"]);
  });
});

describe("PRIORITY_ORDER", () => {
  test("is most urgent first", () => {
    expect(PRIORITY_ORDER).toEqual([...PRIORITIES]);
  });
});
