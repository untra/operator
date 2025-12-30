/**
 * Kanban Column Component
 *
 * Single column with header and scrollable ticket list.
 */

import React from 'react';
import { Card, CardBody, Flex, Text, Box } from '@backstage/ui';
import {
  RiTimeLine,
  RiPlayCircleLine,
  RiPauseCircleLine,
  RiCheckDoubleLine,
} from '@remixicon/react';
import { KanbanCard } from './KanbanCard';
import type { KanbanColumnConfig, KanbanTicketCard } from './types';

const iconMap = {
  queue: RiTimeLine,
  running: RiPlayCircleLine,
  awaiting: RiPauseCircleLine,
  done: RiCheckDoubleLine,
};

interface KanbanColumnProps {
  config: KanbanColumnConfig;
  tickets: KanbanTicketCard[];
}

export function KanbanColumn({ config, tickets }: KanbanColumnProps) {
  const Icon = iconMap[config.key];

  return (
    <Card>
      <CardBody>
        <Flex direction="column" gap="3">
          {/* Column Header */}
          <Flex align="center" justify="between" p="2">
            <Flex align="center" gap="2">
              <Box
                style={{
                  width: 4,
                  height: 24,
                  backgroundColor: config.color,
                  borderRadius: 2,
                }}
              />
              <Icon size={18} color={config.color} />
              <Text variant="title-small">{config.title}</Text>
            </Flex>
            <Box
              style={{
                backgroundColor: `${config.color}20`,
                color: config.color,
                minWidth: 24,
                textAlign: 'center',
                padding: '2px 8px',
                borderRadius: 12,
                fontSize: '0.75rem',
                fontWeight: 600,
              }}
            >
              {tickets.length}
            </Box>
          </Flex>

          {/* Ticket List */}
          <Flex
            direction="column"
            gap="2"
            style={{
              maxHeight: 'calc(100vh - 300px)',
              overflowY: 'auto',
            }}
          >
            {tickets.length === 0 ? (
              <Flex
                align="center"
                justify="center"
                p="4"
                style={{ opacity: 0.5 }}
              >
                <Text variant="body-small" color="secondary">
                  No tickets
                </Text>
              </Flex>
            ) : (
              tickets.map(ticket => (
                <KanbanCard key={ticket.id} ticket={ticket} />
              ))
            )}
          </Flex>
        </Flex>
      </CardBody>
    </Card>
  );
}
