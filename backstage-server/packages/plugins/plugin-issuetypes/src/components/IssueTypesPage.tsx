/**
 * Issue Types List Page
 *
 * Displays all available issue types from the Operator REST API.
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
  LinkButton,
} from '@backstage/core-components';
import { useIssueTypes } from '../hooks';
import type { IssueTypeSummary } from '../api/types';
import { Chip } from './ui';

const columns: TableColumn<IssueTypeSummary>[] = [
  {
    title: 'Glyph',
    field: 'glyph',
    width: '60px',
    render: (row) => (
      <span style={{ fontFamily: 'monospace', fontSize: '1.2em' }}>
        {row.glyph}
      </span>
    ),
  },
  {
    title: 'Key',
    field: 'key',
    render: (row) => (
      <RouterLink
        to={row.key}
        style={{ textDecoration: 'none', color: 'inherit', fontWeight: 'bold' }}
      >
        {row.key}
      </RouterLink>
    ),
  },
  { title: 'Name', field: 'name' },
  {
    title: 'Mode',
    field: 'mode',
    render: (row) => (
      <Chip
        label={row.mode}
        variant={row.mode === 'autonomous' ? 'primary' : 'secondary'}
        size="small"
      />
    ),
  },
  {
    title: 'Source',
    field: 'source',
    render: (row) => (
      <Chip
        label={row.source}
        variant={row.source === 'builtin' ? 'default' : 'primary'}
        size="small"
      />
    ),
  },
  { title: 'Steps', field: 'step_count', width: '80px' },
];

export const IssueTypesPage = () => {
  const { issueTypes, loading, error, retry } = useIssueTypes();

  return (
    <Page themeId="tool">
      <Header
        title="Issue Types"
        subtitle="Manage Operator issue types and workflows"
        data-testid="issuetypes-page-banner"
      >
        <HeaderLabel label="Source" value="Operator REST API" />
      </Header>
      <Content>
        <ContentHeader title="All Issue Types">
          <LinkButton to="new" color="primary" variant="contained">
            Create Issue Type
          </LinkButton>
          <LinkButton
            to="collections"
            color="default"
            variant="outlined"
            style={{ marginLeft: '8px' }}
          >
            Collections
          </LinkButton>
        </ContentHeader>
        {error ? (
          <div style={{ color: 'red', padding: '16px' }}>
            <p>Error: {error.message}</p>
            <button onClick={retry}>Retry</button>
          </div>
        ) : (
          <Table
            title="Issue Types"
            columns={columns}
            data={issueTypes || []}
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
