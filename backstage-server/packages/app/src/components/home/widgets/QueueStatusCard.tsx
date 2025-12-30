/**
 * Queue Status Widget
 *
 * Displays ticket queue status with counts by priority level.
 * Fetches data from the Operator REST API.
 */

import React from 'react';
import { Card, CardBody, Flex, Text, Box } from '@backstage/ui';
import { Progress } from '@backstage/core-components';
import {
  RiListCheck2,
  RiTimeLine,
  RiCheckDoubleLine,
  RiAlertLine,
  RiBugLine,
  RiLightbulbLine,
  RiSearchLine,
} from '@remixicon/react';
import { useQueueStatusQuery } from '../../../api/queries';
import { ErrorState } from '../../common/ErrorState';

const priorityConfig = {
  inv: { label: 'Investigation', icon: RiAlertLine, color: '#E05D44' },
  fix: { label: 'Bug Fix', icon: RiBugLine, color: '#E9A820' },
  feat: { label: 'Feature', icon: RiLightbulbLine, color: '#66AA99' },
  spike: { label: 'Spike', icon: RiSearchLine, color: '#6688AA' },
} as const;

interface PriorityBadgeProps {
  type: keyof typeof priorityConfig;
  count: number;
}

function PriorityBadge({ type, count }: PriorityBadgeProps) {
  const config = priorityConfig[type];
  const Icon = config.icon;

  return (
    <Flex align="center" gap="2" style={{ minWidth: 80 }}>
      <Icon size={16} color={config.color} />
      <Text variant="body-small" style={{ color: config.color, fontWeight: 600 }}>
        {count}
      </Text>
      <Text variant="body-small" color="secondary">
        {config.label}
      </Text>
    </Flex>
  );
}

interface StatusRowProps {
  icon: React.ReactNode;
  label: string;
  count: number;
  color?: string;
}

function StatusRow({ icon, label, count, color }: StatusRowProps) {
  return (
    <Flex align="center" justify="between" p="2">
      <Flex align="center" gap="2">
        {icon}
        <Text variant="body-medium">{label}</Text>
      </Flex>
      <Box
        style={{
          backgroundColor: color || 'var(--bui-color-surface-2)',
          minWidth: 32,
          textAlign: 'center',
          padding: '2px 8px',
          borderRadius: 4,
          fontSize: '0.875rem',
          fontWeight: 500,
        }}
      >
        {count}
      </Box>
    </Flex>
  );
}

export function QueueStatusCard() {
  const { data: status, isLoading, error, refetch } = useQueueStatusQuery();

  if (isLoading) {
    return (
      <Card>
        <CardBody>
          <Flex direction="column" gap="3" p="2">
            <Text variant="title-small">Queue Status</Text>
            <Progress />
          </Flex>
        </CardBody>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardBody>
          <Flex direction="column" gap="3" p="2">
            <Flex align="center" justify="between">
              <Text variant="title-small">Queue Status</Text>
              <RiListCheck2 size={20} color="var(--bui-color-text-secondary)" />
            </Flex>
            <ErrorState
              title="Failed to load"
              message="Unable to load queue status"
              onRetry={() => refetch()}
              compact
            />
          </Flex>
        </CardBody>
      </Card>
    );
  }

  return (
    <Card>
      <CardBody>
        <Flex direction="column" gap="3" p="2">
          <Flex align="center" justify="between">
            <Text variant="title-small">Queue Status</Text>
            <RiListCheck2 size={20} color="var(--bui-color-text-secondary)" />
          </Flex>

          {/* Status counts */}
          <Flex direction="column" gap="1">
            <StatusRow
              icon={<RiTimeLine size={18} color="#6688AA" />}
              label="Queued"
              count={status?.queued || 0}
            />
            <StatusRow
              icon={<RiTimeLine size={18} color="#E9A820" />}
              label="In Progress"
              count={status?.in_progress || 0}
              color="#E9A82020"
            />
            <StatusRow
              icon={<RiTimeLine size={18} color="#9966AA" />}
              label="Awaiting"
              count={status?.awaiting || 0}
              color="#9966AA20"
            />
            <StatusRow
              icon={<RiCheckDoubleLine size={18} color="#66AA99" />}
              label="Completed"
              count={status?.completed || 0}
              color="#66AA9920"
            />
          </Flex>

          {/* Type breakdown */}
          <Flex direction="column" gap="2" style={{ marginTop: 8 }}>
            <Text variant="body-small" color="secondary">
              By Type
            </Text>
            <Flex gap="3" style={{ flexWrap: 'wrap' }}>
              <PriorityBadge type="inv" count={status?.by_type.inv || 0} />
              <PriorityBadge type="fix" count={status?.by_type.fix || 0} />
              <PriorityBadge type="feat" count={status?.by_type.feat || 0} />
              <PriorityBadge type="spike" count={status?.by_type.spike || 0} />
            </Flex>
          </Flex>
        </Flex>
      </CardBody>
    </Card>
  );
}
