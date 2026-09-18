/**
 * Presentation for the two closed ticket fields.
 *
 * Both maps are keyed by a generated union, so a variant added in Rust is a
 * TypeScript error here rather than a silent fallback on the board.
 */

import type { TicketPriority } from "../generated/TicketPriority";
import type { TicketStatus } from "../generated/TicketStatus";

export const STATUS_GLYPH: Record<TicketStatus, string> = {
  queued: "•",
  running: "▶",
  awaiting: "⏸",
  completed: "✓",
};

/** Feeds the `data-priority` hook the board stylesheet selects on. */
export const PRIORITY_KEY: Record<TicketPriority, string> = {
  "P0-critical": "p0",
  "P1-high": "p1",
  "P2-medium": "p2",
  "P3-low": "p3",
};

/** Most urgent first, matching the Rust enum's declaration order. */
export const PRIORITY_ORDER = Object.keys(PRIORITY_KEY) as TicketPriority[];
