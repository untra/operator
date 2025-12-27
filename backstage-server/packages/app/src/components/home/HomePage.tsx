/**
 * Operator Portal Home Page
 *
 * Landing page with quick links and overview widgets.
 * Uses BUI (Backstage UI) components for consistent theming.
 */

import { Content, Page, Header } from '@backstage/core-components';
import { Grid, Card, CardBody, Text, Flex, Link } from '@backstage/ui';
import {
  RiDashboardLine,
  RiSearchLine,
  RiFileList3Line,
  RiFolderLine,
} from '@remixicon/react';
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

function HomePage() {
  return (
    <Page themeId="home">
      <Header title="Operator! Portal" subtitle="Developer portal and ticket management" />
      <Content>
        <Flex direction="column" gap="6" p="6">
          {/* Welcome Card */}
          <Card className="welcome-card">
            <CardBody>
              <Flex direction="column" gap="2" p="4">
                <Text variant="title-large" className="welcome-title">
                  Welcome to Operator
                </Text>
                <Text variant="body-medium" className="welcome-text">
                  Your central hub for managing Claude Code agents, tickets, and software catalog.
                  Explore the catalog, create tickets, and track your work.
                </Text>
              </Flex>
            </CardBody>
          </Card>

          {/* Quick Links Section */}
          <Flex direction="column" gap="4">
            <Text variant="title-medium">Quick Links</Text>
            <Grid.Root columns={{ initial: '1', sm: '2', md: '4' }} gap="4">
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
                  to="/issuetypes"
                  icon={<RiFileList3Line size={48} />}
                  title="Issue Types"
                  description="Manage ticket templates and workflows"
                />
              </Grid.Item>
              <Grid.Item>
                <QuickLinkCard
                  to="/issuetypes/collections"
                  icon={<RiFolderLine size={48} />}
                  title="Collections"
                  description="Browse issue type collections"
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

export { HomePage };
export default HomePage;
