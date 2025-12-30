/**
 * Active Agents Widget
 *
 * Displays currently running Claude Code agents with status.
 * Shows agent name, project, elapsed time, and mode.
 */

import React from 'react';
import { Card, CardBody, Flex, Text, Box, Link } from '@backstage/ui';
import { Progress } from '@backstage/core-components';
import {
  RiRobot2Line,
  RiTimeLine,
  RiUserLine,
  RiCodeLine,
  RiPlayCircleLine,
  RiPauseCircleLine,
} from '@remixicon/react';
import { useActiveAgentsQuery, type Agent } from '../../../api/queries';
import { ErrorState } from '../../common/ErrorState';

function formatElapsed(startedAt: string): string {
  const start = new Date(startedAt).getTime();
  const now = Date.now();
  const elapsed = Math.floor((now - start) / 1000);

  if (elapsed < 60) return `${elapsed}s`;
  if (elapsed < 3600) return `${Math.floor(elapsed / 60)}m`;
  const hours = Math.floor(elapsed / 3600);
  const mins = Math.floor((elapsed % 3600) / 60);
  return `${hours}h ${mins}m`;
}

interface AgentRowProps {
  agent: Agent;
}

function AgentRow({ agent }: AgentRowProps) {
  const statusColors: Record<string, string> = {
    running: '#66AA99',
    awaiting_input: '#E9A820',
    completing: '#6688AA',
  };

  const modeLabels: Record<string, string> = {
    autonomous: 'Auto',
    paired: 'Paired',
  };

  const statusColor = statusColors[agent.status] || '#6688AA';
  const StatusIcon = agent.status === 'running' ? RiPlayCircleLine : RiPauseCircleLine;

  return (
    <Flex
      direction="column"
      gap="1"
      p="2"
      style={{
        borderRadius: 4,
        backgroundColor: 'var(--bui-color-surface-1)',
      }}
    >
      <Flex align="center" justify="between">
        <Flex align="center" gap="2">
          <StatusIcon size={16} color={statusColor} />
          <Text variant="body-medium" style={{ fontWeight: 500 }}>
            {agent.ticket_id}
          </Text>
        </Flex>
        <Box
          style={{
            backgroundColor: `${statusColor}20`,
            color: statusColor,
            fontSize: '0.75rem',
            padding: '2px 8px',
            borderRadius: 4,
          }}
        >
          {modeLabels[agent.mode] || agent.mode}
        </Box>
      </Flex>

      <Flex align="center" gap="3" style={{ marginLeft: 24 }}>
        <Flex align="center" gap="1">
          <RiCodeLine size={14} color="var(--bui-color-text-secondary)" />
          <Text variant="body-small" color="secondary">
            {agent.project}
          </Text>
        </Flex>
        <Flex align="center" gap="1">
          <RiTimeLine size={14} color="var(--bui-color-text-secondary)" />
          <Text variant="body-small" color="secondary">
            {formatElapsed(agent.started_at)}
          </Text>
        </Flex>
        {agent.current_step && (
          <Text variant="body-small" color="secondary">
            Step: {agent.current_step}
          </Text>
        )}
      </Flex>

      <Flex align="center" gap="1" style={{ marginLeft: 24 }}>
        <Link href={`/issuetypes/${agent.ticket_type}`}>
          <Text variant="body-small" style={{ color: 'var(--bui-color-primary)' }}>
            {agent.ticket_type}
          </Text>
        </Link>
      </Flex>
    </Flex>
  );
}

export function ActiveAgentsCard() {
  const { data, isLoading, error, refetch } = useActiveAgentsQuery();

  if (isLoading) {
    return (
      <Card>
        <CardBody>
          <Flex direction="column" gap="3" p="2">
            <Text variant="title-small">Active Agents</Text>
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
              <Text variant="title-small">Active Agents</Text>
              <RiRobot2Line size={20} color="var(--bui-color-text-secondary)" />
            </Flex>
            <ErrorState
              title="Failed to load"
              message="Unable to load active agents"
              onRetry={() => refetch()}
              compact
            />
          </Flex>
        </CardBody>
      </Card>
    );
  }

  const agents = data?.agents || [];

  return (
    <Card>
      <CardBody>
        <Flex direction="column" gap="3" p="2">
          <Flex align="center" justify="between">
            <Flex align="center" gap="2">
              <Text variant="title-small">Active Agents</Text>
              {agents.length > 0 && (
                <Box
                  style={{
                    backgroundColor: '#66AA9920',
                    color: '#66AA99',
                    minWidth: 20,
                    textAlign: 'center',
                    padding: '2px 6px',
                    borderRadius: 4,
                    fontSize: '0.75rem',
                  }}
                >
                  {agents.length}
                </Box>
              )}
            </Flex>
            <RiRobot2Line size={20} color="var(--bui-color-text-secondary)" />
          </Flex>

          {agents.length === 0 ? (
            <Flex
              direction="column"
              align="center"
              justify="center"
              gap="2"
              p="4"
              style={{ opacity: 0.7 }}
            >
              <RiUserLine size={32} color="var(--bui-color-text-secondary)" />
              <Text variant="body-small" color="secondary">
                No agents running
              </Text>
            </Flex>
          ) : (
            <Flex direction="column" gap="2">
              {agents.map(agent => (
                <AgentRow key={agent.id} agent={agent} />
              ))}
            </Flex>
          )}
        </Flex>
      </CardBody>
    </Card>
  );
}
