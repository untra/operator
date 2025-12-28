/**
 * Plugins Page
 *
 * Displays installed Backstage plugins with metadata including
 * routes, APIs, and status information.
 */

import React from 'react';
import {
  Content,
  ContentHeader,
  Header,
  Page,
  Table,
  TableColumn,
} from '@backstage/core-components';

interface PluginInfo {
  id: string;
  name: string;
  description: string;
  routes: string[];
  apis: string[];
  status: 'active' | 'inactive';
}

const columns: TableColumn<PluginInfo>[] = [
  { title: 'Plugin ID', field: 'id' },
  { title: 'Name', field: 'name' },
  { title: 'Description', field: 'description' },
  {
    title: 'Routes',
    field: 'routes',
    render: (row) => row.routes.join(', ') || 'None',
  },
  {
    title: 'APIs',
    field: 'apis',
    render: (row) => row.apis.join(', ') || 'None',
  },
  {
    title: 'Status',
    field: 'status',
    render: (row) => (
      <span style={{ color: row.status === 'active' ? '#4caf50' : '#9e9e9e' }}>
        {row.status}
      </span>
    ),
  },
];

// Static plugin info based on registered plugins in App.tsx
const installedPlugins: PluginInfo[] = [
  {
    id: 'catalog',
    name: 'Backstage Catalog',
    description: 'Software catalog for tracking components, APIs, and resources',
    routes: ['/catalog', '/catalog/:namespace/:kind/:name'],
    apis: ['catalogApiRef'],
    status: 'active',
  },
  {
    id: 'search',
    name: 'Backstage Search',
    description: 'Full-text search across catalog entities',
    routes: ['/search'],
    apis: ['searchApiRef'],
    status: 'active',
  },
  {
    id: 'issuetypes',
    name: 'Operator Issue Types',
    description: 'Manage issue types, workflows, and collections for Operator',
    routes: ['/issuetypes', '/issuetypes/:key', '/issuetypes/new', '/issuetypes/collections'],
    apis: ['operatorApiRef'],
    status: 'active',
  },
];

export const PluginsPage = () => {
  return (
    <Page themeId="tool">
      <Header
        title="Installed Plugins"
        subtitle="Backstage plugins configured in this portal"
        data-testid="plugins-page-banner"
      />
      <Content>
        <ContentHeader title="Plugin Registry" />
        <Table
          title="Plugins"
          columns={columns}
          data={installedPlugins}
          options={{
            search: true,
            paging: false,
          }}
        />
      </Content>
    </Page>
  );
};
