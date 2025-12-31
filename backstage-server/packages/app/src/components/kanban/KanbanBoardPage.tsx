/**
 * Kanban Board Page
 *
 * Full-page view displaying tickets organized by status columns.
 * Read-only view with auto-refresh.
 */

import React from 'react';
import { Page, Header, Content } from '@backstage/core-components';
import { Flex, Text } from '@backstage/ui';
import { RiLayoutColumnLine } from '@remixicon/react';
import { KanbanBoard } from './KanbanBoard';

export function KanbanBoardPage() {
  return (
    <Page themeId="tool">
      <Header
        title="Ticket Board"
        subtitle="View all tickets organized by status"
      />
      <Content>
        <Flex direction="column" gap="4" p="4">
          <Flex align="center" gap="2">
            <RiLayoutColumnLine size={24} />
            <Text variant="title-medium">Kanban Board</Text>
          </Flex>
          <KanbanBoard />
        </Flex>
      </Content>
    </Page>
  );
}
