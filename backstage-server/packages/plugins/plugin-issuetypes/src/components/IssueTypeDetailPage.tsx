/**
 * Issue Type Detail Page
 *
 * Displays detailed information about a specific issue type.
 */

import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  Page,
  InfoCard,
  LinkButton,
  Table,
  TableColumn,
} from '@backstage/core-components';
import { useIssueType, useDeleteIssueType } from '../hooks';
import type { FieldResponse, StepResponse } from '../api/types';
import { Chip } from './ui';
import styles from './IssueTypeDetailPage.module.css';

const fieldColumns: TableColumn<FieldResponse>[] = [
  { title: 'Name', field: 'name' },
  { title: 'Type', field: 'field_type' },
  {
    title: 'Required',
    field: 'required',
    render: (row) => (row.required ? 'Yes' : 'No'),
  },
  { title: 'Default', field: 'default', emptyValue: '-' },
  {
    title: 'Editable',
    field: 'user_editable',
    render: (row) => (row.user_editable ? 'Yes' : 'No'),
  },
];

const stepColumns: TableColumn<StepResponse>[] = [
  { title: 'Name', field: 'name' },
  { title: 'Display Name', field: 'display_name', emptyValue: '-' },
  {
    title: 'Outputs',
    field: 'outputs',
    render: (row) => row.outputs.join(', ') || '-',
  },
  {
    title: 'Review',
    field: 'requires_review',
    render: (row) => (row.requires_review ? 'Yes' : 'No'),
  },
  { title: 'Next Step', field: 'next_step', emptyValue: '(end)' },
  { title: 'Mode', field: 'permission_mode' },
];

export const IssueTypeDetailPage = () => {
  const { key } = useParams<{ key: string }>();
  const navigate = useNavigate();
  const { issueType, loading, error, retry } = useIssueType(key || '');
  const { deleteIssueType, deleting } = useDeleteIssueType();
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const isBuiltin = issueType?.source === 'builtin';

  const handleDelete = async () => {
    if (!issueType) return;
    try {
      await deleteIssueType(issueType.key);
      navigate('..');
    } catch {
      // Error is handled by the hook
    }
  };

  if (loading) {
    return (
      <Page themeId="tool">
        <Content>Loading...</Content>
      </Page>
    );
  }

  if (error || !issueType) {
    return (
      <Page themeId="tool">
        <Content>
          <p className={styles.error}>{error?.message || 'Issue type not found'}</p>
          <button onClick={retry}>Retry</button>
        </Content>
      </Page>
    );
  }

  return (
    <Page themeId="tool">
      <Header
        title={`${issueType.glyph} ${issueType.name}`}
        subtitle={issueType.key}
      />
      <Content>
        <ContentHeader title="Issue Type Details">
          {!isBuiltin && (
            <>
              <LinkButton to="edit" color="primary" variant="contained">
                Edit
              </LinkButton>
              <button
                onClick={() => setShowDeleteConfirm(true)}
                disabled={deleting}
                style={{
                  marginLeft: '8px',
                  padding: '8px 16px',
                  backgroundColor: '#d32f2f',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: deleting ? 'not-allowed' : 'pointer',
                }}
              >
                {deleting ? 'Deleting...' : 'Delete'}
              </button>
            </>
          )}
          {isBuiltin && (
            <Chip label="Read-only (builtin)" variant="default" size="small" />
          )}
        </ContentHeader>

        {showDeleteConfirm && (
          <div
            style={{
              padding: '16px',
              marginBottom: '16px',
              backgroundColor: '#fff3e0',
              border: '1px solid #ff9800',
              borderRadius: '4px',
            }}
          >
            <p>
              Are you sure you want to delete <strong>{issueType.key}</strong>?
            </p>
            <button
              onClick={handleDelete}
              style={{
                marginRight: '8px',
                padding: '8px 16px',
                backgroundColor: '#d32f2f',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Confirm Delete
            </button>
            <button
              onClick={() => setShowDeleteConfirm(false)}
              style={{
                padding: '8px 16px',
                backgroundColor: '#e0e0e0',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Cancel
            </button>
          </div>
        )}

        <div className={styles.grid}>
          <div className={styles.gridItemHalf}>
            <InfoCard title="Overview">
              <p className={styles.body1}>{issueType.description}</p>
              <h4 className={styles.subtitle}>Mode</h4>
              <Chip
                label={issueType.mode}
                variant={issueType.mode === 'autonomous' ? 'primary' : 'secondary'}
                size="small"
              />
              <h4 className={styles.subtitleSpaced}>Source</h4>
              <Chip label={issueType.source} size="small" />
              <h4 className={styles.subtitleSpaced}>Project Required</h4>
              <span>{issueType.project_required ? 'Yes' : 'No'}</span>
              {issueType.color && (
                <>
                  <h4 className={styles.subtitleSpaced}>Color</h4>
                  <span
                    style={{
                      display: 'inline-block',
                      width: '20px',
                      height: '20px',
                      backgroundColor: issueType.color,
                      borderRadius: '4px',
                      verticalAlign: 'middle',
                      marginRight: '8px',
                    }}
                  />
                  {issueType.color}
                </>
              )}
            </InfoCard>
          </div>
        </div>

        {issueType.fields && issueType.fields.length > 0 && (
          <div style={{ marginTop: '16px' }}>
            <InfoCard title="Fields">
              <Table
                columns={fieldColumns}
                data={issueType.fields}
                options={{ search: false, paging: false }}
              />
            </InfoCard>
          </div>
        )}

        {issueType.steps && issueType.steps.length > 0 && (
          <div style={{ marginTop: '16px' }}>
            <InfoCard title="Workflow Steps">
              <Table
                columns={stepColumns}
                data={issueType.steps}
                options={{ search: false, paging: false }}
              />
            </InfoCard>
          </div>
        )}
      </Content>
    </Page>
  );
};
