/**
 * ErrorState Component
 *
 * Reusable error state display for async components.
 * Shows error icon, message, and optional retry button.
 */

import React from 'react';
import { Card, CardBody, Flex, Text, Button } from '@backstage/ui';
import { RiErrorWarningLine, RiRefreshLine } from '@remixicon/react';

interface ErrorStateProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
  compact?: boolean;
}

export function ErrorState({
  title = 'Error',
  message = 'Unable to load data',
  onRetry,
  compact = false,
}: ErrorStateProps) {
  const content = (
    <div role="alert">
      <Flex
        direction="column"
        align="center"
        justify="center"
        gap="3"
        p={compact ? '3' : '4'}
      >
      <RiErrorWarningLine
        size={compact ? 24 : 32}
        color="var(--bui-color-error, #E05D44)"
      />
      <Flex direction="column" align="center" gap="1">
        <Text
          variant={compact ? 'body-medium' : 'title-small'}
          style={{ color: 'var(--bui-color-error, #E05D44)' }}
        >
          {title}
        </Text>
        <Text variant="body-small" color="secondary">
          {message}
        </Text>
      </Flex>
      {onRetry && (
        <Button
          variant="secondary"
          size="small"
          onClick={onRetry}
          aria-label="Retry loading"
        >
          <Flex align="center" gap="1">
            <RiRefreshLine size={16} />
            <span>Retry</span>
          </Flex>
        </Button>
      )}
      </Flex>
    </div>
  );

  if (compact) {
    return content;
  }

  return (
    <Card>
      <CardBody>{content}</CardBody>
    </Card>
  );
}
