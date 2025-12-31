/**
 * Kanban Board TypeScript Types
 *
 * These types mirror the Rust DTOs from the operator backend.
 */

/** Ticket type keys */
export type TicketType = 'INV' | 'FIX' | 'FEAT' | 'SPIKE';

/** Ticket status values mapped to kanban columns */
export type TicketStatus = 'queued' | 'running' | 'awaiting' | 'completed' | 'done';

/** Priority levels */
export type Priority = 'P0-critical' | 'P1-high' | 'P2-medium' | 'P3-low';

/** A single ticket card on the kanban board */
export interface KanbanTicketCard {
  id: string;
  summary: string;
  ticket_type: string;
  project: string;
  status: string;
  step: string;
  step_display_name?: string;
  priority: string;
  timestamp: string;
}

/** Kanban board response grouped by column */
export interface KanbanBoardResponse {
  queue: KanbanTicketCard[];
  running: KanbanTicketCard[];
  awaiting: KanbanTicketCard[];
  done: KanbanTicketCard[];
  total_count: number;
  last_updated: string;
}

/** Column configuration for rendering */
export interface KanbanColumnConfig {
  key: 'queue' | 'running' | 'awaiting' | 'done';
  title: string;
  color: string;
}

/** Type badge configuration */
export const TYPE_BADGE_CONFIG: Record<string, { label: string; color: string }> = {
  INV: { label: 'Investigation', color: '#E05D44' },
  FIX: { label: 'Bug Fix', color: '#E9A820' },
  FEAT: { label: 'Feature', color: '#66AA99' },
  SPIKE: { label: 'Research', color: '#6688AA' },
};

/** Priority color configuration */
export const PRIORITY_COLORS: Record<string, string> = {
  'P0-critical': '#E05D44',
  'P1-high': '#E9A820',
  'P2-medium': '#6688AA',
  'P3-low': '#888888',
};

/** Column definitions */
export const KANBAN_COLUMNS: KanbanColumnConfig[] = [
  { key: 'queue', title: 'Queue', color: '#6688AA' },
  { key: 'running', title: 'Running', color: '#66AA99' },
  { key: 'awaiting', title: 'Awaiting', color: '#E9A820' },
  { key: 'done', title: 'Done', color: '#888888' },
];
