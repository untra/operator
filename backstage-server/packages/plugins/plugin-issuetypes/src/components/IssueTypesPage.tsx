/**
 * Issue Types List Page
 *
 * Displays all available issue types from the Operator REST API.
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

interface IssueType {
  key: string;
  name: string;
  glyph: string;
  description: string;
  mode: 'autonomous' | 'paired';
  source: 'builtin' | 'user' | 'imported';
}

const columns: TableColumn<IssueType>[] = [
  { title: 'Glyph', field: 'glyph', width: '60px' },
  { title: 'Key', field: 'key' },
  { title: 'Name', field: 'name' },
  { title: 'Mode', field: 'mode' },
  { title: 'Source', field: 'source' },
];

export const IssueTypesPage = () => {
  const configApi = useApi(configApiRef);
  const [issueTypes, setIssueTypes] = useState<IssueType[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchIssueTypes = async () => {
      try {
        const backendUrl = configApi.getString('backend.baseUrl');
        const response = await fetch(`${backendUrl}/api/proxy/operator/api/issuetypes`);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const data = await response.json();
        setIssueTypes(data.types || []);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch issue types');
      } finally {
        setLoading(false);
      }
    };

    fetchIssueTypes();
  }, [configApi]);

  return (
    <Page themeId="tool">
      <Header title="Issue Types" subtitle="Manage Operator issue types and workflows">
        <HeaderLabel label="Source" value="Operator REST API" />
      </Header>
      <Content>
        <ContentHeader title="All Issue Types" />
        {error ? (
          <div style={{ color: 'red', padding: '16px' }}>
            Error: {error}
          </div>
        ) : (
          <Table
            title="Issue Types"
            columns={columns}
            data={issueTypes}
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
