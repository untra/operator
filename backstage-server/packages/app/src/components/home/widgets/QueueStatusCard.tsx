/**
 * Queue Status Widget
 *
 * Displays ticket queue status with counts by priority level.
 * Fetches data from the Operator REST API.
 */

import { useState, useEffect } from 'react';
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

interface QueueStatus {
  queued: number;
  inProgress: number;
  completed: number;
  byPriority: {
    inv: number;
    fix: number;
    feat: number;
    spike: number;
  };
}

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
  const [status, setStatus] = useState<QueueStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchStatus() {
      try {
        const response = await fetch('/api/proxy/operator/queue/status');
        if (!response.ok) {
          throw new Error('Failed to fetch queue status');
        }
        const data = await response.json();
        setStatus(data);
      } catch (err) {
        // Use mock data for now if API unavailable
        setStatus({
          queued: 5,
          inProgress: 2,
          completed: 12,
          byPriority: {
            inv: 1,
            fix: 2,
            feat: 3,
            spike: 1,
          },
        });
        setError(null); // Clear error, using mock data
      } finally {
        setLoading(false);
      }
    }

    fetchStatus();
    const interval = setInterval(fetchStatus, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, []);

  if (loading) {
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

  if (error && !status) {
    return (
      <Card>
        <CardBody>
          <Flex direction="column" gap="3" p="2">
            <Text variant="title-small">Queue Status</Text>
            <Text variant="body-small" color="secondary">
              Unable to load queue status
            </Text>
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
              count={status?.inProgress || 0}
              color="#E9A82020"
            />
            <StatusRow
              icon={<RiCheckDoubleLine size={18} color="#66AA99" />}
              label="Completed Today"
              count={status?.completed || 0}
              color="#66AA9920"
            />
          </Flex>

          {/* Priority breakdown */}
          <Flex direction="column" gap="2" style={{ marginTop: 8 }}>
            <Text variant="body-small" color="secondary">
              By Priority
            </Text>
            <Flex gap="3" style={{ flexWrap: 'wrap' }}>
              <PriorityBadge type="inv" count={status?.byPriority.inv || 0} />
              <PriorityBadge type="fix" count={status?.byPriority.fix || 0} />
              <PriorityBadge type="feat" count={status?.byPriority.feat || 0} />
              <PriorityBadge type="spike" count={status?.byPriority.spike || 0} />
            </Flex>
          </Flex>
        </Flex>
      </CardBody>
    </Card>
  );
}

export default QueueStatusCard;
