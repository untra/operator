/**
 * Collections Page
 *
 * Manage issue type collections (sets of issue types for different workflows).
 */

import React from 'react';
import { Link as RouterLink } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  HeaderLabel,
  Page,
  Table,
  TableColumn,
} from '@backstage/core-components';
import { useCollections, useActivateCollection } from '../hooks';
import type { CollectionResponse } from '../api/types';
import { Chip } from './ui';

export const CollectionsPage = () => {
  const { collections, loading, error, retry } = useCollections();
  const { activateCollection, activating } = useActivateCollection();

  const handleActivate = async (name: string) => {
    try {
      await activateCollection(name);
      retry(); // Refresh the list
    } catch {
      // Error is handled by the hook
    }
  };

  const columns: TableColumn<CollectionResponse>[] = [
    {
      title: 'Name',
      field: 'name',
      render: (row) => (
        <span style={{ fontWeight: row.is_active ? 'bold' : 'normal' }}>
          {row.name}
          {row.is_active && (
            <Chip
              label="Active"
              variant="primary"
              size="small"
              style={{ marginLeft: '8px' }}
            />
          )}
        </span>
      ),
    },
    { title: 'Description', field: 'description' },
    {
      title: 'Types',
      field: 'types',
      render: (row) => (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
          {row.types.map((type) => (
            <RouterLink
              key={type}
              to={`../${type}`}
              style={{ textDecoration: 'none' }}
            >
              <Chip label={type} size="small" variant="default" />
            </RouterLink>
          ))}
        </div>
      ),
    },
    {
      title: 'Actions',
      field: 'name',
      render: (row) =>
        !row.is_active ? (
          <button
            onClick={() => handleActivate(row.name)}
            disabled={activating}
            style={{
              padding: '4px 12px',
              backgroundColor: '#1976d2',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: activating ? 'not-allowed' : 'pointer',
            }}
          >
            {activating ? 'Activating...' : 'Activate'}
          </button>
        ) : (
          <span style={{ color: '#4caf50' }}>Current</span>
        ),
    },
  ];

  return (
    <Page themeId="tool">
      <Header
        title="Collections"
        subtitle="Issue type collections for different workflows"
      >
        <HeaderLabel label="Source" value="Operator REST API" />
      </Header>
      <Content>
        <ContentHeader title="All Collections" />
        {error ? (
          <div style={{ color: 'red', padding: '16px' }}>
            <p>Error: {error.message}</p>
            <button onClick={retry}>Retry</button>
          </div>
        ) : (
          <Table
            title="Collections"
            columns={columns}
            data={collections || []}
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
