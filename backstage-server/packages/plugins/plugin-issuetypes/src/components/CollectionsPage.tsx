/**
 * Collections Page
 *
 * Manage issue type collections (sets of issue types for different workflows).
 */

import React, { useEffect, useState } from 'react';
import {
  Content,
  ContentHeader,
  Header,
  HeaderLabel,
  Page,
  Table,
  TableColumn,
} from '@backstage/core-components';
import { useApi, configApiRef } from '@backstage/core-plugin-api';

interface Collection {
  name: string;
  description: string;
  types: string[];
  priority: string[];
}

const columns: TableColumn<Collection>[] = [
  { title: 'Name', field: 'name' },
  { title: 'Description', field: 'description' },
  {
    title: 'Types',
    field: 'types',
    render: (row) => row.types.join(', '),
  },
];

export const CollectionsPage = () => {
  const configApi = useApi(configApiRef);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchCollections = async () => {
      try {
        const backendUrl = configApi.getString('backend.baseUrl');
        const response = await fetch(`${backendUrl}/api/proxy/operator/api/collections`);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const data = await response.json();
        setCollections(data.collections || []);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch collections');
      } finally {
        setLoading(false);
      }
    };

    fetchCollections();
  }, [configApi]);

  return (
    <Page themeId="tool">
      <Header title="Collections" subtitle="Issue type collections for different workflows">
        <HeaderLabel label="Source" value="Operator REST API" />
      </Header>
      <Content>
        <ContentHeader title="All Collections" />
        {error ? (
          <div style={{ color: 'red', padding: '16px' }}>
            Error: {error}
          </div>
        ) : (
          <Table
            title="Collections"
            columns={columns}
            data={collections}
            isLoading={loading}
            options={{
              search: true,
              paging: true,
              pageSize: 10,
            }}
          />
        )}
      </Content>
    </Page>
  );
};
