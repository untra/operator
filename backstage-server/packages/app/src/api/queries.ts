/**
 * API Query Functions
 *
 * Centralized fetch functions for use with TanStack Query.
 * Each function handles its own error throwing for proper error propagation.
 */

import { useQuery } from '@tanstack/react-query';
import type { KanbanBoardResponse } from '../components/kanban/types';

// ============================================================================
// Types
// ============================================================================

export interface Agent {
  id: string;
  ticket_id: string;
  ticket_type: string;
  project: string;
  status: string;
  mode: string;
  started_at: string;
  current_step: string | null;
}

export interface AgentsResponse {
  agents: Agent[];
  count: number;
}

export interface QueueStatus {
  queued: number;
  in_progress: number;
  awaiting: number;
  completed: number;
  by_type: {
    inv: number;
    fix: number;
    feat: number;
    spike: number;
  };
}

export interface IssueType {
  key: string;
  name: string;
  mode: string;
  collection?: string;
}

// ============================================================================
// Query Keys
// ============================================================================

export const queryKeys = {
  kanbanBoard: ['kanban-board'] as const,
  activeAgents: ['active-agents'] as const,
  queueStatus: ['queue-status'] as const,
  issueTypes: ['issue-types'] as const,
};

// ============================================================================
// Fetch Functions
// ============================================================================

async function fetchKanbanBoard(): Promise<KanbanBoardResponse> {
  const response = await fetch('/api/proxy/operator/api/v1/queue/kanban');
  if (!response.ok) {
    throw new Error(`Failed to fetch kanban board: ${response.status}`);
  }
  return response.json();
}

async function fetchActiveAgents(): Promise<AgentsResponse> {
  const response = await fetch('/api/proxy/operator/api/v1/agents/active');
  if (!response.ok) {
    throw new Error(`Failed to fetch active agents: ${response.status}`);
  }
  return response.json();
}

async function fetchQueueStatus(): Promise<QueueStatus> {
  const response = await fetch('/api/proxy/operator/api/v1/queue/status');
  if (!response.ok) {
    throw new Error(`Failed to fetch queue status: ${response.status}`);
  }
  return response.json();
}

async function fetchIssueTypes(): Promise<IssueType[]> {
  const response = await fetch('/api/proxy/operator/api/v1/issuetypes');
  if (!response.ok) {
    throw new Error(`Failed to fetch issue types: ${response.status}`);
  }
  return response.json();
}

// ============================================================================
// Query Hooks
// ============================================================================

export function useKanbanBoardQuery() {
  return useQuery({
    queryKey: queryKeys.kanbanBoard,
    queryFn: fetchKanbanBoard,
    refetchInterval: 15000, // 15 seconds
  });
}

export function useActiveAgentsQuery() {
  return useQuery({
    queryKey: queryKeys.activeAgents,
    queryFn: fetchActiveAgents,
    refetchInterval: 10000, // 10 seconds
  });
}

export function useQueueStatusQuery() {
  return useQuery({
    queryKey: queryKeys.queueStatus,
    queryFn: fetchQueueStatus,
    refetchInterval: 30000, // 30 seconds
  });
}

export function useIssueTypesQuery() {
  return useQuery({
    queryKey: queryKeys.issueTypes,
    queryFn: fetchIssueTypes,
  });
}
