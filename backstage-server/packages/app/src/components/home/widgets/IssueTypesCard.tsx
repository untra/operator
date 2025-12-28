/**
 * Issue Types Widget
 *
 * Quick access to issue types management.
 * Shows recent/pinned types with create action.
 */

import { useState, useEffect } from 'react';
import { Card, CardBody, Flex, Text, Link, Button } from '@backstage/ui';
import { Progress } from '@backstage/core-components';
import {
  RiFileList3Line,
  RiAddLine,
  RiFolderLine,
  RiArrowRightLine,
} from '@remixicon/react';

interface IssueType {
  key: string;
  name: string;
  mode: string;
  collection?: string;
}

interface IssueTypeChipProps {
  issueType: IssueType;
}

function IssueTypeChip({ issueType }: IssueTypeChipProps) {
  const modeColors: Record<string, string> = {
    autonomous: '#66AA99',
    paired: '#E9A820',
    investigation: '#E05D44',
  };

  const color = modeColors[issueType.mode] || '#6688AA';

  return (
    <Link
      href={`/issuetypes/${issueType.key}`}
      style={{ textDecoration: 'none' }}
    >
      <Flex
        align="center"
        gap="2"
        p="2"
        style={{
          borderRadius: 4,
          backgroundColor: 'var(--bui-color-surface-1)',
          border: `1px solid ${color}40`,
          cursor: 'pointer',
          transition: 'all 0.15s ease',
        }}
        className="issue-type-chip"
      >
        <div
          style={{
            width: 8,
            height: 8,
            borderRadius: 2,
            backgroundColor: color,
          }}
        />
        <Text variant="body-small" style={{ fontWeight: 500 }}>
          {issueType.key}
        </Text>
        <Text variant="body-small" color="secondary">
          {issueType.name}
        </Text>
      </Flex>
    </Link>
  );
}

export function IssueTypesCard() {
  const [issueTypes, setIssueTypes] = useState<IssueType[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchIssueTypes() {
      try {
        const response = await fetch('/api/proxy/operator/issuetypes?limit=5');
        if (!response.ok) {
          throw new Error('Failed to fetch issue types');
        }
        const data = await response.json();
        setIssueTypes(data);
      } catch (err) {
        // Use mock data if API unavailable
        setIssueTypes([
          { key: 'FEAT', name: 'Feature', mode: 'autonomous' },
          { key: 'FIX', name: 'Bug Fix', mode: 'autonomous' },
          { key: 'SPIKE', name: 'Research Spike', mode: 'paired' },
          { key: 'INV', name: 'Investigation', mode: 'investigation' },
        ]);
      } finally {
        setLoading(false);
      }
    }

    fetchIssueTypes();
  }, []);

  if (loading) {
    return (
      <Card>
        <CardBody>
          <Flex direction="column" gap="3" p="2">
            <Text variant="title-small">Issue Types</Text>
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
            <Text variant="title-small">Issue Types</Text>
            <RiFileList3Line size={20} color="var(--bui-color-text-secondary)" />
          </Flex>

          {/* Issue type chips */}
          <Flex direction="column" gap="2">
            {issueTypes.map(issueType => (
              <IssueTypeChip key={issueType.key} issueType={issueType} />
            ))}
          </Flex>

          {/* Actions */}
          <Flex gap="2" style={{ marginTop: 4 }}>
            <Link href="/issuetypes/new" style={{ flex: 1 }}>
              <Button
                variant="secondary"
                size="small"
                style={{ width: '100%', justifyContent: 'center' }}
              >
                <Flex align="center" gap="1">
                  <RiAddLine size={16} />
                  <span>New Type</span>
                </Flex>
              </Button>
            </Link>
            <Link href="/issuetypes/collections" style={{ flex: 1 }}>
              <Button
                variant="secondary"
                size="small"
                style={{ width: '100%', justifyContent: 'center' }}
              >
                <Flex align="center" gap="1">
                  <RiFolderLine size={16} />
                  <span>Collections</span>
                </Flex>
              </Button>
            </Link>
          </Flex>

          {/* View all link */}
          <Link href="/issuetypes">
            <Flex
              align="center"
              justify="center"
              gap="1"
              p="2"
              style={{
                borderRadius: 4,
                backgroundColor: 'var(--bui-color-surface-1)',
              }}
            >
              <Text variant="body-small" style={{ color: 'var(--bui-color-primary)' }}>
                View All Issue Types
              </Text>
              <RiArrowRightLine size={14} color="var(--bui-color-primary)" />
            </Flex>
          </Link>
        </Flex>
      </CardBody>
    </Card>
  );
}

export default IssueTypesCard;
