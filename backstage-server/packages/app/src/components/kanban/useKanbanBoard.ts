/**
 * Hook for fetching kanban board data with auto-refresh.
 *
 * Uses TanStack Query for server state management.
 * Returns loading, error, and data states for proper UI feedback.
 */

import { useKanbanBoardQuery } from '../../api/queries';
import type { KanbanBoardResponse } from './types';

interface UseKanbanBoardResult {
  data: KanbanBoardResponse | undefined;
  loading: boolean;
  error: Error | null;
  lastUpdated: Date | null;
  refresh: () => void;
}

export function useKanbanBoard(): UseKanbanBoardResult {
  const { data, isLoading, error, dataUpdatedAt, refetch } = useKanbanBoardQuery();

  return {
    data,
    loading: isLoading,
    error: error as Error | null,
    lastUpdated: dataUpdatedAt ? new Date(dataUpdatedAt) : null,
    refresh: () => refetch(),
  };
}
