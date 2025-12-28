/**
 * Active Agents Widget
 *
 * Displays currently running Claude Code agents with status.
 * Shows agent name, project, elapsed time, and mode.
 */

import { useState, useEffect } from 'react';
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

interface Agent {
  id: string;
  name: string;
  project: string;
  ticketKey: string;
  mode: 'autonomous' | 'paired' | 'awaiting-input';
  startedAt: string;
  status: 'running' | 'paused' | 'waiting';
}

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
  const statusColors = {
    running: '#66AA99',
    paused: '#E9A820',
    waiting: '#6688AA',
  };

  const modeLabels = {
    autonomous: 'Auto',
    paired: 'Paired',
    'awaiting-input': 'Waiting',
  };

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
          <StatusIcon size={16} color={statusColors[agent.status]} />
          <Text variant="body-medium" style={{ fontWeight: 500 }}>
            {agent.name}
          </Text>
        </Flex>
        <Box
          style={{
            backgroundColor: `${statusColors[agent.status]}20`,
            color: statusColors[agent.status],
            fontSize: '0.75rem',
            padding: '2px 8px',
            borderRadius: 4,
          }}
        >
          {modeLabels[agent.mode]}
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
            {formatElapsed(agent.startedAt)}
          </Text>
        </Flex>
      </Flex>

      {agent.ticketKey && (
        <Flex align="center" gap="1" style={{ marginLeft: 24 }}>
          <Link href={`/issuetypes/${agent.ticketKey.split('-')[0]}`}>
            <Text variant="body-small" style={{ color: 'var(--bui-color-primary)' }}>
              {agent.ticketKey}
            </Text>
          </Link>
        </Flex>
      )}
    </Flex>
  );
}

export function ActiveAgentsCard() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchAgents() {
      try {
        const response = await fetch('/api/proxy/operator/agents/active');
        if (!response.ok) {
          throw new Error('Failed to fetch agents');
        }
        const data = await response.json();
        setAgents(data);
      } catch {
        // Use mock data for now if API unavailable
        setAgents([
          {
            id: '1',
            name: 'Agent Alpha',
            project: 'gamesvc',
            ticketKey: 'FEAT-042',
            mode: 'autonomous',
            startedAt: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
            status: 'running',
          },
          {
            id: '2',
            name: 'Agent Beta',
            project: 'operator',
            ticketKey: 'FIX-017',
            mode: 'paired',
            startedAt: new Date(Date.now() - 45 * 60 * 1000).toISOString(),
            status: 'waiting',
          },
        ]);
      } finally {
        setLoading(false);
      }
    }

    fetchAgents();
    const interval = setInterval(fetchAgents, 10000); // Refresh every 10s
    return () => clearInterval(interval);
  }, []);

  if (loading) {
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
