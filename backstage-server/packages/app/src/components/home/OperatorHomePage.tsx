/**
 * Operator Home Page
 *
 * Main homepage container for the Operator portal.
 * Supports dynamic widget slots for extension-based composition.
 */

import React from 'react';
import { Page, Header, Content } from '@backstage/core-components';
import { Grid, Card, CardBody, Flex, Text, Link } from '@backstage/ui';
import {
  RiDashboardLine,
  RiSearchLine,
} from '@remixicon/react';
import { QueueStatusCard, ActiveAgentsCard, IssueTypesCard } from './widgets';
import './HomePage.css';

interface QuickLinkProps {
  to: string;
  icon: React.ReactNode;
  title: string;
  description: string;
}

function QuickLinkCard({ to, icon, title, description }: QuickLinkProps) {
  return (
    <Link href={to} className="quick-link-card">
      <Card className="quick-link-card-inner">
        <CardBody>
          <Flex direction="column" align="center" gap="3" p="4">
            <div className="quick-link-icon">{icon}</div>
            <Text variant="title-small" className="quick-link-title">
              {title}
            </Text>
            <Text variant="body-small" color="secondary" className="quick-link-description">
              {description}
            </Text>
          </Flex>
        </CardBody>
      </Card>
    </Link>
  );
}

export interface OperatorHomePageProps {
  /**
   * Dynamic widgets passed from extension system.
   * When using the new frontend system, widgets are extensions
   * that attach to this page's widget input.
   */
  widgets?: React.ReactNode[];
}

export function OperatorHomePage({ widgets }: OperatorHomePageProps) {
  // Default widgets when not using extension system
  const defaultWidgets = [
    <QueueStatusCard key="queue" />,
    <ActiveAgentsCard key="agents" />,
    <IssueTypesCard key="issuetypes" />,
  ];

  const displayWidgets = widgets && widgets.length > 0 ? widgets : defaultWidgets;

  return (
    <Page themeId="home">
      <Header
        title="Operator! Portal"
        subtitle="Developer portal and agent orchestration"
      />
      <Content>
        <Flex direction="column" gap="6" p="6">
          {/* Welcome Section */}
          <Card className="welcome-card">
            <CardBody>
              <Flex direction="column" gap="2" p="4">
                <Text variant="title-large" className="welcome-title">
                  Welcome to Operator
                </Text>
                <Text variant="body-medium" className="welcome-text">
                  Manage Claude Code agents, track tickets, and explore your software catalog.
                  Monitor your queue, launch agents, and define issue type templates.
                </Text>
              </Flex>
            </CardBody>
          </Card>

          {/* Widgets Grid */}
          <Flex direction="column" gap="4">
            <Text variant="title-medium">Dashboard</Text>
            <Grid.Root columns={{ initial: '1', md: '2', lg: '3' }} gap="4">
              {displayWidgets.map((widget, index) => (
                <Grid.Item key={index}>{widget}</Grid.Item>
              ))}
            </Grid.Root>
          </Flex>

          {/* Quick Links Section */}
          <Flex direction="column" gap="4">
            <Text variant="title-medium">Quick Links</Text>
            <Grid.Root columns={{ initial: '1', sm: '2' }} gap="4">
              <Grid.Item>
                <QuickLinkCard
                  to="/catalog"
                  icon={<RiDashboardLine size={48} />}
                  title="Software Catalog"
                  description="Browse components, APIs, and systems"
                />
              </Grid.Item>
              <Grid.Item>
                <QuickLinkCard
                  to="/search"
                  icon={<RiSearchLine size={48} />}
                  title="Search"
                  description="Find anything in the catalog"
                />
              </Grid.Item>
            </Grid.Root>
          </Flex>
        </Flex>
      </Content>
    </Page>
  );
}

export default OperatorHomePage;
