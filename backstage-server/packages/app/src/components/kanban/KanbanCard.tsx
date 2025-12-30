/**
 * Kanban Card Component
 *
 * Individual ticket card with all details.
 */

import React from 'react';
import { Flex, Text, Box } from '@backstage/ui';
import { RiCodeLine, RiArrowRightSLine } from '@remixicon/react';
import { TypeBadge } from './TypeBadge';
import { KanbanTicketCard, PRIORITY_COLORS } from './types';

interface KanbanCardProps {
  ticket: KanbanTicketCard;
}

export function KanbanCard({ ticket }: KanbanCardProps) {
  const priorityColor = PRIORITY_COLORS[ticket.priority] || '#888888';

  return (
    <Box
      style={{
        backgroundColor: 'var(--bui-color-surface-1)',
        borderRadius: 8,
        padding: 12,
        borderLeft: `3px solid ${priorityColor}`,
      }}
    >
      <Flex direction="column" gap="2">
        {/* Header: ID + Type Badge */}
        <Flex align="center" justify="between">
          <Text
            variant="body-medium"
            style={{ fontWeight: 600, fontFamily: 'monospace' }}
          >
            {ticket.id}
          </Text>
          <TypeBadge type={ticket.ticket_type} />
        </Flex>

        {/* Summary */}
        <Text
          variant="body-small"
          style={{
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            display: '-webkit-box',
            WebkitLineClamp: 2,
            WebkitBoxOrient: 'vertical',
          }}
        >
          {ticket.summary}
        </Text>

        {/* Project + Step */}
        <Flex align="center" gap="2">
          <Flex align="center" gap="1">
            <RiCodeLine size={14} color="var(--bui-color-text-secondary)" />
            <Text variant="body-small" color="secondary">
              {ticket.project}
            </Text>
          </Flex>
          {ticket.step_display_name && (
            <>
              <RiArrowRightSLine
                size={14}
                color="var(--bui-color-text-secondary)"
              />
              <Text variant="body-small" color="secondary">
                {ticket.step_display_name}
              </Text>
            </>
          )}
        </Flex>

        {/* Priority */}
        <Flex align="center" gap="1">
          <Box
            style={{
              width: 8,
              height: 8,
              borderRadius: 4,
              backgroundColor: priorityColor,
            }}
          />
          <Text variant="body-small" color="secondary">
            {ticket.priority.split('-')[1] || ticket.priority}
          </Text>
        </Flex>
      </Flex>
    </Box>
  );
}
