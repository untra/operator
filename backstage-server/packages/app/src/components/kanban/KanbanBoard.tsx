/**
 * Kanban Board Container
 *
 * Responsive grid layout with four columns.
 * Uses CSS Grid for consistent column widths.
 */

import React from 'react';
import { Progress } from '@backstage/core-components';
import { Flex, Text } from '@backstage/ui';
import { KanbanColumn } from './KanbanColumn';
import { useKanbanBoard } from './useKanbanBoard';
import { KANBAN_COLUMNS } from './types';
import { ErrorState } from '../common/ErrorState';

export function KanbanBoard() {
  const { data, loading, error, lastUpdated, refresh } = useKanbanBoard();

  if (loading && !data) {
    return <Progress />;
  }

  if (error && !data) {
    return (
      <ErrorState
        title="Failed to load board"
        message={error.message || 'Unable to load kanban board'}
        onRetry={refresh}
        compact
      />
    );
  }

  return (
    <Flex direction="column" gap="3">
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
          gap: '16px',
          width: '100%',
        }}
      >
        {KANBAN_COLUMNS.map(column => (
          <KanbanColumn
            key={column.key}
            config={column}
            tickets={data?.[column.key] || []}
          />
        ))}
      </div>
      {lastUpdated && (
        <Flex justify="end">
          <Text variant="body-small" color="secondary">
            Last updated: {lastUpdated.toLocaleTimeString()}
          </Text>
        </Flex>
      )}
    </Flex>
  );
}
